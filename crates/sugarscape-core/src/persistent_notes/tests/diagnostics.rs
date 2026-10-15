use serde::{
    de::{DeserializeSeed, Error as _, MapAccess, Visitor},
    Deserializer,
};
struct CompareSummary<'a>(&'a Report);
impl<'de> DeserializeSeed<'de> for CompareSummary<'_> {
    type Value = String;
    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<String, D::Error> {
        d.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for CompareSummary<'_> {
    type Value = String;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("complete exact summary projection")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<String, A::Error> {
        let mut seen = std::collections::BTreeSet::new();
        let mut hash = None;
        while let Some(key) = a.next_key::<String>()? {
            if !seen.insert(key.clone()) {
                return Err(A::Error::custom("duplicate summary field"));
            }
            match key.as_str() {
                "means" => a.next_value_seed(references::Compare(&self.0.means, 1325))?,
                "request_contrasts" => {
                    a.next_value_seed(references::Compare(&self.0.request_contrasts, 1342))?
                }
                "contrasts" => a.next_value_seed(references::Compare(&self.0.contrasts, 1334))?,
                "envelopes" => a.next_value_seed(references::Compare(&self.0.envelopes, 141))?,
                "source" => assert_eq!(a.next_value::<SourceIdentity>()?, self.0.source),
                "reference" => assert_eq!(a.next_value::<SourceIdentity>()?, self.0.reference),
                "counters" => assert_eq!(a.next_value::<Counters>()?, self.0.counters),
                "version" => assert_eq!(a.next_value::<String>()?, "persistent-notes-summary-v1"),
                "protocol_id" => assert_eq!(a.next_value::<String>()?, "persistent-notes-v1"),
                "protocol_version" => assert_eq!(a.next_value::<u64>()?, 1),
                "report_sha256" => hash = Some(a.next_value::<String>()?),
                _ => return Err(A::Error::custom("unknown summary field")),
            }
        }
        if seen.len() != 11 {
            return Err(A::Error::custom("missing summary field"));
        }
        hash.ok_or_else(|| A::Error::missing_field("report_sha256"))
    }
}
use super::super::*;
#[test]
fn identities_bind_distinct_domains_and_full_reference_bytes() {
    let source = source_identity().unwrap();
    let reference = reference_identity().unwrap();
    assert_eq!(source.len(), 64);
    assert_ne!(source, reference);
}
#[test]
fn missing_required_null_and_nested_duplicates_fail() {
    let config = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Right);
    let episode = run_episode(&config).unwrap();
    let mut value = serde_json::to_value(&episode).unwrap();
    value["checkpoints"][0]["own"][1]
        .as_object_mut()
        .unwrap()
        .remove("known_request");
    assert!(
        diagnostics::decode_record::<Episode>(&serde_json::to_vec(&value).unwrap(), 98766).is_err()
    );
    assert!(diagnostics::decode_record::<Episode>(b"{\"key\":[],\"key\":[]}", 98766).is_err());
}
#[test]
fn oversized_record_is_typed_resource_failure() {
    let error = diagnostics::decode_record::<Episode>(&vec![b' '; 98767], 98766).unwrap_err();
    assert_eq!(
        error,
        Error::ResourceLimit {
            resource: "record_bytes".into(),
            limit: 98766
        }
    );
}
/// Engineering fixture reconstruction; never a collected production Report.
fn report_from_sealed_references() -> Result<Report, Error> {
    diagnostics::from_evaluation(references::engineering_evaluation()?)
}
#[test]
fn complete_engineering_report_saved_replay_summary_and_tamper_checks() -> Result<(), Error> {
    let assembly = std::time::Instant::now();
    let mut report = report_from_sealed_references()?;
    eprintln!(
        "engineering sealed-reference report assembly_ms={}",
        assembly.elapsed().as_millis()
    );
    let limits = Limits::registered()?;
    let validation = std::time::Instant::now();
    validate_report(&report, &limits)?;
    eprintln!(
        "engineering saved validation_ms={}",
        validation.elapsed().as_millis()
    );
    let original = report.rows[0].outcome.clone();
    report.rows[0].outcome.spent[0] += 1;
    assert!(validate_report(&report, &limits).is_err());
    report.rows[0].outcome = original.clone();
    report.rows[0].outcome.net[0] = -99;
    assert!(validate_report(&report, &limits).is_err());
    report.rows[0].outcome = original.clone();
    report.rows[0].outcome.correct = !original.correct;
    assert!(validate_report(&report, &limits).is_err());
    report.rows[0].outcome = original;
    report.rows.swap(0, 1);
    assert!(validate_report(&report, &limits).is_err());
    report.rows.swap(0, 1);
    let row = report.rows.remove(0);
    assert!(validate_report(&report, &limits).is_err());
    report.rows.insert(0, row);
    let belief = report.rows[0].checkpoints[0].own[1].belief.clone();
    report.rows[0].checkpoints[0].own[1].belief = [Ratio::ONE, Ratio::ZERO];
    assert!(validate_report(&report, &limits).is_err());
    report.rows[0].checkpoints[0].own[1].belief = belief;
    report.rows[0].checkpoints[0].own[1].known_request = Some(Good::Right);
    assert!(validate_report(&report, &limits).is_err());
    report.rows[0].checkpoints[0].own[1].known_request = None;
    let receipt = report.reference.receipts[0].sha256.clone();
    report.reference.receipts[0].sha256 = "0".repeat(64);
    assert!(validate_report(&report, &limits).is_err());
    report.reference.receipts[0].sha256 = receipt;
    report.limits.episode_rows += 1;
    assert!(validate_report(&report, &limits).is_err());
    report.limits.episode_rows -= 1;
    report.means[0].metrics.correctness.denominator += 1;
    assert!(validate_report(&report, &limits).is_err());
    report.means[0].metrics.correctness.denominator -= 1;
    report.counters.events += 1;
    assert!(validate_report(&report, &limits).is_err());
    report.counters.events -= 1;
    let hash = report.payload_sha256.clone();
    report.payload_sha256 = "0".repeat(64);
    assert!(validate_report(&report, &limits).is_err());
    report.payload_sha256 = hash;
    let summary = summary_json(&report)?;
    let mut decoder = serde_json::Deserializer::from_str(&summary);
    let full_digest = CompareSummary(&report).deserialize(&mut decoder).unwrap();
    decoder.end().unwrap();
    let mut full_hash = sha256::Sha256::new();
    write_report(&mut full_hash, &report)?;
    assert_eq!(full_digest, full_hash.finish());
    assert_ne!(full_digest, report.payload_sha256);
    drop(summary);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../.superpowers/sdd/2026-10-14-paid-persistent-notes/task-4-engineering-report.json",
    );
    let mut output =
        std::io::BufWriter::with_capacity(65536, std::fs::File::create(&path).unwrap());
    let count = write_report(&mut output, &report)?;
    eprintln!("engineering canonical_report_bytes={count}");
    assert!(count < limits.report_bytes);
    drop(output);
    drop(report);
    let reader = std::io::BufReader::with_capacity(65536, std::fs::File::open(&path).unwrap());
    let report = report_from_reader(reader)?;
    validate_report(&report, &limits)?;
    let mut hash1 = sha256::Sha256::new();
    let mut hash2 = sha256::Sha256::new();
    assert_eq!(
        write_report(&mut hash1, &report)?,
        write_report(&mut hash2, &report)?
    );
    assert_eq!(hash1.finish(), hash2.finish());
    Ok(())
}
#[test]
fn hostile_record_numeric_strings_and_arrays() {
    for raw in [b"-0".as_slice(), b"1.0", b"1e0", b"18446744073709551616"] {
        assert!(diagnostics::decode_record::<u64>(raw, 128).is_err());
    }
    let raw = format!("\"{}\"", "x".repeat(193));
    assert!(diagnostics::decode_record::<String>(raw.as_bytes(), 4096).is_err());
    let raw = b"{\"numerator\":2,\"denominator\":2}";
    assert!(diagnostics::decode_record::<Ratio>(raw, 128).is_err());
    let raw = b"{\"numerator\":0,\"denominator\":2}";
    assert!(diagnostics::decode_record::<Ratio>(raw, 128).is_err());
    let raw = b"{\"numerator\":1,\"denominator\":0}";
    assert!(diagnostics::decode_record::<Ratio>(raw, 128).is_err());
}
#[test]
fn every_nullable_field_remains_required_on_the_wire() {
    let config = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Right);
    let value = serde_json::to_value(run_episode(&config).unwrap()).unwrap();
    fn nulls(v: &serde_json::Value, path: String, out: &mut Vec<String>) {
        match v {
            serde_json::Value::Object(fields) => {
                for (k, v) in fields {
                    let next = format!("{path}/{k}");
                    if v.is_null() {
                        out.push(next);
                    } else {
                        nulls(v, next, out);
                    }
                }
            }
            serde_json::Value::Array(items) => {
                for (i, v) in items.iter().enumerate() {
                    nulls(v, format!("{path}/{i}"), out);
                }
            }
            _ => {}
        }
    }
    let mut paths = Vec::new();
    nulls(&value, String::new(), &mut paths);
    assert!(!paths.is_empty());
    for path in paths {
        let mut altered = value.clone();
        let (parent, name) = path.rsplit_once('/').unwrap();
        altered
            .pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(name);
        assert!(
            diagnostics::decode_record::<Episode>(&serde_json::to_vec(&altered).unwrap(), 98766)
                .is_err(),
            "omitted {path}"
        );
    }
}
#[test]
fn limits_and_output_preflight_fail_with_typed_context() {
    let mut report = Report {
        version: "persistent-notes-diagnostic-v1".into(),
        protocol_version: 1,
        protocol_id: "persistent-notes-v1".into(),
        source: SourceIdentity {
            domain: "persistent-notes-source-v1".into(),
            receipts: vec![],
            sha256: "0".repeat(64),
        },
        reference: SourceIdentity {
            domain: "persistent-notes-reference-v1".into(),
            receipts: vec![],
            sha256: "0".repeat(64),
        },
        limits: Limits::default(),
        rows: vec![],
        means: vec![],
        request_contrasts: vec![],
        contrasts: vec![],
        envelopes: vec![],
        counters: Counters::default(),
        payload_sha256: "0".repeat(64),
    };
    report.limits.report_bytes = 1;
    assert_eq!(
        write_report(&mut Vec::new(), &report).unwrap_err(),
        Error::ResourceLimit {
            resource: "report_bytes".into(),
            limit: 1
        }
    );
    report.limits = Limits::default();
    report.limits.header_bytes = 1;
    assert_eq!(
        write_report(&mut Vec::new(), &report).unwrap_err(),
        Error::ResourceLimit {
            resource: "header_bytes".into(),
            limit: 1
        }
    );
}
#[test]
fn role_history_and_map_bounds_reject_before_native_replay() {
    let config = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Right);
    let value = serde_json::to_value(run_episode(&config).unwrap()).unwrap();
    for (role, cap) in [(0, 4), (1, 6)] {
        let mut altered = value.clone();
        let feedback = value["checkpoints"][1]["own"][0]["history"][0].clone();
        altered["checkpoints"][0]["own"][role]["history"] =
            serde_json::Value::Array(vec![feedback; cap + 1]);
        assert!(diagnostics::decode_record::<Episode>(
            &serde_json::to_vec(&altered).unwrap(),
            98766
        )
        .is_err());
    }
    let mut altered = value.clone();
    let cell = value["checkpoints"][0]["own"][0]["map"]["cells"][0].clone();
    altered["checkpoints"][0]["own"][0]["map"]["cells"] = serde_json::Value::Array(vec![cell; 4]);
    assert!(
        diagnostics::decode_record::<Episode>(&serde_json::to_vec(&altered).unwrap(), 98766)
            .is_err()
    );
}
