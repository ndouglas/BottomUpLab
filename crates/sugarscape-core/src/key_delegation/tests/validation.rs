use super::*;
#[test]
fn strict_profile_atoms_and_duplicate_fields_reject() {
    for raw in [
        r#"{"giver":false,"receiver":0}"#,
        r#"{"giver":1.0,"receiver":0}"#,
        r#"{"giver":"1","receiver":0}"#,
        r#"{"giver":-0,"receiver":0}"#,
        r#"{"giver":-1,"receiver":0}"#,
        r#"{"giver":1e0,"receiver":0}"#,
        r#"{"giver":65536,"receiver":0}"#,
        r#"{"giver":0,"giver":1,"receiver":0}"#,
        r#"{"giver":0,"\u0067iver":1,"receiver":0}"#,
        r#"[0,0]"#,
        r#"{"giver":729,"receiver":0}"#,
        r#"{"giver":0,"receiver":64}"#,
    ] {
        assert!(serde_json::from_str::<Profile>(raw).is_err(), "{raw}");
    }
}
#[test]
fn complete_episode_rejects_forged_receipts_and_null_omission() {
    let e = run_episode(&config(), Profile::new(486, 60).unwrap()).unwrap();
    let mut v = serde_json::to_value(&e).unwrap();
    v["histories"][1]
        .as_array_mut()
        .unwrap()
        .retain(|entry| entry.get("Ownership").is_none());
    assert!(serde_json::from_value::<Episode>(v).is_err());
    let mut v = serde_json::to_value(&e).unwrap();
    v["histories"][0][0]["Public"]
        .as_object_mut()
        .unwrap()
        .remove("termination");
    assert!(serde_json::from_value::<Episode>(v).is_err());
    let raw = serde_json::to_string(&e).unwrap().replacen(
        "\"key_owner\":\"Giver\"",
        "\"key_owner\":\"Giver\",\"key_owner\":\"Receiver\"",
        1,
    );
    assert!(serde_json::from_str::<Episode>(&raw).is_err());
}
#[test]
fn tampered_typed_episode_cannot_be_hashed() {
    let mut e = run_episode(&config(), Profile::new(486, 60).unwrap()).unwrap();
    e.outcome.net[0] += 1;
    assert!(trace_hash(&e).is_err());
}

#[test]
fn strict_nested_schemas_reject_unknowns_arrays_and_false_ownership() {
    let e = run_episode(&config(), Profile::new(486, 60).unwrap()).unwrap();
    let original = serde_json::to_value(&e).unwrap();
    for path in [
        "role",
        "cell",
        "token",
        "extra",
        "two_owners",
        "missing_role",
        "unit_object",
        "struct_array",
        "negative_zero",
    ] {
        let mut v = original.clone();
        match path {
            "role" => v["events"][0]["kind"]["Action"]["role"] = "Helper".into(),
            "cell" => v["events"][0]["before"]["positions"][0] = "X".into(),
            "token" => {
                v["events"][0]["kind"]["Action"]["token"] = serde_json::json!({"Giver":"Hold"})
            }
            "extra" => v["histories"][0][0]["Public"]["extra"] = true.into(),
            "two_owners" => {
                v["histories"][0][1]["Observed"]["cells"][1]["occupants"][1]["key"] = true.into()
            }
            "missing_role" => {
                v["histories"][0][0]["Public"]["clock"]
                    .as_object_mut()
                    .unwrap()
                    .remove("role");
            }
            "unit_object" => v["config"]["initial_owner"] = serde_json::json!({"Giver":null}),
            "struct_array" => {
                v["histories"][0][0]["Public"] =
                    serde_json::json!([{"round":0,"phase":"Initial","role":null},null])
            }
            "negative_zero" => {
                let raw = serde_json::to_string(&v)
                    .unwrap()
                    .replacen("\"net\":2", "\"net\":-0", 1);
                assert!(serde_json::from_str::<Episode>(&raw).is_err());
                continue;
            }
            _ => unreachable!(),
        }
        assert!(serde_json::from_value::<Episode>(v).is_err(), "{path}");
    }
}

#[test]
fn choose_rejects_forged_local_ownership_and_passive_action_clock() {
    let mut before = None;
    run_with_capture(&config(), Profile::new(486, 60).unwrap(), &mut |c| {
        if c.clock.round == 1
            && c.clock.phase == Phase::BeforeAction
            && c.clock.role == Some(Role::Receiver)
        {
            before = Some(c.clone());
        }
        Ok(())
    })
    .unwrap();
    let c = before.unwrap();
    assert_eq!(choose(&c.views[1]).unwrap(), Intent::Move(Cell::C));
    assert!(choose(&c.views[0]).is_err());
    let mut forged = c.views[1].clone();
    forged.own_key = false;
    assert!(choose(&forged).is_err());
    let mut forged = c.views[1].clone();
    forged
        .history
        .retain(|entry| !matches!(entry, LocalEntry::Ownership(_)));
    assert!(choose(&forged).is_err());
}

#[test]
fn typed_nested_values_are_validated_without_wire_roundtrip() {
    let mut initial = None;
    run_with_capture(&config(), Profile::new(486, 60).unwrap(), &mut |c| {
        if c.clock.phase == Phase::Initial {
            initial = Some(c.clone());
        }
        Ok(())
    })
    .unwrap();
    let initial = initial.unwrap();
    let mut observation = initial.views[0].last_observation.clone();
    observation.cells[1].occupants[0].id.clear();
    assert!(observation.validate().is_err());
    let mut observation = initial.views[0].last_observation.clone();
    observation.cells[1].occupants.reverse();
    assert!(observation.validate().is_err());
}

fn registered_case(name: &str) -> serde_json::Value {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/kernel-reference.json")).unwrap();
    fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|case| case["name"] == name)
        .unwrap()
        .clone()
}
fn rejects_typed_and_wire<T: serde::Serialize + serde::de::DeserializeOwned>(
    value: &T,
    result: Result<(), Error>,
    label: &str,
) {
    let wire = serde_json::to_string(value).unwrap();
    assert!(result.is_err(), "typed accepted {label}: {wire}");
    assert!(
        serde_json::from_str::<T>(&wire).is_err(),
        "wire accepted {label}: {wire}"
    );
}
#[test]
fn review_r1_impossible_observations_reject_typed_and_wire() {
    let case = registered_case("all-wait");
    let initial: Observation =
        serde_json::from_value(case["boundaries"][0]["views"][0]["last_observation"].clone())
            .unwrap();
    for label in [
        "missing visible owner",
        "duplicate visible ID",
        "cargo plus loose resource",
    ] {
        let mut o = initial.clone();
        match label {
            "missing visible owner" => {
                for a in &mut o.cells[1].occupants {
                    a.key = false;
                }
            }
            "duplicate visible ID" => {
                o.cells[1].occupants[1].id = o.cells[1].occupants[0].id.clone()
            }
            "cargo plus loose resource" => o.cells[1].occupants[1].cargo = true,
            _ => unreachable!(),
        }
        rejects_typed_and_wire(&o, o.validate(), label);
    }
}
#[test]
fn review_r1_all_wait_keyless_receiver_cannot_choose() {
    let case = registered_case("all-wait");
    let boundary = case["boundaries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| {
            b["clock"] == serde_json::json!({"phase":"BeforeAction","role":"Receiver","round":1})
        })
        .unwrap();
    let mut view: View = serde_json::from_value(boundary["views"][1].clone()).unwrap();
    for entry in &mut view.history {
        if let LocalEntry::Observed(o) = entry {
            for c in &mut o.cells {
                for a in &mut c.occupants {
                    if a.role == Role::Giver {
                        a.key = false;
                    }
                }
            }
        }
    }
    for c in &mut view.last_observation.cells {
        for a in &mut c.occupants {
            if a.role == Role::Giver {
                a.key = false;
            }
        }
    }
    assert!(choose(&view).is_err());
    rejects_typed_and_wire(&view, view.validate(), "all-Wait keyless Receiver View");
}
#[test]
fn review_r1_initial_peer_and_resource_constraints_reject() {
    let case = registered_case("all-wait");
    let initial: View = serde_json::from_value(case["boundaries"][0]["views"][0].clone()).unwrap();
    for label in [
        "initial peer at cache",
        "initial peer cargo",
        "missing initial cache",
    ] {
        let mut v = initial.clone();
        let LocalEntry::Observed(o) = &mut v.history[1] else {
            unreachable!()
        };
        match label {
            "initial peer at cache" => {
                let receiver = o.cells[1].occupants.pop().unwrap();
                o.cells[2].occupants.push(receiver);
            }
            "initial peer cargo" => {
                o.cells[1].occupants[1].cargo = true;
                o.cells[2].loose_token = false;
            }
            "missing initial cache" => o.cells[2].loose_token = false,
            _ => unreachable!(),
        }
        v.last_observation = o.clone();
        rejects_typed_and_wire(&v, v.validate(), label);
    }
}
#[test]
fn review_r1_offscreen_owner_and_stale_passive_map_remain_valid() {
    let c = Config::new(
        Stratum::new(true, 1, 3, 1).unwrap(),
        Ids::default(),
        InitialOwner::Receiver,
    )
    .unwrap();
    let mut offscreen = None;
    run_with_capture(&c, Profile::new(0, 60).unwrap(), &mut |p| {
        if p.clock.round == 2
            && p.clock.phase == Phase::BeforeAction
            && p.clock.role == Some(Role::Giver)
        {
            offscreen = Some(p.views[0].last_observation.clone());
        }
        Ok(())
    })
    .unwrap();
    let o = offscreen.unwrap();
    assert_eq!(
        o.cells
            .iter()
            .flat_map(|c| &c.occupants)
            .filter(|a| a.key)
            .count(),
        0
    );
    assert!(o.validate().is_ok());
    assert_eq!(
        serde_json::from_str::<Observation>(&serde_json::to_string(&o).unwrap()).unwrap(),
        o
    );
    let case = registered_case("supplied-near");
    let passive: View = serde_json::from_value(case["boundaries"][2]["views"][1].clone()).unwrap();
    assert!(passive.own_key);
    assert_eq!(passive.last_observation.clock.phase, Phase::Initial);
    assert!(passive.validate().is_ok());
}
#[test]
fn review_r2_outcome_domain_and_counter_ledger_reject_mutations() {
    let case = registered_case("all-wait");
    let base: Outcome = serde_json::from_value(case["episode"]["outcome"].clone()).unwrap();
    for label in [
        "reward overflow",
        "giver spent 25",
        "receiver spent 7",
        "approach overflow",
        "pass overflow",
        "delivery too early",
        "unused credits",
        "net bounds",
        "group bounds",
        "blocked exceeds attempts",
        "blocked Wait",
        "too many actions",
        "approach formula",
        "pass formula",
        "giver spending formula",
        "receiver spending formula",
        "transfer formula",
        "delivery counter",
        "reward without delivery",
    ] {
        let mut o = base.clone();
        match label {
            "reward overflow" => {
                o.reward = [255, 255];
                o.net = [255, 255];
                o.group_net = 510;
            }
            "giver spent 25" => {
                o.spent[0] = 25;
                o.net[0] = -25;
                o.group_net = -25;
                o.unused_credits[0] = 7;
            }
            "receiver spent 7" => {
                o.spent[1] = 7;
                o.net[1] = -7;
                o.group_net = -7;
                o.unused_credits[1] = 25;
            }
            "approach overflow" => o.approach_spent = 255,
            "pass overflow" => o.pass_spent = 255,
            "delivery too early" => o.delivery_round = Some(1),
            "unused credits" => o.unused_credits[0] = 33,
            "net bounds" => o.net = [7, -7],
            "group bounds" => o.group_net = 13,
            "blocked exceeds attempts" => o.giver_blocked[2] = 1,
            "blocked Wait" => o.giver_blocked[0] = 1,
            "too many actions" => o.giver_attempts[1] = 1,
            "approach formula" => o.approach_spent = 1,
            "pass formula" => {
                o.giver_attempts = [5, 0, 1];
                o.giver_blocked[2] = 1;
                o.pass_spent = 3;
                o.spent[0] = 3;
                o.net[0] = -3;
                o.group_net = -3;
                o.unused_credits[0] = 29;
            }
            "giver spending formula" => {
                o.spent[0] = 1;
                o.net[0] = -1;
                o.group_net = -1;
                o.unused_credits[0] = 31;
            }
            "receiver spending formula" => {
                o.spent[1] = 1;
                o.net[1] = -1;
                o.group_net = -1;
                o.unused_credits[1] = 31;
            }
            "transfer formula" => o.transfer_count = 1,
            "delivery counter" => o.delivery_round = Some(6),
            "reward without delivery" => {
                o.reward = [3, 9];
                o.net = [3, 9];
                o.group_net = 12;
            }
            _ => unreachable!(),
        }
        rejects_typed_and_wire(&o, o.validate(), label);
    }
}
#[test]
fn review_r2_physical_role_bounds_and_reward_context_reject() {
    let case = registered_case("all-wait");
    let base: PhysicalState =
        serde_json::from_value(case["boundaries"][0]["physical"].clone()).unwrap();
    for label in [
        "giver spent 25",
        "receiver spent 7",
        "giver reward 7",
        "receiver reward 13",
        "unregistered share",
        "reward before delivery",
    ] {
        let mut s = base.clone();
        match label {
            "giver spent 25" => s.spent[0] = 25,
            "receiver spent 7" => s.spent[1] = 7,
            "giver reward 7" => s.reward = [7, 5],
            "receiver reward 13" => s.reward = [0, 13],
            "unregistered share" => {
                s.resource = ResourceLocation::Delivered;
                s.reward = [1, 11];
            }
            "reward before delivery" => s.reward = [3, 9],
            _ => unreachable!(),
        }
        rejects_typed_and_wire(&s, s.validate(), label);
    }
}

#[test]
fn review_r2_all_registered_partial_outcomes_remain_valid_including_unsettled_delivery() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/kernel-reference.json")).unwrap();
    let mut boundaries = 0;
    let mut unsettled = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let config: Config = serde_json::from_value(case["config"].clone()).unwrap();
        let profile: Profile = serde_json::from_value(case["profile"].clone()).unwrap();
        super::super::kernel::run_checked(
            &config,
            profile,
            &mut |e, _| {
                e.outcome.validate()?;
                let wire = serde_json::to_string(&e.outcome).unwrap();
                assert_eq!(serde_json::from_str::<Outcome>(&wire).unwrap(), e.outcome);
                boundaries += 1;
                if e.outcome.delivery_round.is_some() && e.outcome.reward == [0, 0] {
                    unsettled += 1;
                }
                Ok(())
            },
            &mut |c| {
                c.physical.validate()?;
                let wire = serde_json::to_string(&c.physical).unwrap();
                assert_eq!(
                    serde_json::from_str::<PhysicalState>(&wire).unwrap(),
                    c.physical
                );
                Ok(())
            },
        )
        .unwrap();
    }
    assert_eq!(boundaries, 304);
    assert!(unsettled > 0);
}

#[test]
fn review_r1_pickup_observation_cannot_duplicate_carried_resource() {
    let case = registered_case("supplied-near");
    let boundary = case["boundaries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|b| {
            b["clock"] == serde_json::json!({"phase":"AfterAction","role":"Receiver","round":2})
        })
        .unwrap();
    let mut observation: Observation =
        serde_json::from_value(boundary["views"][1]["last_observation"].clone()).unwrap();
    assert!(observation
        .cells
        .iter()
        .flat_map(|c| &c.occupants)
        .any(|a| a.cargo));
    observation
        .cells
        .iter_mut()
        .find(|c| c.cell == Cell::C)
        .unwrap()
        .loose_token = true;
    rejects_typed_and_wire(
        &observation,
        observation.validate(),
        "post-pickup cargo plus loose cache",
    );
}
#[test]
fn review_r2_registered_extreme_spending_is_valid() {
    let c = Config::new(
        Stratum::new(false, 4, 0, 0).unwrap(),
        Ids::default(),
        InitialOwner::Giver,
    )
    .unwrap();
    let episode = run_episode(&c, Profile::new(728, 63).unwrap()).unwrap();
    assert_eq!(episode.outcome.spent, [24, 6]);
    assert_eq!(episode.outcome.net, [-24, -6]);
    assert_eq!(episode.outcome.group_net, -30);
    assert_eq!(episode.outcome.unused_credits, [8, 26]);
    assert!(episode.outcome.validate().is_ok());
    assert_eq!(
        serde_json::from_str::<Outcome>(&serde_json::to_string(&episode.outcome).unwrap()).unwrap(),
        episode.outcome
    );
}
