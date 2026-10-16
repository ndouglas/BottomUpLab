use super::*;
fn incomplete() -> Report {
    Report {
        version: "key-delegation-diagnostic-v1".into(),
        source_hash: source_identity().unwrap(),
        source_receipts: source_receipts().unwrap(),
        reference_receipts: reference_receipts().unwrap(),
        limits: Limits::registered().unwrap(),
        panels: vec![],
        contrasts: vec![],
        checks: registered_checks(),
        passed: true,
    }
}
#[test]
fn incomplete_report_cannot_pass_integrity_or_summary() {
    let r = incomplete();
    assert!(report_integrity(&r, &r.limits).is_err());
    assert!(summary_json(&r, &r.limits).is_err());
}
#[test]
fn report_transport_rejects_duplicates_unknowns_and_noninteger_atoms() {
    let r = incomplete();
    let s = serde_json::to_string(&r).unwrap();
    assert_eq!(serde_json::from_str::<Report>(&s).unwrap(), r);
    for bad in [
        s.replacen("{", "{\"passed\":false,", 1),
        s.replacen("{", "{\"extra\":0,", 1),
        s.replacen("93312", "93312.0", 1),
        s.replacen("93312", "9.3312e4", 1),
        s.replacen("93312", "true", 1),
        s.replacen("93312", "-0", 1),
    ] {
        assert!(serde_json::from_str::<Report>(&bad).is_err(), "{bad}");
    }
    assert!(serde_json::from_str::<Report>("[]").is_err());
}
#[test]
fn source_closure_is_ordered_and_complete() {
    let s = source_receipts().unwrap();
    let r = reference_receipts().unwrap();
    assert_eq!(s.len(), 25);
    assert_eq!(r.len(), 3);
    assert_eq!(
        s[14].path,
        "crates/sugarscape-core/src/key_delegation/sha256.rs"
    );
    assert_eq!(s[24].path, "Cargo.lock");
    assert_ne!(source_identity().unwrap(), s[0].sha256);
}
#[test]
fn stale_source_is_an_error_before_reconstruction() {
    let mut r = incomplete();
    r.source_hash = "0".repeat(64);
    let e = report_integrity(&r, &r.limits).unwrap_err().to_string();
    assert!(e.contains("source"), "{e}");
}
fn tiny_panel() -> ReportPanel {
    let refs = super::super::references::load().unwrap();
    let p = &refs.game.panels[0];
    let e = run_episode(
        &Config::new(p.stratum.clone(), Ids::default(), InitialOwner::Giver).unwrap(),
        Profile::new(0, 0).unwrap(),
    )
    .unwrap();
    ReportPanel {
        stratum: p.stratum.clone(),
        rows: vec![Row {
            outcome: e.outcome.clone(),
            trace_hash: trace_hash(&e).unwrap(),
        }],
        analysis: p.analysis.clone(),
        controls: p.controls.clone(),
        stats: p.stats.clone(),
    }
}
#[test]
fn strict_panel_requires_nullable_fields_at_every_level() {
    let p = tiny_panel();
    let value = serde_json::to_value(&p).unwrap();
    for pointer in [
        "/analysis/lex_pure",
        "/controls/0/unavailable",
        "/controls/0/episode/outcome/delivery_round",
        "/controls/5/selection/classification",
    ] {
        let mut v = value.clone();
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        v.pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(
            serde_json::from_value::<ReportPanel>(v).is_err(),
            "{pointer}"
        );
    }
    let s = serde_json::to_string(&p).unwrap();
    for bad in [
        s.replacen("\"rows\":", "\"rows\":[],\"r\\u006fws\":", 1),
        s.replacen("\"spent\":[0,0]", "\"spent\":[0.0,0]", 1),
        s.replacen("\"spent\":[0,0]", "\"spent\":[-0,0]", 1),
    ] {
        assert_ne!(bad, s);
        assert!(serde_json::from_str::<ReportPanel>(&bad).is_err());
    }
}
#[test]
fn initially_equipped_has_separate_ownership_and_no_classification() {
    let panel = tiny_panel();
    let check = super::super::diagnostics::check_controls;
    check(&panel.stratum, &panel.controls).unwrap();
    let mut controls = panel.controls.clone();
    controls[5].selection.as_mut().unwrap().classification = controls[0]
        .selection
        .as_ref()
        .unwrap()
        .classification
        .clone();
    assert!(check(&panel.stratum, &controls)
        .unwrap_err()
        .to_string()
        .contains("classification"));
    let mut controls = panel.controls.clone();
    controls[5].selection.as_mut().unwrap().initial_owner = InitialOwner::Giver;
    assert!(check(&panel.stratum, &controls)
        .unwrap_err()
        .to_string()
        .contains("ownership"));
    let mut controls = panel.controls.clone();
    controls[5].selection.as_mut().unwrap().profile = Profile::new(0, 63).unwrap();
    assert!(check(&panel.stratum, &controls).is_err());
    let mut controls = panel.controls.clone();
    controls.swap(0, 1);
    assert!(check(&panel.stratum, &controls).is_err());
    let mut controls = panel.controls.clone();
    controls[0].episode.as_mut().unwrap().histories[0].pop();
    assert!(check(&panel.stratum, &controls).is_err());
}
#[test]
fn streamed_canonical_report_equals_independent_small_value_codec() {
    let mut r = incomplete();
    r.panels.push(tiny_panel());
    let mut bytes = vec![];
    super::super::diagnostics::canonical_report(&r, &mut bytes).unwrap();
    assert_eq!(bytes, super::super::canonical::canonical_bytes(&r).unwrap());
    assert_ne!(
        super::super::sha256::sha256(&bytes),
        super::super::canonical::receipt_identity("key-delegation-report-v1", &bytes).unwrap()
    );
    let parsed: Report = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(parsed, r);
}
#[test]
fn report_metadata_rejects_changed_caps_and_receipt_order() {
    let original = incomplete();
    let mut r = original.clone();
    r.limits.profiles -= 1;
    assert!(report_integrity(&r, &original.limits)
        .unwrap_err()
        .to_string()
        .contains("limits"));
    let mut r = original.clone();
    r.source_receipts.swap(0, 1);
    assert!(report_integrity(&r, &original.limits)
        .unwrap_err()
        .to_string()
        .contains("source"));
    let mut r = original.clone();
    r.reference_receipts.pop();
    assert!(report_integrity(&r, &original.limits)
        .unwrap_err()
        .to_string()
        .contains("reference"));
    let mut r = original.clone();
    r.source_receipts[1] = r.source_receipts[0].clone();
    assert!(report_integrity(&r, &original.limits).is_err());
    for passed in [true, false] {
        let mut r = original.clone();
        r.passed = passed;
        assert!(summary_json(&r, &r.limits).is_err());
    }
}
#[test]
fn receipt_paths_and_hashes_are_strict() {
    let r = source_receipts().unwrap().remove(0);
    for path in [
        "",
        "/absolute",
        "a//b",
        "a/../b",
        "./a",
        "a\\b",
        "a\"b",
        "é",
        "a\u{7f}",
    ] {
        let mut x = r.clone();
        x.path = path.into();
        assert!(x.validate().is_err(), "{path}");
    }
    let mut x = r;
    x.sha256 = "A".repeat(64);
    assert!(x.validate().is_err());
}

#[test]
fn row_receipts_bind_every_outcome_trace_and_order() {
    use super::super::diagnostics::raw_rows;
    let panel = tiny_panel();
    let mut rows = panel.rows.clone();
    let e = run_episode(
        &Config::new(panel.stratum, Ids::default(), InitialOwner::Giver).unwrap(),
        Profile::new(1, 1).unwrap(),
    )
    .unwrap();
    rows.push(Row {
        outcome: e.outcome.clone(),
        trace_hash: trace_hash(&e).unwrap(),
    });
    let expected = raw_rows("rows", &rows).unwrap();
    for action in 0..5 {
        let mut changed = rows.clone();
        match action {
            0 => changed.swap(0, 1),
            1 => {
                changed.pop();
            }
            2 => changed.push(changed[0].clone()),
            3 => changed[0].trace_hash = "0".repeat(64),
            _ => changed[0].outcome.giver_attempts[0] -= 1,
        }
        assert_ne!(raw_rows("rows", &changed).unwrap(), expected);
    }
    let bytes = super::super::canonical::canonical_bytes(&rows).unwrap();
    assert_eq!(expected.bytes, bytes.len() as u64);
    assert_eq!(expected.sha256, super::super::sha256::sha256(&bytes));
}
#[test]
fn panel_census_rejects_missing_duplicated_and_reordered_strata_before_compile() {
    let mut r = incomplete();
    let p = tiny_panel();
    r.panels = strata()
        .into_iter()
        .map(|stratum| ReportPanel {
            stratum,
            ..p.clone()
        })
        .collect();
    // Even valid stratum objects at the wrong index are not an alternative ordering.
    r.panels[0].stratum = strata()[1].clone();
    assert!(super::super::diagnostics::structure(&r)
        .unwrap_err()
        .to_string()
        .contains("stratum order"));
    r.panels[0].stratum = strata()[0].clone();
    assert!(super::super::diagnostics::structure(&r)
        .unwrap_err()
        .to_string()
        .contains("46656"));
    r.panels.pop();
    assert!(super::super::diagnostics::structure(&r)
        .unwrap_err()
        .to_string()
        .contains("36"));
}

fn registered_checks() -> Vec<Check> {
    super::super::protocol::CHECK_NAMES
        .iter()
        .map(|name| Check {
            name: (*name).into(),
            passed: true,
            detail: "synthetic explanation".into(),
        })
        .collect()
}
#[test]
fn review_r1_nonempty_unicode_detail_does_not_change_scientific_comparison() {
    let actual = registered_checks();
    let mut reported = actual.clone();
    reported[4].detail = "Recomputed exactly — Agent α; résultat 同一 🚪".into();
    assert!(super::super::diagnostics::checks_match(&reported, &actual).unwrap());
    let mut report = incomplete();
    report.checks = actual;
    let mut before = vec![];
    super::super::diagnostics::canonical_report(&report, &mut before).unwrap();
    report.checks = reported;
    let mut after = vec![];
    super::super::diagnostics::canonical_report(&report, &mut after).unwrap();
    assert_ne!(
        super::super::canonical::receipt_identity("key-delegation-report-v1", &before).unwrap(),
        super::super::canonical::receipt_identity("key-delegation-report-v1", &after).unwrap()
    );
}
#[test]
fn review_r1_empty_detail_is_malformed_before_reconstruction() {
    let mut report = incomplete();
    report.checks = registered_checks();
    report.checks[3].detail.clear();
    let error = report_integrity(&report, &report.limits)
        .unwrap_err()
        .to_string();
    assert!(error.contains("empty check detail"), "{error}");
    assert!(super::super::diagnostics::checks_match(&report.checks, &registered_checks()).is_err());
}
#[test]
fn review_r1_wrong_names_order_and_passed_bits_do_not_match() {
    let actual = registered_checks();
    let check = super::super::diagnostics::checks_match;
    let mut reported = actual.clone();
    reported[5].name = "wrong".into();
    assert!(check(&reported, &actual).is_err());
    let mut reported = actual.clone();
    reported.swap(6, 7);
    assert!(check(&reported, &actual).is_err());
    let mut reported = actual.clone();
    reported[8].passed = false;
    assert!(!check(&reported, &actual).unwrap());
    let mut failed = actual.clone();
    failed[8].passed = false;
    assert!(!check(&actual, &failed).unwrap());
    let mut reported = actual.clone();
    reported.pop();
    assert!(check(&reported, &actual).is_err());
}
#[test]
fn review_r2_native_panel_comparison_rejects_outcome_trace_and_row_order_changes() {
    let mut actual = tiny_panel();
    let e = run_episode(
        &Config::new(actual.stratum.clone(), Ids::default(), InitialOwner::Giver).unwrap(),
        Profile::new(1, 1).unwrap(),
    )
    .unwrap();
    actual.rows.push(Row {
        outcome: e.outcome.clone(),
        trace_hash: trace_hash(&e).unwrap(),
    });
    let agrees = |p: &ReportPanel| {
        super::super::diagnostics::panel_matches(
            p,
            &actual.rows,
            &actual.analysis,
            &actual.controls,
            &actual.stats,
        )
    };
    assert!(agrees(&actual));
    for mutation in 0..5 {
        let mut reported = actual.clone();
        match mutation {
            0 => reported.rows[0].outcome.giver_attempts[0] -= 1,
            1 => reported.rows[0].trace_hash = "0".repeat(64),
            2 => reported.rows.swap(0, 1),
            3 => {
                reported.rows.pop();
            }
            _ => reported.rows.push(reported.rows[0].clone()),
        }
        assert!(!agrees(&reported), "accepted mutation {mutation}");
    }
}
#[test]
fn review_r2_actual_contrast_comparison_rejects_report_mutation() {
    let actual = super::super::references::load().unwrap().game.contrasts;
    let mut report = incomplete();
    report.contrasts = actual[..1].to_vec();
    assert!(super::super::diagnostics::contrasts_match(
        &report.contrasts,
        &actual[..1]
    ));
    report.contrasts[0].fixed_words.physical_equal_count += 1;
    assert!(!super::super::diagnostics::contrasts_match(
        &report.contrasts,
        &actual[..1]
    ));
}
#[test]
fn review_r2_later_duplicate_and_reordered_panels_reach_order_boundary() {
    let mut report = incomplete();
    report.checks = registered_checks();
    let panel = tiny_panel();
    report.panels = strata()
        .into_iter()
        .map(|stratum| ReportPanel {
            stratum,
            ..panel.clone()
        })
        .collect();
    // Keep every row array deliberately short; the label check must reach panel 32 first.
    report.panels.swap(32, 33);
    let error = super::super::diagnostics::structure(&report)
        .unwrap_err()
        .to_string();
    assert!(error.contains("panel 32 stratum order"), "{error}");
    report.panels.swap(32, 33);
    report.panels[35].stratum = report.panels[34].stratum.clone();
    let error = super::super::diagnostics::structure(&report)
        .unwrap_err()
        .to_string();
    assert!(error.contains("panel 35 stratum order"), "{error}");
}
