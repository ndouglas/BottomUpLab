use super::super::game::{rank, response_sets};
use super::super::*;

#[test]
fn deviation_activates_an_unexecuted_suffix() -> Result<(), Error> {
    let cfg = Config {
        stratum: Stratum::new(1, 3, 0)?,
        ids: Ids::default(),
    };
    assert_eq!(
        run_episode(&cfg, Profile::new(540, 60)?)?.events,
        run_episode(&cfg, Profile::new(546, 60)?)?.events
    );
    assert_eq!(
        run_episode(&cfg, Profile::new(540, 15)?)?
            .outcome
            .delivery_round,
        None
    );
    assert_eq!(
        run_episode(&cfg, Profile::new(546, 15)?)?
            .outcome
            .delivery_round,
        Some(6)
    );
    Ok(())
}
#[test]
fn all_private_payoff_ties_are_retained() -> Result<(), Error> {
    let r = response_sets(2, 2, &[[0, 0]; 4])?;
    assert_eq!(
        r.pure,
        vec![
            Profile::new(0, 0)?,
            Profile::new(0, 1)?,
            Profile::new(1, 0)?,
            Profile::new(1, 1)?
        ]
    );
    Ok(())
}
#[test]
fn private_response_axes_use_own_unilateral_payoffs() -> Result<(), Error> {
    let r = response_sets(2, 2, &[[3, 8], [7, 2], [5, 4], [1, 9]])?;
    assert_eq!(r.holder_best, vec![5, 7]);
    assert_eq!(r.retriever_best, vec![8, 9]);
    assert!(r.pure.is_empty());
    let r = response_sets(2, 2, &[[1, 4], [7, 8], [5, 3], [9, 6]])?;
    assert_eq!(r.pure, vec![Profile::new(1, 1)?]);
    Ok(())
}
#[test]
fn matching_pennies_has_no_fabricated_pure_profile() -> Result<(), Error> {
    assert!(response_sets(2, 2, &[[1, -1], [-1, 1], [-1, 1], [1, -1]])?
        .pure
        .is_empty());
    Ok(())
}
#[test]
fn response_dimensions_and_lengths_are_checked() {
    for (h, r, p) in [
        (0, 2, vec![]),
        (2, 0, vec![]),
        (730, 1, vec![]),
        (1, 65, vec![]),
        (2, 2, vec![[0, 0]; 3]),
    ] {
        assert!(response_sets(h, r, &p).is_err());
    }
}
#[test]
fn team_rank_uses_numeric_group_then_spending_then_time_then_ids() -> Result<(), Error> {
    let mut a = run_episode(
        &Config {
            stratum: Stratum::new(1, 3, 0)?,
            ids: Ids::default(),
        },
        Profile::new(0, 0)?,
    )?
    .outcome;
    a.group_net = 10;
    a.spent = [2, 3];
    a.delivery_round = Some(6);
    let mut b = a.clone();
    b.group_net = 2;
    b.spent = [0, 0];
    b.delivery_round = Some(4);
    assert!(rank(&a, Profile::new(20, 10)?)? < rank(&b, Profile::new(0, 0)?)?);
    b.group_net = 10;
    b.spent = [3, 3];
    assert!(rank(&a, Profile::new(20, 10)?)? < rank(&b, Profile::new(0, 0)?)?);
    b.spent = a.spent;
    b.delivery_round = None;
    assert!(rank(&a, Profile::new(20, 10)?)? < rank(&b, Profile::new(0, 0)?)?);
    b.delivery_round = a.delivery_round;
    assert!(rank(&b, Profile::new(2, 10)?)? < rank(&a, Profile::new(10, 2)?)?);
    Ok(())
}

#[test]
fn atomic_failed_reservations_do_not_consume_any_resource() -> Result<(), Error> {
    use super::super::budget::Budget;
    let limits = Limits {
        profiles: 2,
        transitions: 2,
        allocation_bytes: 2000,
    };
    let mut budget = Budget::new(&limits)?;
    let before = budget.stats.clone();
    for (p, t, retained, scratch) in [
        (3, 0, 0, 0),
        (1, 3, 0, 0),
        (1, 1, 406, 0),
        (1, 1, 0, 406),
        (u64::MAX, 0, u64::MAX, 0),
    ] {
        assert!(budget.reserve(p, t, retained, scratch).is_err());
        assert_eq!(budget.stats, before);
    }
    budget.reserve(2, 2, 405, 0)?;
    assert_eq!(
        budget.stats,
        CompileStats {
            profiles: 2,
            transitions: 2,
            peak_bytes: 2000
        }
    );
    Ok(())
}
#[test]
fn releasing_episode_scratch_keeps_cumulative_work_and_peak() -> Result<(), Error> {
    use super::super::budget::Budget;
    let mut budget = Budget::new(&Limits {
        profiles: 2,
        transitions: 3,
        allocation_bytes: 5000,
    })?;
    budget.reserve(1, 2, 10, 2000)?;
    budget.release_episode();
    budget.reserve(1, 1, 10, 100)?;
    assert_eq!(
        budget.stats,
        CompileStats {
            profiles: 2,
            transitions: 3,
            peak_bytes: 3605
        }
    );
    assert!(budget.begin_profile().is_err());
    Ok(())
}
#[test]
fn budgeted_kernel_counts_early_delivery_and_rejects_before_excess_event() -> Result<(), Error> {
    use super::super::{budget::Budget, kernel::run_budgeted};
    let cfg = Config {
        stratum: Stratum::new(1, 3, 0)?,
        ids: Ids::default(),
    };
    for (p, count) in [(Profile::new(540, 60)?, 13), (Profile::new(0, 0)?, 19)] {
        let mut limits = Limits::registered()?;
        limits.transitions = count;
        let mut b = Budget::new(&limits)?;
        b.begin_profile()?;
        let actual = run_budgeted(&cfg, p, &mut b)?;
        assert_eq!(actual, run_episode(&cfg, p)?);
        assert_eq!(b.stats.transitions, count);
        assert_eq!(
            super::super::canonical::trusted_trace_hash(&actual)?,
            trace_hash(&actual)?
        );
        limits.transitions = count - 1;
        let mut b = Budget::new(&limits)?;
        b.begin_profile()?;
        assert!(
            matches!(run_budgeted(&cfg,p,&mut b),Err(Error::ResourceLimit{resource,..}) if resource=="transitions")
        );
        assert_eq!(b.stats.transitions, count - 1);
    }
    Ok(())
}
#[test]
fn compile_caps_fail_instead_of_returning_partial_rows() -> Result<(), Error> {
    let s = Stratum::new(1, 3, 0)?;
    for limits in [
        Limits {
            profiles: 0,
            transitions: 100,
            allocation_bytes: 100000,
        },
        Limits {
            profiles: 1,
            transitions: 100,
            allocation_bytes: 100000,
        },
        Limits {
            profiles: 100,
            transitions: 0,
            allocation_bytes: 100000,
        },
        Limits {
            profiles: 100,
            transitions: 100,
            allocation_bytes: 1594,
        },
    ] {
        assert!(matches!(
            compile(&s, &limits),
            Err(Error::ResourceLimit { .. })
        ));
    }
    Ok(())
}
#[test]
fn trusted_hash_matches_every_approved_witness_and_public_rejects_corruption() -> Result<(), Error>
{
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/kernel-reference.json")).unwrap();
    for w in fixture["witnesses"].as_array().unwrap() {
        let expected: Episode = serde_json::from_value(w["episode"].clone()).unwrap();
        let mut e = run_episode(&expected.config, expected.profile)?;
        assert_eq!(
            super::super::canonical::native_trace_bytes(&e)?,
            super::super::canonical::canonical_bytes(&e)?
        );
        assert_eq!(
            super::super::canonical::trusted_trace_hash(&e)?,
            w["trace_hash"].as_str().unwrap()
        );
        e.events.pop();
        assert!(trace_hash(&e).is_err());
    }
    Ok(())
}
#[test]
fn game_wire_rejects_positional_duplicate_missing_and_coerced_fields() {
    for input in [
        r#"[1,2,3]"#,
        r#"{"profiles":1,"transitions":2,"allocation_bytes":3,"profiles":1}"#,
        r#"{"profiles":1.0,"transitions":2,"allocation_bytes":3}"#,
        r#"{"profiles":1,"transitions":2}"#,
    ] {
        assert!(serde_json::from_str::<Limits>(input).is_err());
    }
    assert!(serde_json::from_str::<Selector>(r#"{"SoloAttempt":null}"#).is_err());
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/game-reference.json")).unwrap();
    let mut a = fixture["panels"][0]["analysis"].clone();
    a.as_object_mut().unwrap().remove("lex_pure");
    assert!(serde_json::from_value::<Analysis>(a).is_err());
}
#[test]
fn retained_output_and_episode_scratch_are_simultaneously_charged() -> Result<(), Error> {
    use super::super::budget::Budget;
    let mut b = Budget::new(&Limits {
        profiles: 1,
        transitions: 1,
        allocation_bytes: 1695,
    })?;
    b.reserve(1, 1, 0, 100)?;
    assert!(b.retain(1).is_err());
    assert_eq!(b.stats.peak_bytes, 1695);
    b.release_episode();
    b.retain(100)?;
    assert!(b.retain(1).is_err());
    Ok(())
}
#[test]
fn compact_reference_near_default_compile_is_complete() -> Result<(), Error> {
    let game: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/game-reference.json")).unwrap();
    let limits: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/limits-reference.json")).unwrap();
    let t = compile(&Stratum::new(1, 3, 0)?, &Limits::registered()?)?;
    assert_eq!(t.rows.len(), 46656);
    assert_eq!(
        serde_json::to_value(&t.analysis).unwrap(),
        game["panels"][2]["analysis"]
    );
    assert_eq!(
        serde_json::to_value(&t.stats).unwrap(),
        limits["panels"][2]["stats"]
    );
    assert_eq!(
        serde_json::to_value(equilibrium_summary(&t)?).unwrap(),
        game["panels"][2]["equilibrium"]
    );
    for control in game["panels"][2]["controls"].as_array().unwrap() {
        let selector = serde_json::from_value(control["selector"].clone()).unwrap();
        let p = resolve(&t, selector);
        assert_eq!(serde_json::to_value(p).unwrap(), control["profile"]);
        assert_eq!(
            serde_json::to_value(p.map(|p| classify(&t, p)).transpose()?).unwrap(),
            control["classification"]
        );
    }
    Ok(())
}
#[test]
fn nonzero_equilibria_retain_distinct_role_values() -> Result<(), Error> {
    let responses = response_sets(2, 2, &[[1, 5], [-2, 3], [-1, -1], [0, 0]])?;
    assert_eq!(responses.holder_best, vec![1, 0]);
    assert_eq!(responses.retriever_best, vec![5, 0]);
    assert_eq!(
        responses.pure,
        vec![Profile::new(0, 0)?, Profile::new(1, 1)?]
    );
    Ok(())
}
