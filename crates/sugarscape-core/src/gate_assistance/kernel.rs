use super::choose;
use super::observation::observe;
use super::types::*;

type CaptureHook<'a> = Option<&'a mut dyn FnMut(&CapturePoint) -> Result<(), Error>>;

struct World {
    state: PhysicalState,
    histories: [Vec<LocalEntry>; 2],
    observations: [Observation; 2],
    words: [OwnWord; 2],
    events: Vec<Event>,
    outcome: Outcome,
}
impl World {
    fn new(config: &Config, profile: Profile) -> Result<Self, Error> {
        let state = PhysicalState {
            positions: [
                if config.stratum.distance() == 0 {
                    Cell::P
                } else {
                    Cell::O
                },
                Cell::O,
            ],
            retriever_cargo: false,
            resource: ResourceLocation::Cache,
            gate_open: false,
            spent: [0, 0],
            reward: [0, 0],
        };
        let clock = Clock {
            round: 0,
            phase: Phase::Initial,
            role: None,
        };
        let observations = [
            observe(&state, Role::Holder, clock),
            observe(&state, Role::Retriever, clock),
        ];
        let histories = std::array::from_fn(|i| {
            vec![
                LocalEntry::Public {
                    clock,
                    termination: None,
                },
                LocalEntry::Observed(observations[i].clone()),
            ]
        });
        Ok(Self {
            state,
            histories,
            observations,
            words: [
                OwnWord::Holder(HolderWord::from_id(profile.holder)?),
                OwnWord::Retriever(RetrieverWord::from_id(profile.retriever)?),
            ],
            events: Vec::new(),
            outcome: Outcome {
                spent: [0, 0],
                reward: [0, 0],
                net: [0, 0],
                group_net: 0,
                delivery_round: None,
                holder_attempts: [0; 3],
                holder_blocked: [0; 3],
                retriever_attempts: [0; 4],
                retriever_blocked: [0; 4],
                retriever_waits: 0,
            },
        })
    }
    fn view(&self, role: Role, clock: Clock) -> View {
        let i = role.index();
        View {
            role,
            word: self.words[i].clone(),
            own_cell: self.state.positions[i],
            own_cargo: role == Role::Retriever && self.state.retriever_cargo,
            credits: 32 - self.state.spent[i],
            own_reward: self.state.reward[i],
            clock,
            history: self.histories[i].clone(),
            last_observation: self.observations[i].clone(),
        }
    }
    fn capture(&self, clock: Clock, capture: &mut CaptureHook<'_>) -> Result<(), Error> {
        let Some(capture) = capture.as_deref_mut() else {
            return Ok(());
        };
        capture(&CapturePoint {
            clock,
            views: [
                self.view(Role::Holder, clock),
                self.view(Role::Retriever, clock),
            ],
            physical: self.state.clone(),
        })
    }
    fn public(&mut self, clock: Clock, termination: Option<Termination>) {
        for history in &mut self.histories {
            history.push(LocalEntry::Public { clock, termination });
        }
    }
    fn observation(&mut self, role: Role, clock: Clock) {
        let i = role.index();
        self.observations[i] = observe(&self.state, role, clock);
        self.histories[i].push(LocalEntry::Observed(self.observations[i].clone()));
    }
    fn action(
        &mut self,
        config: &Config,
        role: Role,
        clock: Clock,
        intent: Primitive,
    ) -> Result<bool, Error> {
        let token = self.words[role.index()].token(clock.round)?;
        let paid = match token {
            Token::Holder(HolderToken::Wait) | Token::Retriever(RetrieverToken::Wait) => 0,
            Token::Holder(HolderToken::Hold) => config.stratum.holding_cost(),
            _ => 1,
        };
        let before = self.state.clone();
        let (after, result) = transaction(&before, role, intent, paid)?;
        validate_feedback(role, token, intent, result, paid)?;
        // Count updates and the transaction are checked before any authoritative commit.
        let mut outcome = self.outcome.clone();
        let blocked = matches!(result, ActionOutcome::Blocked(_));
        let (attempts, blocks, index) = match role {
            Role::Holder => {
                let index = match token {
                    Token::Holder(HolderToken::Wait) => 0,
                    Token::Holder(HolderToken::Travel) => 1,
                    Token::Holder(HolderToken::Hold) => 2,
                    _ => return Err(history_error("Holder token mismatch")),
                };
                (
                    &mut outcome.holder_attempts[..],
                    &mut outcome.holder_blocked[..],
                    index,
                )
            }
            Role::Retriever => {
                let index = match intent {
                    Primitive::Move(Cell::C) => 0,
                    Primitive::Pickup => 1,
                    Primitive::Move(Cell::O) => 2,
                    Primitive::Deliver => 3,
                    Primitive::Wait => {
                        outcome.retriever_waits = outcome
                            .retriever_waits
                            .checked_add(1)
                            .ok_or(Error::ArithmeticOverflow)?;
                        usize::MAX
                    }
                    _ => return Err(history_error("Retriever intent mismatch")),
                };
                (
                    &mut outcome.retriever_attempts[..],
                    &mut outcome.retriever_blocked[..],
                    index,
                )
            }
        };
        if index != usize::MAX {
            attempts[index] = attempts[index]
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
            if blocked {
                blocks[index] = blocks[index]
                    .checked_add(1)
                    .ok_or(Error::ArithmeticOverflow)?;
            }
        }
        let delivered = intent == Primitive::Deliver && result == ActionOutcome::Success;
        if delivered {
            outcome.delivery_round = Some(clock.round);
        }
        self.state = after;
        self.outcome = outcome;
        self.events.push(Event {
            clock,
            kind: EventKind::Action {
                role,
                token,
                intent,
                result,
                paid,
            },
            before,
            after: self.state.clone(),
        });
        self.public(clock, None);
        self.histories[role.index()].push(LocalEntry::OwnAction {
            clock,
            token,
            intent,
            result,
            paid,
        });
        self.observation(role, clock);
        Ok(delivered)
    }
    fn finish(
        mut self,
        config: &Config,
        profile: Profile,
        round: u8,
        termination: Termination,
        capture: &mut CaptureHook<'_>,
    ) -> Result<Episode, Error> {
        let clock = Clock {
            round,
            phase: Phase::Finished,
            role: None,
        };
        let before = self.state.clone();
        let mut after = before.clone();
        after.gate_open = false;
        if termination == Termination::Delivered {
            after.reward = [
                config.stratum.holder_share(),
                12 - config.stratum.holder_share(),
            ];
        }
        after.validate()?;
        self.state = after;
        self.events.push(Event {
            clock,
            kind: EventKind::Terminal(termination),
            before,
            after: self.state.clone(),
        });
        self.public(clock, Some(termination));
        for role in [Role::Holder, Role::Retriever] {
            let i = role.index();
            self.histories[i].push(LocalEntry::Settlement {
                clock,
                reward: self.state.reward[i],
            });
        }
        self.outcome.spent = self.state.spent;
        self.outcome.reward = self.state.reward;
        self.outcome.net = std::array::from_fn(|i| {
            i16::from(self.state.reward[i]) - i16::from(self.state.spent[i])
        });
        self.outcome.group_net = self.outcome.net[0]
            .checked_add(self.outcome.net[1])
            .ok_or(Error::ArithmeticOverflow)?;
        self.outcome.validate()?;
        self.capture(clock, capture)?;
        Ok(Episode {
            version: 1,
            config: config.clone(),
            profile,
            events: self.events,
            histories: self.histories,
            outcome: self.outcome,
        })
    }
}

/// Validate and charge a candidate state, returning it only after all invariants pass.
pub(super) fn transaction(
    before: &PhysicalState,
    role: Role,
    intent: Primitive,
    paid: u8,
) -> Result<(PhysicalState, ActionOutcome), Error> {
    before.validate()?;
    let mut after = before.clone();
    let i = role.index();
    after.spent[i] = after.spent[i]
        .checked_add(paid)
        .ok_or(Error::ArithmeticOverflow)?;
    if after.spent[i] > 32 {
        return Err(history_error("action exceeds own credits"));
    }
    let cell = before.positions[i];
    let result = match (role, intent) {
        (_, Primitive::Wait) => ActionOutcome::Success,
        (Role::Holder, Primitive::Move(Cell::P)) => {
            if cell == Cell::P {
                ActionOutcome::Blocked(BlockReason::AlreadyAtPlate)
            } else {
                after.positions[i] = Cell::P;
                ActionOutcome::Success
            }
        }
        (Role::Holder, Primitive::OperatePlate) => {
            if cell != Cell::P {
                ActionOutcome::Blocked(BlockReason::AwayFromPlate)
            } else {
                after.gate_open = true;
                ActionOutcome::Success
            }
        }
        (Role::Retriever, Primitive::Move(destination @ (Cell::O | Cell::C))) => {
            let from = if destination == Cell::C {
                Cell::O
            } else {
                Cell::C
            };
            let expected_cargo = destination == Cell::O;
            if cell != from {
                ActionOutcome::Blocked(BlockReason::WrongPosition)
            } else if before.retriever_cargo != expected_cargo {
                ActionOutcome::Blocked(if expected_cargo {
                    BlockReason::EmptyHands
                } else {
                    BlockReason::FullHands
                })
            } else if !before.gate_open {
                ActionOutcome::Blocked(BlockReason::GateClosed)
            } else {
                after.positions[i] = destination;
                ActionOutcome::Success
            }
        }
        (Role::Retriever, Primitive::Pickup) => {
            if cell != Cell::C {
                ActionOutcome::Blocked(BlockReason::WrongPosition)
            } else if before.retriever_cargo {
                ActionOutcome::Blocked(BlockReason::FullHands)
            } else if before.resource != ResourceLocation::Cache {
                ActionOutcome::Blocked(BlockReason::NoToken)
            } else {
                after.retriever_cargo = true;
                after.resource = ResourceLocation::Carried;
                ActionOutcome::Success
            }
        }
        (Role::Retriever, Primitive::Deliver) => {
            if cell != Cell::O {
                ActionOutcome::Blocked(BlockReason::WrongPosition)
            } else if !before.retriever_cargo {
                ActionOutcome::Blocked(BlockReason::EmptyHands)
            } else {
                after.retriever_cargo = false;
                after.resource = ResourceLocation::Delivered;
                ActionOutcome::Success
            }
        }
        _ => {
            return Err(history_error(
                "primitive violates role/capability or destination",
            ))
        }
    };
    after.validate()?;
    Ok((after, result))
}

pub fn run_episode(config: &Config, profile: Profile) -> Result<Episode, Error> {
    execute(config, profile, None)
}
pub fn run_with_capture<F>(
    config: &Config,
    profile: Profile,
    capture: &mut F,
) -> Result<Episode, Error>
where
    F: FnMut(&CapturePoint) -> Result<(), Error>,
{
    execute(config, profile, Some(capture))
}
fn execute(
    config: &Config,
    profile: Profile,
    mut capture: CaptureHook<'_>,
) -> Result<Episode, Error> {
    config.validate()?;
    profile.validate()?;
    let mut world = World::new(config, profile)?;
    world.capture(
        Clock {
            round: 0,
            phase: Phase::Initial,
            role: None,
        },
        &mut capture,
    )?;
    for round in 1..=6 {
        let clock = Clock {
            round,
            phase: Phase::Reset,
            role: None,
        };
        let before = world.state.clone();
        world.state.gate_open = false;
        world.events.push(Event {
            clock,
            kind: EventKind::Reset,
            before,
            after: world.state.clone(),
        });
        world.public(clock, None);
        world.capture(clock, &mut capture)?;
        for role in [Role::Holder, Role::Retriever] {
            let clock = Clock {
                round,
                phase: Phase::BeforeAction,
                role: Some(role),
            };
            world.public(clock, None);
            world.observation(role, clock);
            world.capture(clock, &mut capture)?;
            let intent = choose(&world.view(role, clock))?;
            let clock = Clock {
                phase: Phase::AfterAction,
                ..clock
            };
            let delivered = world.action(config, role, clock, intent)?;
            world.capture(clock, &mut capture)?;
            if delivered {
                return world.finish(config, profile, round, Termination::Delivered, &mut capture);
            }
        }
        if round == 6 {
            return world.finish(config, profile, round, Termination::Deadline, &mut capture);
        }
    }
    Err(history_error("six-round kernel failed to terminate"))
}
