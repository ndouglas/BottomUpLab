use super::super::*;
#[test]
fn outcome_delta_keeps_each_role_and_non_delivery_unavailable() -> Result<(), Error> {
    let cfg = Config {
        stratum: Stratum::new(1, 3, 0)?,
        ids: Ids::default(),
    };
    let a = run_episode(&cfg, Profile::new(0, 63)?)?.outcome;
    let b = run_episode(&cfg, Profile::new(540, 60)?)?.outcome;
    let delta = outcome_delta(&a, &b)?;
    assert_eq!(delta.spent, [2, -2]);
    assert_eq!(delta.net, [1, 11]);
    assert_eq!(delta.group_net, 12);
    assert_eq!(delta.delivery_round, None);
    assert_eq!(
        delta.delivery_unavailable.as_deref(),
        Some("delivery_round_unavailable")
    );
    Ok(())
}

fn algebraic_table() -> GameTable {
    // Explicit role values: both profiles are equilibria but have different
    // payoffs; a representative must never replace complete set endpoints.
    let outcome = |net: [i16; 2], spent: [u8; 2], reward: [u8; 2], delivery_round| Outcome {
        net,
        spent,
        reward,
        group_net: net[0] + net[1],
        delivery_round,
        holder_attempts: [0; 3],
        holder_blocked: [0; 3],
        retriever_attempts: [0; 4],
        retriever_blocked: [0; 4],
        retriever_waits: 0,
    };
    // Embed the literal 2x2 payoff matrix [[1,5],[-2,3],[-1,-1],[0,0]]
    // at the production row stride; unrelated cells have strictly worse payoffs.
    let mut rows = vec![
        Row {
            outcome: outcome([-5, -5], [5, 5], [0, 0], None),
            trace_hash: "f".repeat(64)
        };
        66
    ];
    rows[0].outcome = outcome([1, 5], [2, 4], [3, 9], Some(4));
    rows[1].outcome = outcome([-2, 3], [5, 6], [3, 9], Some(6));
    rows[64].outcome = outcome([-1, -1], [1, 1], [0, 0], None);
    rows[65].outcome = outcome([0, 0], [0, 0], [0, 0], None);
    GameTable {
        stratum: Stratum::new(1, 3, 0).unwrap(),
        rows,
        analysis: Analysis {
            holder_best: vec![1, 0],
            retriever_best: vec![5, 0],
            pure: vec![Profile::new(0, 0).unwrap(), Profile::new(1, 1).unwrap()],
            team: Profile::new(0, 0).unwrap(),
            lex_pure: Some(Profile::new(0, 0).unwrap()),
            best_group_pure: Some(Profile::new(0, 0).unwrap()),
            solo_best: 5,
        },
        stats: CompileStats {
            profiles: 66,
            transitions: 0,
            peak_bytes: 0,
        },
    }
}
#[test]
fn classify_uses_each_roles_own_unilateral_maximum_and_team_gap() -> Result<(), Error> {
    let t = algebraic_table();
    let c = classify(&t, Profile::new(0, 1)?)?;
    assert_eq!(
        (c.holder_regret, c.retriever_regret, c.team_gap, c.is_pure),
        (2, 2, 5, false)
    );
    Ok(())
}
#[test]
fn pure_ranges_include_all_payoffs_and_empty_sets_stay_unavailable() -> Result<(), Error> {
    let mut t = algebraic_table();
    let e = equilibrium_summary(&t)?;
    assert_eq!(
        e.ranges,
        PayoffRanges {
            holder: Some([0, 1]),
            retriever: Some([0, 5]),
            group: Some([0, 6])
        }
    );
    assert_eq!((e.pure_count, e.delivering_pure_count), (2, 1));
    t.analysis.pure.clear();
    t.analysis.lex_pure = None;
    t.analysis.best_group_pure = None;
    let e = equilibrium_summary(&t)?;
    assert_eq!(
        e.ranges,
        PayoffRanges {
            holder: None,
            retriever: None,
            group: None
        }
    );
    assert_eq!(e.range_unavailable.as_deref(), Some("no_pure_equilibrium"));
    assert_eq!(resolve(&t, Selector::NashLex), None);
    assert_eq!(resolve(&t, Selector::NashBestGroup), None);
    Ok(())
}
#[test]
fn compact_reference_settings_have_18_18_9_contrasts_and_share_invariant_supplied_group_net(
) -> Result<(), Error> {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/game-reference.json")).unwrap();
    let limits: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/limits-reference.json")).unwrap();
    let settings = fixture["panels"]
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let controls = p["controls"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| SummaryControl {
                    selector: serde_json::from_value(c["selector"].clone()).unwrap(),
                    profile: serde_json::from_value(c["profile"].clone()).unwrap(),
                    classification: serde_json::from_value(c["classification"].clone()).unwrap(),
                    unavailable: None,
                })
                .collect();
            SummaryStratum {
                stratum: serde_json::from_value(p["stratum"].clone()).unwrap(),
                stats: serde_json::from_value(limits["panels"][i]["stats"].clone()).unwrap(),
                equilibrium: serde_json::from_value(p["equilibrium"].clone()).unwrap(),
                team: Representative {
                    profile: serde_json::from_value(p["analysis"]["team"].clone()).unwrap(),
                    classification: serde_json::from_value(
                        p["controls"][2]["classification"].clone(),
                    )
                    .unwrap(),
                },
                solo_best: p["analysis"]["solo_best"].as_i64().unwrap() as i16,
                controls,
            }
        })
        .collect::<Vec<_>>();
    let actual = contrasts(&settings)?;
    assert_eq!(actual.len(), 45);
    assert_eq!(
        actual
            .iter()
            .filter(|c| c.axis == Axis::HoldingCost)
            .count(),
        18
    );
    assert_eq!(
        actual
            .iter()
            .filter(|c| c.axis == Axis::HolderShare)
            .count(),
        18
    );
    assert_eq!(
        actual
            .iter()
            .filter(|c| c.axis == Axis::HolderDistance)
            .count(),
        9
    );
    for c in actual {
        if c.axis == Axis::HolderShare {
            assert_eq!(c.controls[1].delta.as_ref().unwrap().group_net, 0);
        }
    }
    Ok(())
}
#[test]
fn metric_difference_checks_signed_overflow() -> Result<(), Error> {
    let mut t = algebraic_table();
    let a = t.rows[0].outcome.clone();
    t.rows[0].outcome.net[0] = i16::MIN;
    let mut b = a;
    b.net[0] = i16::MAX;
    assert!(matches!(
        outcome_delta(&t.rows[0].outcome, &b),
        Err(Error::ArithmeticOverflow)
    ));
    Ok(())
}
#[test]
fn contrasts_preserve_one_sided_and_both_sided_unavailability() -> Result<(), Error> {
    let table = algebraic_table();
    let base_equilibrium = equilibrium_summary(&table)?;
    let base_classification = classify(&table, Profile::new(0, 0)?)?;
    let mut settings = registered_strata()
        .into_iter()
        .map(|stratum| SummaryStratum {
            stratum,
            stats: table.stats.clone(),
            equilibrium: base_equilibrium.clone(),
            team: Representative {
                profile: Profile::new(0, 0).unwrap(),
                classification: base_classification.clone(),
            },
            solo_best: 0,
            controls: super::super::game::SELECTORS
                .into_iter()
                .map(|selector| SummaryControl {
                    selector,
                    profile: Some(Profile::new(0, 0).unwrap()),
                    classification: Some(base_classification.clone()),
                    unavailable: None,
                })
                .collect(),
        })
        .collect::<Vec<_>>();
    for index in [0, 1, 12] {
        let e = &mut settings[index].equilibrium;
        e.pure_count = 0;
        e.delivering_pure_count = 0;
        e.ranges = PayoffRanges {
            holder: None,
            retriever: None,
            group: None,
        };
        e.lex = None;
        e.best_group = None;
        e.range_unavailable = Some("no_pure_equilibrium".into());
        for c in &mut settings[index].controls[3..] {
            c.profile = None;
            c.classification = None;
            c.unavailable = Some("no_pure_equilibrium".into());
        }
    }
    let contrasts = contrasts(&settings)?;
    for (lower, upper, control_reason, range_reason) in [
        (0, 1, "both_controls_unavailable", "both_pure_sets_empty"),
        (0, 6, "lower_control_unavailable", "lower_pure_set_empty"),
        (6, 12, "upper_control_unavailable", "upper_pure_set_empty"),
    ] {
        let c = contrasts
            .iter()
            .find(|c| c.lower == settings[lower].stratum && c.upper == settings[upper].stratum)
            .unwrap();
        assert_eq!(c.controls[3].unavailable.as_deref(), Some(control_reason));
        assert!(c.controls[3].delta.is_none());
        assert_eq!(
            c.equilibria.range_unavailable.as_deref(),
            Some(range_reason)
        );
        assert!(c.equilibria.range_delta.is_none());
    }
    Ok(())
}
