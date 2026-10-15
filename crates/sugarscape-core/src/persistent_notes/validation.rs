use super::types::*;
use serde::{
    de::{DeserializeOwned, Error as DeError, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;

/// Preserve decoded duplicate detection and integer atom syntax before any map exists.
fn strict_value<'de, D: Deserializer<'de>>(d: D) -> Result<Value, D::Error> {
    struct Strict;
    impl<'de> Visitor<'de> for Strict {
        type Value = Value;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("exact native JSON")
        }
        fn visit_bool<E: DeError>(self, v: bool) -> Result<Value, E> {
            Ok(v.into())
        }
        fn visit_i64<E: DeError>(self, v: i64) -> Result<Value, E> {
            Ok(v.into())
        }
        fn visit_u64<E: DeError>(self, v: u64) -> Result<Value, E> {
            Ok(v.into())
        }
        fn visit_f64<E: DeError>(self, _: f64) -> Result<Value, E> {
            Err(E::custom("noninteger atom or negative zero"))
        }
        fn visit_str<E: DeError>(self, v: &str) -> Result<Value, E> {
            Ok(v.into())
        }
        fn visit_string<E: DeError>(self, v: String) -> Result<Value, E> {
            Ok(v.into())
        }
        fn visit_unit<E: DeError>(self) -> Result<Value, E> {
            Ok(Value::Null)
        }
        fn visit_none<E: DeError>(self) -> Result<Value, E> {
            Ok(Value::Null)
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
            let mut out = Vec::new();
            while let Some(v) = a.next_element::<StrictValue>()? {
                out.push(v.0);
            }
            Ok(Value::Array(out))
        }
        fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
            let mut out = serde_json::Map::new();
            while let Some(k) = a.next_key::<String>()? {
                if out.contains_key(&k) {
                    return Err(A::Error::custom(format!("duplicate field {k}")));
                }
                out.insert(k, a.next_value::<StrictValue>()?.0);
            }
            Ok(Value::Object(out))
        }
    }
    struct StrictValue(Value);
    impl<'de> Deserialize<'de> for StrictValue {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            strict_value(d).map(Self)
        }
    }
    d.deserialize_any(Strict)
}

/// Round-trip shape equality rejects positional structs, omitted nulls and unit objects.
fn decode<T: DeserializeOwned + Serialize, E: DeError>(v: Value) -> Result<T, E> {
    let wire: T = serde_json::from_value(v.clone()).map_err(E::custom)?;
    if serde_json::to_value(&wire).map_err(E::custom)? != v {
        return Err(E::custom("noncanonical native schema shape"));
    }
    Ok(wire)
}
pub(super) fn parse<T: DeserializeOwned + Serialize>(text: &str) -> Result<T, Error> {
    if text.len() > 4096 {
        return Err(Error::ResourceLimit {
            resource: "config_bytes".into(),
            limit: 4096,
        });
    }
    // Examine integer tokens outside strings before serde_json can convert oversized atoms to f64.
    let bytes = text.as_bytes();
    let mut i = 0;
    let mut quoted = false;
    let mut escaped = false;
    while i < bytes.len() {
        let c = bytes[i];
        if quoted {
            if escaped {
                escaped = false;
            } else if c == b'\\' {
                escaped = true;
            } else if c == b'"' {
                quoted = false;
            }
            i += 1;
            continue;
        }
        if c == b'"' {
            quoted = true;
            i += 1;
            continue;
        }
        if c == b'-' || c.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < bytes.len()
                && (bytes[i].is_ascii_digit()
                    || matches!(bytes[i], b'.' | b'e' | b'E' | b'+' | b'-'))
            {
                i += 1;
            }
            let token = &text[start..i];
            if token.contains(['.', 'e', 'E']) || token == "-0" {
                return Err(Error::InvalidInput("integer required".into()));
            }
            if token.starts_with('-') {
                token
                    .parse::<i64>()
                    .map_err(|_| Error::ArithmeticOverflow)?;
            } else {
                token
                    .parse::<u64>()
                    .map_err(|_| Error::ArithmeticOverflow)?;
            }
        } else {
            i += 1;
        }
    }
    struct Raw(Value);
    impl<'de> Deserialize<'de> for Raw {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            strict_value(d).map(Self)
        }
    }
    let raw: Raw = serde_json::from_str(text).map_err(|e| Error::InvalidInput(e.to_string()))?;
    decode::<T, serde_json::Error>(raw.0).map_err(|e| Error::InvalidInput(e.to_string()))
}
pub fn config_from_json(text: &str) -> Result<Config, Error> {
    let config = config_wire_from_json(text)?;
    config.validate()?;
    Ok(config)
}
/// Decode the exact wire language; callers must validate registered scientific settings.
pub fn config_wire_from_json(text: &str) -> Result<Config, Error> {
    parse(text)
}
impl Config {
    pub fn validate(&self) -> Result<(), Error> {
        self.validate_private()?;
        let s = &self.stratum;
        if ![1, 4].contains(&s.acquisition_cost)
            || ![1, 2, 4, 6].contains(&s.write_cost)
            || ![1, 2].contains(&s.note_read_cost)
            || ![2, 4, 8].contains(&s.manifest_inspection_cost)
            || ![0, 3, 6].contains(&s.writer_share)
        {
            return Err(Error::InvalidInput("stratum domain".into()));
        }
        Ok(())
    }
    pub(super) fn validate_private(&self) -> Result<(), Error> {
        for id in [&self.ids.writer, &self.ids.reader] {
            if id.is_empty()
                || id.len() > 32
                || !id
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-'))
            {
                return Err(Error::InvalidInput("ID domain".into()));
            }
        }
        if self.ids.writer == self.ids.reader {
            return Err(Error::InvalidInput("IDs must be distinct".into()));
        }
        if self.stratum.writer_share > 12 {
            return Err(Error::InvalidInput("reward share".into()));
        }
        Ok(())
    }
}
impl Primitive {
    pub fn validate(&self) -> Result<(), Error> {
        let valid = match self.kind {
            PrimitiveKind::Move => {
                self.cell.is_some() && self.note.is_none() && self.good.is_none()
            }
            PrimitiveKind::WriteNote => {
                matches!(self.note, Some(Note::NoteLeft | Note::NoteRight))
                    && self.cell.is_none()
                    && self.good.is_none()
            }
            PrimitiveKind::Pickup => {
                self.good.is_some() && self.cell.is_none() && self.note.is_none()
            }
            _ => self.cell.is_none() && self.note.is_none() && self.good.is_none(),
        };
        if valid {
            Ok(())
        } else {
            Err(Error::InvalidInput("primitive payload".into()))
        }
    }
}
pub fn primitive_from_json(text: &str) -> Result<Primitive, Error> {
    let primitive: Primitive = parse(text)?;
    primitive.validate()?;
    Ok(primitive)
}
pub(super) fn validate_checkpoint(p: &Checkpoint) -> Result<(), Error> {
    let fail = || Error::InvalidHistory("inconsistent state".into());
    for role in [Role::Writer, Role::Reader] {
        let a = &p.physical.participants[role.index()];
        let v = &p.own[role.index()];
        if a.role != role
            || a.key != (p.physical.key_owner == role)
            || a.credits.checked_add(a.spent) != Some(32)
            || a.active != a.cell.is_some()
            || v.role != role
            || v.clock != p.clock
            || v.status != p.public
            || v.cell != a.cell
            || v.key != a.key
            || v.cargo != a.cargo
            || v.credits != a.credits
            || v.spent != a.spent
            || v.reward != a.reward
            || v.history.len() > if role == Role::Writer { 4 } else { 6 }
            || v.map.cells.len() > 3
        {
            return Err(fail());
        }
        let sum = v.history.iter().try_fold(0u64, |s, f| {
            s.checked_add(f.cost).ok_or(Error::ArithmeticOverflow)
        })?;
        if sum != a.spent {
            return Err(fail());
        }
        if v.map.cells.windows(2).any(|c| c[0].cell >= c[1].cell) {
            return Err(fail());
        }
    }
    if p.public.writer_active != p.physical.participants[0].active
        || !p.public.reader_active
        || p.clock.writer_slot > 4
        || p.clock.reader_slot > 6
        || p.own[0].history.len() as u64 != p.clock.writer_slot
        || p.own[1].history.len() as u64 != p.clock.reader_slot
        || p.physical.participants[0].cargo.is_some()
        || p.physical.settled != (p.public.terminal_reason == Some(TerminalReason::Delivered))
        || p.public.terminal != p.public.terminal_reason.is_some()
    {
        return Err(fail());
    }
    if let Some(g) = p.physical.participants[1].cargo {
        if p.physical.goods[g.index()] {
            return Err(fail());
        }
    }
    if (p.physical.field == Note::Blank) != p.physical.lineage.is_none() {
        return Err(fail());
    }
    Ok(())
}
