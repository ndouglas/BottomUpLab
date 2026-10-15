//! Exact source-bound diagnostics, streaming reports and saved-action reconstruction.
use super::{
    budget::{encoded_len, Budget},
    canonical,
    kernel::World,
    metrics::*,
    protocol::*,
    references, sha256, source_inputs,
    types::*,
    Counters, Limits,
};
use serde::{
    de::{DeserializeOwned, DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use serde_json::Value;
use source_inputs::SourceIdentity;
use std::{
    fmt,
    io::{Read, Write},
};
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Report {
    pub version: String,
    pub protocol_version: u64,
    pub protocol_id: String,
    pub source: SourceIdentity,
    pub reference: SourceIdentity,
    pub limits: Limits,
    pub rows: Vec<Episode>,
    pub means: Vec<MeanRow>,
    pub request_contrasts: Vec<RequestContrast>,
    pub contrasts: Vec<MeanContrast>,
    pub envelopes: Vec<Envelope>,
    pub counters: Counters,
    pub payload_sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Summary {
    pub version: String,
    pub protocol_version: u64,
    pub protocol_id: String,
    pub source: SourceIdentity,
    pub reference: SourceIdentity,
    pub report_sha256: String,
    pub means: Vec<MeanRow>,
    pub request_contrasts: Vec<RequestContrast>,
    pub contrasts: Vec<MeanContrast>,
    pub envelopes: Vec<Envelope>,
    pub counters: Counters,
}
fn invalid(message: &str) -> Error {
    Error::InvalidReport(message.into())
}
fn io(e: std::io::Error) -> Error {
    Error::Io(e.to_string())
}
struct Key;
impl<'de> DeserializeSeed<'de> for Key {
    type Value = String;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
        d.deserialize_str(self)
    }
}
impl<'de> Visitor<'de> for Key {
    type Value = String;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded ASCII object key")
    }
    fn visit_str<E: serde::de::Error>(self, key: &str) -> Result<String, E> {
        if key.len() > 32 || !key.is_ascii() {
            return Err(E::custom("object key domain"));
        }
        Ok(key.to_owned())
    }
}
struct Reject;
impl<'de> Deserialize<'de> for Reject {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Extra;
        impl<'de> Visitor<'de> for Extra {
            type Value = Reject;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("end of bounded array")
            }
        }
        d.deserialize_any(Extra)
    }
}
// Limits apply before reserving each record/array member. Values never contain whole reports.
struct RecordBudget {
    remaining: u64,
    key_length: usize,
}
impl RecordBudget {
    fn charge<E: serde::de::Error>(&mut self, bytes: u64) -> Result<(), E> {
        self.remaining = self
            .remaining
            .checked_sub(bytes)
            .ok_or_else(|| E::custom("record byte budget"))?;
        Ok(())
    }
}
struct ExactValue<'a> {
    field: &'a str,
    depth: usize,
    budget: &'a mut RecordBudget,
    role: Option<Role>,
}
impl<'de> DeserializeSeed<'de> for ExactValue<'_> {
    type Value = Value;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for ExactValue<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("bounded exact record")
    }
    fn visit_bool<E: serde::de::Error>(self, v: bool) -> Result<Value, E> {
        self.budget.charge::<E>(v.to_string().len() as u64)?;
        Ok(v.into())
    }
    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Value, E> {
        self.budget.charge::<E>(v.to_string().len() as u64)?;
        Ok(v.into())
    }
    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Value, E> {
        self.budget.charge::<E>(v.to_string().len() as u64)?;
        Ok(v.into())
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Value, E> {
        Err(E::custom("integer required"))
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Value, E> {
        self.budget.charge::<E>(4)?;
        Ok(Value::Null)
    }
    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Value, E> {
        let cap = match self.field {
            "writer" | "reader" => 32,
            "sha256" | "payload_sha256" => 64,
            "path" => 192,
            "cell" => 1,
            "good" | "cargo" | "request" | "known_request" | "observed_request" | "picked_good" => {
                5
            }
            "note" | "symbol" | "field" | "read_symbol" | "phase" | "terminal_reason" => 9,
            "role" | "actor" | "key_owner" | "key_holder" => 6,
            "status" => 7,
            "block_reason" | "writer_program" => 13,
            "reader_routine" | "routine" => 12,
            "arm" | "kind" => 15,
            "version" => REPORT_VERSION.len(),
            "protocol_id" => PROTOCOL_ID.len(),
            "domain" => "persistent-notes-reference-v1".len(),
            _ => 128,
        };
        if v.len() > cap || !v.is_ascii() {
            return Err(E::custom("string domain"));
        }
        self.budget
            .charge::<E>(encoded_len(&v, 1152).map_err(E::custom)?)?;
        Ok(v.into())
    }
    fn visit_string<E: serde::de::Error>(self, v: String) -> Result<Value, E> {
        self.visit_str(&v)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
        if self.depth >= 24 {
            return Err(A::Error::custom("record depth"));
        }
        let cap = match self.field {
            "checkpoints" => 16,
            "history" => {
                if self.role == Some(Role::Writer) {
                    4
                } else {
                    6
                }
            }
            "cells" | "ties" => 3,
            "receipts" => 32,
            "key" => self.budget.key_length,
            "belief" | "request_prior" | "own" | "participants" | "goods" | "occupants"
            | "attempted" | "successful" | "blocked" | "spent" | "reward" | "net"
            | "inspections" | "acquisition" | "writing" | "reading" | "inspection" | "movement"
            | "handoff" | "pickup" | "delivery" | "wait" => 2,
            _ => return Err(A::Error::custom("unexpected array")),
        };
        self.budget.charge::<A::Error>(2)?;
        let mut out = Vec::new();
        for index in 0..cap {
            let Some(v) = a.next_element_seed(ExactValue {
                field: "",
                depth: self.depth + 1,
                budget: self.budget,
                role: if self.field == "own" {
                    Some(if index == 0 {
                        Role::Writer
                    } else {
                        Role::Reader
                    })
                } else {
                    self.role
                },
            })?
            else {
                return Ok(Value::Array(out));
            };
            if index != 0 {
                self.budget.charge::<A::Error>(1)?;
            }
            out.push(v);
        }
        if a.next_element::<Reject>()?.is_some() {
            return Err(A::Error::custom("array length"));
        }
        Ok(Value::Array(out))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Value, A::Error> {
        if self.depth >= 24 {
            return Err(A::Error::custom("record depth"));
        }
        self.budget.charge::<A::Error>(2)?;
        let mut out = serde_json::Map::new();
        while let Some(k) = a.next_key_seed(Key)? {
            if k.len() > 32 || out.len() >= 32 || out.contains_key(&k) {
                return Err(A::Error::custom(
                    "unknown, excess or duplicate record field",
                ));
            }
            self.budget.charge::<A::Error>(
                encoded_len(&k, 192).map_err(A::Error::custom)? + 1 + u64::from(!out.is_empty()),
            )?;
            let v = a.next_value_seed(ExactValue {
                field: &k,
                depth: self.depth + 1,
                budget: self.budget,
                role: self.role,
            })?;
            if k == "denominator" && v.as_u64().is_none_or(|n| n == 0) {
                return Err(A::Error::custom("ratio denominator"));
            }
            out.insert(k, v);
        }
        if out.contains_key("numerator") && out.contains_key("denominator") {
            let n = out["numerator"]
                .as_i64()
                .ok_or_else(|| A::Error::custom("ratio numerator"))?;
            let d = out["denominator"]
                .as_u64()
                .ok_or_else(|| A::Error::custom("ratio denominator"))?;
            let (mut x, mut y) = (n.unsigned_abs(), d);
            while y != 0 {
                let t = x % y;
                x = y;
                y = t;
            }
            if x != 1 {
                return Err(A::Error::custom("unreduced ratio"));
            }
        }
        Ok(Value::Object(out))
    }
}
pub(super) struct Record<T> {
    max: u64,
    _kind: std::marker::PhantomData<T>,
}
impl<T> Record<T> {
    pub(super) fn new(max: u64) -> Self {
        Self {
            max,
            _kind: std::marker::PhantomData,
        }
    }
}
impl<'de, T: DeserializeOwned + Serialize> DeserializeSeed<'de> for Record<T> {
    type Value = T;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<T, D::Error> {
        let mut budget = RecordBudget {
            remaining: self.max,
            key_length: match self.max {
                1325 | 1334 => 7,
                141 => 6,
                _ => 8,
            },
        };
        let value = ExactValue {
            field: "",
            depth: 0,
            budget: &mut budget,
            role: None,
        }
        .deserialize(d)?;
        encoded_len(&value, self.max).map_err(D::Error::custom)?;
        let result: T = serde_json::from_value(value.clone()).map_err(D::Error::custom)?;
        if serde_json::to_value(&result).map_err(D::Error::custom)? != value {
            return Err(D::Error::custom("required field/schema mismatch"));
        }
        Ok(result)
    }
}
#[cfg(test)]
pub(crate) fn decode_record<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
    max: u64,
) -> Result<T, Error> {
    if bytes.len() as u64 > max {
        return Err(Error::ResourceLimit {
            resource: "record_bytes".into(),
            limit: max,
        });
    }
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let record = Record::new(max)
        .deserialize(&mut decoder)
        .map_err(|e| Error::InvalidInput(e.to_string()))?;
    decoder
        .end()
        .map_err(|e| Error::InvalidInput(e.to_string()))?;
    Ok(record)
}
pub(super) struct Array<T> {
    length: usize,
    max: u64,
    _kind: std::marker::PhantomData<T>,
}
impl<T> Array<T> {
    pub(super) fn new(length: usize, max: u64) -> Self {
        Self {
            length,
            max,
            _kind: std::marker::PhantomData,
        }
    }
}
impl<'de, T: DeserializeOwned + Serialize> DeserializeSeed<'de> for Array<T> {
    type Value = Vec<T>;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<Vec<T>, D::Error> {
        d.deserialize_seq(self)
    }
}
impl<'de, T: DeserializeOwned + Serialize> Visitor<'de> for Array<T> {
    type Value = Vec<T>;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("complete bounded ordered array")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<Vec<T>, A::Error> {
        let mut out = Vec::with_capacity(self.length);
        for _ in 0..self.length {
            out.push(
                a.next_element_seed(Record::new(self.max))?
                    .ok_or_else(|| A::Error::custom("missing row"))?,
            );
        }
        if a.next_element::<Reject>()?.is_some() {
            return Err(A::Error::custom("extra row"));
        }
        Ok(out)
    }
}
impl<'de> Deserialize<'de> for Report {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        struct Fields;
        impl<'de> Visitor<'de> for Fields {
            type Value = Report;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("registered report")
            }
            fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<Report, A::Error> {
                let (
                    mut version,
                    mut protocol_version,
                    mut protocol_id,
                    mut source,
                    mut reference,
                    mut limits,
                    mut rows,
                    mut means,
                    mut request_contrasts,
                    mut contrasts,
                    mut envelopes,
                    mut counters,
                    mut payload_sha256,
                ) = (
                    None, None, None, None, None, None, None, None, None, None, None, None, None,
                );
                let mut seen = std::collections::BTreeSet::new();
                while let Some(k) = a.next_key_seed(Key)? {
                    if !seen.insert(k.clone()) {
                        return Err(A::Error::custom("duplicate report field"));
                    }
                    match k.as_str() {
                        "version" => version = Some(a.next_value_seed(Record::new(128))?),
                        "protocol_version" => protocol_version = Some(a.next_value()?),
                        "protocol_id" => protocol_id = Some(a.next_value_seed(Record::new(128))?),
                        "source" => source = Some(a.next_value_seed(Record::new(65536))?),
                        "reference" => reference = Some(a.next_value_seed(Record::new(65536))?),
                        "limits" => limits = Some(a.next_value_seed(Record::new(65536))?),
                        "counters" => counters = Some(a.next_value_seed(Record::new(65536))?),
                        "payload_sha256" => {
                            payload_sha256 = Some(a.next_value_seed(Record::new(128))?)
                        }
                        "rows" => rows = Some(a.next_value_seed(Array::new(4608, 98766))?),
                        "means" => means = Some(a.next_value_seed(Array::new(2304, 1325))?),
                        "request_contrasts" => {
                            request_contrasts = Some(a.next_value_seed(Array::new(4608, 1342))?)
                        }
                        "contrasts" => contrasts = Some(a.next_value_seed(Array::new(2304, 1334))?),
                        "envelopes" => envelopes = Some(a.next_value_seed(Array::new(576, 141))?),
                        _ => return Err(A::Error::custom("unknown report field")),
                    }
                }
                macro_rules! required {
                    ($n:ident) => {
                        $n.ok_or_else(|| A::Error::missing_field(stringify!($n)))?
                    };
                }
                Ok(Report {
                    version: required!(version),
                    protocol_version: required!(protocol_version),
                    protocol_id: required!(protocol_id),
                    source: required!(source),
                    reference: required!(reference),
                    limits: required!(limits),
                    rows: required!(rows),
                    means: required!(means),
                    request_contrasts: required!(request_contrasts),
                    contrasts: required!(contrasts),
                    envelopes: required!(envelopes),
                    counters: required!(counters),
                    payload_sha256: required!(payload_sha256),
                })
            }
        }
        d.deserialize_map(Fields)
    }
}
/// Reader guard bounds serde's token scratch space before allocation; report bytes count raw input.
struct Guard<R> {
    reader: R,
    bytes: u64,
    quoted: bool,
    escape: bool,
    string_bytes: u64,
    atom_bytes: u64,
}
impl<R: Read> Read for Guard<R> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        let n = self.reader.read(out)?;
        for &b in &out[..n] {
            self.bytes += 1;
            if self.bytes > 467585152 {
                return Err(std::io::Error::other("report_bytes limit"));
            }
            if self.quoted {
                self.string_bytes += 1;
                if self.string_bytes > 1153 {
                    return Err(std::io::Error::other("string token limit"));
                }
                if self.escape {
                    self.escape = false;
                } else if b == b'\\' {
                    self.escape = true;
                } else if b == b'"' {
                    self.quoted = false;
                }
            } else if b == b'"' {
                self.quoted = true;
                self.string_bytes = 0;
                self.atom_bytes = 0;
            } else if b.is_ascii_digit() || matches!(b, b'-' | b'+' | b'.' | b'e' | b'E') {
                self.atom_bytes += 1;
                if self.atom_bytes > 20 {
                    return Err(std::io::Error::other("integer token limit"));
                }
            } else {
                self.atom_bytes = 0;
            }
        }
        Ok(n)
    }
}
pub fn report_from_reader<R: Read>(reader: R) -> Result<Report, Error> {
    // Reserve the complete registered typed storage before any report array allocation.
    let mut storage = Budget::new(&Limits::registered()?)?;
    for (resource, count, bytes) in [
        ("episode_rows", 4608, 29463),
        ("mean_rows", 2304, 522),
        ("request_contrast_rows", 4608, 523),
        ("contrast_rows", 2304, 522),
        ("envelope_rows", 576, 68),
    ] {
        for _ in 0..count {
            storage.row(resource, bytes)?;
        }
    }

    let guarded = Guard {
        reader,
        bytes: 0,
        quoted: false,
        escape: false,
        string_bytes: 0,
        atom_bytes: 0,
    };
    let mut decoder = serde_json::Deserializer::from_reader(guarded);
    let report = Report::deserialize(&mut decoder).map_err(|e| {
        if e.is_io() {
            Error::Io(e.to_string())
        } else {
            Error::InvalidInput(e.to_string())
        }
    })?;
    decoder
        .end()
        .map_err(|e| Error::InvalidInput(e.to_string()))?;
    Ok(report)
}
fn array<W: Write, T: Serialize>(writer: &mut W, records: &[T], max: u64) -> Result<(), Error> {
    writer.write_all(b"[").map_err(io)?;
    for (i, record) in records.iter().enumerate() {
        if i != 0 {
            writer.write_all(b",").map_err(io)?;
        }
        canonical::write_record(writer, record, max)?;
    }
    writer.write_all(b"]").map_err(io)
}
fn report_payload<W: Write>(writer: &mut W, r: &Report, include_hash: bool) -> Result<(), Error> {
    macro_rules! field {
        ($name:literal,$v:expr) => {{
            writer
                .write_all(concat!("\"", $name, "\":").as_bytes())
                .map_err(io)?;
            canonical::write_record(writer, $v, 65536)?;
        }};
    }
    writer.write_all(b"{\"contrasts\":").map_err(io)?;
    array(writer, &r.contrasts, 1334)?;
    writer.write_all(b",").map_err(io)?;
    field!("counters", &r.counters);
    writer.write_all(b",\"envelopes\":").map_err(io)?;
    array(writer, &r.envelopes, 141)?;
    writer.write_all(b",").map_err(io)?;
    field!("limits", &r.limits);
    writer.write_all(b",\"means\":").map_err(io)?;
    array(writer, &r.means, 1325)?;
    if include_hash {
        writer.write_all(b",").map_err(io)?;
        field!("payload_sha256", &r.payload_sha256);
    }
    writer.write_all(b",").map_err(io)?;
    field!("protocol_id", &r.protocol_id);
    writer.write_all(b",").map_err(io)?;
    field!("protocol_version", &r.protocol_version);
    writer.write_all(b",").map_err(io)?;
    field!("reference", &r.reference);
    writer.write_all(b",\"request_contrasts\":").map_err(io)?;
    array(writer, &r.request_contrasts, 1342)?;
    writer.write_all(b",\"rows\":").map_err(io)?;
    array(writer, &r.rows, 98766)?;
    writer.write_all(b",").map_err(io)?;
    field!("source", &r.source);
    writer.write_all(b",").map_err(io)?;
    field!("version", &r.version);
    writer.write_all(b"}").map_err(io)
}
fn payload_hash(r: &Report) -> Result<String, Error> {
    let mut hash = sha256::Sha256::new();
    report_payload(&mut hash, r, false)?;
    Ok(hash.finish())
}
struct Cap<W> {
    writer: W,
    bytes: u64,
    limit: u64,
}
impl<W: Write> Write for Cap<W> {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if self
            .bytes
            .checked_add(b.len() as u64)
            .is_none_or(|n| n > self.limit)
        {
            return Err(std::io::Error::other("report_bytes limit"));
        }
        let n = self.writer.write(b)?;
        self.bytes += n as u64;
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
/// Serialize and flush an already validated report, enforcing header and byte caps.
pub fn write_report<W: Write>(writer: &mut W, report: &Report) -> Result<u64, Error> {
    preflight(report)?;
    let mut cap = Cap {
        writer,
        bytes: 0,
        limit: report.limits.report_bytes,
    };
    report_payload(&mut cap, report, true)?;
    cap.flush().map_err(io)?;
    Ok(cap.bytes)
}
pub(crate) fn from_evaluation(e: super::Evaluation) -> Result<Report, Error> {
    let mut report = Report {
        version: REPORT_VERSION.into(),
        protocol_version: PROTOCOL_VERSION,
        protocol_id: PROTOCOL_ID.into(),
        source: source_inputs::source()?,
        reference: source_inputs::reference()?,
        limits: Limits::registered()?,
        rows: e.rows,
        means: e.means,
        request_contrasts: e.request_contrasts,
        contrasts: e.contrasts,
        envelopes: e.envelopes,
        counters: e.counters,
        payload_sha256: String::new(),
    };
    preflight(&report)?;
    report.payload_sha256 = payload_hash(&report)?;
    Ok(report)
}
pub fn diagnose(limits: &Limits) -> Result<Report, Error> {
    if *limits != Limits::registered()? {
        return Err(invalid("unregistered diagnostic limits"));
    }
    let report = from_evaluation(super::evaluate(limits)?)?;
    validate_report(&report, limits)?;
    Ok(report)
}
fn replay(episode: &Episode, budget: &mut Budget) -> Result<(), Error> {
    episode
        .config
        .validate()
        .map_err(|_| invalid("saved config"))?;
    if episode.config.key() != episode.key
        || episode.config.ids.writer != "Agent-A"
        || episode.config.ids.reader != "Agent-B"
    {
        return Err(invalid("registered census key/IDs"));
    }
    if episode.checkpoints.is_empty() || episode.checkpoints.len() > 16 {
        return Err(invalid("checkpoint count"));
    }
    budget.row("episode_rows", 29463)?;
    budget.charge("checkpoints", 1)?;
    budget.charge("map_cell_visits", 2)?;
    let mut world = World::new(&episode.config)?;
    if world.checkpoint() != &episode.checkpoints[0] {
        return Err(invalid("initial private state/view"));
    }
    for (index, saved) in episode.checkpoints.iter().enumerate().skip(1) {
        let event = saved
            .event
            .as_ref()
            .ok_or_else(|| invalid("missing saved event"))?;
        let mut compare = |actual: &Checkpoint| {
            if actual == saved {
                Ok(())
            } else {
                Err(invalid("replayed state/view/event"))
            }
        };
        match event.kind {
            EventKind::Action => {
                let actor = event.actor.ok_or_else(|| invalid("missing saved actor"))?;
                let primitive = event
                    .feedback
                    .as_ref()
                    .ok_or_else(|| invalid("missing saved feedback"))?
                    .primitive
                    .clone();
                budget.capture(
                    world.checkpoint(),
                    Some(&primitive),
                    Some(actor),
                    true,
                    index as u64,
                )?;
                world.act_with_capture(actor, primitive, &mut compare)?;
            }
            EventKind::Departure => {
                budget.capture(world.checkpoint(), None, None, true, index as u64)?;
                world.depart_with_capture(&mut compare)?;
            }
            EventKind::Settlement => {
                budget.capture(world.checkpoint(), None, None, true, index as u64)?;
                world.finish_with_capture(&mut compare)?;
            }
        }
    }
    if world.checkpoint().clock.phase != Phase::Terminal
        || world.outcome() != Ok(episode.outcome.clone())
    {
        return Err(invalid("saved outcome/terminal marker"));
    }
    budget.encoded(episode, budget.counters.episode_rows > 1)
}
fn reductions(report: &Report, budget: &mut Budget) -> Result<(), Error> {
    let arms = [
        Arm::Retained,
        Arm::Erased,
        Arm::NoNote,
        Arm::NoAcquireNoNote,
    ];
    let routines = [
        ReaderRoutine::BlindGuess,
        ReaderRoutine::InspectFirst,
        ReaderRoutine::ReadAdaptive,
        ReaderRoutine::Oracle,
    ];
    let labels = [
        (0, 1, Contrast::RetainedErased),
        (0, 2, Contrast::RetainedNoNote),
        (1, 2, Contrast::ErasedNoNote),
        (0, 3, Contrast::RetainedNoAcquireNoNote),
    ];
    let (mut m, mut q, mut c, mut e) = (0, 0, 0, 0);
    for (stratum_index, stratum) in super::registered_strata().iter().enumerate() {
        let (s, w, r, i, b) = (
            stratum.acquisition_cost,
            stratum.write_cost,
            stratum.note_read_cost,
            stratum.manifest_inspection_cost,
            stratum.writer_share,
        );
        let start = stratum_index * 32;
        let mean_start = m;
        for (arm_index, arm) in arms.iter().enumerate() {
            for (routine_index, routine) in routines.iter().enumerate() {
                let n = start + arm_index * 8 + routine_index * 2;
                for (j, request) in [Good::Left, Good::Right].iter().enumerate() {
                    if report.rows[n + j].key != (s, w, r, i, b, *arm, *routine, *request) {
                        return Err(invalid("row order/key"));
                    }
                }
                budget.row("mean_rows", 522)?;
                budget.charge("metric_atom_operations", 60)?;
                let mean = MeanRow {
                    key: (s, w, r, i, b, *arm, *routine),
                    metrics: combine(&report.rows[n].outcome, &report.rows[n + 1].outcome, true)?,
                };
                if mean != report.means[m] {
                    return Err(invalid("saved mean"));
                }
                budget.encoded(&mean, m != 0)?;
                m += 1;
            }
        }
        for (routine_index, routine) in routines.iter().enumerate() {
            for (request_index, request) in [Good::Left, Good::Right].iter().enumerate() {
                for (left, right, label) in labels {
                    budget.row("request_contrast_rows", 523)?;
                    budget.charge("metric_atom_operations", 60)?;
                    let outcome = |arm: usize| {
                        &report.rows[start + arm * 8 + routine_index * 2 + request_index].outcome
                    };
                    let contrast = RequestContrast {
                        key: (s, w, r, i, b, *routine, *request, label),
                        metrics: combine(outcome(left), outcome(right), false)?,
                    };
                    if contrast != report.request_contrasts[q] {
                        return Err(invalid("saved request contrast"));
                    }
                    budget.encoded(&contrast, q != 0)?;
                    q += 1;
                }
            }
            for (left, right, label) in labels {
                budget.row("contrast_rows", 522)?;
                budget.charge("metric_atom_operations", 60)?;
                let metrics =
                    |arm: usize| &report.means[mean_start + arm * 4 + routine_index].metrics;
                let contrast = MeanContrast {
                    key: (s, w, r, i, b, *routine, label),
                    metrics: difference(metrics(left), metrics(right))?,
                };
                if contrast != report.contrasts[c] {
                    return Err(invalid("saved mean contrast"));
                }
                budget.encoded(&contrast, c != 0)?;
                c += 1;
            }
        }
        for (arm_index, arm) in arms.iter().enumerate() {
            budget.row("envelope_rows", 68)?;
            budget.charge("metric_atom_operations", 5)?;
            let candidates =
                &report.means[mean_start + arm_index * 4..mean_start + arm_index * 4 + 3];
            let mut best = &candidates[0].metrics.net[1];
            let mut ties = vec![routines[0]];
            for k in 1..3 {
                match compare(&candidates[k].metrics.net[1], best)? {
                    std::cmp::Ordering::Greater => {
                        best = &candidates[k].metrics.net[1];
                        ties.clear();
                        ties.push(routines[k]);
                    }
                    std::cmp::Ordering::Equal => ties.push(routines[k]),
                    std::cmp::Ordering::Less => {}
                }
            }
            let envelope = Envelope {
                key: (s, w, r, i, b, *arm),
                reader_net: best.clone(),
                ties,
            };
            if envelope != report.envelopes[e] {
                return Err(invalid("saved envelope/ties"));
            }
            budget.encoded(&envelope, e != 0)?;
            e += 1;
        }
    }
    Ok(())
}
pub fn validate_report(report: &Report, limits: &Limits) -> Result<(), Error> {
    if report.version != REPORT_VERSION
        || report.protocol_version != PROTOCOL_VERSION
        || report.protocol_id != PROTOCOL_ID
    {
        return Err(invalid("report version/protocol"));
    }
    if *limits != Limits::registered()? || report.limits != *limits {
        return Err(invalid("registered limits"));
    }
    if report.source != source_inputs::source()? || report.reference != source_inputs::reference()?
    {
        return Err(invalid("source/reference identity/receipts"));
    }
    if (
        report.rows.len(),
        report.means.len(),
        report.request_contrasts.len(),
        report.contrasts.len(),
        report.envelopes.len(),
    ) != (4608, 2304, 4608, 2304, 576)
    {
        return Err(invalid("complete ordered arrays"));
    }
    preflight(report)?;
    let strata = super::registered_strata();
    let arms = [
        Arm::Retained,
        Arm::Erased,
        Arm::NoNote,
        Arm::NoAcquireNoNote,
    ];
    let routines = [
        ReaderRoutine::BlindGuess,
        ReaderRoutine::InspectFirst,
        ReaderRoutine::ReadAdaptive,
        ReaderRoutine::Oracle,
    ];
    for (index, episode) in report.rows.iter().enumerate() {
        let s = &strata[index / 32];
        let expected = (
            s.acquisition_cost,
            s.write_cost,
            s.note_read_cost,
            s.manifest_inspection_cost,
            s.writer_share,
            arms[(index % 32) / 8],
            routines[(index % 8) / 2],
            [Good::Left, Good::Right][index % 2],
        );
        if episode.key != expected {
            return Err(invalid("registered row order/key"));
        }
    }
    let mut budget = Budget::new(limits)?;
    for episode in &report.rows {
        replay(episode, &mut budget).map_err(|error| match error {
            Error::ResourceLimit { .. } | Error::ArithmeticOverflow | Error::Io(_) => error,
            _ => invalid("saved transition/view/outcome"),
        })?;
    }
    reductions(report, &mut budget)?;
    if report.counters != budget.counters || report.counters != references::counters()? {
        return Err(invalid("saved/recomputed/sealed counters"));
    }
    references::compare(report)?;
    if report.payload_sha256 != payload_hash(report)? {
        return Err(invalid("payload digest"));
    }
    Ok(())
}
pub fn summary_json(report: &Report) -> Result<String, Error> {
    validate_report(report, &Limits::registered()?)?;
    // The summary is bounded to its registered aggregate maxima, never the Episode arrays.
    let summary = Summary {
        version: SUMMARY_VERSION.into(),
        protocol_version: PROTOCOL_VERSION,
        protocol_id: PROTOCOL_ID.into(),
        source: report.source.clone(),
        reference: report.reference.clone(),
        report_sha256: full_report_hash(report)?,
        means: report.means.clone(),
        request_contrasts: report.request_contrasts.clone(),
        contrasts: report.contrasts.clone(),
        envelopes: report.envelopes.clone(),
        counters: report.counters.clone(),
    };
    let summary_cap = report.limits.header_bytes
        + 2304 * (1325 + 1)
        + 4608 * (1342 + 1)
        + 2304 * (1334 + 1)
        + 576 * (141 + 1);
    let length = encoded_len(&summary, summary_cap)?;
    let mut bytes =
        Vec::with_capacity(usize::try_from(length).map_err(|_| Error::ArithmeticOverflow)?);
    // Write aggregate records separately: no whole-summary Value tree.
    bytes.extend_from_slice(b"{\"contrasts\":");
    array(&mut bytes, &summary.contrasts, 1334)?;
    bytes.extend_from_slice(b",\"counters\":");
    canonical::write_record(&mut bytes, &summary.counters, 65536)?;
    bytes.extend_from_slice(b",\"envelopes\":");
    array(&mut bytes, &summary.envelopes, 141)?;
    bytes.extend_from_slice(b",\"means\":");
    array(&mut bytes, &summary.means, 1325)?;
    bytes.extend_from_slice(b",\"protocol_id\":");
    canonical::write_record(&mut bytes, &summary.protocol_id, 128)?;
    bytes.extend_from_slice(b",\"protocol_version\":1,\"reference\":");
    canonical::write_record(&mut bytes, &summary.reference, 65536)?;
    bytes.extend_from_slice(b",\"report_sha256\":");
    canonical::write_record(&mut bytes, &summary.report_sha256, 128)?;
    bytes.extend_from_slice(b",\"request_contrasts\":");
    array(&mut bytes, &summary.request_contrasts, 1342)?;
    bytes.extend_from_slice(b",\"source\":");
    canonical::write_record(&mut bytes, &summary.source, 65536)?;
    bytes.extend_from_slice(b",\"version\":\"persistent-notes-summary-v1\"}");
    String::from_utf8(bytes).map_err(|e| invalid(&e.to_string()))
}

fn preflight(report: &Report) -> Result<(), Error> {
    #[derive(Serialize)]
    struct Header<'a> {
        version: &'a str,
        protocol_version: u64,
        protocol_id: &'a str,
        source: &'a SourceIdentity,
        reference: &'a SourceIdentity,
        limits: &'a Limits,
        counters: &'a Counters,
        payload_sha256: &'a str,
    }
    let header = Header {
        version: &report.version,
        protocol_version: report.protocol_version,
        protocol_id: &report.protocol_id,
        source: &report.source,
        reference: &report.reference,
        limits: &report.limits,
        counters: &report.counters,
        payload_sha256: &report.payload_sha256,
    };
    // Five array names, separators and delimiters fit in the separate 128-byte allowance.
    encoded_len(&header, report.limits.header_bytes.saturating_sub(128)).map_err(|e| match e {
        Error::ResourceLimit { .. } => Error::ResourceLimit {
            resource: "header_bytes".into(),
            limit: report.limits.header_bytes,
        },
        other => other,
    })?;
    encoded_len(report, report.limits.report_bytes).map_err(|e| match e {
        Error::ResourceLimit { .. } => Error::ResourceLimit {
            resource: "report_bytes".into(),
            limit: report.limits.report_bytes,
        },
        other => other,
    })?;
    Ok(())
}
pub fn write_episode<W: Write>(writer: &mut W, episode: &Episode) -> Result<(), Error> {
    canonical::write_record_flush(writer, episode, 98766)
}

fn full_report_hash(report: &Report) -> Result<String, Error> {
    let mut hash = sha256::Sha256::new();
    report_payload(&mut hash, report, true)?;
    Ok(hash.finish())
}
