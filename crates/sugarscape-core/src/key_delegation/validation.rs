use super::types::*;
use serde::{
    de::{DeserializeOwned, Error as DeError, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;

/// Preserve decoded duplicate detection and integer atom syntax before any map exists.
pub(super) fn strict_value<'de, D: Deserializer<'de>>(d: D) -> Result<Value, D::Error> {
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
pub(super) fn decode<T: DeserializeOwned + Serialize, E: DeError>(v: Value) -> Result<T, E> {
    let wire: T = serde_json::from_value(v.clone()).map_err(E::custom)?;
    if serde_json::to_value(&wire).map_err(E::custom)? != v {
        return Err(E::custom("noncanonical native schema shape"));
    }
    Ok(wire)
}
impl Stratum {
    pub fn validate(&self) -> Result<(), Error> {
        if ![1, 2, 4].contains(&self.transfer_cost())
            || ![0, 3, 6].contains(&self.giver_share())
            || self.distance() > 1
        {
            Err(invalid("stratum domain"))
        } else {
            Ok(())
        }
    }
}
impl Ids {
    pub fn validate(&self) -> Result<(), Error> {
        if self.giver().is_empty() || self.receiver().is_empty() || self.giver() == self.receiver()
        {
            Err(invalid("IDs must be nonempty and distinct"))
        } else {
            Ok(())
        }
    }
}
impl Config {
    pub fn validate(&self) -> Result<(), Error> {
        self.stratum().validate()?;
        self.ids().validate()
    }
}
impl Profile {
    pub fn validate(&self) -> Result<(), Error> {
        if self.giver() > 728 || self.receiver() > 63 {
            Err(invalid("profile domain"))
        } else {
            Ok(())
        }
    }
}
impl Clock {
    pub fn validate(&self) -> Result<(), Error> {
        let valid = match self.phase {
            Phase::Initial => self.round == 0 && self.role.is_none(),
            Phase::BeforeAction | Phase::AfterAction => {
                (1..=6).contains(&self.round) && self.role.is_some()
            }
            Phase::Finished => (1..=6).contains(&self.round) && self.role.is_none(),
        };
        if valid {
            Ok(())
        } else {
            Err(history_error("clock domain"))
        }
    }
}
fn validate_finances(spent: [u8; 2], reward: [u8; 2]) -> Result<(), Error> {
    if spent[0] > 24 || spent[1] > 6 || !matches!(reward, [0, 0] | [0, 12] | [3, 9] | [6, 6]) {
        return Err(history_error("role spending or registered reward bounds"));
    }
    Ok(())
}
impl PhysicalState {
    pub fn validate(&self) -> Result<(), Error> {
        validate_finances(self.spent, self.reward)?;
        if self.positions[0] == Cell::C
            || self.positions[1] == Cell::P
            || self.receiver_cargo != (self.resource == ResourceLocation::Carried)
            || self.reward != [0, 0] && self.resource != ResourceLocation::Delivered
        {
            return Err(history_error("physical state"));
        }
        Ok(())
    }
}
impl VisibleOccupant {
    pub fn validate(&self) -> Result<(), Error> {
        if self.id.is_empty() || (self.role == Role::Giver && self.cargo) {
            Err(history_error("visible occupant"))
        } else {
            Ok(())
        }
    }
}
impl VisibleCell {
    pub fn validate(&self) -> Result<(), Error> {
        for occupant in &self.occupants {
            occupant.validate()?;
        }
        if self.occupants.len() > 2
            || self.occupants.windows(2).any(|w| w[0].role >= w[1].role)
            || self.loose_token && self.cell != Cell::C
            || self.occupants.iter().any(|o| {
                (o.role == Role::Giver && self.cell == Cell::C)
                    || (o.role == Role::Receiver && self.cell == Cell::P)
            })
        {
            Err(history_error("visible cell"))
        } else {
            Ok(())
        }
    }
}
impl Observation {
    pub fn validate(&self) -> Result<(), Error> {
        self.clock.validate()?;
        for cell in &self.cells {
            cell.validate()?;
        }
        let cells: Vec<_> = self.cells.iter().map(|c| c.cell).collect();
        if ![
            vec![Cell::P, Cell::O],
            vec![Cell::P, Cell::O, Cell::C],
            vec![Cell::O, Cell::C],
        ]
        .contains(&cells)
        {
            return Err(history_error("visible cell order"));
        }
        let occupants: Vec<_> = self.cells.iter().flat_map(|c| &c.occupants).collect();
        let visible_keys = occupants.iter().filter(|o| o.key).count();
        if visible_keys > 1
            || occupants.len() == 2 && visible_keys != 1
            || occupants
                .iter()
                .enumerate()
                .any(|(i, a)| occupants[i + 1..].iter().any(|b| a.id == b.id))
            || occupants.iter().any(|a| a.cargo) && self.cells.iter().any(|c| c.loose_token)
            || [Role::Giver, Role::Receiver]
                .iter()
                .any(|r| occupants.iter().filter(|o| o.role == *r).count() > 1)
        {
            return Err(history_error("visible Agent, key or resource conservation"));
        }
        Ok(())
    }
}
impl OwnershipReceipt {
    pub fn validate(&self) -> Result<(), Error> {
        self.clock.validate()?;
        if self.clock.phase != Phase::AfterAction
            || self.clock.role != Some(Role::Giver)
            || self.counterpart.is_empty()
            || self.before == self.after
        {
            Err(history_error("ownership receipt"))
        } else {
            Ok(())
        }
    }
}
impl Event {
    pub fn validate(&self) -> Result<(), Error> {
        self.clock.validate()?;
        self.before.validate()?;
        self.after.validate()?;
        match &self.kind {
            EventKind::Action {
                role,
                token,
                intent,
                result: _,
                paid,
            } => {
                if self.clock.phase != Phase::AfterAction
                    || self.clock.role != Some(*role)
                    || *paid > 4
                    || !matches!(
                        (role, token),
                        (Role::Giver, Token::Giver(_)) | (Role::Receiver, Token::Receiver(_))
                    )
                {
                    return Err(history_error("action envelope"));
                }
                super::controller::own_intent(
                    *role,
                    token.clone(),
                    self.before.positions[role.index()],
                    *role == Role::Receiver && self.before.receiver_cargo,
                )
                .and_then(|expected| {
                    if expected == *intent {
                        Ok(())
                    } else {
                        Err(history_error("intent mismatch"))
                    }
                })
            }
            EventKind::Terminal(_) => {
                if self.clock.phase == Phase::Finished {
                    Ok(())
                } else {
                    Err(history_error("terminal clock"))
                }
            }
        }
    }
}
impl Outcome {
    pub fn validate(&self) -> Result<(), Error> {
        validate_finances(self.spent, self.reward)?;
        if self.transfer_count > 1
            || self.delivery_round.is_some_and(|q| !(4..=6).contains(&q))
            || self
                .giver_attempts
                .iter()
                .chain(&self.giver_blocked)
                .chain(&self.receiver_attempts)
                .chain(&self.receiver_blocked)
                .any(|v| *v > 6)
            || self.receiver_waits > 6
            || self.approach_spent > 6
            || self.pass_spent > 24
            || !(-24..=6).contains(&self.net[0])
            || !(-6..=12).contains(&self.net[1])
            || !(-30..=12).contains(&self.group_net)
            || !(8..=32).contains(&self.unused_credits[0])
            || !(26..=32).contains(&self.unused_credits[1])
        {
            return Err(history_error("outcome bounds"));
        }
        for i in 0..2 {
            if self.net[i] != i16::from(self.reward[i]) - i16::from(self.spent[i])
                || self.unused_credits[i] != 32 - self.spent[i]
            {
                return Err(history_error("outcome ledger"));
            }
        }
        if self.group_net
            != self.net[0]
                .checked_add(self.net[1])
                .ok_or(Error::ArithmeticOverflow)?
        {
            return Err(history_error("group ledger"));
        }
        let sum = |counts: &[u8]| {
            counts.iter().try_fold(0u8, |total, count| {
                total.checked_add(*count).ok_or(Error::ArithmeticOverflow)
            })
        };
        let giver_actions = sum(&self.giver_attempts)?;
        let receiver_advances = sum(&self.receiver_attempts)?;
        let receiver_actions = receiver_advances
            .checked_add(self.receiver_waits)
            .ok_or(Error::ArithmeticOverflow)?;
        if giver_actions > 6
            || receiver_actions > 6
            || giver_actions
                .checked_sub(receiver_actions)
                .is_none_or(|gap| gap > 1)
        {
            return Err(history_error("action counts exceed the public schedule"));
        }
        let mut giver_successes = [0; 3];
        let mut receiver_successes = [0; 4];
        for (i, successes) in giver_successes.iter_mut().enumerate() {
            *successes = self.giver_attempts[i]
                .checked_sub(self.giver_blocked[i])
                .ok_or_else(|| history_error("Giver blocked exceeds attempts"))?;
        }
        for (i, successes) in receiver_successes.iter_mut().enumerate() {
            *successes = self.receiver_attempts[i]
                .checked_sub(self.receiver_blocked[i])
                .ok_or_else(|| history_error("Receiver blocked exceeds attempts"))?;
        }
        if self.giver_blocked[0] != 0
            || giver_successes[1] > 1
            || giver_successes[2] != self.transfer_count
            || self.approach_spent != self.giver_attempts[1]
            || ![1u8, 2, 4]
                .iter()
                .any(|cost| self.giver_attempts[2].checked_mul(*cost) == Some(self.pass_spent))
            || self.approach_spent.checked_add(self.pass_spent) != Some(self.spent[0])
            || receiver_advances != self.spent[1]
        {
            return Err(history_error("action and spending ledgers disagree"));
        }
        if receiver_successes.iter().any(|n| *n > 1)
            || receiver_successes.windows(2).any(|pair| pair[0] < pair[1])
            || self.receiver_blocked[1..].iter().any(|n| *n != 0)
            || self.delivery_round.is_some() != (receiver_successes[3] == 1)
            || self
                .delivery_round
                .is_some_and(|q| giver_actions != q || receiver_actions != q)
            || self.reward != [0, 0] && self.delivery_round.is_none()
        {
            return Err(history_error("resource progress or delivery ledger"));
        }
        // Delivery commits before settlement: a delivery round with zero rewards is valid.
        Ok(())
    }
}
impl Episode {
    pub fn validate(&self) -> Result<(), Error> {
        self.config.validate()?;
        self.profile.validate()?;
        let expected = super::run_episode(&self.config, self.profile)?;
        if self != &expected {
            Err(history_error("complete Episode differs from native replay"))
        } else {
            Ok(())
        }
    }
}
impl View {
    pub fn validate(&self) -> Result<(), Error> {
        super::observation::validate_view(self)
    }
}
impl CapturePoint {
    pub fn validate(&self) -> Result<(), Error> {
        self.clock.validate()?;
        self.physical.validate()?;
        for (i, v) in self.views.iter().enumerate() {
            v.validate()?;
            if v.role.index() != i
                || v.clock != self.clock
                || v.own_cell != self.physical.positions[i]
                || v.own_key != (self.physical.key_owner == v.role)
                || v.own_cargo != (v.role == Role::Receiver && self.physical.receiver_cargo)
                || v.credits != 32 - self.physical.spent[i]
                || v.own_reward != self.physical.reward[i]
            {
                return Err(history_error("capture own projection"));
            }
        }
        Ok(())
    }
}
