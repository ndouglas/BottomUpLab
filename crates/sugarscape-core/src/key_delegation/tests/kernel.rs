use super::*;
#[test]
fn supplied_near_handoff_charges_sender_and_delivers_once() {
    let s = Stratum::new(true, 1, 3, 0).unwrap();
    let ids = Ids::new("Agent-A".into(), "Agent-B".into()).unwrap();
    let c = Config::new(s, ids, InitialOwner::Giver).unwrap();
    let e = run_episode(&c, Profile::new(486, 60).unwrap()).unwrap();
    assert_eq!(
        (e.outcome.spent, e.outcome.net, e.outcome.delivery_round),
        ([1, 4], [2, 5], Some(4))
    );
}

#[test]
fn every_registered_event_history_and_capture_matches_typed_reference() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/kernel-reference.json")).unwrap();
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 14);
    for case in cases {
        let config: Config = serde_json::from_value(case["config"].clone()).unwrap();
        let profile: Profile = serde_json::from_value(case["profile"].clone()).unwrap();
        let expected: Episode = serde_json::from_value(case["episode"].clone()).unwrap();
        let mut captures = Vec::new();
        let actual = run_with_capture(&config, profile, &mut |c| {
            captures.push(c.clone());
            Ok(())
        })
        .unwrap();
        assert_eq!(actual, expected, "{}", case["name"]);
        assert_eq!(trace_hash(&actual).unwrap(), case["trace_hash"]);
        let boundaries = case["boundaries"].as_array().unwrap();
        assert_eq!(captures.len(), boundaries.len());
        for (capture, boundary) in captures.iter().zip(boundaries) {
            let expected = CapturePoint {
                clock: serde_json::from_value(boundary["clock"].clone()).unwrap(),
                views: serde_json::from_value(boundary["views"].clone()).unwrap(),
                physical: serde_json::from_value(boundary["physical"].clone()).unwrap(),
            };
            assert_eq!(*capture, expected, "{}", case["name"]);
        }
    }
}

#[test]
fn prospective_boundaries_match_registered_payload_charges() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/kernel-reference.json")).unwrap();
    for case in fixture["cases"].as_array().unwrap() {
        let config: Config = serde_json::from_value(case["config"].clone()).unwrap();
        let profile: Profile = serde_json::from_value(case["profile"].clone()).unwrap();
        let mut boundaries = Vec::new();
        super::super::kernel::run_checked(
            &config,
            profile,
            &mut |e, n| {
                boundaries.push((e.clone(), n));
                Ok(())
            },
            &mut |_| Ok(()),
        )
        .unwrap();
        assert_eq!(
            boundaries.len(),
            case["boundaries"].as_array().unwrap().len()
        );
        for ((partial, added), expected) in boundaries
            .iter()
            .zip(case["boundaries"].as_array().unwrap())
        {
            assert_eq!(partial.events.len() as u64, expected["event_count"]);
            assert_eq!(
                partial.histories.each_ref().map(Vec::len),
                serde_json::from_value::<[usize; 2]>(expected["history_lengths"].clone()).unwrap()
            );
            let bytes = super::super::canonical::canonical_bytes(partial).unwrap();
            let receipt = serde_json::json!([{"path":"key-delegation-trace-v1","bytes":bytes.len(),"sha256":"0".repeat(64)}]);
            let scratch = 2 * bytes.len() + serde_json::to_vec(&receipt).unwrap().len() + 64;
            assert_eq!(
                scratch as u64, expected["scratch_bytes"],
                "{} at {}",
                case["name"], expected["clock"]
            );
            assert_eq!(
                *added,
                if ["AfterAction", "Finished"]
                    .contains(&expected["clock"]["phase"].as_str().unwrap())
                {
                    1
                } else {
                    0
                }
            );
        }
    }
}

#[test]
fn pass_receipts_precede_sender_sense_without_passive_refresh() {
    let mut captures = Vec::new();
    run_with_capture(&config(), Profile::new(486, 60).unwrap(), &mut |c| {
        captures.push(c.clone());
        Ok(())
    })
    .unwrap();
    let c = &captures[2];
    assert_eq!(c.physical.key_owner, Role::Receiver);
    assert!(!c.views[0].own_key && c.views[1].own_key);
    assert_eq!(
        c.views[1].last_observation,
        captures[0].views[1].last_observation
    );
    assert!(matches!(
        c.views[0].history[c.views[0].history.len() - 2],
        LocalEntry::Ownership(_)
    ));
    assert!(matches!(
        c.views[1].history.last(),
        Some(LocalEntry::Ownership(_))
    ));
    assert_eq!(c.views[1].credits, 32);
    let after_delivery = &captures[captures.len() - 2];
    assert_eq!(after_delivery.physical.reward, [0, 0]);
    let finished = captures.last().unwrap();
    assert_eq!(finished.physical.reward, [3, 9]);
    for i in 0..2 {
        assert_eq!(
            finished.views[i].last_observation,
            after_delivery.views[i].last_observation
        );
        assert_eq!(finished.views[i].own_reward, [3, 9][i]);
    }
}

#[test]
fn marker_numeric_unicode_and_escaped_ids_remain_opaque() {
    let baseline = run_episode(&config(), Profile::new(486, 60).unwrap()).unwrap();
    for (g, r) in [
        ("@T@", "@B@"),
        ("@B@", "@T@"),
        ("001", "-0"),
        ("é\"\\\n😀", "e\u{301}"),
        ("Z", "A"),
        (" ", "\t"),
    ] {
        let config = Config::new(
            config().stratum().clone(),
            Ids::new(g.into(), r.into()).unwrap(),
            InitialOwner::Giver,
        )
        .unwrap();
        let episode = run_episode(&config, baseline.profile).unwrap();
        assert_eq!(episode.outcome, baseline.outcome);
        assert_eq!(episode.words, baseline.words);
        let raw = serde_json::to_string(&episode).unwrap();
        assert_eq!(serde_json::from_str::<Episode>(&raw).unwrap(), episode);
        assert_ne!(
            trace_hash(&episode).unwrap(),
            trace_hash(&baseline).unwrap()
        );
        for (i, expected) in [(0, r), (1, g)] {
            let receipt = episode.histories[i]
                .iter()
                .find_map(|entry| {
                    if let LocalEntry::Ownership(o) = entry {
                        Some(o)
                    } else {
                        None
                    }
                })
                .unwrap();
            assert_eq!(receipt.counterpart, expected);
        }
        // Undo only the explicit Agent-name fields; every other trace field must agree.
        let mut normalized = episode.clone();
        normalized.config = baseline.config.clone();
        for (i, history) in normalized.histories.iter_mut().enumerate() {
            for entry in history {
                match entry {
                    LocalEntry::Observed(observation) => {
                        for cell in &mut observation.cells {
                            for occupant in &mut cell.occupants {
                                occupant.id = baseline.config.ids().get(occupant.role).into();
                            }
                        }
                    }
                    LocalEntry::Ownership(receipt) => {
                        receipt.counterpart = baseline
                            .config
                            .ids()
                            .get(if i == 0 { Role::Receiver } else { Role::Giver })
                            .into();
                    }
                    _ => {}
                }
            }
        }
        assert_eq!(normalized, baseline);
    }
}

#[test]
fn resource_and_capture_failures_abort_before_success() {
    let mut count = 0;
    let err = super::super::kernel::run_checked(
        &config(),
        Profile::new(486, 60).unwrap(),
        &mut |_, events| {
            if events > 0 {
                Err(Error::ResourceLimit {
                    resource: "transitions".into(),
                    limit: 0,
                })
            } else {
                Ok(())
            }
        },
        &mut |_| {
            count += 1;
            Ok(())
        },
    )
    .unwrap_err();
    assert!(matches!(err, Error::ResourceLimit { .. }));
    assert_eq!(count, 2);
    let mut count = 0;
    let err = run_with_capture(&config(), Profile::new(486, 60).unwrap(), &mut |_| {
        count += 1;
        if count == 3 {
            Err(Error::Io("capture stopped".into()))
        } else {
            Ok(())
        }
    })
    .unwrap_err();
    assert_eq!(err, Error::Io("capture stopped".into()));
    assert_eq!(count, 3);
}

#[test]
fn direct_primitives_check_physics_after_role_capability() {
    let state = PhysicalState {
        positions: [Cell::O, Cell::O],
        receiver_cargo: false,
        resource: ResourceLocation::Cache,
        key_owner: Role::Receiver,
        spent: [0; 2],
        reward: [0; 2],
    };
    for (cell, cargo, resource, intent, reason) in [
        (
            Cell::O,
            false,
            ResourceLocation::Cache,
            Intent::Pickup,
            BlockReason::WrongPosition,
        ),
        (
            Cell::C,
            true,
            ResourceLocation::Carried,
            Intent::Pickup,
            BlockReason::FullHands,
        ),
        (
            Cell::C,
            false,
            ResourceLocation::Delivered,
            Intent::Pickup,
            BlockReason::NoToken,
        ),
        (
            Cell::O,
            false,
            ResourceLocation::Cache,
            Intent::Deliver,
            BlockReason::EmptyHands,
        ),
        (
            Cell::C,
            false,
            ResourceLocation::Cache,
            Intent::Move(Cell::C),
            BlockReason::WrongPosition,
        ),
    ] {
        let mut before = state.clone();
        before.positions[1] = cell;
        before.receiver_cargo = cargo;
        before.resource = resource;
        let (after, result, paid) = super::super::kernel::action(
            &config(),
            &before,
            Role::Receiver,
            &Token::Receiver(ReceiverToken::Advance),
            &intent,
        )
        .unwrap();
        let mut expected = before;
        expected.spent[1] = 1;
        assert_eq!(
            (after, result, paid),
            (expected, ActionOutcome::Blocked(reason), 1)
        );
    }
    assert!(super::super::kernel::action(
        &config(),
        &state,
        Role::Giver,
        &Token::Giver(GiverToken::Approach),
        &Intent::Pickup
    )
    .is_err());
}
