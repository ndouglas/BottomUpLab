use super::super::*;

#[test]
fn helper_witnesses_match_declared_policy_original_ids() {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/planning-reference.json")).unwrap();
    for initial in reference["initial_values"].as_array().unwrap() {
        let protocol: Protocol = serde_json::from_value(initial["protocol"].clone()).unwrap();
        let prior = serde_json::from_value(initial["prior"].clone()).unwrap();
        let limits = Limits::registered().unwrap();
        let policy = CompiledPolicy::build(&protocol, prior, &limits).unwrap();
        let replay = Replay::build(&protocol, &policy, &limits).unwrap();
        for row in reference["prefix_witnesses"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                r["protocol"] == initial["protocol"]
                    && r["prior"] == initial["prior"]
                    && r["inference"] != "InvestigatorInference"
            })
        {
            let prefix: Prefix = serde_json::from_value(row["prefix"].clone()).unwrap();
            if row["belief"].is_null() {
                assert!(
                    matches!(replay.infer(&prefix), Err(Error::UnsupportedHistory { .. })),
                    "{}",
                    row["label"]
                );
            } else {
                assert_eq!(
                    serde_json::to_value(replay.infer(&prefix).unwrap()).unwrap(),
                    row["belief"],
                    "{}",
                    row["label"]
                );
                let support = replay.candidate_support(&prefix).unwrap();
                let mass = Probability::new(1, support.len() as u64).unwrap();
                let actual: Vec<_> = support
                    .iter()
                    .map(|id| serde_json::json!({"id":id,"mass":mass}))
                    .collect();
                assert_eq!(
                    serde_json::json!(actual),
                    row["candidate_support"],
                    "{}",
                    row["label"]
                );
            }
        }
    }
}

#[test]
fn investigator_intervention_is_supported_while_helper_replay_rejects_off_policy_round() {
    let protocol = Protocol::new(Role::A, PolicyKind::NoProbe);
    let limits = Limits::registered().unwrap();
    let policy = CompiledPolicy::build(&protocol, OwnPrior::Uniform, &limits).unwrap();
    let replay = Replay::build(&protocol, &policy, &limits).unwrap();
    let mut state = EpisodeState::new(
        &protocol,
        Environment::InFamily(Mechanism::SharedPersistent),
        EpisodeBits::from_index(0).unwrap(),
        [
            OwnPrior::Uniform,
            OwnPrior::PointMass(Mechanism::SharedPersistent),
        ],
    )
    .unwrap();
    select_choice(&mut state, Choice::Continue).unwrap();
    advance_one(&mut state).unwrap();
    assert!(policy.infer(state.prefix(Role::A)).is_ok());
    assert!(matches!(
        replay.infer(state.prefix(Role::B)),
        Err(Error::UnsupportedHistory { .. })
    ));
}

#[test]
fn released_workspace_preserves_peak_and_failed_reservation_is_atomic() {
    use super::super::budget::Budget;
    let mut budget = Budget::new(&Limits {
        states: 2,
        branches: 2,
        allocation_bytes: 10,
    })
    .unwrap();
    budget.allocate(8).unwrap();
    budget.release(5).unwrap();
    budget.allocate(6).unwrap();
    assert_eq!(budget.stats.allocation_bytes, 9);
    let before = budget.stats.clone();
    assert!(matches!(
        budget.allocate(2),
        Err(Error::ResourceLimit { .. })
    ));
    assert_eq!(budget.stats, before);
    budget.release(9).unwrap();
    assert_eq!(budget.stats.allocation_bytes, 9);
    assert!(matches!(budget.release(1), Err(Error::ArithmeticOverflow)));
    budget.allocate(10).unwrap();
}

#[test]
fn replay_and_hosts_share_cumulative_limits_and_computational_errors_stay_errors() {
    let protocol = Protocol::new(Role::A, PolicyKind::KnownRaw);
    let prior = OwnPrior::PointMass(Mechanism::Inert);
    let limits = Limits::registered().unwrap();
    let policy = CompiledPolicy::build(&protocol, prior, &limits).unwrap();
    let replay = Replay::build(&protocol, &policy, &limits).unwrap();
    let after_build = replay.stats();
    assert!(after_build.decision_states > policy.stats().decision_states);
    run_episode(
        &protocol,
        Environment::InFamily(Mechanism::Inert),
        0,
        &policy,
        &replay,
    )
    .unwrap();
    let after_first = replay.stats();
    run_episode(
        &protocol,
        Environment::InFamily(Mechanism::Inert),
        1,
        &policy,
        &replay,
    )
    .unwrap();
    let after_second = replay.stats();
    assert!(
        after_second.decision_states > after_first.decision_states
            && after_first.decision_states > after_build.decision_states
    );
    assert!(after_second.allocation_bytes > after_first.allocation_bytes);
    let restricted = Limits {
        states: after_build.decision_states + 1,
        ..limits
    };
    let policy = CompiledPolicy::build(&protocol, prior, &restricted).unwrap();
    let replay = Replay::build(&protocol, &policy, &restricted).unwrap();
    assert!(
        matches!(run_episode(&protocol,Environment::InFamily(Mechanism::Inert),0,&policy,&replay),Err(Error::ResourceLimit {resource,..}) if resource=="states")
    );
    assert_eq!(replay.stats().decision_states, restricted.states);
    let failed_peak = replay.stats().allocation_bytes;
    assert!(matches!(
        run_episode(
            &protocol,
            Environment::InFamily(Mechanism::Inert),
            1,
            &policy,
            &replay
        ),
        Err(Error::ResourceLimit { .. })
    ));
    assert_eq!(replay.stats().allocation_bytes, failed_peak);
}

#[test]
fn raw_dataflip_early_own_intervention_contradiction_keeps_its_original_prefix() {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/planning-reference.json")).unwrap();
    let row = reference["prefix_witnesses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| {
            r["label"]
                .as_str()
                .unwrap()
                .starts_with("DataFlip investigator-only")
        })
        .unwrap();
    let protocol: Protocol = serde_json::from_value(row["protocol"].clone()).unwrap();
    let prior = serde_json::from_value(row["prior"].clone()).unwrap();
    let prefix: Prefix = serde_json::from_value(row["prefix"].clone()).unwrap();
    let policy = CompiledPolicy::build(&protocol, prior, &Limits::registered().unwrap()).unwrap();
    assert!(prefix
        .entries
        .iter()
        .all(|e| !matches!(e, Entry::Physical(LocalEntry::PrivateBit { .. }))));
    assert_eq!(
        policy.infer(&prefix),
        Err(Error::UnsupportedHistory { prefix })
    );
}
