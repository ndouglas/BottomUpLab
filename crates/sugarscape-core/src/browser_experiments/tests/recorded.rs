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
