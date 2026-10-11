use super::super::*;

// Detect incorrect finite-horizon Bellman rewards, prices, or policy constraints.
#[test]
fn initial_values_match_independent_reference() {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/planning-reference.json")).unwrap();
    for row in reference["initial_values"].as_array().unwrap() {
        let protocol: Protocol = serde_json::from_value(row["protocol"].clone()).unwrap();
        let prior: OwnPrior = serde_json::from_value(row["prior"].clone()).unwrap();
        let expected: ScoreFraction = serde_json::from_value(row["value"].clone()).unwrap();
        let started = std::time::Instant::now();
        let policy =
            CompiledPolicy::build(&protocol, prior, &Limits::registered().unwrap()).unwrap();
        eprintln!(
            "planner engineering {:?} {:?} {:?}: {:?}, elapsed {:?}",
            protocol.investigator,
            protocol.policy,
            prior,
            policy.stats(),
            started.elapsed()
        );
        assert_eq!(
            policy.initial_value(),
            expected,
            "{:?} {:?} {:?}",
            protocol.investigator,
            protocol.policy,
            prior
        );
    }
}

// Complete independently generated alternatives catch marginal-only cache aliasing,
// changed tie order, hidden-A correlation loss, and forced-policy maximization.
#[test]
fn complete_investigator_witnesses_preserve_original_joint_mass_and_choices() {
    let reference: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/planning-reference.json")).unwrap();
    for initial in reference["initial_values"].as_array().unwrap() {
        let protocol: Protocol = serde_json::from_value(initial["protocol"].clone()).unwrap();
        let prior: OwnPrior = serde_json::from_value(initial["prior"].clone()).unwrap();
        let policy =
            CompiledPolicy::build(&protocol, prior, &Limits::registered().unwrap()).unwrap();
        for row in reference["prefix_witnesses"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| {
                r["protocol"] == initial["protocol"]
                    && r["prior"] == initial["prior"]
                    && r["inference"] == "InvestigatorInference"
            })
        {
            let prefix: Prefix = serde_json::from_value(row["prefix"].clone()).unwrap();
            if row["belief"].is_null() {
                assert!(
                    matches!(policy.infer(&prefix), Err(Error::UnsupportedHistory { .. })),
                    "{}",
                    row["label"]
                );
                continue;
            }
            let expected: Belief = serde_json::from_value(row["belief"].clone()).unwrap();
            assert_eq!(policy.infer(&prefix).unwrap(), expected, "{}", row["label"]);
            let support = policy.candidate_support(&prefix).unwrap();
            let mass = Probability::new(1, support.len() as u64).unwrap();
            let rows: Vec<_> = support
                .iter()
                .map(|id| serde_json::json!({"id":id,"mass":mass}))
                .collect();
            assert_eq!(
                serde_json::json!(rows),
                row["candidate_support"],
                "{}",
                row["label"]
            );
            let view = View::from_prefix(prefix, expected).unwrap();
            if !row["decision"].is_null() {
                let decision = policy.decide(&view).unwrap();
                assert_eq!(
                    serde_json::to_value(&decision).unwrap(),
                    row["decision"],
                    "{}",
                    row["label"]
                );
                let selected = decision
                    .alternatives
                    .iter()
                    .find(|a| a.choice == decision.choice)
                    .unwrap()
                    .value;
                assert_eq!(
                    serde_json::to_value(selected).unwrap(),
                    row["continuation_value"],
                    "{}",
                    row["label"]
                );
            } else {
                assert!(
                    matches!(policy.decide(&view), Err(Error::InvalidHistory(_))),
                    "{}",
                    row["label"]
                );
            }
        }
    }
}

#[test]
fn forged_views_and_cross_protocol_prefixes_are_rejected() {
    let protocol = Protocol::new(Role::A, PolicyKind::KnownRaw);
    let prior = OwnPrior::PointMass(Mechanism::Inert);
    let policy = CompiledPolicy::build(&protocol, prior, &Limits::registered().unwrap()).unwrap();
    let prefix = Prefix {
        role: Role::A,
        ids: protocol.ids.clone(),
        own_prior: prior,
        checkpoint: Checkpoint::Boundary(Boundary::Probe { completed: 0 }),
        entries: vec![],
    };
    let view = View::from_prefix(
        prefix.clone(),
        Belief {
            models: [
                Probability::new(0, 1).unwrap(),
                Probability::new(0, 1).unwrap(),
                Probability::new(0, 1).unwrap(),
                Probability::new(1, 1).unwrap(),
            ],
            target: None,
        },
    )
    .unwrap();
    for alteration in 0..3 {
        let mut forged = view.clone();
        match alteration {
            0 => forged.credits = 47,
            1 => forged.private_bit = Some(true),
            _ => forged.belief.models.swap(0, 3),
        }
        assert!(matches!(
            policy.decide(&forged),
            Err(Error::InvalidHistory(_))
        ));
    }
    let mut other = prefix.clone();
    other.ids.surface = "another-surface".into();
    assert!(matches!(
        policy.infer(&other),
        Err(Error::InvalidHistory(_))
    ));
    other = prefix;
    other.own_prior = OwnPrior::PointMass(Mechanism::SharedPersistent);
    assert!(matches!(
        policy.infer(&other),
        Err(Error::InvalidHistory(_))
    ));
    assert!(matches!(
        CompiledPolicy::build(&protocol, OwnPrior::Uniform, &Limits::registered().unwrap()),
        Err(Error::InvalidProtocol(_))
    ));
}

#[test]
fn resource_limits_abort_before_planning_growth() {
    let protocol = Protocol::new(Role::A, PolicyKind::RawAdaptive);
    for which in 0..3 {
        let mut limits = Limits::registered().unwrap();
        match which {
            0 => limits.states = 1,
            1 => limits.branches = 1,
            _ => limits.allocation_bytes = 1,
        }
        assert!(matches!(
            CompiledPolicy::build(&protocol, OwnPrior::Uniform, &limits),
            Err(Error::ResourceLimit { .. })
        ));
    }
}

// Removing pre-growth checks would mutate these counters or admit allocation.
#[test]
fn rejected_budget_growth_is_atomic() {
    use super::super::budget::Budget;
    let mut budget = Budget::new(&Limits {
        states: 1,
        branches: 1,
        allocation_bytes: 10,
    })
    .unwrap();
    budget.state(10).unwrap();
    budget.branch().unwrap();
    let before = budget.stats.clone();
    assert!(matches!(budget.state(1), Err(Error::ResourceLimit { .. })));
    assert_eq!(budget.stats, before);
    assert!(matches!(
        budget.allocate(1),
        Err(Error::ResourceLimit { .. })
    ));
    assert_eq!(budget.stats, before);
    assert!(matches!(budget.branch(), Err(Error::ResourceLimit { .. })));
    assert_eq!(budget.stats, before);
}

#[test]
fn known_prior_keeps_original_model_ids_and_rejects_helper_role() {
    let protocol = Protocol::new(Role::B, PolicyKind::KnownRaw);
    let prior = OwnPrior::PointMass(Mechanism::Inert);
    let policy = CompiledPolicy::build(&protocol, prior, &Limits::registered().unwrap()).unwrap();
    let mut prefix = Prefix {
        role: Role::B,
        ids: protocol.ids.clone(),
        own_prior: prior,
        checkpoint: Checkpoint::Boundary(Boundary::Probe { completed: 0 }),
        entries: vec![],
    };
    assert_eq!(
        policy.candidate_support(&prefix).unwrap(),
        (768..1024).collect::<Vec<u16>>()
    );
    prefix.role = Role::A;
    assert!(matches!(
        policy.infer(&prefix),
        Err(Error::InvalidHistory(_))
    ));
}

// A stronger helper in Raw would invalidate the restricted-policy embedding.
// Replay the exact same supplied owned actions through both physical kernels
// before asserting dominance of the separately optimized role expectations.
#[test]
fn supplied_actions_embed_with_identical_helper_before_role_expected_dominance() {
    use std::collections::BTreeMap;
    fn trace(
        protocol: &Protocol,
        model: Mechanism,
        actions: Option<&BTreeMap<Position, Action>>,
    ) -> [Prefix; 2] {
        let mut priors = [OwnPrior::PointMass(model); 2];
        priors[protocol.investigator.index()] = OwnPrior::Uniform;
        let mut state = EpisodeState::new(
            protocol,
            Environment::InFamily(model),
            EpisodeBits::from_index(0).unwrap(),
            priors,
        )
        .unwrap();
        loop {
            match state.next_boundary() {
                Some(Boundary::Probe { completed: 3 }) => break,
                Some(Boundary::Probe { .. }) => {
                    select_choice(&mut state, Choice::Continue).unwrap()
                }
                Some(Boundary::Calibration(position)) => {
                    select_choice(&mut state, Choice::Calibration(actions.unwrap()[&position]))
                        .unwrap()
                }
                Some(Boundary::Trial { .. }) => panic!("unexpected live boundary"),
                None => {
                    advance_one(&mut state).unwrap();
                }
            }
        }
        [state.prefix(Role::A).clone(), state.prefix(Role::B).clone()]
    }
    for role in [Role::A, Role::B] {
        let raw = Protocol::new(role, PolicyKind::RawAdaptive);
        let supplied = Protocol::new(role, PolicyKind::SuppliedAdaptive);
        for model in Mechanism::ALL {
            let expected = trace(&supplied, model, None);
            let actions = expected[role.index()]
                .entries
                .iter()
                .filter_map(|entry| match entry {
                    Entry::Physical(LocalEntry::Action(e)) => Some((e.position, e.action)),
                    _ => None,
                })
                .collect();
            let actual = trace(&raw, model, Some(&actions));
            assert_eq!(actual[1 - role.index()], expected[1 - role.index()]);
            let physical = |prefix: &Prefix| {
                prefix
                    .entries
                    .iter()
                    .filter_map(|e| match e {
                        Entry::Physical(p) => Some(p.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(
                physical(&actual[role.index()]),
                physical(&expected[role.index()])
            );
        }
        let limits = Limits::registered().unwrap();
        let raw = CompiledPolicy::build(&raw, OwnPrior::Uniform, &limits).unwrap();
        let supplied = CompiledPolicy::build(&supplied, OwnPrior::Uniform, &limits).unwrap();
        assert_ne!(
            raw.initial_value()
                .checked_cmp(supplied.initial_value())
                .unwrap(),
            std::cmp::Ordering::Less
        );
    }
}
