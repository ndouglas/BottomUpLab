use super::*;
#[test]
fn synthetic_negative_ties_keep_all_private_best_responses() {
    let payoff = [[[-2, -2], [-2, -2]], [[-2, -2], [-2, -2]]];
    let a = super::super::game::test_matrix(&payoff);
    assert_eq!(a.pure, vec![(0, 0), (0, 1), (1, 0), (1, 1)]);
    assert_eq!(a.giver_best, vec![-2, -2]);
    assert_eq!(a.receiver_best, vec![-2, -2]);
}
#[test]
fn matching_pennies_has_no_pure_equilibrium() {
    let a = super::super::game::test_matrix(&[[[1, -1], [-1, 1]], [[-1, 1], [1, -1]]]);
    assert!(a.pure.is_empty());
}
#[test]
fn unexecuted_suffix_remains_a_distinct_strategy_and_can_activate() {
    let a = run_episode(&config(), Profile::new(486, 60).unwrap()).unwrap();
    let b = run_episode(&config(), Profile::new(487, 60).unwrap()).unwrap();
    assert_eq!(a.events, b.events);
    assert_ne!(trace_hash(&a).unwrap(), trace_hash(&b).unwrap());
    let a = run_episode(&config(), Profile::new(486, 15).unwrap()).unwrap();
    let b = run_episode(&config(), Profile::new(487, 15).unwrap()).unwrap();
    assert_eq!(b.outcome.net[0], a.outcome.net[0] - 1);
}
#[test]
fn registered_limits_and_zero_budgets() {
    let l = Limits::registered().unwrap();
    assert_eq!(
        (l.profiles, l.transitions, l.allocation_bytes),
        (93312, 1213056, 35939858)
    );
    for l in [
        Limits {
            profiles: 0,
            ..l.clone()
        },
        Limits {
            transitions: 0,
            ..l.clone()
        },
        Limits {
            allocation_bytes: 0,
            ..l.clone()
        },
    ] {
        assert!(matches!(
            compile(config().stratum().clone(), &l),
            Err(Error::ResourceLimit { .. })
        ));
    }
}
#[test]
fn public_choice_matches_every_reachable_native_action_in_reference_cases() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/kernel-reference.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let c: Config = serde_json::from_value(case["config"].clone()).unwrap();
        let p: Profile = serde_json::from_value(case["profile"].clone()).unwrap();
        let mut chosen = None;
        run_with_capture(&c, p, &mut |point| {
            if point.clock.phase == Phase::BeforeAction {
                let role = point.clock.role.unwrap();
                chosen = Some(choose(&point.views[role.index()])?);
            }
            if point.clock.phase == Phase::AfterAction {
                let role = point.clock.role.unwrap();
                let intent = point.views[role.index()]
                    .history
                    .iter()
                    .rev()
                    .find_map(|entry| {
                        if let LocalEntry::OwnAction { intent, .. } = entry {
                            Some(intent)
                        } else {
                            None
                        }
                    })
                    .unwrap();
                assert_eq!(chosen.as_ref(), Some(intent));
            }
            Ok(())
        })
        .unwrap();
    }
}
