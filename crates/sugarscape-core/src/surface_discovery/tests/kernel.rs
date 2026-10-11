use super::super::*;
use crate::shared_surface::Outcome;

fn state(role: Role, policy: PolicyKind, environment: Environment) -> EpisodeState {
    let nominal = match environment {
        Environment::InFamily(m) => m,
        Environment::DataFlip => Mechanism::SharedPersistent,
    };
    let mut priors = [OwnPrior::PointMass(nominal); 2];
    if policy != PolicyKind::KnownRaw {
        priors[role.index()] = OwnPrior::Uniform;
    }
    EpisodeState::new(
        &Protocol::new(role, policy),
        environment,
        EpisodeBits::from_index(1).unwrap(),
        priors,
    )
    .unwrap()
}
fn raw(role: Role) -> EpisodeState {
    state(
        role,
        PolicyKind::RawAdaptive,
        Environment::InFamily(Mechanism::SharedPersistent),
    )
}
fn check(state: &EpisodeState) {
    for role in [Role::A, Role::B] {
        state
            .protocol()
            .validate_prefix(state.prefix(role))
            .unwrap();
        let wire = serde_json::to_string(state.prefix(role)).unwrap();
        assert_eq!(
            serde_json::from_str::<Prefix>(&wire).unwrap(),
            *state.prefix(role)
        );
    }
}
fn next_paid(state: &mut EpisodeState) -> (Role, Event) {
    loop {
        let step = advance_one(state).unwrap();
        check(state);
        if let Some(event) = step.events.first() {
            return event.clone();
        }
    }
}
fn finish_round(state: &mut EpisodeState, action: Action) -> Vec<(Role, Event)> {
    let mut events = Vec::new();
    loop {
        if let Some(Boundary::Calibration(_)) = state.next_boundary() {
            select_choice(state, Choice::Calibration(action)).unwrap();
            check(state);
        }
        let step = advance_one(state).unwrap();
        check(state);
        events.extend(step.events);
        if matches!(step.next, Some(Boundary::Probe { .. })) {
            return events;
        }
    }
}
fn reach_trial(state: &mut EpisodeState) {
    select_choice(state, Choice::Stop).unwrap();
    check(state);
    loop {
        let step = advance_one(state).unwrap();
        check(state);
        if matches!(step.next, Some(Boundary::Trial { .. })) {
            return;
        }
    }
}

// Catches paying, resetting, issuing a bit, or changing delivered clocks on free Stop0.
#[test]
fn stop_zero_is_free_public_and_queued() {
    let mut s = raw(Role::A);
    let checkpoints = [s.prefix(Role::A).checkpoint, s.prefix(Role::B).checkpoint];
    select_choice(&mut s, Choice::Stop).unwrap();
    assert_eq!([s.credits(Role::A), s.credits(Role::B)], [48, 48]);
    assert_eq!(
        [s.prefix(Role::A).checkpoint, s.prefix(Role::B).checkpoint],
        checkpoints
    );
    assert_eq!(
        s.prefix(Role::B).entries,
        [Entry::PublicProbeStop { completed: 0 }]
    );
    assert_eq!(s.prefix(Role::A).entries.len(), 2);
    assert!(s.next_boundary().is_none());
    assert!(select_choice(&mut s, Choice::Stop).is_err());
    check(&s);
    let step = advance_one(&mut s).unwrap();
    assert!(step.events.is_empty());
    assert_eq!(step.next, Some(Boundary::Trial { trial: 0 }));
    assert_eq!(s.prefix(Role::B).entries.len(), 3);
    check(&s);
}
// Catches incorrect A/B/A/B ownership, midround stops, missing paid waits, or early spending.
#[test]
fn raw_round_commits_a_b_b_a_with_private_slot_choices() {
    for role in [Role::A, Role::B] {
        let mut s = raw(role);
        let helper = if role == Role::A { Role::B } else { Role::A };
        let helper_before = s.prefix(helper).clone();
        select_choice(&mut s, Choice::Continue).unwrap();
        assert_eq!(*s.prefix(helper), helper_before);
        assert_eq!(s.credits(role), 48);
        check(&s);
        let entered = advance_one(&mut s).unwrap();
        assert!(entered.events.is_empty());
        let events = finish_round(&mut s, Action::Wait);
        assert_eq!(
            events.iter().map(|e| e.0).collect::<Vec<_>>(),
            [Role::A, Role::B, Role::B, Role::A]
        );
        assert_eq!([s.credits(Role::A), s.credits(Role::B)], [46, 46]);
        assert_eq!(
            s.prefix(role)
                .entries
                .iter()
                .filter(|e| matches!(
                    e,
                    Entry::OwnChoice {
                        boundary: Boundary::Calibration(_),
                        ..
                    }
                ))
                .count(),
            2
        );
        assert!(!s
            .prefix(helper)
            .entries
            .iter()
            .any(|e| matches!(e, Entry::OwnChoice { .. })));
    }
}
// Catches B choosing before A pays, and disclosing A's action through B's own entries.
#[test]
fn b_live_boundary_follows_unobserved_a_paid_after_slot() {
    let mut s = raw(Role::B);
    select_choice(&mut s, Choice::Stop).unwrap();
    let entry = advance_one(&mut s).unwrap();
    assert!(entry.next.is_none());
    let (role, event) = next_paid(&mut s);
    assert_eq!(
        (role, event.position.round, event.position.slot),
        (Role::A, 1, 1)
    );
    assert_eq!(
        s.prefix(Role::B).checkpoint,
        Checkpoint::AfterSlot(event.position)
    );
    assert!(!s
        .prefix(Role::B)
        .entries
        .iter()
        .any(|e| matches!(e, Entry::Physical(LocalEntry::Action(_)))));
    assert!(s.next_boundary().is_none());
    let step = advance_one(&mut s).unwrap();
    assert_eq!(step.next, Some(Boundary::Trial { trial: 0 }));
    check(&s);
    let helper = s.prefix(Role::A).clone();
    select_choice(&mut s, Choice::Inspect).unwrap();
    assert_eq!(*s.prefix(Role::A), helper);
    check(&s);
    let (_, event) = next_paid(&mut s);
    assert_eq!((event.position.slot, event.action), (2, Action::Wait));
}
// Catches permitting a fourth purchased round.
#[test]
fn three_round_stop_remains_mandatory() {
    let mut s = raw(Role::B);
    for completed in 0..3 {
        assert_eq!(s.next_boundary(), Some(Boundary::Probe { completed }));
        select_choice(&mut s, Choice::Continue).unwrap();
        advance_one(&mut s).unwrap();
        finish_round(&mut s, Action::Read);
    }
    assert!(select_choice(&mut s, Choice::Continue).is_err());
    select_choice(&mut s, Choice::Stop).unwrap();
    check(&s);
}
// Catches illegal actions mutating a selected state or a committed round allowing Stop.
#[test]
fn illegal_selection_preserves_histories_credits_and_boundary() {
    let mut s = raw(Role::A);
    select_choice(&mut s, Choice::Continue).unwrap();
    advance_one(&mut s).unwrap();
    let before = [s.prefix(Role::A).clone(), s.prefix(Role::B).clone()];
    for choice in [
        Choice::Stop,
        Choice::Inspect,
        Choice::Calibration(Action::Write(Symbol::Blank)),
        Choice::Calibration(Action::InspectOwnTarget),
    ] {
        assert!(select_choice(&mut s, choice).is_err());
        assert_eq!(
            [s.prefix(Role::A).clone(), s.prefix(Role::B).clone()],
            before
        );
        assert_eq!(s.credits(Role::A), 48);
        assert!(s.next_boundary().is_some());
    }
}
// Catches excluding any nonblank symbol, mechanism-specific acceptance, or lost DataFlip physics.
#[test]
fn all_six_writes_accept_generically_and_dataflip_inverts_data_only() {
    for symbol in [
        Symbol::Probe0,
        Symbol::Ack0,
        Symbol::Probe1,
        Symbol::Ack1,
        Symbol::Data0,
        Symbol::Data1,
    ] {
        let mut s = state(Role::A, PolicyKind::RawAdaptive, Environment::DataFlip);
        select_choice(&mut s, Choice::Continue).unwrap();
        advance_one(&mut s).unwrap();
        select_choice(&mut s, Choice::Calibration(Action::Write(symbol))).unwrap();
        check(&s);
        let (_, event) = next_paid(&mut s);
        assert_eq!(event.outcome, Outcome::Accepted);
        let (role, event) = next_paid(&mut s);
        assert_eq!(role, Role::B);
        let stored = match symbol {
            Symbol::Data0 => Symbol::Data1,
            Symbol::Data1 => Symbol::Data0,
            _ => symbol,
        };
        assert_eq!(event.outcome, Outcome::Read(stored));
    }
}
// Catches round reset occurring inside the paid last-slot return.
#[test]
fn paid_last_slot_returns_before_shared_resetting_round_finish() {
    let mut s = state(
        Role::B,
        PolicyKind::RawAdaptive,
        Environment::InFamily(Mechanism::SharedResetting),
    );
    select_choice(&mut s, Choice::Continue).unwrap();
    advance_one(&mut s).unwrap();
    loop {
        if matches!(s.next_boundary(), Some(Boundary::Calibration(_))) {
            select_choice(&mut s, Choice::Calibration(Action::Write(Symbol::Ack1))).unwrap();
        }
        let step = advance_one(&mut s).unwrap();
        check(&s);
        if step
            .events
            .first()
            .is_some_and(|(_, e)| e.position.slot == 4)
        {
            break;
        }
    }
    assert!(s.read_lineage(Role::A).is_some());
    advance_one(&mut s).unwrap();
    assert!(s.read_lineage(Role::A).is_none());
}
// Catches wrong live cost, early future bits, or prediction/finish skipping owned opportunities.
#[test]
fn inspect_and_attempt_charge_nine_and_six_per_trial_with_maximum_forty_two() {
    for role in [Role::A, Role::B] {
        for routine in [Choice::Inspect, Choice::Attempt] {
            let mut s = raw(role);
            for _ in 0..3 {
                select_choice(&mut s, Choice::Continue).unwrap();
                advance_one(&mut s).unwrap();
                finish_round(&mut s, Action::Wait);
            }
            reach_trial(&mut s);
            for trial in 0..4 {
                select_choice(&mut s, routine).unwrap();
                check(&s);
                loop {
                    let step = advance_one(&mut s).unwrap();
                    check(&s);
                    if s.prefix(role).checkpoint == (Checkpoint::Prediction { trial }) {
                        break;
                    }
                    assert!(!step.histories[role.index()].entries.iter().any(|e|matches!(e,Entry::Physical(LocalEntry::PrivateBit { trial: t,.. }) if *t>trial)));
                }
                assert_eq!(
                    s.credits(role),
                    42 - (trial + 1) * if routine == Choice::Inspect { 9 } else { 6 }
                );
                if trial < 3 {
                    loop {
                        let step = advance_one(&mut s).unwrap();
                        check(&s);
                        if step.next.is_some() {
                            break;
                        }
                    }
                }
            }
            advance_one(&mut s).unwrap();
            check(&s);
            assert!(s.is_finished());
            assert!(advance_one(&mut s).is_err());
        }
    }
}
// Catches oracle/transfer KnownRaw, wrong owner prior, and invalid opaque identifiers.
#[test]
fn checked_construction_rejects_wrong_priors_and_known_transfer() {
    let p = Protocol::new(Role::A, PolicyKind::KnownRaw);
    for (environment, priors) in [
        (
            Environment::DataFlip,
            [OwnPrior::PointMass(Mechanism::SharedPersistent); 2],
        ),
        (
            Environment::InFamily(Mechanism::Inert),
            [OwnPrior::PointMass(Mechanism::SharedPersistent); 2],
        ),
    ] {
        assert!(
            EpisodeState::new(&p, environment, EpisodeBits::from_index(0).unwrap(), priors)
                .is_err()
        );
    }
    let mut p = Protocol::new(Role::B, PolicyKind::RawAdaptive);
    assert!(EpisodeState::new(
        &p,
        Environment::InFamily(Mechanism::Inert),
        EpisodeBits::from_index(0).unwrap(),
        [OwnPrior::Uniform; 2]
    )
    .is_err());
    p.ids.agents[1] = p.ids.agents[0].clone();
    assert!(p.validate().is_err());
}
// Catches inconsistent transported credits/bits, unknown fields, and ownership tampering.
#[test]
fn serde_checks_own_clock_credits_bits_roles_and_unknown_fields() {
    let mut s = raw(Role::A);
    reach_trial(&mut s);
    select_choice(&mut s, Choice::Inspect).unwrap();
    next_paid(&mut s);
    let belief = Belief {
        models: [Probability::new(1, 4).unwrap(); 4],
        target: Some(Probability::new(1, 2).unwrap()),
    };
    let view = View::from_prefix(s.prefix(Role::A).clone(), belief).unwrap();
    let value = serde_json::to_value(&view).unwrap();
    for field in ["credits", "private_bit"] {
        let mut forged = value.clone();
        forged[field] = if field == "credits" {
            serde_json::json!(48)
        } else {
            serde_json::json!(false)
        };
        assert!(serde_json::from_value::<View>(forged).is_err());
    }
    let mut prefix = serde_json::to_value(s.prefix(Role::A)).unwrap();
    prefix["role"] = serde_json::json!("B");
    assert!(serde_json::from_value::<Prefix>(prefix).is_err());
    let mut prefix = serde_json::to_value(s.prefix(Role::A)).unwrap();
    prefix["entries"].as_array_mut().unwrap().retain(|e| {
        e.get("Physical")
            .and_then(|p| p.get("PrivateBit"))
            .is_none()
    });
    assert!(serde_json::from_value::<Prefix>(prefix).is_err());
    let mut forged = value.clone();
    forged["hidden_peer"] = serde_json::json!(true);
    assert!(serde_json::from_value::<View>(forged).is_err());
    let mut forged = value;
    forged.as_object_mut().unwrap().remove("private_bit");
    assert!(serde_json::from_value::<View>(forged).is_err());
    assert!(serde_json::from_str::<Boundary>(r#"{"Probe":{"completed":4}}"#).is_err());
}
// Catches chronology rejection of independently authored selected/queued prefixes.
#[test]
fn independent_registered_prefix_witnesses_validate() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/planning-reference.json")).unwrap();
    for witness in fixture["prefix_witnesses"].as_array().unwrap() {
        let prefix: Prefix = serde_json::from_value(witness["prefix"].clone())
            .unwrap_or_else(|e| panic!("{}: {e}", witness["label"]));
        let protocol: Protocol = serde_json::from_value(witness["protocol"].clone()).unwrap();
        protocol
            .validate_prefix(&prefix)
            .unwrap_or_else(|e| panic!("{}: {e}", witness["label"]));
    }
}
// Catches changing the original physical recipes or introducing raw slot decisions for supplied controls.
#[test]
fn supplied_transitions_match_original_active_kernel_for_both_roles() {
    use crate::active_surface as old;
    for role in [Role::A, Role::B] {
        for model in Mechanism::ALL {
            let environment = Environment::InFamily(model);
            let mut s = state(role, PolicyKind::SuppliedAdaptive, environment);
            let mut priors = [OwnPrior::PointMass(model); 2];
            priors[role.index()] = OwnPrior::Uniform;
            let mut legacy = old::EpisodeState::new(
                &old::Protocol::new(role, old::PolicyKind::Adaptive),
                environment,
                EpisodeBits::from_index(1).unwrap(),
                priors,
            )
            .unwrap();
            for (choice, old_choice) in [
                (Choice::Continue, old::Choice::ContinueProbe),
                (Choice::Continue, old::Choice::ContinueProbe),
                (Choice::Continue, old::Choice::ContinueProbe),
                (Choice::Stop, old::Choice::StopProbing),
                (Choice::Inspect, old::Choice::Inspect),
                (Choice::Attempt, old::Choice::AttemptCommunication),
                (Choice::Inspect, old::Choice::Inspect),
                (Choice::Attempt, old::Choice::AttemptCommunication),
            ] {
                select_choice(&mut s, choice).unwrap();
                old::select_choice(&mut legacy, old_choice).unwrap();
                loop {
                    let step = advance_one(&mut s).unwrap();
                    let previous = old::advance_one(&mut legacy).unwrap();
                    assert_eq!(step.events, previous.events);
                    assert_eq!(step.spent_delta, previous.spent_delta);
                    for local_role in [Role::A, Role::B] {
                        let actual = s
                            .prefix(local_role)
                            .entries
                            .iter()
                            .filter_map(|e| {
                                if let Entry::Physical(e) = e {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        let expected = legacy
                            .prefix(local_role)
                            .entries
                            .iter()
                            .filter_map(|e| {
                                if let old::Entry::Physical(e) = e {
                                    Some(e)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        assert_eq!(actual, expected);
                    }
                    check(&s);
                    assert_eq!(step.next.is_some(), previous.next.is_some());
                    if step.next.is_some() || s.is_finished() {
                        break;
                    }
                }
            }
            assert!(!s.prefix(role).entries.iter().any(|e| matches!(
                e,
                Entry::OwnChoice {
                    boundary: Boundary::Calibration(_),
                    ..
                }
            )));
        }
    }
}
// Catches accepting a legal paid action that violates the supplied controller's own-bit routine or calibration recipe.
#[test]
fn declared_supplied_actions_reject_forged_legal_events() {
    let mut s = state(
        Role::B,
        PolicyKind::RawAdaptive,
        Environment::InFamily(Mechanism::SharedPersistent),
    );
    select_choice(&mut s, Choice::Stop).unwrap();
    advance_one(&mut s).unwrap();
    next_paid(&mut s);
    let mut prefix = s.prefix(Role::A).clone();
    for entry in &mut prefix.entries {
        if let Entry::Physical(LocalEntry::Action(event)) = entry {
            event.action = Action::Write(Symbol::Data0);
        }
    }
    prefix.validate().unwrap();
    assert!(s.protocol().validate_prefix(&prefix).is_err());

    let mut s = state(
        Role::A,
        PolicyKind::SuppliedAdaptive,
        Environment::InFamily(Mechanism::SharedPersistent),
    );
    select_choice(&mut s, Choice::Continue).unwrap();
    advance_one(&mut s).unwrap();
    next_paid(&mut s);
    let mut prefix = s.prefix(Role::A).clone();
    for entry in &mut prefix.entries {
        if let Entry::Physical(LocalEntry::Action(event)) = entry {
            event.action = Action::Wait;
            event.outcome = Outcome::Waited;
        }
    }
    prefix.validate().unwrap();
    assert!(s.protocol().validate_prefix(&prefix).is_err());
}

// Catches treating an own legal counterfactual intervention as evidence of why a forced control chose it.
#[test]
fn forced_control_counterfactual_choices_execute_with_unchanged_declared_continuation() {
    for policy in [PolicyKind::NoProbe, PolicyKind::InspectOnly] {
        for role in [Role::A, Role::B] {
            let mut s = state(
                role,
                policy,
                Environment::InFamily(Mechanism::SharedPersistent),
            );
            select_choice(&mut s, Choice::Continue).unwrap();
            check(&s);
            advance_one(&mut s).unwrap();
            finish_round(&mut s, Action::Wait);
            assert_eq!(s.credits(role), 46);
            reach_trial(&mut s);
            select_choice(&mut s, Choice::Attempt).unwrap();
            check(&s);
            loop {
                let step = advance_one(&mut s).unwrap();
                check(&s);
                if s.prefix(role).checkpoint == (Checkpoint::Prediction { trial: 0 }) {
                    break;
                }
                assert!(step.next.is_none());
            }
            assert_eq!(s.credits(role), 40);
            assert_eq!(s.protocol().policy, policy);
        }
    }
}

fn assert_review_history_rejected(protocol: &Protocol, prefix: &Prefix) {
    assert!(prefix.validate().is_err());
    assert!(protocol.validate_prefix(prefix).is_err());
    assert!(serde_json::from_value::<Prefix>(serde_json::to_value(prefix).unwrap()).is_err());
}

// Catches using a future Continue as permission for an earlier calibration Reset.
#[test]
fn review_rejects_calibration_reset_before_first_continue() {
    let mut s = raw(Role::A);
    select_choice(&mut s, Choice::Continue).unwrap();
    advance_one(&mut s).unwrap();
    let mut forged = s.prefix(Role::A).clone();
    forged.entries.swap(0, 1);
    assert_review_history_rejected(s.protocol(), &forged);
}

// Catches treating B's two own paid actions as completion before A4 and free round finish.
#[test]
fn review_rejects_b_purchase_before_a_last_slot_and_round_finish() {
    let mut s = raw(Role::B);
    select_choice(&mut s, Choice::Continue).unwrap();
    advance_one(&mut s).unwrap();
    next_paid(&mut s);
    advance_one(&mut s).unwrap();
    for _ in 0..2 {
        select_choice(&mut s, Choice::Calibration(Action::Wait)).unwrap();
        next_paid(&mut s);
        if matches!(
            s.prefix(Role::B).checkpoint,
            Checkpoint::AfterSlot(Position { slot: 2, .. })
        ) {
            advance_one(&mut s).unwrap();
        }
    }
    let mut forged = s.prefix(Role::B).clone();
    forged.entries.push(Entry::OwnChoice {
        boundary: Boundary::Probe { completed: 1 },
        choice: Choice::Continue,
    });
    assert_review_history_rejected(s.protocol(), &forged);
}

// Catches a B live selection before the unseen A1 payment and subsequent public advance.
#[test]
fn review_rejects_b_trial_selection_before_unseen_a_paid_action() {
    let mut s = raw(Role::B);
    select_choice(&mut s, Choice::Stop).unwrap();
    advance_one(&mut s).unwrap();
    let mut forged = s.prefix(Role::B).clone();
    forged.entries.push(Entry::OwnChoice {
        boundary: Boundary::Trial { trial: 0 },
        choice: Choice::Inspect,
    });
    assert_review_history_rejected(s.protocol(), &forged);
}

fn review_belief(target: Option<Probability>) -> Belief {
    Belief {
        models: [Probability::new(1, 4).unwrap(); 4],
        target,
    }
}
fn review_prediction(role: Role) -> EpisodeState {
    let mut s = raw(role);
    reach_trial(&mut s);
    select_choice(&mut s, Choice::Inspect).unwrap();
    while s.prefix(role).checkpoint != (Checkpoint::Prediction { trial: 0 }) {
        advance_one(&mut s).unwrap();
    }
    s
}
fn review_null_target_wire(prefix: &Prefix) -> serde_json::Value {
    let view = View::from_prefix(
        prefix.clone(),
        review_belief(Some(Probability::new(1, 2).unwrap())),
    )
    .unwrap();
    let mut wire = serde_json::to_value(view).unwrap();
    wire["belief"]["target"] = serde_json::Value::Null;
    wire
}

// Catches a checked live decision View without the posterior the decision requires.
#[test]
fn review_typed_trial_view_requires_target_before_and_after_selection() {
    for role in [Role::A, Role::B] {
        let mut s = raw(role);
        reach_trial(&mut s);
        assert!(View::from_prefix(s.prefix(role).clone(), review_belief(None)).is_err());
        select_choice(&mut s, Choice::Inspect).unwrap();
        assert!(View::from_prefix(s.prefix(role).clone(), review_belief(None)).is_err());
    }
}
#[test]
fn review_serde_trial_view_rejects_null_target_before_and_after_selection() {
    for role in [Role::A, Role::B] {
        let mut s = raw(role);
        reach_trial(&mut s);
        assert!(serde_json::from_value::<View>(review_null_target_wire(s.prefix(role))).is_err());
        select_choice(&mut s, Choice::Inspect).unwrap();
        assert!(serde_json::from_value::<View>(review_null_target_wire(s.prefix(role))).is_err());
    }
}
#[test]
fn review_typed_prediction_view_requires_target() {
    for role in [Role::A, Role::B] {
        let s = review_prediction(role);
        assert!(View::from_prefix(s.prefix(role).clone(), review_belief(None)).is_err());
    }
}
#[test]
fn review_serde_prediction_view_rejects_null_target() {
    for role in [Role::A, Role::B] {
        let s = review_prediction(role);
        assert!(serde_json::from_value::<View>(review_null_target_wire(s.prefix(role))).is_err());
    }
}
// Catches overcorrecting null rejection in calibration or rejecting valid live/queued decision mass.
#[test]
fn review_valid_calibration_null_and_live_target_views_remain_checked() {
    for role in [Role::A, Role::B] {
        let mut s = raw(role);
        select_choice(&mut s, Choice::Continue).unwrap();
        check(&s);
        advance_one(&mut s).unwrap();
        while !matches!(s.next_boundary(), Some(Boundary::Calibration(_))) {
            advance_one(&mut s).unwrap();
        }
        let view = View::from_prefix(s.prefix(role).clone(), review_belief(None)).unwrap();
        assert_eq!(
            serde_json::from_value::<View>(serde_json::to_value(&view).unwrap()).unwrap(),
            view
        );
        select_choice(&mut s, Choice::Calibration(Action::Wait)).unwrap();
        check(&s);
        View::from_prefix(s.prefix(role).clone(), review_belief(None)).unwrap();
        let mut s = raw(role);
        reach_trial(&mut s);
        for queued in [false, true] {
            if queued {
                select_choice(&mut s, Choice::Inspect).unwrap();
            }
            let view = View::from_prefix(
                s.prefix(role).clone(),
                review_belief(Some(Probability::new(1, 2).unwrap())),
            )
            .unwrap();
            assert_eq!(
                serde_json::from_value::<View>(serde_json::to_value(&view).unwrap()).unwrap(),
                view
            );
        }
        let s = review_prediction(role);
        View::from_prefix(
            s.prefix(role).clone(),
            review_belief(Some(Probability::new(1, 2).unwrap())),
        )
        .unwrap();
    }
}
