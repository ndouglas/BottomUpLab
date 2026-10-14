//! Exact finite engineering records. References are immutable, independently sealed inputs.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{cmp::Ordering, io::Write, path::Path};
use sugarscape_core::negotiable::{
    cake_payoff, cake_rows, classify, history_weights, CakeRow, Classification, HistoryResult,
    Observation, ParetoKind, Policy, Ratio, WireRatio,
};

use super::f5_source_identity as source_identity;
include!(concat!(env!("OUT_DIR"), "/f5_source_identity.rs"));

pub(super) const CAP: usize = 8 * 1024 * 1024;
const SCHEMA: &str = "negotiable-foundation-v1";
const MAIN_PDF: &str = "8eaba6509a3fc39a08905ccadac6ad2cd69c308d9b8cb404248fb6a944a60492";
const SUPPLEMENT_PDF: &str = "643655853ddf456513aa430094449db81bb939447ac1ad63bf5b864826b5c140";
pub(super) const REFERENCES: [&[u8]; 3] = [
    include_bytes!(
        "../../../crates/sugarscape-core/src/negotiable/tests/fixtures/cake-reference.json"
    ),
    include_bytes!(
        "../../../crates/sugarscape-core/src/negotiable/tests/fixtures/pareto-reference.json"
    ),
    include_bytes!(
        "../../../crates/sugarscape-core/src/negotiable/tests/fixtures/priority-reference.json"
    ),
];
const REFERENCE_HASHES: [&str; 3] = [
    "42e738fe443809b7598e61c3d80879ab975147e435fd32c3c7e1d0703573da76",
    "fafca8c4c1c19009aa3ec78b171e39cc3e6a754aba265d387c39e564125e73ae",
    "0e2e19f4d7847a6277d99394b27d3c142fc1fc82d5367e54160227117c1d4b41",
];
const SETTINGS_HASHES: [&str; 3] = [
    "accf03a47855ec65c7bad5d36ce902483fd0923d5f2f9478783595430874b59b",
    "351d52110096e72095d63df66e101d3840f6677788528dd5be63ebd15e8a1158",
    "60a734b6b532e37a20351cc7b8cd5668d1319aaa00eb7c0481831de4687e5956",
];

#[derive(Clone, Copy, Debug)]
pub(super) enum RecordKind {
    Cake,
    Pareto,
    Priority,
}
impl RecordKind {
    pub(super) fn index(self) -> usize {
        match self {
            Self::Cake => 0,
            Self::Pareto => 1,
            Self::Priority => 2,
        }
    }
    pub(super) fn count(self) -> usize {
        [225, 9, 4599][self.index()]
    }
    fn name(self) -> &'static str {
        ["cake", "pareto", "priority"][self.index()]
    }
}

/// Mathematical disagreement is distinct from inability to perform the check.
#[derive(Debug)]
pub(super) enum Failure {
    Mathematics(String),
    Operational(String),
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mathematics(message) | Self::Operational(message) => f.write_str(message),
        }
    }
}
impl From<sugarscape_core::negotiable::Error> for Failure {
    fn from(error: sugarscape_core::negotiable::Error) -> Self {
        Self::Operational(format!("negotiable exact arithmetic/input: {error}"))
    }
}
impl From<serde_json::Error> for Failure {
    fn from(error: serde_json::Error) -> Self {
        Self::Operational(format!("negotiable JSON encoding/reference: {error}"))
    }
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) struct BoundedRecordWriter {
    bytes: Vec<u8>,
    limit: usize,
    failed: bool,
}
impl BoundedRecordWriter {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
            failed: false,
        }
    }
    pub(super) fn finish(self) -> Result<Vec<u8>, String> {
        if self.failed {
            Err("negotiable record serialization failed; partial bytes unavailable".into())
        } else {
            Ok(self.bytes)
        }
    }
}
impl Write for BoundedRecordWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.failed || bytes.len() > self.limit.saturating_sub(self.bytes.len()) {
            self.failed = true;
            return Err(std::io::Error::other(
                "negotiable record exceeds byte cap; partial output refused",
            ));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        if self.failed {
            Err(std::io::Error::other(
                "negotiable record writer already failed",
            ))
        } else {
            Ok(())
        }
    }
}

pub(super) fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("survey repository parent")
}
pub(super) fn source_entries(root: &Path) -> Result<Vec<(String, Vec<u8>)>, Failure> {
    let mut entries = Vec::new();
    for relative in source_identity::DIRECTORIES
        .iter()
        .chain(source_identity::FILES)
    {
        source_identity::read_inputs(root, Path::new(relative), &mut entries)
            .map_err(Failure::Operational)?;
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries.dedup_by(|a, b| a.0 == b.0);
    for kind in [RecordKind::Cake, RecordKind::Pareto, RecordKind::Priority] {
        let path = format!(
            "crates/sugarscape-core/src/negotiable/tests/fixtures/{}-reference.json",
            kind.name()
        );
        if !entries.iter().any(|entry| entry.0 == path) {
            return Err(Failure::Operational(format!(
                "missing required negotiable reference input {path}"
            )));
        }
    }
    Ok(entries)
}
pub(super) fn source_hash(root: &Path) -> Result<String, Failure> {
    source_identity::fingerprint(source_entries(root)?.into_iter().map(Ok))
        .map_err(Failure::Operational)
}
fn verify_binding(root: &Path, expected: &str) -> Result<(), Failure> {
    let actual = source_hash(root)?;
    if actual != expected {
        return Err(Failure::Operational(format!(
            "negotiable stale source binding at {}: compiled {expected}, current {actual}",
            root.display()
        )));
    }
    Ok(())
}
/// Recheck after construction as well: a changed source must revoke successful bytes.
pub(super) fn source_bound(
    root: &Path,
    expected: &str,
    build: impl FnOnce() -> Result<Vec<u8>, Failure>,
) -> Result<Vec<u8>, Failure> {
    verify_binding(root, expected)?;
    let bytes = build()?;
    verify_binding(root, expected)?;
    Ok(bytes)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Reference {
    schema: String,
    pub(super) settings: Value,
    pub(super) rows: Vec<Value>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParetoRow {
    policy_id: u8,
    payoff: [WireRatio; 2],
    paper_weak: Classification,
    strong: Classification,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
enum PriorityResult {
    Supported { weights: [WireRatio; 2] },
    Impossible { first_impossible_prefix: usize },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PriorityRow {
    model_id: u8,
    prior_index: u8,
    history: Vec<u8>,
    result: PriorityResult,
}

fn invalid(message: impl Into<String>) -> Failure {
    Failure::Operational(format!(
        "negotiable malformed reference: {}",
        message.into()
    ))
}
fn required_keys(value: &Value, keys: &[&str]) -> Result<(), Failure> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid("expected object"))?;
    if object.len() != keys.len() || !keys.iter().all(|key| object.contains_key(*key)) {
        return Err(invalid(format!(
            "expected keys {keys:?}, found {:?}",
            object.keys().collect::<Vec<_>>()
        )));
    }
    Ok(())
}
fn canonical_fractions(value: &Value) -> Result<(), Failure> {
    match value {
        Value::Object(object) => {
            if object.contains_key("num") || object.contains_key("den") {
                required_keys(value, &["num", "den"])?;
                let wire: WireRatio = serde_json::from_value(value.clone())?;
                Ratio::from_wire(wire).map_err(|error| invalid(format!("fraction: {error}")))?;
            } else {
                for child in object.values() {
                    canonical_fractions(child)?;
                }
            }
        }
        Value::Array(array) => {
            for child in array {
                canonical_fractions(child)?;
            }
        }
        _ => {}
    }
    Ok(())
}

/// Internal supplied-byte boundary for refusal tests; production always uses the sealed hashes.
pub(super) fn validate_reference(
    kind: RecordKind,
    bytes: &[u8],
    expected_hash: &str,
) -> Result<Reference, Failure> {
    if bytes.len() > CAP || sha256(bytes) != expected_hash {
        return Err(invalid(format!(
            "{} reference byte cap or sealed SHA-256 mismatch",
            kind.name()
        )));
    }
    let value: Value = serde_json::from_slice(bytes)?;
    // Canonical round-trip also detects duplicate keys instead of silently discarding them.
    let mut canonical = serde_json::to_vec(&value)?;
    canonical.push(b'\n');
    if canonical != bytes {
        return Err(invalid("noncanonical or duplicate JSON keys"));
    }
    let reference: Reference = serde_json::from_value(value.clone())?;
    if reference.schema != SCHEMA {
        return Err(invalid("schema"));
    }
    let mut settings_bytes = serde_json::to_vec(&reference.settings)?;
    settings_bytes.push(b'\n');
    if sha256(&settings_bytes) != SETTINGS_HASHES[kind.index()] {
        return Err(invalid(format!(
            "{} fixed settings SHA-256 mismatch",
            kind.name()
        )));
    }
    canonical_fractions(&value)?;
    validate_census(kind, &reference)?;
    Ok(reference)
}
fn histories() -> impl Iterator<Item = Vec<u8>> {
    (0..=8).flat_map(|length| {
        (0..(1usize << length))
            .map(move |bits| (0..length).rev().map(|i| ((bits >> i) & 1) as u8).collect())
    })
}
fn validate_census(kind: RecordKind, reference: &Reference) -> Result<(), Failure> {
    if reference.rows.len() != kind.count() {
        return Err(invalid(format!(
            "{} requires exactly {} rows",
            kind.name(),
            kind.count()
        )));
    }
    match kind {
        RecordKind::Cake => {
            let weights = reference.settings["weights"]
                .as_array()
                .ok_or_else(|| invalid("weights"))?;
            for (index, value) in reference.rows.iter().enumerate() {
                let row: CakeRow = serde_json::from_value(value.clone())?;
                if value["weight"] != weights[index / 9] || usize::from(row.policy_id) != index % 9
                {
                    return Err(invalid(format!(
                        "cake missing/duplicate/reordered cell {index}"
                    )));
                }
                for ids in [&row.global_maximizers, &row.local_maximizers] {
                    if ids.is_empty()
                        || ids.len() > 9
                        || ids.iter().any(|id| *id > 8)
                        || ids.windows(2).any(|w| w[0] >= w[1])
                    {
                        return Err(invalid(format!("cake maximizing IDs at row {index}")));
                    }
                }
            }
        }
        RecordKind::Pareto => {
            for (index, value) in reference.rows.iter().enumerate() {
                for key in ["paper_weak", "strong"] {
                    required_keys(&value[key], &["efficient", "witness"])?;
                }
                let row: ParetoRow = serde_json::from_value(value.clone())?;
                if usize::from(row.policy_id) != index {
                    return Err(invalid(format!(
                        "pareto missing/duplicate/reordered policy {index}"
                    )));
                }
                // Validate the reference against its own payoff geometry. A
                // production disagreement belongs to the mathematical check.
                let points = reference
                    .rows
                    .iter()
                    .map(|value| {
                        let row: ParetoRow = serde_json::from_value(value.clone())?;
                        parse_pair(row.payoff)
                    })
                    .collect::<Result<Vec<_>, Failure>>()?;
                for (classification, kind) in [
                    (&row.paper_weak, ParetoKind::PaperWeak),
                    (&row.strong, ParetoKind::Strong),
                ] {
                    verify_witness(classification, points[index], &points, kind).map_err(
                        |error| {
                            invalid(format!("pareto reference witness policy {index}: {error}"))
                        },
                    )?;
                }
            }
        }
        RecordKind::Priority => {
            let mut index = 0;
            for model_id in 0..3 {
                for prior_index in 0..3 {
                    for history in histories() {
                        let row: PriorityRow =
                            serde_json::from_value(reference.rows[index].clone())?;
                        if row.model_id != model_id
                            || row.prior_index != prior_index
                            || row.history != history
                        {
                            return Err(invalid(format!(
                                "priority missing/duplicate/reordered cell {index}"
                            )));
                        }
                        match row.result {
                            PriorityResult::Supported { weights } => {
                                let [alice, bob] = parse_pair(weights)?;
                                if alice.num() < 0
                                    || bob.num() < 0
                                    || alice.checked_add(bob)? != Ratio::new(1, 1)?
                                {
                                    return Err(invalid(format!(
                                        "priority normalized weights at {index}"
                                    )));
                                }
                            }
                            PriorityResult::Impossible {
                                first_impossible_prefix,
                            } => {
                                if first_impossible_prefix == 0
                                    || first_impossible_prefix > row.history.len()
                                {
                                    return Err(invalid(format!(
                                        "priority impossible prefix at {index}"
                                    )));
                                }
                            }
                        }
                        index += 1;
                    }
                }
            }
        }
    }
    Ok(())
}
fn parse_pair(pair: [WireRatio; 2]) -> Result<[Ratio; 2], Failure> {
    let [alice, bob] = pair;
    Ok([Ratio::from_wire(alice)?, Ratio::from_wire(bob)?])
}
fn cake_points() -> Result<Vec<[Ratio; 2]>, Failure> {
    (0..9)
        .map(|id| Ok(cake_payoff(Policy::from_id(id)?)?))
        .collect()
}
/// Reconstruct witnesses independently of the hull search, checking mass, payoff and inequalities.
fn verify_witness(
    c: &Classification,
    target: [Ratio; 2],
    points: &[[Ratio; 2]],
    kind: ParetoKind,
) -> Result<(), Failure> {
    let mismatch =
        |message: &str| Failure::Mathematics(format!("invalid dominance witness: {message}"));
    match (&c.witness, c.efficient) {
        (None, true) => return Ok(()),
        (Some(_), false) => {}
        _ => return Err(mismatch("efficiency/witness availability")),
    }
    let witness = c.witness.as_ref().expect("witness matched above");
    if witness.parts.is_empty()
        || witness.parts.len() > 3
        || witness
            .parts
            .windows(2)
            .any(|w| w[0].policy_id >= w[1].policy_id)
    {
        return Err(mismatch("parts count/order"));
    }
    let zero = Ratio::new(0, 1)?;
    let one = Ratio::new(1, 1)?;
    let mut mass = zero;
    let mut payoff = [zero; 2];
    for part in &witness.parts {
        let weight = Ratio::from_wire(part.weight.clone())?;
        if weight.checked_cmp(zero)? == Ordering::Less {
            return Err(mismatch("negative mass"));
        }
        let point = points
            .get(usize::from(part.policy_id))
            .ok_or_else(|| mismatch("unknown policy"))?;
        mass = mass.checked_add(weight)?;
        for principal in 0..2 {
            payoff[principal] =
                payoff[principal].checked_add(weight.checked_mul(point[principal])?)?;
        }
    }
    if mass != one || payoff != parse_pair(witness.payoff.clone())? {
        return Err(mismatch("mass or reconstructed payoff"));
    }
    let a = payoff[0].checked_cmp(target[0])?;
    let b = payoff[1].checked_cmp(target[1])?;
    let dominates = match kind {
        ParetoKind::PaperWeak => a == Ordering::Greater && b == Ordering::Greater,
        ParetoKind::Strong => {
            a != Ordering::Less
                && b != Ordering::Less
                && (a == Ordering::Greater || b == Ordering::Greater)
        }
    };
    if !dominates {
        return Err(mismatch("declared coordinate inequalities"));
    }
    Ok(())
}

pub(super) fn cake_failure(error: sugarscape_core::negotiable::Error) -> Failure {
    // cake_rows has fixed validated inputs; its sole InvalidFixture branch is
    // disagreement between whole-policy and observation-local maximizing sets.
    // The typed variant is enough here; no message-prefix classification.
    match error {
        sugarscape_core::negotiable::Error::InvalidFixture(message) => Failure::Mathematics(
            format!("cake whole/local mathematical disagreement: {message}"),
        ),
        error => Failure::from(error),
    }
}

fn production_rows(kind: RecordKind, reference: &Reference) -> Result<Vec<Value>, Failure> {
    let mismatch = |index| {
        Failure::Mathematics(format!(
            "{} finite reference disagreement at row {index}",
            kind.name()
        ))
    };
    let rows = match kind {
        RecordKind::Cake => cake_rows()
            .map_err(cake_failure)?
            .into_iter()
            .map(serde_json::to_value)
            .collect::<Result<Vec<_>, _>>()?,
        RecordKind::Pareto => {
            let points = cake_points()?;
            let mut rows = Vec::with_capacity(9);
            for index in 0..9 {
                let expected: ParetoRow = serde_json::from_value(reference.rows[index].clone())?;
                let paper_weak = classify(&points, index, ParetoKind::PaperWeak)?;
                let strong = classify(&points, index, ParetoKind::Strong)?;
                verify_witness(&paper_weak, points[index], &points, ParetoKind::PaperWeak)?;
                verify_witness(&strong, points[index], &points, ParetoKind::Strong)?;
                if expected.paper_weak.efficient != paper_weak.efficient
                    || expected.strong.efficient != strong.efficient
                    || parse_pair(expected.payoff)? != points[index]
                {
                    return Err(mismatch(index));
                }
                rows.push(serde_json::to_value(ParetoRow {
                    policy_id: index as u8,
                    payoff: points[index].map(Ratio::wire),
                    paper_weak,
                    strong,
                })?);
            }
            rows
        }
        RecordKind::Priority => {
            let models = [(9, 10, 1, 10), (1, 2, 1, 2), (1, 1, 4, 5)];
            let priors = [(0, 1), (1, 2), (1, 1)];
            let mut rows = Vec::with_capacity(4599);
            for (model_id, (a, ad, b, bd)) in models.into_iter().enumerate() {
                let red = [Ratio::new(a, ad)?, Ratio::new(b, bd)?];
                for (prior_index, (num, den)) in priors.into_iter().enumerate() {
                    let alice = Ratio::new(num, den)?;
                    let prior = [alice, Ratio::new(1, 1)?.checked_sub(alice)?];
                    for history in histories() {
                        let observations: Vec<_> = history
                            .iter()
                            .map(|o| {
                                if *o == 0 {
                                    Observation::Red
                                } else {
                                    Observation::Green
                                }
                            })
                            .collect();
                        let result = match history_weights(prior, red, &observations)? {
                            HistoryResult::Supported(weights) => PriorityResult::Supported {
                                weights: weights.map(Ratio::wire),
                            },
                            HistoryResult::Impossible {
                                first_impossible_prefix,
                            } => PriorityResult::Impossible {
                                first_impossible_prefix,
                            },
                        };
                        rows.push(serde_json::to_value(PriorityRow {
                            model_id: model_id as u8,
                            prior_index: prior_index as u8,
                            history,
                            result,
                        })?);
                    }
                }
            }
            rows
        }
    };
    compare_rows(kind, &rows, &reference.rows)?;
    Ok(rows)
}
/// Pareto witnesses may differ in valid decomposition; all other cells compare exactly.
pub(super) fn compare_rows(
    kind: RecordKind,
    actual: &[Value],
    expected: &[Value],
) -> Result<(), Failure> {
    if actual.len() != kind.count() || expected.len() != kind.count() {
        return Err(invalid("partial production/reference census"));
    }
    for (index, (a, b)) in actual.iter().zip(expected).enumerate() {
        let mut a = a.clone();
        let mut b = b.clone();
        if matches!(kind, RecordKind::Pareto) {
            for key in ["paper_weak", "strong"] {
                a[key]["witness"] = Value::Null;
                b[key]["witness"] = Value::Null;
            }
        }
        if a != b {
            return Err(Failure::Mathematics(format!(
                "{} finite reference disagreement at row {index}",
                kind.name()
            )));
        }
    }
    Ok(())
}
fn envelope(kind: RecordKind, settings: Value, source: &str, rows: Vec<Value>) -> Value {
    json!({"schema":SCHEMA,"scope":"finite_exact_foundation","binding":{
        "main_pdf_sha256":MAIN_PDF,"supplement_pdf_sha256":SUPPLEMENT_PDF,
        "reference_sha256":{"cake":REFERENCE_HASHES[0],"pareto":REFERENCE_HASHES[1],"priority":REFERENCE_HASHES[2]},
        "settings_sha256":SETTINGS_HASHES[kind.index()],"compiled_source_sha256":source
    },"settings":settings,"rows":rows})
}

/// Complete wire-size certificate: maximal i128 fraction strings, all keys,
/// punctuation, nine tied IDs, three witness parts, eight observations, envelope
/// and fixed settings. Each census is multiplied by its largest possible row.
pub(super) fn worst_case_bytes(kind: RecordKind, settings: &Value) -> Result<usize, Failure> {
    // Signed i128 numerator needs <=40 ASCII bytes; positive denominator <=39.
    let fraction = json!({"num":i128::MIN.to_string(),"den":i128::MAX.to_string()});
    let witness = json!({"parts":(0..3).map(|id|json!({"policy_id":id,"weight":fraction})).collect::<Vec<_>>(),"payoff":[fraction,fraction]});
    let classification = json!({"efficient":false,"witness":witness});
    let row = match kind {
        RecordKind::Cake => {
            json!({"weight":fraction,"policy_id":8,"expected":[fraction,fraction],"objective":fraction,"global_maximizers":(0..9).collect::<Vec<_>>(),"local_maximizers":(0..9).collect::<Vec<_>>()})
        }
        RecordKind::Pareto => {
            json!({"policy_id":8,"payoff":[fraction,fraction],"paper_weak":classification,"strong":classification})
        }
        RecordKind::Priority => {
            json!({"model_id":2,"prior_index":2,"history":[1,1,1,1,1,1,1,1],"result":{"status":"supported","weights":[fraction,fraction]}})
        }
    };
    let row_bytes = serde_json::to_vec(&row)?.len();
    let fixed =
        serde_json::to_vec(&envelope(kind, settings.clone(), &"f".repeat(64), vec![]))?.len();
    // Empty rows [] already accounts for brackets. Add N rows and N-1 commas,
    // and the canonical final newline. The impossible branch is smaller.
    Ok(fixed + kind.count() * row_bytes + kind.count() - 1 + 1)
}

pub(super) fn build_inner(
    kind: RecordKind,
    references: [&[u8]; 3],
    source: &str,
    limit: usize,
) -> Result<Vec<u8>, Failure> {
    if references.iter().map(|b| b.len()).sum::<usize>() > 16 * 1024 * 1024 {
        return Err(invalid("combined reference byte cap"));
    }
    let mut validated = Vec::with_capacity(3);
    for kind in [RecordKind::Cake, RecordKind::Pareto, RecordKind::Priority] {
        validated.push(validate_reference(
            kind,
            references[kind.index()],
            REFERENCE_HASHES[kind.index()],
        )?);
    }
    let reference = &validated[kind.index()];
    let proof = worst_case_bytes(kind, &reference.settings)?;
    if proof > CAP {
        return Err(Failure::Operational(format!(
            "{} complete wire bound {proof} exceeds fixed cap {CAP}",
            kind.name()
        )));
    }
    let rows = production_rows(kind, reference)?;
    if rows.len() != kind.count() {
        return Err(invalid("production row count before serialization"));
    }
    let record = envelope(kind, reference.settings.clone(), source, rows);
    let mut writer = BoundedRecordWriter::new(limit.min(CAP));
    serde_json::to_writer(&mut writer, &record)?;
    writer.write_all(b"\n").map_err(|error| {
        Failure::Operational(format!("negotiable final newline encoding: {error}"))
    })?;
    writer.finish().map_err(Failure::Operational)
}
pub(super) fn build_checked(kind: RecordKind) -> Result<Vec<u8>, Failure> {
    source_bound(root(), COMPILED_SOURCE_IDENTITY, || {
        build_inner(kind, REFERENCES, COMPILED_SOURCE_IDENTITY, CAP)
    })
}
#[cfg(test)]
pub(super) fn build_record(kind: RecordKind) -> Result<Vec<u8>, String> {
    build_checked(kind).map_err(|error| error.to_string())
}
