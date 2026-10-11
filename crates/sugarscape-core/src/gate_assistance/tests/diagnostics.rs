//! Boundary tests deliberately never collect a successful diagnostic.
use super::super::diagnostics::{empty_report, validate_identity};
use super::super::*;

#[test]
fn saved_passed_flag_cannot_authorize_missing_census_or_summary() {
    let mut report = empty_report(&Limits::registered().unwrap()).unwrap();
    report.passed = true;
    assert!(!report_integrity(&report, &report.limits).unwrap());
    assert!(summary_json(&report).is_err());
}
#[test]
fn current_raw_receipts_reject_forgery_missing_duplicate_and_reordering() {
    let original = empty_report(&Limits::registered().unwrap()).unwrap();
    validate_identity(&original).unwrap();
    for reference in [false, true] {
        for mutation in 0..5 {
            let mut report = original.clone();
            let receipts = if reference {
                &mut report.reference_receipts
            } else {
                &mut report.source_receipts
            };
            match mutation {
                0 => receipts[0].sha256 = "0".repeat(64),
                1 => {
                    receipts.pop();
                }
                2 => receipts.push(receipts[0].clone()),
                3 => receipts.swap(0, 1),
                _ => receipts[0].bytes += 1,
            }
            assert!(report_integrity(&report, &report.limits).is_err());
        }
    }
    let mut report = original;
    report.source_hash = "0".repeat(64);
    assert!(report_integrity(&report, &report.limits).is_err());
}
#[test]
fn report_wire_rejects_object_substitutes_nested_duplicates_and_number_coercion() {
    let report = empty_report(&Limits::registered().unwrap()).unwrap();
    let json = serde_json::to_string(&report).unwrap();
    assert_eq!(serde_json::from_str::<Report>(&json).unwrap(), report);
    assert!(serde_json::from_str::<Report>("[]").is_err());
    let duplicated = json.replacen("\"profiles\":", "\"profiles\":1,\"profiles\":", 1);
    assert!(serde_json::from_str::<Report>(&duplicated).is_err());
    for atom in ["true", "1.0", "\"1\"", "18446744073709551616", "-1"] {
        let mut value = serde_json::to_value(&report).unwrap();
        value["limits"]["profiles"] = serde_json::from_str(atom).unwrap();
        assert!(serde_json::from_value::<Report>(value).is_err(), "{atom}");
    }
    assert!(serde_json::from_str::<ControlRecord>(
        r#"{"selector":"NashLex","profile":null,"episode":null}"#
    )
    .is_err());
    assert!(
        serde_json::from_str::<Receipt>(r#"{"path":"a","bytes":0,"bytes":1,"sha256":"x"}"#)
            .is_err()
    );
}
#[test]
fn reduced_caps_are_operational_errors_before_collection() {
    let mut limits = Limits::registered().unwrap();
    limits.profiles = 0;
    assert!(matches!(
        diagnose(&limits),
        Err(Error::ResourceLimit { .. })
    ));
}
#[test]
fn reconstruction_requires_registered_panel_and_canonical_raw_mapping() {
    let report = empty_report(&Limits::registered().unwrap()).unwrap();
    assert!(reconstruct_episode(&report, 0, Profile::new(0, 0).unwrap()).is_err());
    assert!(reconstruct_episode(&report, usize::MAX, Profile::new(0, 0).unwrap()).is_err());
}

fn synthetic_table() -> GameTable {
    let stratum = Stratum::new(1, 3, 0).unwrap();
    let episode = run_episode(
        &Config {
            stratum: stratum.clone(),
            ids: Ids::default(),
        },
        Profile::new(0, 0).unwrap(),
    )
    .unwrap();
    let mut rows = vec![
        Row {
            outcome: episode.outcome.clone(),
            trace_hash: trace_hash(&episode).unwrap()
        };
        46656
    ];
    rows[1].trace_hash = "1".repeat(64);
    rows[64].trace_hash = "2".repeat(64);
    GameTable {
        stratum,
        rows,
        analysis: Analysis {
            holder_best: vec![0; 64],
            retriever_best: vec![0; 729],
            pure: vec![Profile::new(0, 0).unwrap()],
            team: Profile::new(0, 0).unwrap(),
            lex_pure: Some(Profile::new(0, 0).unwrap()),
            best_group_pure: Some(Profile::new(0, 0).unwrap()),
            solo_best: 0,
        },
        stats: CompileStats {
            profiles: 46656,
            transitions: 1,
            peak_bytes: 1,
        },
    }
}
fn retained(table: &GameTable) -> Panel {
    Panel {
        stratum: table.stratum.clone(),
        rows: table.rows.clone(),
        analysis: table.analysis.clone(),
        controls: super::super::diagnostics::controls(table).unwrap(),
        stats: table.stats.clone(),
    }
}
#[test]
fn panel_comparison_rejects_missing_duplicate_reordered_transposed_and_forged_rows() {
    use super::super::diagnostics::compare_panel;
    let table = synthetic_table();
    let original = retained(&table);
    assert!(compare_panel(&original, &table)
        .unwrap()
        .into_iter()
        .all(|x| x));
    for mutation in 0..6 {
        let mut p = original.clone();
        match mutation {
            0 => {
                p.rows.pop();
            }
            1 => {
                p.rows[1] = p.rows[0].clone();
            }
            2 => p.rows.swap(0, 1),
            3 => p.rows.swap(1, 64),
            4 => p.rows[0].trace_hash = "f".repeat(64),
            _ => p.rows[0].outcome.net[0] += 1,
        }
        assert!(
            !compare_panel(&p, &table).unwrap()[0],
            "mutation {mutation}"
        );
    }
}
#[test]
fn panel_comparison_rejects_maxima_pure_sets_representatives_regret_and_availability() {
    use super::super::diagnostics::compare_panel;
    let table = synthetic_table();
    let original = retained(&table);
    for mutation in 0..7 {
        let mut p = original.clone();
        let index = match mutation {
            0 => {
                p.analysis.holder_best[0] += 1;
                1
            }
            1 => {
                p.analysis.retriever_best[0] += 1;
                1
            }
            2 => {
                p.analysis.pure.push(Profile::new(0, 0).unwrap());
                2
            }
            3 => {
                p.analysis.team = Profile::new(1, 0).unwrap();
                3
            }
            4 => {
                p.controls[0].classification.as_mut().unwrap().holder_regret += 1;
                4
            }
            5 => {
                p.controls[3].profile = None;
                p.controls[3].episode = None;
                p.controls[3].classification = None;
                4
            }
            _ => {
                p.controls.swap(0, 1);
                4
            }
        };
        assert!(
            !compare_panel(&p, &table).unwrap()[index],
            "mutation {mutation}"
        );
    }
}
#[test]
fn census_order_and_check_order_are_required_before_expensive_compilation() {
    use super::super::{diagnostics::shape, protocol::CHECK_NAMES};
    let template = synthetic_table();
    let mut report = empty_report(&Limits::registered().unwrap()).unwrap();
    report.checks = CHECK_NAMES
        .into_iter()
        .map(|name| Check {
            name: name.into(),
            passed: true,
            detail: "synthetic shape only".into(),
        })
        .collect();
    for stratum in registered_strata() {
        let mut t = template.clone();
        t.stratum = stratum;
        let mut p = retained(&t);
        // These deliberately synthetic classifications provide well-formed shape;
        // the complete scientific verifier is never called on this baseline.
        for c in &mut p.controls {
            c.classification.as_mut().unwrap().outcome =
                c.episode.as_ref().unwrap().outcome.clone();
        }
        report.panels.push(p);
    }
    assert!(shape(&report).unwrap());
    let last = report.panels.pop().unwrap();
    assert!(!report_integrity(&report, &report.limits).unwrap());
    report.panels.push(last);
    report.panels.swap(0, 1);
    assert!(!report_integrity(&report, &report.limits).unwrap());
    report.panels.swap(0, 1);
    let original = report.panels[1].stratum.clone();
    report.panels[1].stratum = report.panels[0].stratum.clone();
    assert!(!report_integrity(&report, &report.limits).unwrap());
    report.panels[1].stratum = original;
    report.checks.swap(0, 1);
    assert!(!report_integrity(&report, &report.limits).unwrap());
    report.checks.swap(0, 1);
    report.checks[1].name = report.checks[0].name.clone();
    assert!(!report_integrity(&report, &report.limits).unwrap());
}
#[test]
fn approved_portable_metadata_and_compact_witnesses_load_without_ignored_files() {
    let refs = super::super::references::References::load().unwrap();
    assert_eq!(refs.witnesses().unwrap(), (true, true));
    assert!(!refs.row_identity(0, &vec![]).unwrap());
}
#[test]
fn nested_episode_duplicates_and_profile_number_boundaries_are_not_erased() {
    let e = super::episode(540, 60);
    let c = ControlRecord {
        selector: Selector::SuppliedHelp,
        profile: Some(e.profile),
        episode: Some(e),
        classification: None,
    };
    let json = serde_json::to_string(&c).unwrap();
    let duplicate = json.replacen("\"holder\":540", "\"holder\":540,\"holder\":541", 1);
    assert!(serde_json::from_str::<ControlRecord>(&duplicate).is_err());
    let duplicate = json.replacen("\"paid\":1", "\"paid\":1,\"paid\":2", 1);
    assert_ne!(duplicate, json);
    assert!(serde_json::from_str::<ControlRecord>(&duplicate).is_err());
    for holder in ["true", "1.0", "\"1\"", "729", "65536", "-1"] {
        let bad = json.replacen("\"holder\":540", &format!("\"holder\":{holder}"), 1);
        assert!(
            serde_json::from_str::<ControlRecord>(&bad).is_err(),
            "{holder}"
        );
    }
}
#[test]
fn forged_resource_statistics_are_rejected_by_current_table_comparison() {
    let table = synthetic_table();
    let mut p = retained(&table);
    p.stats.peak_bytes += 1;
    assert!(!super::super::diagnostics::compare_panel(&p, &table).unwrap()[5]);
}

#[test]
fn summary_and_unavailable_control_wire_preserve_required_nulls_and_strict_objects() {
    let report = empty_report(&Limits::registered().unwrap()).unwrap();
    let summary = Summary {
        version: "gate-assistance-summary-v1".into(),
        report_hash: "0".repeat(64),
        source_hash: report.source_hash,
        settings: vec![],
        contrasts: vec![],
        limits: report.limits,
        source_receipts: report.source_receipts,
        reference_receipts: report.reference_receipts,
    };
    let json = serde_json::to_string(&summary).unwrap();
    assert_eq!(serde_json::from_str::<Summary>(&json).unwrap(), summary);
    assert!(serde_json::from_str::<Summary>("[]").is_err());
    let extra = json.replacen('{', "{\"unregistered\":1,", 1);
    assert!(serde_json::from_str::<Summary>(&extra).is_err());
    let nulls = r#"{"selector":"NashLex","profile":null,"episode":null,"classification":null}"#;
    let control = serde_json::from_str::<ControlRecord>(nulls).unwrap();
    assert_eq!(serde_json::to_string(&control).unwrap(), nulls);
}
