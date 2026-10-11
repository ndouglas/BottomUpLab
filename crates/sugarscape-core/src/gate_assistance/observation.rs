use super::controller::own_primitive;
use super::types::*;

/// Invoked only by the kernel at initial and before/after own-action boundaries.
pub(super) fn observe(state: &PhysicalState, role: Role, clock: Clock) -> Observation {
    let cells: &[Cell] = match state.positions[role.index()] {
        Cell::P => &[Cell::P, Cell::O],
        Cell::O => &[Cell::P, Cell::O, Cell::C],
        Cell::C => &[Cell::O, Cell::C],
    };
    Observation {
        clock,
        cells: cells
            .iter()
            .map(|cell| VisibleCell {
                cell: *cell,
                occupants: [Role::Holder, Role::Retriever]
                    .into_iter()
                    .filter(|role| state.positions[role.index()] == *cell)
                    .map(|role| (role, role == Role::Retriever && state.retriever_cargo))
                    .collect(),
                loose_token: *cell == Cell::C && state.resource == ResourceLocation::Cache,
            })
            .collect(),
        gate: if state.positions[role.index()] == Cell::P {
            None
        } else {
            Some(state.gate_open)
        },
    }
}

fn observe_own(o: &Observation, role: Role, cell: Cell, cargo: bool) -> Result<(), Error> {
    o.validate()?;
    let wanted: &[Cell] = match cell {
        Cell::P => &[Cell::P, Cell::O],
        Cell::O => &[Cell::P, Cell::O, Cell::C],
        Cell::C => &[Cell::O, Cell::C],
    };
    if !o.cells.iter().map(|c| c.cell).eq(wanted.iter().copied()) {
        return Err(history_error("own observation visibility mismatch"));
    }
    let mut own = Vec::new();
    for c in &o.cells {
        for (r, has_cargo) in &c.occupants {
            if *r == role {
                own.push((c.cell, *has_cargo));
            }
        }
    }
    if own != [(cell, cargo)] {
        return Err(history_error("observation disagrees with own state"));
    }
    Ok(())
}

fn clocks_until(end: Clock) -> Result<Vec<Clock>, Error> {
    let mut clocks = vec![Clock {
        round: 0,
        phase: Phase::Initial,
        role: None,
    }];
    if end == clocks[0] {
        return Ok(clocks);
    }
    for round in 1..=6 {
        clocks.push(Clock {
            round,
            phase: Phase::Reset,
            role: None,
        });
        if end == *clocks.last().unwrap() {
            return Ok(clocks);
        }
        for role in [Role::Holder, Role::Retriever] {
            for phase in [Phase::BeforeAction, Phase::AfterAction] {
                clocks.push(Clock {
                    round,
                    phase,
                    role: Some(role),
                });
                if end == *clocks.last().unwrap() {
                    return Ok(clocks);
                }
            }
        }
        if end.phase == Phase::Finished && end.round == round {
            clocks.push(end);
            return Ok(clocks);
        }
    }
    Err(history_error("history end clock outside chronology"))
}

/// Validate only the committed local interface; peer schedules/physics are absent.
pub(super) fn validate_view(view: &View) -> Result<(), Error> {
    view.clock.validate()?;
    if !matches!(
        (&view.word, view.role),
        (OwnWord::Holder(_), Role::Holder) | (OwnWord::Retriever(_), Role::Retriever)
    ) || view.credits > 32
        || (view.role == Role::Holder && (view.own_cell == Cell::C || view.own_cargo))
        || (view.role == Role::Retriever && view.own_cell == Cell::P)
    {
        return Err(history_error(
            "invalid own role, word, cell, cargo or credits",
        ));
    }
    let first = match view.history.get(1) {
        Some(LocalEntry::Observed(o)) => o,
        _ => return Err(history_error("missing initial observation")),
    };
    let initial_cells: Vec<_> = first
        .cells
        .iter()
        .filter(|c| c.occupants.contains(&(view.role, false)))
        .map(|c| c.cell)
        .collect();
    let [mut cell] = initial_cells.as_slice() else {
        return Err(history_error("missing own initial location"));
    };
    if (view.role == Role::Retriever && cell != Cell::O)
        || (view.role == Role::Holder && cell == Cell::C)
    {
        return Err(history_error("invalid initial own location"));
    }
    let mut cargo = false;
    let mut delivered = false;
    let mut spent = 0u8;
    let mut reward = 0u8;
    let mut last: Option<&Observation> = None;
    let mut cursor = 0;
    let mut holding_cost = None;
    for clock in clocks_until(view.clock)? {
        if delivered && clock.phase != Phase::Finished {
            return Err(history_error(
                "public action opportunity after own delivery",
            ));
        }
        let termination = match view.history.get(cursor) {
            Some(LocalEntry::Public {
                clock: c,
                termination,
            }) if *c == clock => *termination,
            _ => return Err(history_error("missing or reordered public clock")),
        };
        cursor += 1;
        if (clock.phase == Phase::Finished) != termination.is_some() {
            return Err(history_error("public termination at wrong phase"));
        }
        let active = clock.role == Some(view.role);
        if active && clock.phase == Phase::AfterAction {
            let Some(LocalEntry::OwnAction {
                clock: c,
                token,
                intent,
                result,
                paid,
            }) = view.history.get(cursor)
            else {
                return Err(history_error("missing own action feedback"));
            };
            if *c != clock
                || *token != view.word.token(clock.round)?
                || *intent != own_primitive(view.role, *token, cell, cargo)?
            {
                return Err(history_error("own feedback does not follow own word/state"));
            }
            validate_feedback(view.role, *token, *intent, *result, *paid)?;
            if view.role == Role::Retriever && matches!(intent, Primitive::Move(Cell::O | Cell::C))
            {
                let observed = last
                    .ok_or_else(|| history_error("crossing lacks own BeforeAction observation"))?;
                let before = Clock {
                    round: clock.round,
                    phase: Phase::BeforeAction,
                    role: Some(view.role),
                };
                if observed.clock != before {
                    return Err(history_error(
                        "crossing lacks fresh own BeforeAction observation",
                    ));
                }
                if !matches!(
                    (observed.gate, *result),
                    (Some(true), ActionOutcome::Success)
                        | (Some(false), ActionOutcome::Blocked(BlockReason::GateClosed))
                ) {
                    return Err(history_error(
                        "crossing feedback contradicts fresh own gate observation",
                    ));
                }
            }
            if *intent == Primitive::OperatePlate {
                if holding_cost.is_some_and(|k| k != *paid) {
                    return Err(history_error("inconsistent holding charge"));
                }
                holding_cost = Some(*paid);
            }
            spent = spent.checked_add(*paid).ok_or(Error::ArithmeticOverflow)?;
            if spent > 32 {
                return Err(history_error("own credits exhausted"));
            }
            match (*intent, *result) {
                (Primitive::Move(Cell::P), ActionOutcome::Success) if cell != Cell::O => {
                    return Err(history_error("Travel success away from O"))
                }
                (Primitive::Move(Cell::P), ActionOutcome::Blocked(BlockReason::AlreadyAtPlate))
                    if cell != Cell::P =>
                {
                    return Err(history_error("Travel block away from P"))
                }
                (Primitive::OperatePlate, ActionOutcome::Success) if cell != Cell::P => {
                    return Err(history_error("Hold success away from P"))
                }
                (Primitive::OperatePlate, ActionOutcome::Blocked(BlockReason::AwayFromPlate))
                    if cell != Cell::O =>
                {
                    return Err(history_error("Hold block at P"))
                }
                (Primitive::Move(to), ActionOutcome::Success) => cell = to,
                (Primitive::Pickup, ActionOutcome::Success) => cargo = true,
                (Primitive::Deliver, ActionOutcome::Success) => {
                    cargo = false;
                    delivered = true;
                }
                (
                    _,
                    ActionOutcome::Blocked(
                        BlockReason::WrongPosition
                        | BlockReason::EmptyHands
                        | BlockReason::FullHands
                        | BlockReason::NoToken,
                    ),
                ) => {
                    return Err(history_error(
                        "impossible block from shortest-route controller",
                    ))
                }
                _ => {}
            }
            cursor += 1;
        }
        if clock.phase == Phase::Initial
            || (active && matches!(clock.phase, Phase::BeforeAction | Phase::AfterAction))
        {
            let Some(LocalEntry::Observed(o)) = view.history.get(cursor) else {
                return Err(history_error("missing own observation"));
            };
            if o.clock != clock {
                return Err(history_error("observation clock mismatch"));
            }
            observe_own(o, view.role, cell, cargo)?;
            last = Some(o);
            cursor += 1;
        }
        if clock.phase == Phase::Finished {
            if view.role == Role::Retriever
                && delivered != (termination == Some(Termination::Delivered))
            {
                return Err(history_error(
                    "public termination disagrees with own delivery",
                ));
            }
            let Some(LocalEntry::Settlement {
                clock: c,
                reward: r,
            }) = view.history.get(cursor)
            else {
                return Err(history_error("missing own settlement"));
            };
            let valid = match termination {
                Some(Termination::Deadline) => clock.round == 6 && *r == 0,
                Some(Termination::Delivered) => {
                    clock.round >= 4
                        && match view.role {
                            Role::Holder => [0, 3, 6].contains(r),
                            Role::Retriever => [6, 9, 12].contains(r),
                        }
                }
                None => false,
            };
            if *c != clock || !valid {
                return Err(history_error("invalid own settlement"));
            }
            reward = *r;
            cursor += 1;
        }
    }
    if cursor != view.history.len()
        || last != Some(&view.last_observation)
        || cell != view.own_cell
        || cargo != view.own_cargo
        || 32 - spent != view.credits
        || reward != view.own_reward
    {
        return Err(history_error("View does not match committed local prefix"));
    }
    Ok(())
}
