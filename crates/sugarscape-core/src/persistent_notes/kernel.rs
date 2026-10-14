use super::{observation, types::*, validation::validate_checkpoint};

/// Researcher state; participant decisions receive a single owned View instead.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct World {
    config: Config,
    state: Checkpoint,
    note_survived: bool,
    completion_clock: Option<Clock>,
}
fn invalid_history(message: &str) -> Error {
    Error::InvalidHistory(message.into())
}
fn commit_candidate<T: Clone, F>(live: &mut T, transaction: F) -> Result<(), Error>
where
    F: FnOnce(&mut T) -> Result<(), Error>,
{
    let mut candidate = live.clone();
    transaction(&mut candidate)?;
    *live = candidate;
    Ok(())
}
pub(super) fn charge(credits: &mut u64, spent: &mut u64, cost: u64) -> Result<(), Error> {
    let next_credits = credits
        .checked_sub(cost)
        .ok_or_else(|| invalid_history("insufficient credits"))?;
    let next_spent = spent.checked_add(cost).ok_or(Error::ArithmeticOverflow)?;
    *credits = next_credits;
    *spent = next_spent;
    Ok(())
}
impl World {
    pub(crate) fn new(config: &Config) -> Result<Self, Error> {
        config.validate()?;
        Self::new_private(config)
    }
    pub(super) fn new_private(config: &Config) -> Result<Self, Error> {
        config.validate_private()?;
        let clock = Clock {
            boundary: 0,
            phase: Phase::Writer,
            writer_slot: 0,
            reader_slot: 0,
        };
        let public = PublicStatus {
            writer_active: true,
            reader_active: true,
            terminal: false,
            terminal_reason: None,
        };
        let participants = [Role::Writer, Role::Reader].map(|role| Participant {
            role,
            cell: Some(if role == Role::Writer {
                Cell::C
            } else {
                Cell::O
            }),
            active: true,
            key: role == Role::Writer,
            cargo: None,
            credits: 32,
            spent: 0,
            reward: 0,
        });
        let physical = Physical {
            request: config.request,
            field: Note::Blank,
            key_owner: Role::Writer,
            participants,
            goods: [true; 2],
            lineage: None,
            settled: false,
        };
        let own = [Role::Writer, Role::Reader]
            .map(|role| observation::initial_view(config, role, &clock, &public, &physical));
        let state = Checkpoint {
            clock,
            public,
            physical,
            own,
            event: None,
        };
        validate_checkpoint(&state)?;
        Ok(Self {
            config: config.clone(),
            state,
            note_survived: false,
            completion_clock: None,
        })
    }
    pub(crate) fn checkpoint(&self) -> &Checkpoint {
        &self.state
    }
    #[cfg(test)]
    pub(crate) fn act(&mut self, actor: Role, primitive: Primitive) -> Result<(), Error> {
        self.act_with_capture(actor, primitive, &mut |_| Ok(()))
    }
    pub(crate) fn act_with_capture<F: FnMut(&Checkpoint) -> Result<(), Error>>(
        &mut self,
        actor: Role,
        primitive: Primitive,
        capture: &mut F,
    ) -> Result<(), Error> {
        validate_checkpoint(&self.state)?;
        primitive.validate()?;
        commit_candidate(self, |candidate| {
            candidate.action(actor, primitive)?;
            validate_checkpoint(&candidate.state)?;
            capture(&candidate.state)
        })
    }
    fn action(&mut self, actor: Role, primitive: Primitive) -> Result<(), Error> {
        let p = &mut self.state;
        let idx = actor.index();
        let a = &p.physical.participants[idx];
        if p.public.terminal || !a.active {
            return Err(invalid_history("actor unavailable"));
        }
        if (actor == Role::Writer && (p.clock.phase != Phase::Writer || p.clock.writer_slot >= 4))
            || (actor == Role::Reader
                && (!matches!(p.clock.phase, Phase::Departure | Phase::Reader)
                    || p.clock.reader_slot >= 6))
        {
            return Err(invalid_history("wrong phase or opportunity"));
        }
        let kind = primitive.kind;
        if matches!(
            (actor, kind),
            (
                Role::Writer,
                PrimitiveKind::ReadNote | PrimitiveKind::Pickup | PrimitiveKind::Deliver
            ) | (
                Role::Reader,
                PrimitiveKind::PassKey | PrimitiveKind::WriteNote
            )
        ) {
            return Err(Error::InvalidInput("wrong role primitive".into()));
        }
        let cell = a.cell.ok_or_else(|| invalid_history("actor unavailable"))?;
        let s = &self.config.stratum;
        let cost = match kind {
            PrimitiveKind::Wait => 0,
            PrimitiveKind::Move
            | PrimitiveKind::PassKey
            | PrimitiveKind::Pickup
            | PrimitiveKind::Deliver => 1,
            PrimitiveKind::InspectManifest => {
                if actor == Role::Writer {
                    s.acquisition_cost
                } else {
                    s.manifest_inspection_cost
                }
            }
            PrimitiveKind::WriteNote => s.write_cost,
            PrimitiveKind::ReadNote => s.note_read_cost,
        };
        let block = match kind {
            PrimitiveKind::Move => {
                let target = primitive
                    .cell
                    .ok_or_else(|| Error::InvalidInput("move payload".into()))?;
                if !matches!(
                    (cell, target),
                    (Cell::P, Cell::O)
                        | (Cell::O, Cell::P)
                        | (Cell::O, Cell::C)
                        | (Cell::C, Cell::O)
                ) {
                    return Err(Error::InvalidInput("nonadjacent move".into()));
                }
                if matches!((cell, target), (Cell::O, Cell::C) | (Cell::C, Cell::O)) && !a.key {
                    Some(BlockReason::NoKey)
                } else {
                    None
                }
            }
            PrimitiveKind::InspectManifest => {
                if cell != Cell::C {
                    Some(BlockReason::WrongLocation)
                } else {
                    None
                }
            }
            PrimitiveKind::PassKey => {
                if cell != Cell::O {
                    Some(BlockReason::WrongLocation)
                } else if !a.key {
                    Some(BlockReason::NoKey)
                } else if !p.physical.participants[1].active
                    || p.physical.participants[1].cell != Some(Cell::O)
                {
                    Some(BlockReason::NoRecipient)
                } else {
                    None
                }
            }
            PrimitiveKind::WriteNote => {
                if p.own[idx].known_request.is_none()
                    || !p.own[idx]
                        .history
                        .iter()
                        .any(|f| f.observed_request.is_some())
                {
                    return Err(invalid_history("missing paid observation"));
                }
                if cell != Cell::O {
                    Some(BlockReason::WrongLocation)
                } else {
                    None
                }
            }
            PrimitiveKind::ReadNote => {
                if cell != Cell::O {
                    Some(BlockReason::WrongLocation)
                } else {
                    None
                }
            }
            PrimitiveKind::Pickup => {
                if cell != Cell::C {
                    Some(BlockReason::WrongLocation)
                } else if a.cargo.is_some() {
                    Some(BlockReason::CargoOccupied)
                } else {
                    if !p.physical.goods[primitive
                        .good
                        .ok_or_else(|| Error::InvalidInput("pickup payload".into()))?
                        .index()]
                    {
                        return Err(invalid_history("unavailable good"));
                    }
                    None
                }
            }
            PrimitiveKind::Deliver => {
                if cell != Cell::O {
                    Some(BlockReason::WrongLocation)
                } else if a.cargo.is_none() {
                    Some(BlockReason::NoCargo)
                } else {
                    None
                }
            }
            PrimitiveKind::Wait => None,
        };
        let a = &mut p.physical.participants[idx];
        charge(&mut a.credits, &mut a.spent, cost)?;
        p.clock.boundary = p
            .clock
            .boundary
            .checked_add(1)
            .ok_or(Error::ArithmeticOverflow)?;
        if actor == Role::Writer {
            p.clock.writer_slot = p
                .clock
                .writer_slot
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
        } else {
            p.clock.reader_slot = p
                .clock
                .reader_slot
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
            p.clock.phase = Phase::Reader;
        }
        let mut feedback = Feedback {
            clock: p.clock.clone(),
            primitive: primitive.clone(),
            status: if block.is_none() {
                ActionStatus::Applied
            } else {
                ActionStatus::Blocked
            },
            block_reason: block,
            cost,
            observed_request: None,
            read_symbol: None,
            picked_good: None,
            reward: 0,
        };
        if block.is_none() {
            match kind {
                PrimitiveKind::Wait => {}
                PrimitiveKind::Move => p.physical.participants[idx].cell = primitive.cell,
                PrimitiveKind::InspectManifest => {
                    feedback.observed_request = Some(p.physical.request);
                    p.own[idx].known_request = Some(p.physical.request);
                    p.own[idx].belief = observation::belief(Some(p.physical.request));
                }
                PrimitiveKind::PassKey => {
                    p.physical.key_owner = Role::Reader;
                    p.physical.participants[0].key = false;
                    p.physical.participants[1].key = true;
                    for role in [Role::Writer, Role::Reader] {
                        p.own[role.index()].key_receipt = Some(PossessionReceipt {
                            clock: p.clock.clone(),
                            key: role == Role::Reader,
                        });
                    }
                }
                PrimitiveKind::WriteNote => {
                    let symbol = primitive
                        .note
                        .ok_or_else(|| Error::InvalidInput("write payload".into()))?;
                    let observation = p.own[idx]
                        .history
                        .iter()
                        .rev()
                        .find(|f| f.observed_request.is_some())
                        .ok_or_else(|| invalid_history("missing paid observation"))?;
                    p.physical.field = symbol;
                    p.physical.lineage = Some(Lineage {
                        writer: Role::Writer,
                        observed_request: observation
                            .observed_request
                            .ok_or_else(|| invalid_history("missing paid observation"))?,
                        observation_clock: observation.clock.clone(),
                        write_clock: p.clock.clone(),
                        symbol,
                    });
                }
                PrimitiveKind::ReadNote => {
                    let symbol = p.physical.field;
                    feedback.read_symbol = Some(symbol);
                    p.own[idx].note_read = Some(ReadRecord {
                        symbol,
                        clock: p.clock.clone(),
                    });
                    let decoded = match symbol {
                        Note::Blank => None,
                        Note::NoteLeft => Some(Good::Left),
                        Note::NoteRight => Some(Good::Right),
                    };
                    if decoded.is_some() {
                        p.own[idx].known_request = decoded;
                        p.own[idx].belief = observation::belief(decoded);
                    }
                }
                PrimitiveKind::Pickup => {
                    let good = primitive
                        .good
                        .ok_or_else(|| Error::InvalidInput("pickup payload".into()))?;
                    p.physical.goods[good.index()] = false;
                    p.physical.participants[idx].cargo = Some(good);
                    feedback.picked_good = Some(good);
                }
                PrimitiveKind::Deliver => {
                    let good = p.physical.participants[idx]
                        .cargo
                        .take()
                        .ok_or_else(|| invalid_history("missing cargo"))?;
                    p.physical.settled = true;
                    p.public.terminal = true;
                    p.public.terminal_reason = Some(TerminalReason::Delivered);
                    self.completion_clock = Some(p.clock.clone());
                    if good == p.physical.request {
                        p.physical.participants[0].reward = s.writer_share;
                        p.physical.participants[1].reward = 12 - s.writer_share;
                        feedback.reward = 12 - s.writer_share;
                    }
                }
            }
        }
        p.own[idx].history.push(feedback.clone());
        p.event = Some(Event {
            kind: EventKind::Action,
            clock: p.clock.clone(),
            actor: Some(actor),
            feedback: Some(feedback),
            erased: false,
            terminal_reason: p.public.terminal_reason,
        });
        observation::synchronize(p);
        if kind != PrimitiveKind::ReadNote {
            observation::sense(&mut p.own[idx], &p.physical, &p.clock);
        }
        Ok(())
    }
    #[cfg(test)]
    pub(crate) fn depart(&mut self) -> Result<(), Error> {
        self.depart_with_capture(&mut |_| Ok(()))
    }
    pub(crate) fn depart_with_capture<F: FnMut(&Checkpoint) -> Result<(), Error>>(
        &mut self,
        capture: &mut F,
    ) -> Result<(), Error> {
        validate_checkpoint(&self.state)?;
        commit_candidate(self, |w| {
            let p = &mut w.state;
            if p.clock.phase != Phase::Writer || p.clock.writer_slot != 4 || p.public.terminal {
                return Err(invalid_history("departure boundary"));
            }
            p.clock.boundary = p
                .clock
                .boundary
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
            p.clock.phase = Phase::Departure;
            p.public.writer_active = false;
            p.physical.participants[0].active = false;
            p.physical.participants[0].cell = None;
            let erased = w.config.arm == Arm::Erased;
            if erased {
                p.physical.field = Note::Blank;
                p.physical.lineage = None;
            }
            w.note_survived = p.physical.field != Note::Blank;
            p.event = Some(Event {
                kind: EventKind::Departure,
                clock: p.clock.clone(),
                actor: None,
                feedback: None,
                erased,
                terminal_reason: None,
            });
            observation::synchronize(p);
            validate_checkpoint(p)?;
            capture(p)
        })
    }
    #[cfg(test)]
    pub(crate) fn finish(&mut self) -> Result<(), Error> {
        self.finish_with_capture(&mut |_| Ok(()))
    }
    pub(crate) fn finish_with_capture<F: FnMut(&Checkpoint) -> Result<(), Error>>(
        &mut self,
        capture: &mut F,
    ) -> Result<(), Error> {
        validate_checkpoint(&self.state)?;
        commit_candidate(self, |w| {
            let p = &mut w.state;
            if p.clock.phase == Phase::Terminal
                || p.clock.writer_slot != 4
                || p.clock.phase != Phase::Reader
                || (!p.public.terminal && p.clock.reader_slot != 6)
            {
                return Err(invalid_history("terminal marker boundary"));
            }
            if !p.public.terminal {
                p.public.terminal = true;
                p.public.terminal_reason = Some(TerminalReason::Deadline);
            }
            p.clock.boundary = p
                .clock
                .boundary
                .checked_add(1)
                .ok_or(Error::ArithmeticOverflow)?;
            p.clock.phase = Phase::Terminal;
            p.event = Some(Event {
                kind: EventKind::Settlement,
                clock: p.clock.clone(),
                actor: None,
                feedback: None,
                erased: false,
                terminal_reason: p.public.terminal_reason,
            });
            observation::synchronize(p);
            validate_checkpoint(p)?;
            capture(p)
        })
    }
    #[cfg(test)]
    pub(super) fn reset_field(&mut self) -> Result<(), Error> {
        commit_candidate(self, |w| {
            w.state.physical.field = Note::Blank;
            w.state.physical.lineage = None;
            w.state.event = None;
            validate_checkpoint(&w.state)
        })
    }
}

impl World {
    pub(crate) fn outcome(&self) -> Result<Outcome, Error> {
        validate_checkpoint(&self.state)?;
        let p = &self.state;
        if p.clock.phase != Phase::Terminal {
            return Err(invalid_history("missing terminal marker"));
        }
        let terminal_reason = p
            .public
            .terminal_reason
            .ok_or_else(|| invalid_history("missing terminal reason"))?;
        let mut costs = Costs::default();
        let mut counts = Counts::default();
        let mut selected_good = None;
        for role in [Role::Writer, Role::Reader] {
            let idx = role.index();
            for f in &p.own[idx].history {
                counts.attempted[idx] = counts.attempted[idx]
                    .checked_add(1)
                    .ok_or(Error::ArithmeticOverflow)?;
                let category = match f.primitive.kind {
                    PrimitiveKind::Wait => &mut costs.wait,
                    PrimitiveKind::Move => &mut costs.movement,
                    PrimitiveKind::InspectManifest => {
                        if role == Role::Writer {
                            &mut costs.acquisition
                        } else {
                            &mut costs.inspection
                        }
                    }
                    PrimitiveKind::PassKey => &mut costs.handoff,
                    PrimitiveKind::WriteNote => &mut costs.writing,
                    PrimitiveKind::ReadNote => &mut costs.reading,
                    PrimitiveKind::Pickup => &mut costs.pickup,
                    PrimitiveKind::Deliver => &mut costs.delivery,
                };
                category[idx] = category[idx]
                    .checked_add(f.cost)
                    .ok_or(Error::ArithmeticOverflow)?;
                if f.status == ActionStatus::Blocked {
                    counts.blocked[idx] += 1;
                } else {
                    counts.successful[idx] += 1;
                    match f.primitive.kind {
                        PrimitiveKind::InspectManifest => counts.inspections[idx] += 1,
                        PrimitiveKind::PassKey => counts.key_passes += 1,
                        PrimitiveKind::WriteNote => counts.writes += 1,
                        PrimitiveKind::ReadNote => {
                            counts.reads += 1;
                            if f.read_symbol != Some(Note::Blank) {
                                counts.nonblank_reads += 1;
                            }
                        }
                        PrimitiveKind::Pickup => selected_good = f.picked_good,
                        _ => {}
                    }
                }
            }
        }
        let spent = p.physical.participants.clone().map(|a| a.spent);
        let reward = p.physical.participants.clone().map(|a| a.reward);
        let mut net = [0i64; 2];
        for idx in 0..2 {
            net[idx] = i64::try_from(reward[idx])
                .map_err(|_| Error::ArithmeticOverflow)?
                .checked_sub(i64::try_from(spent[idx]).map_err(|_| Error::ArithmeticOverflow)?)
                .ok_or(Error::ArithmeticOverflow)?;
        }
        let completed = terminal_reason == TerminalReason::Delivered;
        Ok(Outcome {
            correct: completed && selected_good == Some(p.physical.request),
            completed,
            deadline: terminal_reason == TerminalReason::Deadline,
            terminal_reason,
            completion_clock: self.completion_clock.clone(),
            selected_good,
            belief: p.own[1].belief.clone(),
            spent,
            reward,
            net,
            group_net: net[0]
                .checked_add(net[1])
                .ok_or(Error::ArithmeticOverflow)?,
            costs,
            counts,
            note_survived: self.note_survived,
        })
    }
}
