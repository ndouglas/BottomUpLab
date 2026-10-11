mod full_reference;
mod game;
mod metrics;
use super::*;

fn config(k: u8, b: u8, d: u8) -> Config {
    Config {
        stratum: Stratum::new(k, b, d).unwrap(),
        ids: Ids::default(),
    }
}
fn episode(h: u16, r: u8) -> Episode {
    run_episode(&config(1, 3, 0), Profile::new(h, r).unwrap()).unwrap()
}
fn captures(cfg: &Config, p: Profile) -> (Episode, Vec<CapturePoint>) {
    let mut points = Vec::new();
    let e = run_with_capture(cfg, p, &mut |point| {
        points.push(point.clone());
        Ok(())
    })
    .unwrap();
    (e, points)
}

#[test]
fn supplied_near_matches_manual_physics() -> Result<(), Error> {
    let cfg = config(1, 3, 0);
    let e = run_episode(&cfg, Profile::new(540, 60)?)?;
    assert_eq!(e.outcome.delivery_round, Some(4));
    assert_eq!(e.outcome.spent, [2, 4]);
    assert_eq!(e.outcome.reward, [3, 9]);
    assert_eq!(e.outcome.net, [1, 5]);
    let solo = run_episode(&cfg, Profile::new(0, 63)?)?;
    assert_eq!(solo.outcome.delivery_round, None);
    assert_eq!(solo.outcome.spent, [0, 6]);
    Ok(())
}
#[test]
fn hold_away_from_plate_is_charged_and_blocked() {
    let e = run_episode(&config(4, 6, 1), Profile::new(728, 63).unwrap()).unwrap();
    assert_eq!(e.outcome.spent, [24, 6]);
    assert_eq!(e.outcome.holder_blocked, [0, 0, 6]);
    assert!(e.events.iter().any(|event| matches!(
        event.kind,
        EventKind::Action {
            result: ActionOutcome::Blocked(BlockReason::AwayFromPlate),
            paid: 4,
            ..
        }
    )));
}
#[test]
fn travel_from_plate_is_charged_and_blocked() {
    let e = episode(364, 63);
    assert_eq!(e.outcome.spent, [6, 6]);
    assert_eq!(e.outcome.holder_blocked, [0, 6, 0]);
    assert!(e.events.iter().any(|event| matches!(
        event.kind,
        EventKind::Action {
            result: ActionOutcome::Blocked(BlockReason::AlreadyAtPlate),
            ..
        }
    )));
}
#[test]
fn cargo_survives_round_closure() {
    let e = episode(540, 60);
    let event = e
        .events
        .iter()
        .find(|e| e.clock.round == 3 && e.clock.phase == Phase::Reset)
        .unwrap();
    assert!(event.after.retriever_cargo);
    assert_eq!(event.after.resource, ResourceLocation::Carried);
    assert_eq!(event.after.positions[1], Cell::C);
    assert!(!event.after.gate_open);
}
#[test]
fn shared_cell_has_no_collision_charge() {
    let (e, points) = captures(&config(1, 3, 1), Profile::new(0, 0).unwrap());
    assert_eq!(points[0].physical.positions, [Cell::O, Cell::O]);
    assert_eq!(e.outcome.spent, [0, 0]);
    assert_eq!(
        points[0].views[0].last_observation.cells[1].occupants,
        vec![(Role::Holder, false), (Role::Retriever, false)]
    );
}
#[test]
fn pickup_and_delivery_occur_once() {
    let e = episode(728, 63);
    assert_eq!(e.outcome.retriever_attempts, [1, 1, 1, 1]);
    assert_eq!(e.outcome.reward, [3, 9]);
    assert_eq!(
        e.events.last().unwrap().after.resource,
        ResourceLocation::Delivered
    );
}
#[test]
fn all_hold_terminates_after_four_rounds_without_suffix_charges() {
    let e = episode(728, 63);
    assert_eq!(e.outcome.spent, [4, 4]);
    assert_eq!(e.outcome.holder_attempts, [0, 0, 4]);
    assert_eq!(e.events.len(), 13);
}
#[test]
fn no_action_follows_terminal_and_suffix_identity_is_retained() {
    let e = episode(540, 60);
    let suffix = episode(546, 60);
    assert_eq!(e.events, suffix.events);
    assert_eq!(e.histories, suffix.histories);
    assert_ne!(trace_hash(&e).unwrap(), trace_hash(&suffix).unwrap());
    assert!(matches!(
        e.events.last().unwrap().kind,
        EventKind::Terminal(Termination::Delivered)
    ));
    assert!(e.events.iter().all(|e| e.clock.round <= 4));
}
#[test]
fn capture_is_read_only_and_passive_observations_remain_stale_at_every_clock() {
    let (e, points) = captures(&config(1, 3, 0), Profile::new(560, 60).unwrap());
    assert_eq!(e, episode(560, 60));
    assert_eq!(points.len(), 22);
    for (index, point) in points.iter().enumerate() {
        for role in [Role::Holder, Role::Retriever] {
            let i = role.index();
            let view = &point.views[i];
            assert_eq!(view.clock, point.clock);
            let observed = view
                .history
                .iter()
                .filter_map(|e| match e {
                    LocalEntry::Observed(o) => Some(o),
                    _ => None,
                })
                .next_back()
                .unwrap();
            assert_eq!(&view.last_observation, observed);
            if index > 0
                && !(point.clock.role == Some(role)
                    && matches!(point.clock.phase, Phase::BeforeAction | Phase::AfterAction))
            {
                assert_eq!(
                    view.last_observation,
                    points[index - 1].views[i].last_observation
                );
            }
            for entry in &view.history {
                if let LocalEntry::OwnAction { clock, token, .. } = entry {
                    assert_eq!(clock.role, Some(role));
                    assert!(matches!(
                        (role, token),
                        (Role::Holder, Token::Holder(_)) | (Role::Retriever, Token::Retriever(_))
                    ));
                }
            }
        }
    }
    let finished = points.last().unwrap();
    assert!(!finished.physical.gate_open);
    assert_eq!(finished.views[1].last_observation.gate, Some(true));
    assert_eq!(finished.views[1].own_reward, 9);
}
#[test]
fn controller_uses_only_own_word_and_state() {
    let (_, points) = captures(&config(1, 3, 0), Profile::new(540, 60).unwrap());
    let own = points
        .iter()
        .find(|p| p.clock.phase == Phase::BeforeAction && p.clock.role == Some(Role::Retriever))
        .unwrap()
        .views[1]
        .clone();
    assert_eq!(choose(&own).unwrap(), Primitive::Move(Cell::C));
    let mut altered = own.clone();
    for entry in &mut altered.history {
        if let LocalEntry::Observed(o) = entry {
            for cell in &mut o.cells {
                cell.occupants.retain(|(r, _)| *r != Role::Holder);
            }
            o.gate = Some(false);
        }
    }
    altered.last_observation = altered
        .history
        .iter()
        .filter_map(|e| {
            if let LocalEntry::Observed(o) = e {
                Some(o.clone())
            } else {
                None
            }
        })
        .next_back()
        .unwrap();
    assert_eq!(choose(&altered).unwrap(), choose(&own).unwrap());
    altered.own_cell = Cell::P;
    assert!(choose(&altered).is_err());
    altered = own;
    altered.word = OwnWord::Holder(HolderWord::from_id(0).unwrap());
    assert!(choose(&altered).is_err());
}
#[test]
fn controller_rejects_nonactive_and_finished_clocks() {
    let (_, points) = captures(&config(1, 3, 0), Profile::new(540, 60).unwrap());
    for point in points {
        for view in point.views {
            if point.clock.phase != Phase::BeforeAction || point.clock.role != Some(view.role) {
                assert!(choose(&view).is_err());
            }
        }
    }
}
#[test]
fn capture_failures_are_operational_errors() {
    let e = run_with_capture(&config(1, 3, 0), Profile::new(0, 0).unwrap(), &mut |_| {
        Err(Error::Io("capture failed".into()))
    });
    assert!(matches!(e, Err(Error::Io(_))));
}
#[test]
fn role_renaming_preserves_physics_but_changes_complete_identity() {
    let original = episode(540, 60);
    let mut cfg = original.config.clone();
    cfg.ids = Ids {
        holder: "Z-holder".into(),
        retriever: "A-retriever".into(),
    };
    let renamed = run_episode(&cfg, original.profile).unwrap();
    assert_eq!(renamed.events, original.events);
    assert_eq!(renamed.histories, original.histories);
    assert_eq!(renamed.outcome, original.outcome);
    assert_ne!(
        trace_hash(&renamed).unwrap(),
        trace_hash(&original).unwrap()
    );
}
#[test]
fn all_word_ids_round_trip_with_six_most_significant_first_tokens() {
    for id in 0..729 {
        let word = HolderWord::from_id(id).unwrap();
        assert_eq!(word.id(), id);
        assert_eq!(
            serde_json::to_value(word)
                .unwrap()
                .as_array()
                .unwrap()
                .len(),
            6
        );
    }
    for id in 0..64 {
        let word = RetrieverWord::from_id(id).unwrap();
        assert_eq!(word.id(), id);
    }
    assert_eq!(
        serde_json::to_value(HolderWord::from_id(540).unwrap()).unwrap(),
        serde_json::json!(["Hold", "Wait", "Hold", "Wait", "Wait", "Wait"])
    );
    assert_eq!(
        serde_json::to_value(RetrieverWord::from_id(60).unwrap()).unwrap(),
        serde_json::json!(["Advance", "Advance", "Advance", "Advance", "Wait", "Wait"])
    );
}
#[test]
fn invalid_strata_labels_and_raw_word_ids_reject() {
    for (k, b, d) in [(0, 3, 0), (3, 3, 0), (1, 1, 0), (1, 3, 2)] {
        assert!(Stratum::new(k, b, d).is_err());
    }
    assert!(Profile::new(729, 0).is_err());
    assert!(Profile::new(0, 64).is_err());
    assert!(HolderWord::from_id(729).is_err());
    assert!(RetrieverWord::from_id(64).is_err());
    for ids in [
        Ids {
            holder: "".into(),
            retriever: "B".into(),
        },
        Ids {
            holder: "A".into(),
            retriever: "A".into(),
        },
    ] {
        let mut cfg = config(1, 3, 0);
        cfg.ids = ids;
        assert!(cfg.validate().is_err());
    }
}
#[test]
fn strict_serialized_inputs_reject_coercions_unknowns_duplicates_and_bounds() {
    for input in [
        r#"{"holder":729,"retriever":0}"#,
        r#"{"holder":0,"retriever":64}"#,
        r#"{"holder":1.0,"retriever":0}"#,
        r#"{"holder":1e0,"retriever":0}"#,
        r#"{"holder":true,"retriever":0}"#,
        r#"{"holder":"1","retriever":0}"#,
        r#"{"holder":0,"holder":1,"retriever":0}"#,
        r#"{"holder":0,"retriever":0,"extra":0}"#,
    ] {
        assert!(serde_json::from_str::<Profile>(input).is_err(), "{input}");
    }
    for input in [
        r#"{"holding_cost":3,"holder_share":3,"distance":0}"#,
        r#"{"holding_cost":1,"holder_share":4,"distance":0}"#,
        r#"{"holding_cost":1,"holder_share":3,"distance":2}"#,
    ] {
        assert!(serde_json::from_str::<Stratum>(input).is_err());
    }
    for input in [
        r#"["Wait"]"#,
        r#"["Wait","Wait","Wait","Wait","Wait","Wait","Wait"]"#,
        r#"["Wait","Wait","Wait","Wait","Wait","Advance"]"#,
    ] {
        assert!(serde_json::from_str::<HolderWord>(input).is_err());
    }
    for input in [
        r#"{"round":0,"phase":"Reset","role":null}"#,
        r#"{"round":7,"phase":"Finished","role":null}"#,
        r#"{"round":1,"phase":"BeforeAction","role":null}"#,
        r#"{"round":1,"phase":"Reset"}"#,
    ] {
        assert!(serde_json::from_str::<Clock>(input).is_err(), "{input}");
    }
    assert!(serde_json::from_str::<Ids>(r#"{"holder":"\ud800","retriever":"B"}"#).is_err());
}
#[test]
fn strict_episode_validation_rejects_missing_nullable_fields_and_corrupt_history() {
    let e = episode(540, 60);
    let mut v = serde_json::to_value(&e).unwrap();
    v["outcome"]
        .as_object_mut()
        .unwrap()
        .remove("delivery_round");
    assert!(serde_json::from_value::<Episode>(v).is_err());
    let mut v = serde_json::to_value(&e).unwrap();
    v["histories"][0][0]["Public"]
        .as_object_mut()
        .unwrap()
        .remove("termination");
    assert!(serde_json::from_value::<Episode>(v).is_err());
    let wire = serde_json::to_string(&e).unwrap().replacen(
        "\"version\":1",
        "\"version\":1,\"version\":1",
        1,
    );
    assert!(serde_json::from_str::<Episode>(&wire).is_err());
    let mut corrupt = e;
    corrupt.histories[0].pop();
    assert!(trace_hash(&corrupt).is_err());
}
#[test]
fn all_ten_independent_witnesses_match_complete_typed_episodes_bytes_and_hashes() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/kernel-reference.json")).unwrap();
    let witnesses = fixture["witnesses"].as_array().unwrap();
    assert_eq!(witnesses.len(), 10);
    for witness in witnesses {
        let expected: Episode = serde_json::from_value(witness["episode"].clone()).unwrap();
        let actual = run_episode(&expected.config, expected.profile).unwrap();
        assert_eq!(actual, expected, "{}", witness["name"]);
        let bytes = canonical::canonical_bytes(&actual).unwrap();
        assert_eq!(
            bytes,
            serde_json::to_vec(&witness["episode"]).unwrap(),
            "{}",
            witness["name"]
        );
        assert_eq!(
            trace_hash(&actual).unwrap(),
            witness["trace_hash"].as_str().unwrap(),
            "{}",
            witness["name"]
        );
    }
}

#[test]
fn standalone_local_entries_reject_invalid_roles_clocks_and_settlements() {
    for wire in [
        r#"{"Public":{"clock":{"round":1,"phase":"Reset","role":null},"termination":"Delivered"}}"#,
        r#"{"OwnAction":{"clock":{"round":1,"phase":"AfterAction","role":"Holder"},"token":{"Retriever":"Advance"},"intent":{"Move":"C"},"result":"Success","paid":1}}"#,
        r#"{"Settlement":{"clock":{"round":1,"phase":"Reset","role":null},"reward":3}}"#,
        r#"{"Settlement":{"clock":{"round":4,"phase":"Finished","role":null},"reward":255}}"#,
    ] {
        assert!(serde_json::from_str::<LocalEntry>(wire).is_err(), "{wire}");
    }
}
#[test]
fn standalone_observations_reject_duplicate_roles_across_cells() {
    let (_, points) = captures(&config(1, 3, 0), Profile::new(0, 0).unwrap());
    let mut observation = points[0].views[1].last_observation.clone();
    observation.cells[2]
        .occupants
        .push((Role::Retriever, false));
    assert!(
        serde_json::from_value::<Observation>(serde_json::to_value(observation).unwrap()).is_err()
    );
}
#[test]
fn standalone_events_reject_physics_inconsistent_with_their_intent() {
    let mut event = episode(540, 60).events[0].clone();
    event.after.spent[0] = 1;
    assert!(serde_json::from_value::<Event>(serde_json::to_value(event).unwrap()).is_err());
    let mut event = episode(540, 60).events[1].clone();
    event.after.gate_open = false;
    assert!(serde_json::from_value::<Event>(serde_json::to_value(event).unwrap()).is_err());
}
#[test]
fn every_capture_round_trips_through_strict_typed_wire() {
    for (cfg, p) in [
        (config(1, 3, 0), Profile::new(560, 60).unwrap()),
        (config(4, 0, 1), Profile::new(728, 63).unwrap()),
        (config(2, 6, 1), Profile::new(423, 30).unwrap()),
    ] {
        let (_, points) = captures(&cfg, p);
        for point in points {
            assert_eq!(
                serde_json::from_str::<CapturePoint>(&serde_json::to_string(&point).unwrap())
                    .unwrap(),
                point
            );
        }
    }
}
#[test]
fn canonical_numbers_are_integer_atoms_with_zero_and_null_preserved() {
    assert!(canonical::canonical_bytes(&serde_json::json!({"n":1.0})).is_err());
    assert!(canonical::canonical_bytes(&serde_json::json!({"n":1e20})).is_err());
    assert_eq!(
        canonical::canonical_bytes(&serde_json::json!({"z":0,"a":null})).unwrap(),
        b"{\"a\":null,\"z\":0}"
    );
    let e = episode(0, 0);
    assert_eq!(e.outcome.net, [0, 0]);
    assert_eq!(e.outcome.delivery_round, None);
}
#[test]
fn native_wire_rejects_nested_duplicates_unknowns_and_missing_nulls() {
    let e = episode(540, 60);
    let wire = serde_json::to_string(&e).unwrap();
    for altered in [
        wire.replacen("\"positions\":", "\"extra\":0,\"positions\":", 1),
        wire.replacen("\"paid\":1", "\"paid\":1,\"paid\":1", 1),
        wire.replacen("\"gate\":null", "\"gate\":null,\"gate\":null", 1),
        wire.replacen(",\"gate\":null", "", 1),
    ] {
        assert!(serde_json::from_str::<Episode>(&altered).is_err());
    }
}

#[test]
fn native_unit_variants_require_strings_instead_of_null_payload_objects() {
    assert!(serde_json::from_str::<Role>(r#"{"Holder":null}"#).is_err());
    assert!(serde_json::from_str::<Primitive>(r#"{"Wait":null}"#).is_err());
    assert!(serde_json::from_str::<ActionOutcome>(r#"{"Success":null}"#).is_err());
    assert!(serde_json::from_str::<EventKind>(r#"{"Reset":null}"#).is_err());
}

#[test]
fn retriever_own_history_rejects_action_opportunities_after_successful_delivery() {
    let (_, points) = captures(&config(1, 3, 0), Profile::new(728, 63).unwrap());
    let mut view = points
        .iter()
        .find(|p| {
            p.clock.round == 4
                && p.clock.phase == Phase::AfterAction
                && p.clock.role == Some(Role::Retriever)
        })
        .unwrap()
        .views[1]
        .clone();
    for (phase, role) in [
        (Phase::Reset, None),
        (Phase::BeforeAction, Some(Role::Holder)),
        (Phase::AfterAction, Some(Role::Holder)),
        (Phase::BeforeAction, Some(Role::Retriever)),
    ] {
        view.history.push(LocalEntry::Public {
            clock: Clock {
                round: 5,
                phase,
                role,
            },
            termination: None,
        });
    }
    view.clock = Clock {
        round: 5,
        phase: Phase::BeforeAction,
        role: Some(Role::Retriever),
    };
    view.last_observation.clock = view.clock;
    view.history
        .push(LocalEntry::Observed(view.last_observation.clone()));
    assert!(choose(&view).is_err());
}
#[test]
fn retriever_own_history_rejects_delivery_settlement_without_own_delivery() {
    let (_, points) = captures(&config(1, 3, 0), Profile::new(0, 0).unwrap());
    let mut view = points.last().unwrap().views[1].clone();
    for entry in &mut view.history {
        match entry {
            LocalEntry::Public { clock, termination } if clock.phase == Phase::Finished => {
                *termination = Some(Termination::Delivered)
            }
            LocalEntry::Settlement { reward, .. } => *reward = 9,
            _ => {}
        }
    }
    view.own_reward = 9;
    assert!(view.validate().is_err());
}

fn positional_fields(value: &impl serde::Serialize, fields: &[&str]) -> serde_json::Value {
    let object = serde_json::to_value(value).unwrap();
    serde_json::Value::Array(fields.iter().map(|field| object[*field].clone()).collect())
}
fn rejects_positional_object<T: serde::Serialize + serde::de::DeserializeOwned>(
    value: &T,
    fields: &[&str],
) {
    let array = positional_fields(value, fields);
    assert!(serde_json::from_str::<T>(&serde_json::to_string(&array).unwrap()).is_err());
    assert!(serde_json::from_value::<T>(array).is_err());
}
#[test]
fn review_native_objects_reject_positional_arrays() {
    let (e, points) = captures(&config(1, 3, 0), Profile::new(540, 60).unwrap());
    rejects_positional_object(&e.profile, &["holder", "retriever"]);
    rejects_positional_object(
        &e.config.stratum,
        &["holding_cost", "holder_share", "distance"],
    );
    rejects_positional_object(&e.config.ids, &["holder", "retriever"]);
    rejects_positional_object(&e.config, &["stratum", "ids"]);
    rejects_positional_object(&points[0].clock, &["round", "phase", "role"]);
    rejects_positional_object(
        &points[0].physical,
        &[
            "positions",
            "retriever_cargo",
            "resource",
            "gate_open",
            "spent",
            "reward",
        ],
    );
    rejects_positional_object(
        &points[0].views[0].last_observation.cells[0],
        &["cell", "occupants", "loose_token"],
    );
    rejects_positional_object(
        &points[0].views[0].last_observation,
        &["clock", "cells", "gate"],
    );
    rejects_positional_object(&e.events[0], &["clock", "kind", "before", "after"]);
    rejects_positional_object(
        &e.outcome,
        &[
            "spent",
            "reward",
            "net",
            "group_net",
            "delivery_round",
            "holder_attempts",
            "holder_blocked",
            "retriever_attempts",
            "retriever_blocked",
            "retriever_waits",
        ],
    );
    rejects_positional_object(
        &e,
        &[
            "version",
            "config",
            "profile",
            "events",
            "histories",
            "outcome",
        ],
    );
    rejects_positional_object(
        &points[0].views[0],
        &[
            "role",
            "word",
            "own_cell",
            "own_cargo",
            "credits",
            "own_reward",
            "clock",
            "history",
            "last_observation",
        ],
    );
    rejects_positional_object(&points[0], &["clock", "views", "physical"]);
}
#[test]
fn review_struct_variant_payloads_reject_positional_arrays() {
    let e = episode(540, 60);
    let action = serde_json::to_value(&e.events[1].kind).unwrap();
    let fields = positional_fields(
        &action["Action"],
        &["role", "token", "intent", "result", "paid"],
    );
    let altered = serde_json::json!({"Action":fields});
    assert!(serde_json::from_str::<EventKind>(&serde_json::to_string(&altered).unwrap()).is_err());
    assert!(serde_json::from_value::<EventKind>(altered).is_err());
    for entry in &e.histories[0] {
        let value = serde_json::to_value(entry).unwrap();
        for (variant, fields) in [
            ("Public", &["clock", "termination"][..]),
            (
                "OwnAction",
                &["clock", "token", "intent", "result", "paid"][..],
            ),
            ("Settlement", &["clock", "reward"][..]),
        ] {
            if let Some(payload) = value.get(variant) {
                let mut altered = value.clone();
                altered[variant] = positional_fields(payload, fields);
                assert!(
                    serde_json::from_str::<LocalEntry>(&serde_json::to_string(&altered).unwrap())
                        .is_err(),
                    "{variant}"
                );
                assert!(
                    serde_json::from_value::<LocalEntry>(altered).is_err(),
                    "{variant}"
                );
            }
        }
    }
}
#[test]
fn review_nested_episode_and_view_objects_reject_positional_arrays() {
    let (e, points) = captures(&config(1, 3, 0), Profile::new(540, 60).unwrap());
    let native = serde_json::to_value(&e).unwrap();
    for (path, fields) in [
        ("/config", &["stratum", "ids"][..]),
        ("/profile", &["holder", "retriever"][..]),
        ("/events/0/clock", &["round", "phase", "role"][..]),
        (
            "/events/0/before",
            &[
                "positions",
                "retriever_cargo",
                "resource",
                "gate_open",
                "spent",
                "reward",
            ][..],
        ),
        ("/histories/0/0/Public", &["clock", "termination"][..]),
    ] {
        let mut altered = native.clone();
        let array = positional_fields(altered.pointer(path).unwrap(), fields);
        *altered.pointer_mut(path).unwrap() = array;
        assert!(
            serde_json::from_str::<Episode>(&serde_json::to_string(&altered).unwrap()).is_err(),
            "{path}"
        );
        assert!(
            serde_json::from_value::<Episode>(altered).is_err(),
            "{path}"
        );
    }
    let native = serde_json::to_value(&points[0].views[0]).unwrap();
    for (path, fields) in [
        ("/clock", &["round", "phase", "role"][..]),
        ("/last_observation", &["clock", "cells", "gate"][..]),
        ("/history/0/Public", &["clock", "termination"][..]),
        (
            "/history/1/Observed/cells/0",
            &["cell", "occupants", "loose_token"][..],
        ),
    ] {
        let mut altered = native.clone();
        let array = positional_fields(altered.pointer(path).unwrap(), fields);
        *altered.pointer_mut(path).unwrap() = array;
        assert!(
            serde_json::from_str::<View>(&serde_json::to_string(&altered).unwrap()).is_err(),
            "{path}"
        );
        assert!(serde_json::from_value::<View>(altered).is_err(), "{path}");
    }
}
#[test]
fn review_intended_words_tuples_and_fixed_arrays_remain_native_arrays() {
    let word: HolderWord =
        serde_json::from_str(r#"["Hold","Wait","Hold","Wait","Wait","Wait"]"#).unwrap();
    assert_eq!(word.id(), 540);
    let word: RetrieverWord =
        serde_json::from_str(r#"["Advance","Advance","Advance","Advance","Wait","Wait"]"#).unwrap();
    assert_eq!(word.id(), 60);
    let cell: VisibleCell = serde_json::from_str(
        r#"{"cell":"O","occupants":[["Holder",false],["Retriever",false]],"loose_token":false}"#,
    )
    .unwrap();
    assert_eq!(
        cell.occupants,
        vec![(Role::Holder, false), (Role::Retriever, false)]
    );
    let state: PhysicalState = serde_json::from_str(r#"{"positions":["O","O"],"retriever_cargo":false,"resource":"Cache","gate_open":false,"spent":[0,0],"reward":[0,0]}"#).unwrap();
    assert_eq!(state.positions, [Cell::O, Cell::O]);
}
#[test]
fn review_crossing_success_rejects_fresh_closed_gate_observation() {
    let (_, points) = captures(&config(1, 3, 0), Profile::new(540, 60).unwrap());
    let mut view = points
        .iter()
        .find(|p| {
            p.clock.round == 1
                && p.clock.phase == Phase::AfterAction
                && p.clock.role == Some(Role::Retriever)
        })
        .unwrap()
        .views[1]
        .clone();
    for entry in &mut view.history {
        if let LocalEntry::Observed(o) = entry {
            if o.clock.round == 1 && o.clock.phase == Phase::BeforeAction {
                o.gate = Some(false);
            }
        }
    }
    assert!(view.validate().is_err());
    assert!(serde_json::from_value::<View>(serde_json::to_value(view).unwrap()).is_err());
}
#[test]
fn review_gate_closed_feedback_rejects_fresh_open_gate_observation() {
    let (_, points) = captures(&config(1, 3, 0), Profile::new(0, 63).unwrap());
    let mut view = points
        .iter()
        .find(|p| {
            p.clock.round == 1
                && p.clock.phase == Phase::AfterAction
                && p.clock.role == Some(Role::Retriever)
        })
        .unwrap()
        .views[1]
        .clone();
    for entry in &mut view.history {
        if let LocalEntry::Observed(o) = entry {
            if o.clock.round == 1 && o.clock.phase == Phase::BeforeAction {
                o.gate = Some(true);
            }
        }
    }
    assert!(view.validate().is_err());
    assert!(serde_json::from_value::<View>(serde_json::to_value(view).unwrap()).is_err());
}
#[test]
fn review_crossing_validation_uses_fresh_own_observation_instead_of_stale_gate() {
    // A round-one successful crossing has stale Initial=false and fresh Before=true.
    // A round-three blocked return has stale own round-two After=true and fresh Before=false.
    for (holder, round, fresh_gate) in [(540, 1, true), (648, 3, false)] {
        let (_, points) = captures(&config(1, 3, 0), Profile::new(holder, 63).unwrap());
        let view = &points
            .iter()
            .find(|p| {
                p.clock.round == round
                    && p.clock.phase == Phase::AfterAction
                    && p.clock.role == Some(Role::Retriever)
            })
            .unwrap()
            .views[1];
        let observed: Vec<_> = view
            .history
            .iter()
            .filter_map(|entry| {
                if let LocalEntry::Observed(o) = entry {
                    Some(o)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(observed[observed.len() - 2].gate, Some(fresh_gate));
        assert_eq!(observed[observed.len() - 3].gate, Some(!fresh_gate));
        assert!(view.validate().is_ok());
        assert_eq!(
            &serde_json::from_str::<View>(&serde_json::to_string(view).unwrap()).unwrap(),
            view
        );
    }
}
