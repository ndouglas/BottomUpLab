use super::observation::observe;
use super::types::*;
use serde::{ser::SerializeSeq, Serialize, Serializer};

pub fn run_episode(config: &Config, profile: Profile) -> Result<Episode, Error> {
    run_engine(config, profile, &mut |_, _| Ok(()), None, None, false)
}
pub fn run_with_capture(
    config: &Config,
    profile: Profile,
    capture: &mut dyn FnMut(&CapturePoint) -> Result<(), Error>,
) -> Result<Episode, Error> {
    run_engine(
        config,
        profile,
        &mut |_, _| Ok(()),
        None,
        Some(capture),
        false,
    )
}
/// Legacy engineering observer runs after allocation preflight and before capture.
#[cfg(test)]
pub(super) fn run_checked(
    config: &Config,
    profile: Profile,
    inspect: &mut dyn FnMut(&Episode, u64) -> Result<(), Error>,
    capture: &mut dyn FnMut(&CapturePoint) -> Result<(), Error>,
) -> Result<Episode, Error> {
    run_engine(
        config,
        profile,
        &mut |_, _| Ok(()),
        Some(inspect),
        Some(capture),
        false,
    )
}
pub(super) fn run_budgeted(
    config: &Config,
    profile: Profile,
    preflight: &mut dyn FnMut(&Prospective<'_>, u64) -> Result<(), Error>,
) -> Result<Episode, Error> {
    run_engine(config, profile, preflight, None, None, true)
}
/// Borrowed observations serialize directly from validated state; no String or Vec exists before charge.
#[derive(Serialize)]
struct BorrowedObservation<'a> {
    clock: Clock,
    cells: Cells<'a>,
}
struct Cells<'a> {
    ids: &'a Ids,
    state: &'a PhysicalState,
    role: Role,
}
impl Serialize for Cells<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let visible: &[Cell] = match self.state.positions[self.role.index()] {
            Cell::P => &[Cell::P, Cell::O],
            Cell::O => &[Cell::P, Cell::O, Cell::C],
            Cell::C => &[Cell::O, Cell::C],
        };
        let mut seq = s.serialize_seq(Some(visible.len()))?;
        for cell in visible {
            #[derive(Serialize)]
            struct Image<'a> {
                cell: Cell,
                occupants: Occupants<'a>,
                loose_token: bool,
            }
            seq.serialize_element(&Image {
                cell: *cell,
                occupants: Occupants {
                    ids: self.ids,
                    state: self.state,
                    cell: *cell,
                },
                loose_token: *cell == Cell::C && self.state.resource == ResourceLocation::Cache,
            })?;
        }
        seq.end()
    }
}
struct Occupants<'a> {
    ids: &'a Ids,
    state: &'a PhysicalState,
    cell: Cell,
}
impl Serialize for Occupants<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let roles = [Role::Giver, Role::Receiver];
        let count = roles
            .iter()
            .filter(|r| self.state.positions[r.index()] == self.cell)
            .count();
        let mut seq = s.serialize_seq(Some(count))?;
        for role in roles {
            if self.state.positions[role.index()] == self.cell {
                #[derive(Serialize)]
                struct Occupant<'a> {
                    role: Role,
                    id: &'a str,
                    cargo: bool,
                    key: bool,
                }
                seq.serialize_element(&Occupant {
                    role,
                    id: self.ids.get(role),
                    cargo: role == Role::Receiver && self.state.receiver_cargo,
                    key: self.state.key_owner == role,
                })?;
            }
        }
        seq.end()
    }
}
#[derive(Serialize)]
enum Pending<'a> {
    Public {
        clock: Clock,
        termination: Option<Termination>,
    },
    Observed(BorrowedObservation<'a>),
    OwnAction {
        clock: Clock,
        token: &'a Token,
        intent: &'a Intent,
        result: &'a ActionOutcome,
        paid: u8,
    },
    Ownership(BorrowedReceipt<'a>),
    Settlement {
        clock: Clock,
        reward: u8,
        net: i16,
    },
}
#[derive(Serialize)]
struct BorrowedReceipt<'a> {
    clock: Clock,
    counterpart: &'a str,
    before: bool,
    after: bool,
}
impl Pending<'_> {
    fn retain(&self) -> LocalEntry {
        #[cfg(test)]
        MATERIALIZED.set(MATERIALIZED.get() + 1);
        match self {
            Self::Public { clock, termination } => LocalEntry::Public {
                clock: *clock,
                termination: *termination,
            },
            Self::Observed(o) => {
                LocalEntry::Observed(observe(o.cells.ids, o.cells.state, o.cells.role, o.clock))
            }
            Self::OwnAction {
                clock,
                token,
                intent,
                result,
                paid,
            } => LocalEntry::OwnAction {
                clock: *clock,
                token: (*token).clone(),
                intent: (*intent).clone(),
                result: (*result).clone(),
                paid: *paid,
            },
            Self::Ownership(r) => LocalEntry::Ownership(OwnershipReceipt {
                clock: r.clock,
                counterpart: r.counterpart.into(),
                before: r.before,
                after: r.after,
            }),
            Self::Settlement { clock, reward, net } => LocalEntry::Settlement {
                clock: *clock,
                reward: *reward,
                net: *net,
            },
        }
    }
}
struct History<'a> {
    old: &'a [LocalEntry],
    pending: [Option<Pending<'a>>; 4],
}
impl Serialize for History<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq =
            s.serialize_seq(Some(self.old.len() + self.pending.iter().flatten().count()))?;
        for e in self.old {
            seq.serialize_element(e)?;
        }
        for e in self.pending.iter().flatten() {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}
struct Events<'a> {
    old: &'a [Event],
    pending: Option<&'a Event>,
}
impl Serialize for Events<'_> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut seq =
            s.serialize_seq(Some(self.old.len() + usize::from(self.pending.is_some())))?;
        for e in self.old.iter().chain(self.pending) {
            seq.serialize_element(e)?;
        }
        seq.end()
    }
}
#[derive(Serialize)]
pub(super) struct Prospective<'a> {
    version: u8,
    config: &'a Config,
    profile: &'a Profile,
    words: &'a [OwnWord; 2],
    events: Events<'a>,
    histories: [History<'a>; 2],
    outcome: Outcome,
    #[serde(skip)]
    pub clock: Clock,
    #[serde(skip)]
    previous_length: Option<(u64, u64)>,
}
impl Prospective<'_> {
    pub fn json_len(&self) -> Result<u64, Error> {
        use super::budget::{add, json_len};
        let Some((previous, old_outcome)) = self.previous_length else {
            return json_len(self);
        };
        let mut n = previous
            .checked_sub(old_outcome)
            .ok_or(Error::ArithmeticOverflow)?;
        n = add(n, json_len(&self.outcome)?)?;
        if let Some(e) = self.events.pending {
            n = add(n, json_len(e)?)?;
            if !self.events.old.is_empty() {
                n = add(n, 1)?;
            }
        }
        for h in &self.histories {
            for (count, e) in (h.old.len()..).zip(h.pending.iter().flatten()) {
                n = add(n, json_len(e)?)?;
                if count > 0 {
                    n = add(n, 1)?;
                }
            }
        }
        Ok(n)
    }
}
fn make_view(e: &Episode, s: &PhysicalState, role: Role, clock: Clock) -> Result<View, Error> {
    let i = role.index();
    let last = e.histories[i]
        .iter()
        .rev()
        .find_map(|x| {
            if let LocalEntry::Observed(o) = x {
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
type Inspect<'a> = Option<&'a mut dyn FnMut(&Episode, u64) -> Result<(), Error>>;
type Capture<'a> = Option<&'a mut dyn FnMut(&CapturePoint) -> Result<(), Error>>;
fn run_engine(
    config: &Config,
    profile: Profile,
    preflight: &mut dyn FnMut(&Prospective<'_>, u64) -> Result<(), Error>,
    mut inspect: Inspect<'_>,
    mut capture: Capture<'_>,
    account: bool,
) -> Result<Episode, Error> {
    config.validate()?;
    profile.validate()?;
    let words = profile.words();
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
        key_owner: if config.initial_owner() == InitialOwner::Giver {
            Role::Giver
        } else {
            Role::Receiver
        },
        spent: [0; 2],
        reward: [0; 2],
    };
    // Empty vectors allocate nothing. Config cloning is delayed until initial preflight succeeds.
    let mut events = Vec::new();
    let mut histories: [Vec<LocalEntry>; 2] = [Vec::new(), Vec::new()];
    let mut clock = Clock {
        round: 0,
        phase: Phase::Initial,
        role: None,
    };
    let mut episode_config = None;
    let mut previous_length = None;
    loop {
        let mut next = state.clone();
        let mut event = None;
        let mut termination = None;
        let active = clock.role;
        if clock.phase == Phase::AfterAction {
            let role = active.ok_or_else(|| history_error("active transaction role"))?;
            let token = words[role.index()].token(clock.round)?;
            // Same own-only decision function as public choose; native prefixes were constructed here.
            let intent = super::controller::own_intent(
                role,
                token.clone(),
                state.positions[role.index()],
                role == Role::Receiver && state.receiver_cargo,
            )?;
            let (n, result, paid) = action(config, &state, role, &token, &intent)?;
            next = n;
            event = Some(Event {
                clock,
                kind: EventKind::Action {
                    role,
                    token,
                    intent,
                    result,
                    paid,
                },
                before: state.clone(),
                after: next.clone(),
            });
        } else if clock.phase == Phase::Finished {
            let term = if state.resource == ResourceLocation::Delivered {
                Termination::Delivered
            } else {
                Termination::Deadline
            };
            termination = Some(term);
            if term == Termination::Delivered {
                next.reward = [
                    config.stratum().giver_share(),
                    12 - config.stratum().giver_share(),
                ];
            }
            event = Some(Event {
                clock,
                kind: EventKind::Terminal(term),
                before: state.clone(),
                after: next.clone(),
            });
        }
        next.validate()?;
        let pending = [Role::Giver, Role::Receiver].map(|role| {
            let mut entries: [Option<Pending<'_>>; 4] = [None, None, None, None];
            let mut n = 0;
            entries[n] = Some(Pending::Public { clock, termination });
            n += 1;
            if let Some(Event {
                kind:
                    EventKind::Action {
                        role: actor,
                        token,
                        intent,
                        result,
                        paid,
                    },
                ..
            }) = &event
            {
                if *actor == role {
                    entries[n] = Some(Pending::OwnAction {
                        clock,
                        token,
                        intent,
                        result,
                        paid: *paid,
                    });
                    n += 1;
                }
                if *intent == Intent::Pass && *result == ActionOutcome::Success {
                    entries[n] = Some(Pending::Ownership(BorrowedReceipt {
                        clock,
                        counterpart: config.ids().get(if role == Role::Giver {
                            Role::Receiver
                        } else {
                            Role::Giver
                        }),
                        before: role == Role::Giver,
                        after: role == Role::Receiver,
                    }));
                    n += 1;
                }
            }
            if clock.phase == Phase::Initial
                || active == Some(role)
                    && matches!(clock.phase, Phase::BeforeAction | Phase::AfterAction)
            {
                entries[n] = Some(Pending::Observed(BorrowedObservation {
                    clock,
                    cells: Cells {
                        ids: config.ids(),
                        state: &next,
                        role,
                    },
                }));
            } else if clock.phase == Phase::Finished {
                entries[n] = Some(Pending::Settlement {
                    clock,
                    reward: next.reward[role.index()],
                    net: i16::from(next.reward[role.index()]) - i16::from(next.spent[role.index()]),
                });
            }
            History {
                old: &histories[role.index()],
                pending: entries,
            }
        });
        let o = outcome(&next, events.iter().chain(event.as_ref()))?;
        let prospective = Prospective {
            version: 1,
            config,
            profile: &profile,
            words: &words,
            events: Events {
                old: &events,
                pending: event.as_ref(),
            },
            histories: pending,
            outcome: o,
            clock,
            previous_length,
        };
        let added = u64::from(event.is_some());
        preflight(&prospective, added)?;
        if account {
            previous_length = Some((
                prospective.json_len()?,
                super::budget::json_len(&prospective.outcome)?,
            ));
        }
        // No pending event/history/config payload is allocated or retained before the preceding guard.
        let retained = prospective.histories.each_ref().map(|h| {
            h.pending
                .each_ref()
                .map(|e| e.as_ref().map(Pending::retain))
        });
        let o = prospective.outcome;
        if episode_config.is_none() {
            episode_config = Some(config.clone());
        }
        for (h, entries) in histories.iter_mut().zip(retained) {
            h.extend(entries.into_iter().flatten());
        }
        if let Some(e) = event {
            events.push(e);
        }
        state = next;
        if inspect.is_some() || capture.is_some() {
            let e = Episode {
                version: 1,
                config: config.clone(),
                profile,
                words: words.clone(),
                events: events.clone(),
                histories: histories.clone(),
                outcome: o.clone(),
            };
            if let Some(f) = inspect.as_mut() {
                f(&e, added)?;
            }
            if let Some(f) = capture.as_mut() {
                f(&CapturePoint {
                    clock,
                    views: [
                        make_view(&e, &state, Role::Giver, clock)?,
                        make_view(&e, &state, Role::Receiver, clock)?,
                    ],
                    physical: state.clone(),
                })?;
            }
        }
        clock = match (clock.phase, clock.role) {
            (Phase::Initial, _) => Clock {
                round: 1,
                phase: Phase::BeforeAction,
                role: Some(Role::Giver),
            },
            (Phase::BeforeAction, role) => Clock {
                phase: Phase::AfterAction,
                role,
                ..clock
            },
            (Phase::AfterAction, _) if state.resource == ResourceLocation::Delivered => Clock {
                phase: Phase::Finished,
                role: None,
                ..clock
            },
            (Phase::AfterAction, Some(Role::Giver)) => Clock {
                phase: Phase::BeforeAction,
                role: Some(Role::Receiver),
                ..clock
            },
            (Phase::AfterAction, Some(Role::Receiver)) if clock.round < 6 => Clock {
                round: clock.round + 1,
                phase: Phase::BeforeAction,
                role: Some(Role::Giver),
            },
            (Phase::AfterAction, _) => Clock {
                phase: Phase::Finished,
                role: None,
                ..clock
            },
            (Phase::Finished, _) => {
                return Ok(Episode {
                    version: 1,
                    config: episode_config
                        .ok_or_else(|| history_error("missing initial config"))?,
                    profile,
                    words,
                    events,
                    histories,
                    outcome: o,
                })
            }
        };
    }
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
fn outcome<'a>(
    s: &PhysicalState,
    events: impl IntoIterator<Item = &'a Event>,
) -> Result<Outcome, Error> {
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
#[cfg(test)]
thread_local! {static MATERIALIZED:std::cell::Cell<usize>=const{std::cell::Cell::new(0)};}
#[cfg(test)]
pub(super) fn test_preflight_order(config: &Config) {
    use std::cell::Cell as Counter;
    let p = Profile::new(486, 60).unwrap();
    let mut counts = Vec::new();
    run_engine(
        config,
        p,
        &mut |e, _| {
            assert_eq!(e.json_len()?, super::budget::json_len(e)?);
            counts.push(e.json_len()?);
            Ok(())
        },
        None,
        None,
        true,
    )
    .unwrap();
    for rejected in 0..counts.len() {
        MATERIALIZED.set(0);
        let captures = Counter::new(0);
        let before = Counter::new(0);
        let calls = Counter::new(0);
        let last_captured_materializations = Counter::new(0);
        let result = run_engine(
            config,
            p,
            &mut |e, _| {
                assert_eq!(MATERIALIZED.get(), last_captured_materializations.get());
                let i = calls.get();
                calls.set(i + 1);
                assert_eq!(e.json_len()?, counts[i]);
                if i == rejected {
                    before.set(MATERIALIZED.get());
                    return Err(Error::ResourceLimit {
                        resource: "test boundary".into(),
                        limit: counts[i] - 1,
                    });
                }
                Ok(())
            },
            None,
            Some(&mut |_| {
                captures.set(captures.get() + 1);
                last_captured_materializations.set(MATERIALIZED.get());
                Ok(())
            }),
            true,
        );
        assert!(result.is_err());
        assert_eq!(captures.get(), rejected);
        assert_eq!(MATERIALIZED.get(), before.get());
    }
}
