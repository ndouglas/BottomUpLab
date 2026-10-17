use crate::claim::Verdict;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn record(id: &str) -> Value {
    let c = super::all()
        .into_iter()
        .find(|c| c.id == id)
        .expect("registered negotiable claim");
    let o = (c.check)(&[]);
    assert_eq!(o.verdict, Verdict::Holds, "{}", o.detail);
    assert!(o.detail.contains("finite"));
    serde_json::from_str(&o.measured).unwrap()
}

#[test]
fn negotiable_registration_is_exact_and_unique() {
    let claims = super::all();
    let ids: Vec<_> = claims
        .iter()
        .filter(|c| c.item == "negotiable")
        .map(|c| c.id)
        .collect();
    assert_eq!(
        ids,
        vec![
            "negotiable.cake",
            "negotiable.pareto",
            "negotiable.priority"
        ]
    );
}

#[test]
fn negotiable_legacy_catalog_metadata_is_byte_identical() {
    let mut rows: Vec<_> = super::all().into_iter().filter(|c|!c.id.starts_with("negotiable.") && !c.id.starts_with("gridworlds.") && !c.id.starts_with("misaligned.")).map(|c| serde_json::json!({"id":c.id,"item":c.item,"source":c.source,"citation":c.citation,"text":c.text})).collect();
    rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    assert_eq!(rows.len(), 817);
    assert_eq!(
        format!("{:x}", Sha256::digest(serde_json::to_vec(&rows).unwrap())),
        "597de12a85e03be52672bc6d28e12fae8a7eccb197a21006c924906421dad05c"
    );
}

#[test]
fn negotiable_merge_preserves_preintegration_main_catalog_metadata() {
    // Literal constructors from preintegration main eb1555b; never execute their checks.
    for (prefix, count, expected) in [
        (
            "gridworlds.",
            10,
            "96ab7daca04892aa31ea5563609c66318e0a4b71c9b63d68d8c8508664cc208f",
        ),
        (
            "misaligned.",
            12,
            "2d029ac03a2ae3ca8266403c38e1babbdf12f0142daffaf37d3447c9690f6b0c",
        ),
    ] {
        let mut rows: Vec<_> = super::all().into_iter().filter(|c| c.id.starts_with(prefix)).map(|c| serde_json::json!({"id":c.id,"item":c.item,"source":c.source,"citation":c.citation,"text":c.text})).collect();
        rows.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
        assert_eq!(rows.len(), count);
        assert_eq!(
            format!("{:x}", Sha256::digest(serde_json::to_vec(&rows).unwrap())),
            expected
        );
    }
}

#[test]
fn negotiable_records_retain_all_counts_scope_and_fixed_settings() {
    for (id, n, kind) in [
        ("negotiable.cake", 225, "cake"),
        ("negotiable.pareto", 9, "pareto"),
        ("negotiable.priority", 4599, "priority"),
    ] {
        let r = record(id);
        assert_eq!(r["rows"].as_array().unwrap().len(), n);
        assert_eq!(r["scope"], "finite_exact_foundation");
        assert_eq!(r["schema"], "negotiable-foundation-v1");
        assert_eq!(r["settings"]["reference_kind"], kind);
        assert_eq!(r["settings"]["history_horizon"], 8);
        assert_eq!(
            r["binding"]["compiled_source_sha256"]
                .as_str()
                .unwrap()
                .len(),
            64
        );
    }
}

#[test]
fn negotiable_cake_retains_exact_fractions_and_boundary_ties() {
    let r = record("negotiable.cake");
    let rows = r["rows"].as_array().unwrap();
    let source = rows.iter().find(|r| r["policy_id"] == 2).unwrap();
    assert_eq!(
        source["expected"],
        serde_json::json!([{"num":"27","den":"1"},{"num":"27","den":"1"}])
    );
    for (num, den) in [("1", "19"), ("2", "11"), ("9", "11"), ("18", "19")] {
        let row = rows
            .iter()
            .find(|r| r["weight"]["num"] == num && r["weight"]["den"] == den)
            .unwrap();
        assert_eq!(row["global_maximizers"], row["local_maximizers"]);
        assert_eq!(row["global_maximizers"].as_array().unwrap().len(), 2);
    }
}

use super::negotiable_records::{self as records, BoundedRecordWriter, Failure, RecordKind};
use std::{
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

fn canonical(value: &Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(value).unwrap();
    bytes.push(b'\n');
    bytes
}
fn kinds() -> [RecordKind; 3] {
    [RecordKind::Cake, RecordKind::Pareto, RecordKind::Priority]
}

#[test]
fn negotiable_bounded_record_writer_does_not_publish_a_partial_success() {
    let mut writer = BoundedRecordWriter::new(3);
    assert!(writer.write_all(b"four").is_err());
    assert!(writer.finish().is_err());
}
#[test]
fn negotiable_partial_unicode_encoding_remains_sticky() {
    let mut writer = BoundedRecordWriter::new(4);
    writer.write_all("é".as_bytes()).unwrap();
    assert!(writer.write_all("雪".as_bytes()).is_err());
    assert!(writer.write_all(b"a").is_err());
    assert!(writer.flush().is_err());
    assert!(writer.finish().is_err());
}
#[test]
fn negotiable_partial_json_serialization_becomes_error_without_measured_bytes() {
    let o = super::negotiable::outcome(
        records::build_inner(RecordKind::Cake, records::REFERENCES, &"a".repeat(64), 32),
        "engineering injection",
    );
    assert_eq!(o.verdict, Verdict::Error);
    assert!(o.measured.is_empty());
    assert!(o.detail.contains("byte cap"));
}
#[test]
fn negotiable_unicode_error_context_is_retained() {
    let o = super::negotiable::outcome(
        Err(Failure::Operational("source 雪/é unavailable".into())),
        "engineering injection",
    );
    assert_eq!(o.verdict, Verdict::Error);
    assert!(o.detail.contains("雪/é"));
    assert!(o.measured.is_empty());
}
#[test]
fn negotiable_mathematical_disagreement_becomes_fails() {
    let reference: Value = serde_json::from_slice(records::REFERENCES[0]).unwrap();
    let mut actual = reference["rows"].as_array().unwrap().clone();
    actual[0]["objective"]["num"] = serde_json::json!("1");
    let result = records::compare_rows(
        RecordKind::Cake,
        &actual,
        reference["rows"].as_array().unwrap(),
    );
    let o = super::negotiable::outcome(result.map(|_| vec![]), "engineering injection");
    assert_eq!(o.verdict, Verdict::Fails);
    assert!(o.detail.contains("row 0"));
    assert!(o.measured.is_empty());
}
#[test]
fn negotiable_reference_truncation_and_stale_hash_are_errors() {
    for kind in kinds() {
        let bytes = records::REFERENCES[kind.index()];
        assert!(records::validate_reference(
            kind,
            &bytes[..bytes.len() - 5],
            &records::sha256(bytes)
        )
        .is_err());
        assert!(records::validate_reference(kind, bytes, &"0".repeat(64)).is_err());
    }
}
#[test]
fn negotiable_reference_invalid_keys_settings_and_fractions_are_errors() {
    for kind in kinds() {
        let original: Value = serde_json::from_slice(records::REFERENCES[kind.index()]).unwrap();
        for variant in 0..5 {
            let mut value = original.clone();
            match variant {
                0 => {
                    value["unexpected"] = serde_json::json!("雪");
                }
                1 => {
                    value["rows"][0]["unexpected"] = serde_json::json!(true);
                }
                2 => {
                    value["settings"]["history_horizon"] = serde_json::json!(9);
                }
                3 => {
                    value["settings"]["source_hashes"]["main_pdf"] =
                        serde_json::json!("0".repeat(64));
                }
                _ => {
                    let fraction = match kind {
                        RecordKind::Cake => &mut value["rows"][0]["objective"],
                        RecordKind::Pareto => &mut value["rows"][0]["payoff"][0],
                        RecordKind::Priority => &mut value["rows"][0]["result"]["weights"][0],
                    };
                    fraction["num"] = serde_json::json!("00");
                }
            }
            let bytes = canonical(&value);
            // Rehashing isolates structural/settings validation from the outer seal.
            assert!(
                records::validate_reference(kind, &bytes, &records::sha256(&bytes)).is_err(),
                "{kind:?} variant {variant}"
            );
        }
        let mut bytes = canonical(&original);
        bytes.splice(
            1..1,
            b"\"schema\":\"negotiable-foundation-v1\",".iter().copied(),
        );
        assert!(records::validate_reference(kind, &bytes, &records::sha256(&bytes)).is_err());
    }
}
#[test]
fn negotiable_every_missing_and_duplicated_census_is_refused() {
    for kind in kinds() {
        let original: Value = serde_json::from_slice(records::REFERENCES[kind.index()]).unwrap();
        // First, middle and last positions distinguish count-only checks from
        // same-count missing/duplicate/reordered input checks in every census.
        for index in [0, kind.count() / 2, kind.count() - 1] {
            for mutation in 0..3 {
                let mut value = original.clone();
                let rows = value["rows"].as_array_mut().unwrap();
                match mutation {
                    0 => {
                        rows.remove(index);
                    }
                    1 => {
                        rows[index] = rows[(index + 1) % kind.count()].clone();
                    }
                    _ => {
                        rows.swap(index, (index + 1) % kind.count());
                    }
                }
                let bytes = canonical(&value);
                assert!(
                    records::validate_reference(kind, &bytes, &records::sha256(&bytes)).is_err(),
                    "{kind:?} row {index} mutation {mutation}"
                );
            }
        }
    }
}

struct SourceTree(PathBuf);
impl SourceTree {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "negotiable-engineering-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        for (name, bytes) in records::source_entries(records::root()).unwrap() {
            let target = path.join(name);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(target, bytes).unwrap();
        }
        Self(path)
    }
}
impl Drop for SourceTree {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn negotiable_changed_source_before_construction_is_error_without_callback() {
    let tree = SourceTree::new();
    let original = records::source_hash(&tree.0).unwrap();
    std::fs::write(
        tree.0.join("survey/src/claims/negotiable.rs"),
        "// changed 雪\n",
    )
    .unwrap();
    let o = super::negotiable::outcome(
        records::source_bound(&tree.0, &original, || {
            panic!("stale source must refuse before construction")
        }),
        "engineering injection",
    );
    assert_eq!(o.verdict, Verdict::Error);
    assert!(o.measured.is_empty());
    assert!(o.detail.contains("stale source binding"));
}
#[test]
fn negotiable_changed_source_after_construction_revokes_successful_bytes() {
    let tree = SourceTree::new();
    let original = records::source_hash(&tree.0).unwrap();
    let result = records::source_bound(&tree.0, &original, || {
        std::fs::write(
            tree.0.join("survey/src/claims/negotiable.rs"),
            "// changed after construction\n",
        )
        .unwrap();
        Ok(b"{\"complete\":true}".to_vec())
    });
    let o = super::negotiable::outcome(result, "engineering injection");
    assert_eq!(o.verdict, Verdict::Error);
    assert!(o.measured.is_empty());
    assert!(o.detail.contains("stale source binding"));
}
#[test]
fn negotiable_missing_reference_input_is_operational_error() {
    let tree = SourceTree::new();
    std::fs::remove_file(
        tree.0
            .join("crates/sugarscape-core/src/negotiable/tests/fixtures/priority-reference.json"),
    )
    .unwrap();
    assert!(records::source_hash(&tree.0)
        .unwrap_err()
        .to_string()
        .contains("missing required negotiable reference"));
}
#[test]
fn negotiable_complete_wire_certificates_cover_records_and_all_registered_rows() {
    for kind in kinds() {
        let bytes = records::build_record(kind).unwrap();
        let r: Value = serde_json::from_slice(&bytes).unwrap();
        let bound = records::worst_case_bytes(kind, &r["settings"]).unwrap();
        let source = records::source_hash(records::root()).unwrap();
        assert_eq!(r["binding"]["compiled_source_sha256"], source);
        eprintln!("engineering current_runtime_and_compiled_source_sha256={source}");
        assert!(bytes.len() <= bound && bound < records::CAP);
        assert_eq!(bytes, canonical(&r));
        assert_eq!(r["rows"].as_array().unwrap().len(), kind.count());
        eprintln!(
            "engineering {} rows={} actual_bytes={} worst_case_bytes={} cap={}",
            r["settings"]["reference_kind"],
            kind.count(),
            bytes.len(),
            bound,
            records::CAP
        );
    }
}

#[test]
fn negotiable_core_cake_disagreement_is_mathematical_but_overflow_is_operational() {
    use sugarscape_core::negotiable::Error;
    assert!(matches!(
        records::cake_failure(Error::InvalidFixture("whole/local disagreement".into())),
        Failure::Mathematics(_)
    ));
    assert!(matches!(
        records::cake_failure(Error::Overflow),
        Failure::Operational(_)
    ));
}

#[test]
fn negotiable_reference_forged_witness_and_missing_null_are_refused() {
    let original: Value = serde_json::from_slice(records::REFERENCES[1]).unwrap();
    for variant in 0..3 {
        let mut value = original.clone();
        match variant {
            0 => {
                value["rows"][0]["strong"]["efficient"] = serde_json::json!(false);
            }
            1 => {
                value["rows"][0]["strong"]
                    .as_object_mut()
                    .unwrap()
                    .remove("witness");
            }
            _ => {
                let classification = &mut value["rows"][0]["strong"];
                classification["efficient"] = serde_json::json!(false);
                classification["witness"] = serde_json::json!({"parts":[{"policy_id":0,"weight":{"num":"1","den":"1"}}],"payoff":[{"num":"30","den":"1"},{"num":"0","den":"1"}]});
            }
        }
        let bytes = canonical(&value);
        assert!(
            records::validate_reference(RecordKind::Pareto, &bytes, &records::sha256(&bytes))
                .is_err()
        );
    }
}

#[test]
fn negotiable_compatibility_bridge_changes_only_the_exact_library_export_receipt() {
    let library = include_bytes!("../../../crates/sugarscape-core/src/lib.rs");
    let text = std::str::from_utf8(library).unwrap();
    let export = "\npub mod negotiable;\n";
    assert_eq!(text.matches(export).count(), 1);
    assert_eq!(export.len(), 21);
    // Remove only the exact Notes/compact-receipt registration additions.
    let additions = [
        "mod identity_bytes;\n",
        "\npub mod persistent_notes;\n\nmod compact_receipts;\n",
    ];
    let mut pre_notes = text.to_owned();
    for addition in additions {
        assert_eq!(pre_notes.matches(addition).count(), 1);
        pre_notes = pre_notes.replacen(addition, "", 1);
    }
    let original = pre_notes.replacen(export, "", 1);
    // Exact eb1555b library, including both pre-existing model registrations.
    assert_eq!(original.len(), 1949);
    assert_eq!(
        records::sha256(original.as_bytes()),
        "260322051e3e1a94834471168a64414a385bb99dfe32f0b65fc5eb5e41a1e3cc"
    );
    assert_eq!(
        library.len(),
        original.len() + 21 + additions.iter().map(|a| a.len()).sum::<usize>()
    );
    let bytes=include_bytes!("../../../crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json");
    let mut metadata: Value = serde_json::from_slice(bytes).unwrap();
    let bindings = metadata["bindings"].as_array_mut().unwrap();
    assert_eq!(bindings.len(), 2);
    for binding in bindings {
        let sources = binding["sources"].as_array_mut().unwrap();
        let rows: Vec<_> = sources
            .iter_mut()
            .filter(|s| s["path"] == "crates/sugarscape-core/src/lib.rs")
            .collect();
        assert_eq!(rows.len(), 1);
        let row = rows.into_iter().next().unwrap();
        assert_eq!(row["bytes"], library.len());
        assert_eq!(row["sha256"], records::sha256(library));
        row["bytes"] = serde_json::json!(1930);
        row["sha256"] =
            serde_json::json!("80b9f3b95d65c38742eea91e7fe773b161a603f9298d41a958ae787e5e9086ea");
    }
    // Restoring only the two lib receipts recovers the complete reviewed lean metadata,
    // including historical identity, native source and reference identities.
    assert_eq!(
        records::sha256(&serde_json::to_vec(&metadata).unwrap()),
        "4c83c2acf3c20e707c9221af01b96421820311a9d2ac51744fef3aa401b4b6d7"
    );
}

#[test]
fn negotiable_reference_witnesses_use_reference_payoffs_before_math_comparison() {
    use sugarscape_core::negotiable::{Ratio, WireRatio};
    fn double(value: &mut Value) {
        let wire: WireRatio = serde_json::from_value(value.clone()).unwrap();
        let scaled = Ratio::from_wire(wire)
            .unwrap()
            .checked_mul(Ratio::new(2, 1).unwrap())
            .unwrap();
        *value = serde_json::to_value(scaled.wire()).unwrap();
    }
    let original: Value = serde_json::from_slice(records::REFERENCES[1]).unwrap();
    let mut value = original.clone();
    // Uniform positive scaling preserves the reference's own witness geometry,
    // but disagrees mathematically with the registered production utilities.
    for row in value["rows"].as_array_mut().unwrap() {
        for fraction in row["payoff"].as_array_mut().unwrap() {
            double(fraction);
        }
        for key in ["paper_weak", "strong"] {
            if !row[key]["witness"].is_null() {
                for fraction in row[key]["witness"]["payoff"].as_array_mut().unwrap() {
                    double(fraction);
                }
            }
        }
    }
    let bytes = canonical(&value);
    let reference =
        records::validate_reference(RecordKind::Pareto, &bytes, &records::sha256(&bytes))
            .expect("internally coherent reference witness geometry must validate independently");
    let o = super::negotiable::outcome(
        records::compare_rows(
            RecordKind::Pareto,
            original["rows"].as_array().unwrap(),
            &reference.rows,
        )
        .map(|_| vec![]),
        "engineering mathematical mismatch injection",
    );
    assert_eq!(o.verdict, Verdict::Fails);
}
