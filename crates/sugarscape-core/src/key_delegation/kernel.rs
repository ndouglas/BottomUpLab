use super::types::*;
use super::{choose, observation::observe};

pub fn run_episode(config: &Config, profile: Profile) -> Result<Episode, Error> {
    run_with_capture(config, profile, &mut |_| Ok(()))
}
pub fn run_with_capture(
    config: &Config,
    profile: Profile,
    capture: &mut dyn FnMut(&CapturePoint) -> Result<(), Error>,
) -> Result<Episode, Error> {
    run_checked(config, profile, &mut |_, _| Ok(()), capture)
}

/// Prospective partial Episode and appended Event count, before authoritative commit.
/// The compiler can reject every registered boundary here without adding sensing.
pub(super) fn run_checked(
    config: &Config,
    profile: Profile,
    charge: &mut dyn FnMut(&Episode, u64) -> Result<(), Error>,
    capture: &mut dyn FnMut(&CapturePoint) -> Result<(), Error>,
) -> Result<Episode, Error> {
    config.validate()?;
    profile.validate()?;
    let mut state = PhysicalState {
        positions: [
            if config.stratum().distance() == 0 {
                Cell::O
            } else {
                Cell::P
            },
            Cell::O,
        ],
        receiver_cargo: false,
        resource: ResourceLocation::Cache,
        key_owner: match config.initial_owner() {
            InitialOwner::Giver => Role::Giver,
            InitialOwner::Receiver => Role::Receiver,
        },
        spent: [0; 2],
        reward: [0; 2],
    };
    let mut episode = Episode {
        version: 1,
        config: config.clone(),
        profile,
        words: profile.words(),
        events: Vec::new(),
        histories: [Vec::new(), Vec::new()],
        outcome: outcome(&state, &[])?,
    };
    let initial = Clock {
        round: 0,
        phase: Phase::Initial,
        role: None,
    };
    let mut candidate = episode.clone();
    public(&mut candidate, initial, None);
    for role in [Role::Giver, Role::Receiver] {
        sense(&mut candidate, &state, role, initial);
    }
    let next = state.clone();
    commit(
        &mut episode,
        &mut state,
        candidate,
        next,
        initial,
        charge,
        capture,
    )?;
    for round in 1..=6 {
        for role in [Role::Giver, Role::Receiver] {
            let before_clock = Clock {
                round,
                phase: Phase::BeforeAction,
                role: Some(role),
            };
            let mut candidate = episode.clone();
            public(&mut candidate, before_clock, None);
            sense(&mut candidate, &state, role, before_clock);
            let next = state.clone();
            commit(
                &mut episode,
                &mut state,
                candidate,
                next,
                before_clock,
                charge,
                capture,
            )?;
            let view = make_view(&episode, &state, role, before_clock)?;
            let token = view.word.token(round)?;
            let intent = choose(&view)?;
            let (next, result, paid) = action(config, &state, role, &token, &intent)?;
            let clock = Clock {
                round,
                phase: Phase::AfterAction,
                role: Some(role),
            };
            let mut candidate = episode.clone();
            candidate.events.push(Event {
                clock,
                kind: EventKind::Action {
                    role,
                    token: token.clone(),
                    intent: intent.clone(),
                    result: result.clone(),
                    paid,
                },
                before: state.clone(),
                after: next.clone(),
            });
            public(&mut candidate, clock, None);
            candidate.histories[role.index()].push(LocalEntry::OwnAction {
                clock,
                token,
                intent: intent.clone(),
                result: result.clone(),
                paid,
            });
            if intent == Intent::Pass && result == ActionOutcome::Success {
                for owner in [Role::Giver, Role::Receiver] {
                    let peer = if owner == Role::Giver {
                        Role::Receiver
                    } else {
                        Role::Giver
                    };
                    candidate.histories[owner.index()].push(LocalEntry::Ownership(
                        OwnershipReceipt {
                            clock,
                            counterpart: config.ids().get(peer).into(),
                            before: owner == Role::Giver,
                            after: owner == Role::Receiver,
                        },
                    ));
                }
            }
            sense(&mut candidate, &next, role, clock);
            commit(
                &mut episode,
                &mut state,
                candidate,
                next,
                clock,
                charge,
                capture,
            )?;
            if state.resource == ResourceLocation::Delivered {
                return finish(
                    episode,
                    state,
                    round,
                    Termination::Delivered,
                    charge,
                    capture,
                );
            }
        }
    }
    finish(episode, state, 6, Termination::Deadline, charge, capture)
}

fn public(e: &mut Episode, clock: Clock, termination: Option<Termination>) {
    for h in &mut e.histories {
        h.push(LocalEntry::Public { clock, termination });
    }
}
fn sense(e: &mut Episode, s: &PhysicalState, role: Role, clock: Clock) {
    let o = observe(e.config.ids(), s, role, clock);
    e.histories[role.index()].push(LocalEntry::Observed(o));
}
fn make_view(e: &Episode, s: &PhysicalState, role: Role, clock: Clock) -> Result<View, Error> {
    let i = role.index();
    let last = e.histories[i]
        .iter()
        .rev()
        .find_map(|entry| {
            if let LocalEntry::Observed(o) = entry {
                Some(o.clone())
            } else {
                None
            }
        })
        .ok_or_else(|| history_error("missing sense"))?;
    Ok(View {
        role,
        word: e.words[i].clone(),
        own_cell: s.positions[i],
        own_cargo: role == Role::Receiver && s.receiver_cargo,
        own_key: s.key_owner == role,
        credits: 32 - s.spent[i],
        own_reward: s.reward[i],
        own_net: i16::from(s.reward[i]) - i16::from(s.spent[i]),
        clock,
        history: e.histories[i].clone(),
        last_observation: last,
    })
}
fn commit(
    e: &mut Episode,
    s: &mut PhysicalState,
    mut candidate: Episode,
    next: PhysicalState,
    clock: Clock,
    charge: &mut dyn FnMut(&Episode, u64) -> Result<(), Error>,
    capture: &mut dyn FnMut(&CapturePoint) -> Result<(), Error>,
) -> Result<(), Error> {
    next.validate()?;
    candidate.outcome = outcome(&next, &candidate.events)?;
    let added = candidate
        .events
        .len()
        .checked_sub(e.events.len())
        .ok_or(Error::ArithmeticOverflow)?;
    charge(
        &candidate,
        u64::try_from(added).map_err(|_| Error::ArithmeticOverflow)?,
    )?;
    *e = candidate;
    *s = next;
    let point = CapturePoint {
        clock,
        views: [
            make_view(e, s, Role::Giver, clock)?,
            make_view(e, s, Role::Receiver, clock)?,
        ],
        physical: s.clone(),
    };
    capture(&point)
}
fn finish(
    mut e: Episode,
    mut s: PhysicalState,
    round: u8,
    termination: Termination,
    charge: &mut dyn FnMut(&Episode, u64) -> Result<(), Error>,
    capture: &mut dyn FnMut(&CapturePoint) -> Result<(), Error>,
) -> Result<Episode, Error> {
    let clock = Clock {
        round,
        phase: Phase::Finished,
        role: None,
    };
    let mut next = s.clone();
    if termination == Termination::Delivered {
        next.reward = [
            e.config.stratum().giver_share(),
            12 - e.config.stratum().giver_share(),
        ];
    }
    let mut candidate = e.clone();
    candidate.events.push(Event {
        clock,
        kind: EventKind::Terminal(termination),
        before: s.clone(),
        after: next.clone(),
    });
    public(&mut candidate, clock, Some(termination));
    for role in [Role::Giver, Role::Receiver] {
        let i = role.index();
        candidate.histories[i].push(LocalEntry::Settlement {
            clock,
            reward: next.reward[i],
            net: i16::from(next.reward[i]) - i16::from(next.spent[i]),
        });
    }
    commit(&mut e, &mut s, candidate, next, clock, charge, capture)?;
    Ok(e)
}

pub(super) fn action(
    config: &Config,
    before: &PhysicalState,
    role: Role,
    token: &Token,
    intent: &Intent,
) -> Result<(PhysicalState, ActionOutcome, u8), Error> {
    config.validate()?;
    before.validate()?;
    // Capability and destination errors are operational; physical failures below pay.
    let capable = matches!(
        (role, token, intent),
        (Role::Giver, Token::Giver(GiverToken::Wait), Intent::Wait)
            | (
                Role::Giver,
                Token::Giver(GiverToken::Approach),
                Intent::Move(Cell::O)
            )
            | (Role::Giver, Token::Giver(GiverToken::Pass), Intent::Pass)
            | (
                Role::Receiver,
                Token::Receiver(ReceiverToken::Wait),
                Intent::Wait
            )
            | (
                Role::Receiver,
                Token::Receiver(ReceiverToken::Advance),
                Intent::Move(Cell::O | Cell::C) | Intent::Pickup | Intent::Deliver
            )
    );
    if !capable {
        return Err(history_error("illegal role, token or intent destination"));
    }
    let mut s = before.clone();
    let paid = match token {
        Token::Giver(GiverToken::Wait) | Token::Receiver(ReceiverToken::Wait) => 0,
        Token::Giver(GiverToken::Pass) => config.stratum().transfer_cost(),
        _ => 1,
    };
    s.spent[role.index()] = s.spent[role.index()]
        .checked_add(paid)
        .filter(|v| *v <= 32)
        .ok_or(Error::ArithmeticOverflow)?;
    let blocked = match intent {
        Intent::Wait => None,
        Intent::Pass => {
            if !config.stratum().transfer_enabled() {
                Some(BlockReason::TransferDisabled)
            } else if s.key_owner != Role::Giver {
                Some(BlockReason::NotKeyOwner)
            } else if s.positions[0] != s.positions[1] {
                Some(BlockReason::NotColocated)
            } else {
                s.key_owner = Role::Receiver;
                None
            }
        }
        Intent::Move(to) if role == Role::Giver => {
            if s.positions[0] == Cell::O {
                Some(BlockReason::AlreadyAtOutside)
            } else {
                s.positions[0] = *to;
                None
            }
        }
        Intent::Move(to) => {
            let from = if *to == Cell::C { Cell::O } else { Cell::C };
            if s.positions[1] != from {
                Some(BlockReason::WrongPosition)
            } else if s.key_owner != Role::Receiver {
                Some(BlockReason::NotKeyOwner)
            } else {
                s.positions[1] = *to;
                None
            }
        }
        Intent::Pickup => {
            if s.positions[1] != Cell::C {
                Some(BlockReason::WrongPosition)
            } else if s.receiver_cargo {
                Some(BlockReason::FullHands)
            } else if s.resource != ResourceLocation::Cache {
                Some(BlockReason::NoToken)
            } else {
                s.receiver_cargo = true;
                s.resource = ResourceLocation::Carried;
                None
            }
        }
        Intent::Deliver => {
            if s.positions[1] != Cell::O {
                Some(BlockReason::WrongPosition)
            } else if !s.receiver_cargo {
                Some(BlockReason::EmptyHands)
            } else {
                s.receiver_cargo = false;
                s.resource = ResourceLocation::Delivered;
                None
            }
        }
    };
    s.validate()?;
    Ok((
        s,
        blocked.map_or(ActionOutcome::Success, ActionOutcome::Blocked),
        paid,
    ))
}
fn increment(v: &mut u8) -> Result<(), Error> {
    *v = v.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
    Ok(())
}
fn outcome(s: &PhysicalState, events: &[Event]) -> Result<Outcome, Error> {
    let net = [
        i16::from(s.reward[0]) - i16::from(s.spent[0]),
        i16::from(s.reward[1]) - i16::from(s.spent[1]),
    ];
    let mut o = Outcome {
        spent: s.spent,
        reward: s.reward,
        net,
        group_net: net[0]
            .checked_add(net[1])
            .ok_or(Error::ArithmeticOverflow)?,
        delivery_round: None,
        giver_attempts: [0; 3],
        giver_blocked: [0; 3],
        receiver_attempts: [0; 4],
        receiver_blocked: [0; 4],
        receiver_waits: 0,
        transfer_count: 0,
        approach_spent: 0,
        pass_spent: 0,
        unused_credits: [32 - s.spent[0], 32 - s.spent[1]],
    };
    for event in events {
        if let EventKind::Action {
            role,
            token,
            intent,
            result,
            paid,
        } = &event.kind
        {
            let blocked = matches!(result, ActionOutcome::Blocked(_));
            match role {
                Role::Giver => {
                    let i = match token {
                        Token::Giver(GiverToken::Wait) => 0,
                        Token::Giver(GiverToken::Approach) => 1,
                        Token::Giver(GiverToken::Pass) => 2,
                        _ => return Err(history_error("Giver token")),
                    };
                    increment(&mut o.giver_attempts[i])?;
                    if blocked {
                        increment(&mut o.giver_blocked[i])?;
                    }
                    if i == 1 {
                        o.approach_spent = o
                            .approach_spent
                            .checked_add(*paid)
                            .ok_or(Error::ArithmeticOverflow)?;
                    }
                    if i == 2 {
                        o.pass_spent = o
                            .pass_spent
                            .checked_add(*paid)
                            .ok_or(Error::ArithmeticOverflow)?;
                        if !blocked {
                            increment(&mut o.transfer_count)?;
                        }
                    }
                }
                Role::Receiver => {
                    let i = match intent {
                        Intent::Wait => {
                            increment(&mut o.receiver_waits)?;
                            continue;
                        }
                        Intent::Move(Cell::C) => 0,
                        Intent::Pickup => 1,
                        Intent::Move(Cell::O) => 2,
                        Intent::Deliver => 3,
                        _ => return Err(history_error("Receiver intent")),
                    };
                    increment(&mut o.receiver_attempts[i])?;
                    if blocked {
                        increment(&mut o.receiver_blocked[i])?;
                    }
                    if i == 3 && !blocked {
                        o.delivery_round = Some(event.clock.round);
                    }
                }
            }
        }
    }
    Ok(o)
}
