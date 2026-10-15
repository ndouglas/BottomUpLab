use super::super::*;
use crate::deduction::{strategic_reporting as strategic, testimony_game as game};
use serde_json::Value;
#[test]
fn complete_recorded_campaigns_preserve_all_seeds_curves_and_pairs() {
    let start = std::time::Instant::now();
    let json = recorded_results_json().unwrap();
    println!(
        "Recorded compact lossless output: {} bytes, {} ms",
        json.len(),
        start.elapsed().as_millis()
    );
    assert!(json.len() <= MAX_EPISODE_BYTES);
    let value: Value = serde_json::from_str(&json).unwrap();
    for (study, summaries, pairs, environments, frozen) in [
        ("testimony_game", 8, 80, 4, 160),
        ("strategic_reporting", 12, 120, 7, 240),
    ] {
        let record = &value["studies"][study];
        assert_eq!(record["label"], "Retained measurements");
        assert_eq!(record["runs"].as_array().unwrap().len(), 40);
        assert_eq!(record["summaries"].as_array().unwrap().len(), summaries);
        assert_eq!(
            record["paired_differences"].as_array().unwrap().len(),
            pairs
        );
        assert_eq!(
            record["environments"].as_array().unwrap().len(),
            environments
        );
        assert_eq!(
            record["frozen_evaluations"].as_array().unwrap().len(),
            frozen
        );
        for method in ["genetic", "random"] {
            for seed in 0..20 {
                let expected_seed = seed.to_string();
                let run = record["runs"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|r| {
                        r["method"] == method && r["seed"].as_str() == Some(expected_seed.as_str())
                    })
                    .unwrap();
                assert_eq!(run["curve"].as_array().unwrap().len(), 3164);
                assert_eq!(run["curve"][3163][0], "3164");
            }
        }
    }
}
#[test]
fn compact_curve_rows_reconstruct_existing_search_dtos() {
    for (study, asset) in [
        (
            "testimony_game",
            include_str!("../fixtures/testimony-search-recorded.json"),
        ),
        (
            "strategic_reporting",
            include_str!("../fixtures/strategic-recorded.json"),
        ),
    ] {
        let value: Value = serde_json::from_str(asset).unwrap();
        for run in value["runs"].as_array().unwrap() {
            let mut original = run.clone();
            original["curve"]=Value::Array(run["curve"].as_array().unwrap().iter().map(|row|serde_json::json!({"evaluations":row[0],"best_fitness_numerator":row[1],"champion":value["curve_champions"][row[2].as_u64().unwrap() as usize]})).collect());
            if study == "testimony_game" {
                let _: game::SearchRun = serde_json::from_value(original).unwrap();
            } else {
                let _: strategic::SearchRun = serde_json::from_value(original).unwrap();
            }
        }
    }
}
#[test]
fn audit_retained_census_and_provenance_are_complete() {
    let value: Value =
        serde_json::from_str(include_str!("../fixtures/audit-recorded.json")).unwrap();
    for (field, count) in [
        ("control_evaluations", 48),
        ("targeted_audits", 8),
        ("nominal_evaluations", 16),
        ("cross_target_evaluations", 32),
        ("policy_provenance", 46),
    ] {
        assert_eq!(value[field].as_array().unwrap().len(), count);
    }
    assert_eq!(
        value["provenance"]["sha256"],
        "213f9e1402839e3ca8f3c60a27951a78484bef6a8afc9af6c89bde2192266dcc"
    );
}
#[test]
fn original_environment_catalog_and_audit_evaluation_dtos_parse() {
    use crate::deduction::{adversarial_audit as audit, strategy_inference as inference};
    let strategic: Value =
        serde_json::from_str(include_str!("../fixtures/strategic-recorded.json")).unwrap();
    for env in strategic["environments"].as_array().unwrap() {
        let _: strategic::PanelDescription = serde_json::from_value(env.clone()).unwrap();
    }
    let strategy: Value =
        serde_json::from_str(include_str!("../fixtures/strategy-recorded.json")).unwrap();
    for catalog in strategy["catalogs"].as_array().unwrap() {
        let _: inference::CatalogRow = serde_json::from_value(catalog.clone()).unwrap();
    }
    let audit: Value =
        serde_json::from_str(include_str!("../fixtures/audit-recorded.json")).unwrap();
    for field in [
        "control_evaluations",
        "targeted_audits",
        "nominal_evaluations",
        "cross_target_evaluations",
    ] {
        for row in audit[field].as_array().unwrap() {
            let mut row = row.clone();
            let env = audit["environments"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["id"] == row["environment_id"])
                .unwrap();
            let rules: strategic::Config = serde_json::from_value(env["rules"].clone()).unwrap();
            for (index, h) in row["evaluation"]["histories"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .enumerate()
            {
                h["observation"] =
                    serde_json::to_value(audit::history_view(&rules, index as u8).unwrap())
                        .unwrap();
            }
            match field {
                "control_evaluations" => {
                    let _: audit::ControlEvaluation = serde_json::from_value(row).unwrap();
                }
                "targeted_audits" => {
                    let _: audit::TargetedAudit = serde_json::from_value(row).unwrap();
                }
                "nominal_evaluations" => {
                    let _: audit::NominalEvaluation = serde_json::from_value(row).unwrap();
                }
                "cross_target_evaluations" => {
                    let _: audit::CrossTargetEvaluation = serde_json::from_value(row).unwrap();
                }
                _ => unreachable!(),
            }
        }
    }
}

#[test]
fn discovery_recorded_summary_preserves_complete_measured_census_and_transfer_nulls() {
    let value: Value = serde_json::from_str(&recorded_results_json().unwrap()).unwrap();
    let summary = &value["studies"]["surface_discovery"];
    assert_eq!(summary["version"], "surface-discovery-summary-v1");
    assert_eq!(
        summary["report_hash"],
        "a4377fcebbbb92c5d389c319a8098303ed4db812dfb528eff36e5d35e4a23606"
    );
    assert_eq!(
        summary["source_hash"],
        "bea9272bcd6e9aab412d1aafd36dc9facd0ffffd8b10ccfd7e5bb2aaac9fa055"
    );
    for (field, count) in [
        ("settings", 48),
        ("comparisons", 24),
        ("role_expectations", 10),
        ("role_projections", 10),
        ("role_contrasts", 6),
        ("compilations", 16),
    ] {
        assert_eq!(summary[field].as_array().unwrap().len(), count, "{field}");
    }
    let transfer = summary["settings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| {
            row["protocol"]["investigator"] == "B"
                && row["protocol"]["policy"] == "InspectOnly"
                && row["environment"] == "DataFlip"
        })
        .unwrap();
    assert_eq!(
        transfer["aggregate"]["spent"][0],
        serde_json::json!({"numerator":"5","denominator":"1"})
    );
    assert_eq!(
        transfer["aggregate"]["spent"][1],
        serde_json::json!({"numerator":"7","denominator":"1"})
    );
    assert!(transfer["aggregate"]["group_net"].is_null());
    assert_eq!(
        transfer["aggregate"]["net"],
        serde_json::json!([null, null])
    );
    assert_eq!(
        transfer["projection"]["unavailable_mass"],
        serde_json::json!({"numerator":"1","denominator":"1"})
    );
    assert!(summary.get("histories").is_none());
    assert!(summary.get("runs").is_none());
    assert!(summary["settings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|row| row.get("episodes").is_none()));
}

#[test]
fn gate_recorded_summary_retains_all_measured_strata_and_control_classifications() {
    let value: Value = serde_json::from_str(&recorded_results_json().unwrap()).unwrap();
    let summary = &value["studies"]["gate_assistance"];
    assert_eq!(summary["version"], "gate-assistance-summary-v1");
    assert_eq!(
        summary["report_hash"],
        "9bac206e4f8ac91cc101653ba4b52295526ad3596bdce0e9f85b83a3e2afed02"
    );
    assert_eq!(
        summary["source_hash"],
        "e774ebf0e4590138c435ebee7fe148f2e22c5c4de214e9feea9b8765549dd2b1"
    );
    assert_eq!(summary["settings"].as_array().unwrap().len(), 18);
    assert_eq!(summary["contrasts"].as_array().unwrap().len(), 45);
    for row in summary["settings"].as_array().unwrap() {
        assert_eq!(row["stats"]["profiles"], "46656");
        assert_eq!(row["controls"].as_array().unwrap().len(), 5);
        assert!(row.get("rows").is_none());
        for control in row["controls"].as_array().unwrap() {
            assert!(control.get("episode").is_none());
            assert!(control["classification"]["holder_regret"].is_string());
        }
    }
}

#[test]
fn key_recorded_summary_matches_every_native_field_losslessly() {
    let raw: crate::key_delegation::Summary =
        serde_json::from_str(include_str!("../fixtures/key-recorded.json")).unwrap();
    let actual: Value = serde_json::from_str(&recorded_results_json().unwrap()).unwrap();
    assert_eq!(
        actual["studies"]["key_delegation"]["version"],
        "key-delegation-summary-v1"
    );
    let native: Value =
        serde_json::from_str(include_str!("../fixtures/key-recorded.json")).unwrap();
    assert_eq!(serde_json::to_value(&raw).unwrap(), native);
    // Independent expected projection: never call the production wire codec.
    fn decimal_strings(value: Value) -> Value {
        match value {
            Value::Number(n) => Value::String(n.to_string()),
            Value::Array(a) => Value::Array(a.into_iter().map(decimal_strings).collect()),
            Value::Object(o) => Value::Object(
                o.into_iter()
                    .map(|(k, v)| (k, decimal_strings(v)))
                    .collect(),
            ),
            value => value,
        }
    }
    assert_eq!(actual["studies"]["key_delegation"], decimal_strings(native));
}

#[test]
fn key_recorded_census_preserves_order_ownership_and_all_receipts() {
    let value: Value = serde_json::from_str(&recorded_results_json().unwrap()).unwrap();
    let s = &value["studies"]["key_delegation"];
    assert_eq!(
        s["report_hash"],
        "7126a4586087408cbad1845ed716039fb774e66fb300043689ffba015c9d2218"
    );
    assert_eq!(
        s["source_hash"],
        "93b3d388b5c652d332811456f174615876db114ba48a0bad48b8314b3a4563d7"
    );
    assert_eq!(s["source_receipts"].as_array().unwrap().len(), 18);
    assert_eq!(s["reference_receipts"].as_array().unwrap().len(), 3);
    assert_eq!(s["checks"].as_array().unwrap().len(), 11);
    assert_eq!(s["passed"], true);
    let rows = s["settings"].as_array().unwrap();
    assert_eq!(rows.len(), 36);
    let selectors = [
        "SoloAttempt",
        "SuppliedHandoff",
        "TeamOptimal",
        "NashLex",
        "NashBestGroup",
        "InitiallyEquipped",
    ];
    let mut i = 0;
    for e in [false, true] {
        for t in ["1", "2", "4"] {
            for b in ["0", "3", "6"] {
                for d in ["0", "1"] {
                    let r = &rows[i];
                    i += 1;
                    assert_eq!(
                        r["stratum"],
                        serde_json::json!({"transfer_enabled":e,"transfer_cost":t,"giver_share":b,"distance":d})
                    );
                    assert_eq!(r["stats"]["profiles"], "46656");
                    let cs = r["controls"].as_array().unwrap();
                    assert_eq!(cs.len(), 6);
                    for (c, selector) in cs.iter().zip(selectors) {
                        assert_eq!(c["selector"], selector);
                        assert!(c["unavailable"].is_null());
                        if selector == "InitiallyEquipped" {
                            assert_eq!(c["classification_scope"], "changed_initial_control");
                            assert_eq!(c["selection"]["initial_owner"], "Receiver");
                            assert!(c["selection"]["classification"].is_null());
                            assert_eq!(c["outcome"]["delivery_round"], "4");
                        } else {
                            assert_eq!(c["selection"]["initial_owner"], "Giver");
                            assert_eq!(c["outcome"], c["selection"]["classification"]["outcome"]);
                        }
                    }
                    assert!(r.get("rows").is_none());
                    assert!(r.get("episodes").is_none());
                }
            }
        }
    }
    let contrasts = s["contrasts"].as_array().unwrap();
    assert_eq!(contrasts.len(), 84);
    for (axis, n) in [
        ("TransferPermission", 18),
        ("TransferCost", 24),
        ("GiverShare", 24),
        ("Distance", 18),
    ] {
        assert_eq!(contrasts.iter().filter(|c| c["axis"] == axis).count(), n);
    }
    fn no_numbers(v: &Value) {
        match v {
            Value::Number(_) => panic!("lossy numeric atom"),
            Value::Array(a) => a.iter().for_each(no_numbers),
            Value::Object(o) => o.values().for_each(no_numbers),
            _ => {}
        }
    }
    no_numbers(s);
}

#[test]
fn paid_notes_recorded_summary_keeps_every_registered_aggregate_and_identity() {
    let value: Value = serde_json::from_str(&recorded_results_json().unwrap()).unwrap();
    let summary = &value["studies"]["persistent_notes"];
    assert_eq!(summary["version"], "persistent-notes-summary-v1");
    for (field, count) in [
        ("means", 2304),
        ("request_contrasts", 4608),
        ("contrasts", 2304),
        ("envelopes", 576),
    ] {
        assert_eq!(summary[field].as_array().unwrap().len(), count);
    }
    assert_eq!(
        summary["source"]["sha256"],
        "28b90c51a6f8d126eae517f1292a32518207c888ff95f60abd9c2b45ee3c572e"
    );
    assert_eq!(
        summary["reference"]["sha256"],
        "2227ad21600c94e54245bd2743509b6aa65399f2f8eb86693c1aecd483225788"
    );
    assert_eq!(summary["report_sha256"].as_str().unwrap().len(), 64);
}

#[test]
fn notes_dictionary_reconstructs_complete_lossless_summary_within_unchanged_transport_cap() {
    let raw = recorded_results_json().unwrap();
    assert!(raw.len() <= MAX_EPISODE_BYTES);
    println!("Combined recorded transport: {} bytes", raw.len());
    let mut all: Value = serde_json::from_str(&raw).unwrap();
    let summary = all["studies"]["persistent_notes"].as_object_mut().unwrap();
    assert_eq!(
        summary.remove("wire_encoding").unwrap(),
        "persistent-notes-metrics-dictionary-v1"
    );
    let dictionary = summary.remove("metrics_dictionary").unwrap();
    for field in ["means", "request_contrasts", "contrasts"] {
        for row in summary.get_mut(field).unwrap().as_array_mut().unwrap() {
            let index: usize = row["metrics"].as_str().unwrap().parse().unwrap();
            row["metrics"] = dictionary[index].clone();
        }
    }
    let original: Value =
        serde_json::from_str(include_str!("../fixtures/notes-recorded.json")).unwrap();
    let expected = super::super::wire::lossless_value(&original).unwrap();
    assert_eq!(Value::Object(summary.clone()), expected);
}
