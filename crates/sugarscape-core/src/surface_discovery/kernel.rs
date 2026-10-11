use super::*;
use crate::shared_surface::{self as physical, Knowledge, LocalPrefix, LocalView, World};

#[derive(Clone, Copy)]
enum Clock {
    Choice(Boundary),
    ProbeStart {
        round: u8,
    },
    EnterLive {
        trial: u8,
    },
    Before {
        position: Position,
        queued: Option<Action>,
    },
    After(Position),
    Prediction {
        trial: u8,
    },
    Finished,
}
/// Privileged candidate/evaluator runtime. Controller APIs accept only Prefix/View.
#[derive(Clone)]
pub struct EpisodeState {
    protocol: Protocol,
    world: World,
    bits: EpisodeBits,
    histories: [Prefix; 2],
    credits: [u8; 2],
    private_bits: [Option<bool>; 2],
    clock: Clock,
    routine: Option<Choice>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    pub histories: [Prefix; 2],
    pub events: Vec<(Role, Event)>,
    pub next: Option<Boundary>,
    pub spent_delta: [u8; 2],
}
impl EpisodeState {
    pub fn new(
        protocol: &Protocol,
        environment: Environment,
        bits: EpisodeBits,
        priors: [OwnPrior; 2],
    ) -> Result<Self, Error> {
        protocol.validate()?;
        if protocol.policy == PolicyKind::KnownRaw && environment == Environment::DataFlip {
            return Err(Error::InvalidProtocol(
                "KnownRaw requires matching in-family mechanism, not DataFlip".into(),
            ));
        }
        let nominal = match environment {
            Environment::InFamily(m) => m,
            Environment::DataFlip => Mechanism::SharedPersistent,
        };
        for role in [Role::A, Role::B] {
            let expected =
                if role == protocol.investigator && protocol.policy != PolicyKind::KnownRaw {
                    OwnPrior::Uniform
                } else {
                    OwnPrior::PointMass(nominal)
                };
            if priors[role.index()] != expected {
                return Err(Error::InvalidProtocol(format!(
                    "Agent {role:?} prior differs from declared control for {environment:?}"
                )));
            }
        }
        Ok(Self {
            protocol: protocol.clone(),
            world: World::new(environment, protocol.ids.clone()),
            bits,
            histories: [Role::A, Role::B].map(|role| Prefix {
                role,
                ids: protocol.ids.clone(),
                own_prior: priors[role.index()],
                checkpoint: Checkpoint::Boundary(Boundary::Probe { completed: 0 }),
                entries: Vec::new(),
            }),
            credits: [48; 2],
            private_bits: [None; 2],
            clock: Clock::Choice(Boundary::Probe { completed: 0 }),
            routine: None,
        })
    }
    pub fn protocol(&self) -> &Protocol {
        &self.protocol
    }
    pub fn prefix(&self, role: Role) -> &Prefix {
        &self.histories[role.index()]
    }
    pub fn credits(&self, role: Role) -> u8 {
        self.credits[role.index()]
    }
    /// Only unselected opportunities are returned; delivered boundaries can remain queued.
    pub fn next_boundary(&self) -> Option<Boundary> {
        if let Clock::Choice(b) = self.clock {
            Some(b)
        } else {
            None
        }
    }
    pub fn is_finished(&self) -> bool {
        matches!(self.clock, Clock::Finished)
    }
    /// Evaluator-only origin snapshot. Capture immediately after paid Read, before
    /// any free reset/round finish. Never put lineage or actual State into View.
    pub fn read_lineage(&self, role: Role) -> Option<physical::Lineage> {
        self.world.read_lineage(role)
    }
    fn checkpoint(&mut self, checkpoint: Checkpoint) {
        for h in &mut self.histories {
            h.checkpoint = checkpoint;
        }
        if let Checkpoint::Boundary(boundary) = checkpoint {
            let position = match boundary {
                Boundary::Calibration(p) => Some(p),
                Boundary::Trial { .. } => Some(
                    boundary
                        .trial_position(self.protocol.investigator)
                        .expect("validated trial"),
                ),
                Boundary::Probe { .. } => None,
            };
            if let Some(position) = position {
                self.histories[1 - self.protocol.investigator.index()].checkpoint =
                    Checkpoint::BeforeSlot(position);
            }
        }
    }
    fn step(&self, before: [u8; 2], events: Vec<(Role, Event)>) -> Step {
        Step {
            histories: self.histories.clone(),
            events,
            next: self.next_boundary(),
            spent_delta: [before[0] - self.credits[0], before[1] - self.credits[1]],
        }
    }
    fn start_phase(&mut self, phase: Phase) {
        self.world.reset(phase);
        for role in [Role::A, Role::B] {
            let index = role.index();
            self.histories[index]
                .entries
                .push(Entry::Physical(LocalEntry::Reset { phase }));
            self.private_bits[index] = match phase {
                Phase::Calibration => None,
                Phase::Live { trial } => {
                    let (a, b) = self.bits.0[usize::from(trial)];
                    let bit = if role == Role::A { a } else { b };
                    self.histories[index]
                        .entries
                        .push(Entry::Physical(LocalEntry::PrivateBit { trial, bit }));
                    Some(bit)
                }
            };
        }
    }
    fn expose(&mut self, position: Position) {
        let owned =
            physical::role_at(position).expect("validated position") == self.protocol.investigator;
        let boundary = match position.phase {
            Phase::Calibration if owned && self.protocol.policy.raw() => {
                Some(Boundary::Calibration(position))
            }
            Phase::Live { trial } if owned && self.routine.is_none() => {
                Some(Boundary::Trial { trial })
            }
            _ => None,
        };
        if let Some(boundary) = boundary {
            self.clock = Clock::Choice(boundary);
            self.checkpoint(Checkpoint::Boundary(boundary));
        } else {
            self.clock = Clock::Before {
                position,
                queued: None,
            };
            self.checkpoint(Checkpoint::BeforeSlot(position));
        }
    }
    fn supplied_action(&self, role: Role, position: Position) -> Result<Action, Error> {
        let prefix = self.prefix(role);
        let models = match prefix.own_prior {
            OwnPrior::Uniform => [Probability::new(1, 4)?; 4],
            OwnPrior::PointMass(model) => Mechanism::ALL
                .map(|m| Probability::new(u64::from(m == model), 1).expect("bounded point prior")),
        };
        let view = LocalView {
            prefix: LocalPrefix {
                role,
                ids: prefix.ids.clone(),
                own_prior: prefix.own_prior,
                checkpoint: physical::Checkpoint::Action(position),
                entries: prefix
                    .entries
                    .iter()
                    .filter_map(|e| {
                        if let Entry::Physical(e) = e {
                            Some(e.clone())
                        } else {
                            None
                        }
                    })
                    .collect(),
            },
            credits: self.credits(role),
            private_bit: self.private_bits[role.index()],
            belief: Belief {
                models,
                target: None,
            },
        };
        Ok(physical::decide(&view, Knowledge::Known)?.action)
    }
    fn action(
        &self,
        role: Role,
        position: Position,
        queued: Option<Action>,
    ) -> Result<Action, Error> {
        if let Some(action) = queued {
            return Ok(action);
        }
        if position.phase == Phase::Calibration || role != self.protocol.investigator {
            return self.supplied_action(role, position);
        }
        let routine = self.routine.ok_or_else(|| {
            Error::InvalidHistory("investigator has no selected live routine".into())
        })?;
        let bit = self.private_bits[role.index()]
            .ok_or_else(|| Error::InvalidHistory("live routine lacks issued own bit".into()))?;
        super::types::live_action(role, position, routine, bit)
    }
}
/// Queue a free owned selection. Delivered clocks, credits and fields stay intact.
pub fn select_choice(state: &mut EpisodeState, choice: Choice) -> Result<(), Error> {
    let Clock::Choice(boundary) = state.clock else {
        return Err(Error::InvalidHistory(
            "choice requires an unselected boundary".into(),
        ));
    };
    state.protocol.validate_choice(boundary, choice)?;
    let required = match choice {
        Choice::Continue => 2,
        Choice::Stop => 0,
        Choice::Calibration(_) => 1,
        Choice::Inspect => 9,
        Choice::Attempt => 6,
    };
    if state.credits(state.protocol.investigator) < required
        || (choice == Choice::Continue && state.credits.iter().any(|c| *c < 2))
    {
        return Err(Error::InvalidHistory(format!(
            "insufficient credits for {choice:?} at {boundary:?}"
        )));
    }
    let next = match (boundary, choice) {
        (Boundary::Probe { completed }, Choice::Continue) => Clock::ProbeStart {
            round: completed + 1,
        },
        (Boundary::Probe { .. }, Choice::Stop) => Clock::EnterLive { trial: 0 },
        (Boundary::Calibration(position), Choice::Calibration(action)) => Clock::Before {
            position,
            queued: Some(action),
        },
        (b @ Boundary::Trial { .. }, _) => Clock::Before {
            position: b.trial_position(state.protocol.investigator)?,
            queued: None,
        },
        _ => unreachable!("checked boundary and choice"),
    };
    state.histories[state.protocol.investigator.index()]
        .entries
        .push(Entry::OwnChoice { boundary, choice });
    if let (Boundary::Probe { completed }, Choice::Stop) = (boundary, choice) {
        for h in &mut state.histories {
            h.entries.push(Entry::PublicProbeStop { completed });
        }
    }
    if matches!(boundary, Boundary::Trial { .. }) {
        state.routine = Some(choice);
    }
    state.clock = next;
    Ok(())
}
fn pay(
    state: &mut EpisodeState,
    role: Role,
    position: Position,
    action: Action,
) -> Result<Event, Error> {
    let target = match position.phase {
        Phase::Calibration => None,
        Phase::Live { trial } => {
            let (a, b) = state.bits.0[usize::from(trial)];
            Some(if role == Role::A { b } else { a })
        }
    };
    // The original World checks all fallible conditions before mutating fields.
    let event = state
        .world
        .apply(role, position, &action, target, state.credits(role))?;
    state.credits[role.index()] = event.credits_after;
    state.histories[role.index()]
        .entries
        .push(Entry::Physical(LocalEntry::Action(event.clone())));
    Ok(event)
}
/// Execute one paid slot or explicit free transition. A paid return always
/// precedes later round finish, reset, issue, or another paid slot.
pub fn advance_one(state: &mut EpisodeState) -> Result<Step, Error> {
    let before = state.credits;
    let mut events = Vec::new();
    match state.clock {
        Clock::Choice(b) => {
            return Err(Error::InvalidHistory(format!(
                "unselected boundary {b:?} requires choice"
            )))
        }
        Clock::Finished => return Err(Error::InvalidHistory("episode is already Finished".into())),
        Clock::ProbeStart { round } => {
            if round == 1 {
                state.start_phase(Phase::Calibration);
            }
            state.expose(Position {
                phase: Phase::Calibration,
                round,
                slot: 1,
            });
        }
        Clock::EnterLive { trial } => enter_live(state, trial),
        Clock::Before { position, queued } => {
            let role = physical::role_at(position)?;
            let action = state.action(role, position, queued)?;
            let event = pay(state, role, position, action)?;
            state.clock = Clock::After(position);
            state.checkpoint(Checkpoint::AfterSlot(position));
            events.push((role, event));
        }
        Clock::After(p) => {
            if p.slot == 4 {
                state.world.finish_round();
            }
            match p.phase {
                Phase::Calibration if p.slot == 4 => {
                    let boundary = Boundary::Probe { completed: p.round };
                    state.clock = Clock::Choice(boundary);
                    state.checkpoint(Checkpoint::Boundary(boundary));
                }
                Phase::Live { trial } if p.round == 3 && p.slot == 4 => {
                    state.clock = Clock::Prediction { trial };
                    state.checkpoint(Checkpoint::Prediction { trial });
                }
                _ => state.expose(if p.slot == 4 {
                    Position {
                        round: p.round + 1,
                        slot: 1,
                        ..p
                    }
                } else {
                    Position {
                        slot: p.slot + 1,
                        ..p
                    }
                }),
            }
        }
        Clock::Prediction { trial: 3 } => {
            state.clock = Clock::Finished;
            state.checkpoint(Checkpoint::Finished);
        }
        Clock::Prediction { trial } => enter_live(state, trial + 1),
    }
    Ok(state.step(before, events))
}
fn enter_live(state: &mut EpisodeState, trial: u8) {
    state.start_phase(Phase::Live { trial });
    state.routine = None;
    state.expose(Position {
        phase: Phase::Live { trial },
        round: 1,
        slot: 1,
    });
}

#[cfg(test)]
mod failure_tests {
    use super::*;
    fn state() -> EpisodeState {
        EpisodeState::new(
            &Protocol::new(Role::A, PolicyKind::RawAdaptive),
            Environment::InFamily(Mechanism::SharedPersistent),
            EpisodeBits::from_index(0).unwrap(),
            [
                OwnPrior::Uniform,
                OwnPrior::PointMass(Mechanism::SharedPersistent),
            ],
        )
        .unwrap()
    }
    // Characterizes rejection-before-effect using the original paid World seam.
    #[test]
    fn insufficient_budget_does_not_change_selected_paid_clock_or_world() {
        let mut s = state();
        select_choice(&mut s, Choice::Continue).unwrap();
        advance_one(&mut s).unwrap();
        select_choice(&mut s, Choice::Calibration(Action::Write(Symbol::Data1))).unwrap();
        s.credits[0] = 0;
        let histories = s.histories.clone();
        assert!(advance_one(&mut s).is_err());
        assert_eq!(s.histories, histories);
        assert_eq!(s.credits, [0, 48]);
        assert!(s.world.read_lineage(Role::A).is_none());
        assert!(s.next_boundary().is_none());
        s.credits[0] = 48;
        let paid = advance_one(&mut s).unwrap();
        assert_eq!(paid.events[0].1.action, Action::Write(Symbol::Data1));
    }
    #[test]
    fn insufficient_whole_round_budget_rejects_before_selection_effects() {
        let mut s = state();
        s.credits[1] = 1;
        let histories = s.histories.clone();
        assert!(select_choice(&mut s, Choice::Continue).is_err());
        assert_eq!(s.histories, histories);
        assert_eq!(s.credits, [48, 1]);
        assert_eq!(s.next_boundary(), Some(Boundary::Probe { completed: 0 }));
    }
    #[test]
    fn wrong_owner_or_phase_paid_failure_preserves_world_and_local_state() {
        let mut s = state();
        let histories = s.histories.clone();
        let position = Position {
            phase: Phase::Calibration,
            round: 1,
            slot: 1,
        };
        assert!(pay(&mut s, Role::B, position, Action::Write(Symbol::Data1)).is_err());
        assert!(pay(&mut s, Role::A, position, Action::InspectOwnTarget).is_err());
        assert_eq!(s.histories, histories);
        assert_eq!(s.credits, [48, 48]);
        assert!(s.world.read_lineage(Role::A).is_none());
        assert_eq!(s.next_boundary(), Some(Boundary::Probe { completed: 0 }));
    }
}
