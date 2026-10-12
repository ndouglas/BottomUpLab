use super::types::*;

pub(super) fn observe(ids: &Ids, state: &PhysicalState, role: Role, clock: Clock) -> Observation {
    let visible = visible_cells(state.positions[role.index()]);
    Observation {
        clock,
        cells: visible
            .iter()
            .map(|cell| VisibleCell {
                cell: *cell,
                occupants: [Role::Giver, Role::Receiver]
                    .into_iter()
                    .filter(|r| state.positions[r.index()] == *cell)
                    .map(|r| VisibleOccupant {
                        role: r,
                        id: ids.get(r).into(),
                        cargo: r == Role::Receiver && state.receiver_cargo,
                        key: state.key_owner == r,
                    })
                    .collect(),
                loose_token: *cell == Cell::C && state.resource == ResourceLocation::Cache,
            })
            .collect(),
    }
}
fn visible_cells(cell: Cell) -> &'static [Cell] {
    match cell {
        Cell::P => &[Cell::P, Cell::O],
        Cell::O => &[Cell::P, Cell::O, Cell::C],
        Cell::C => &[Cell::O, Cell::C],
    }
}
fn clocks(end: Clock) -> Result<Vec<Clock>, Error> {
    end.validate()?;
    let mut result = vec![Clock {
        round: 0,
        phase: Phase::Initial,
        role: None,
    }];
    if end == result[0] {
        return Ok(result);
    }
    for round in 1..=6 {
        for role in [Role::Giver, Role::Receiver] {
            for phase in [Phase::BeforeAction, Phase::AfterAction] {
                let c = Clock {
                    round,
                    phase,
                    role: Some(role),
                };
                result.push(c);
                if end == c {
                    return Ok(result);
                }
            }
        }
        if end.phase == Phase::Finished && end.round == round {
            result.push(end);
            return Ok(result);
        }
    }
    Err(history_error("unreachable local clock"))
}
fn own_image(o: &Observation, role: Role) -> Result<(Cell, &VisibleOccupant), Error> {
    let mut found = o.cells.iter().flat_map(|c| {
        c.occupants
            .iter()
            .filter(move |a| a.role == role)
            .map(move |a| (c.cell, a))
    });
    let own = found
        .next()
        .ok_or_else(|| history_error("missing own occupant"))?;
    if found.next().is_some() {
        return Err(history_error("duplicate own occupant"));
    }
    Ok(own)
}
/// Replay only the own history. No peer schedule or authoritative world is available.
pub(super) fn validate_view(v: &View) -> Result<(), Error> {
    v.clock.validate()?;
    if !matches!(
        (&v.word, v.role),
        (OwnWord::Giver(_), Role::Giver) | (OwnWord::Receiver(_), Role::Receiver)
    ) {
        return Err(history_error("own word role"));
    }
    let first = match v.history.get(1) {
        Some(LocalEntry::Observed(o)) => o,
        _ => return Err(history_error("initial own observation")),
    };
    first.validate()?;
    let (receiver_cell, receiver) = own_image(first, Role::Receiver)?;
    if receiver_cell != Cell::O
        || receiver.cargo
        || first
            .cells
            .iter()
            .any(|c| c.cell == Cell::C && !c.loose_token)
    {
        return Err(history_error("initial Receiver position, cargo or cache"));
    }
    let (mut cell, own) = own_image(first, v.role)?;
    let own_id = own.id.clone();
    let mut key = own.key;
    let mut cargo = false;
    let mut spent = 0u8;
    let mut reward = 0u8;
    let mut cursor = 0;
    let mut last = None;
    let mut delivered = false;
    let peer_role = if v.role == Role::Giver {
        Role::Receiver
    } else {
        Role::Giver
    };
    let peer_id = first
        .cells
        .iter()
        .flat_map(|c| &c.occupants)
        .find(|o| o.role == peer_role)
        .ok_or_else(|| history_error("initial peer occupant"))?
        .id
        .clone();
    if own_id == peer_id
        || own.cargo
        || v.role == Role::Receiver && cell != Cell::O
        || v.role == Role::Giver && cell == Cell::C
    {
        return Err(history_error("initial own state"));
    }
    for clock in clocks(v.clock)? {
        let termination = match v.history.get(cursor) {
            Some(LocalEntry::Public {
                clock: c,
                termination,
            }) if *c == clock => *termination,
            _ => return Err(history_error("public chronology")),
        };
        cursor += 1;
        if clock.phase != Phase::Finished && termination.is_some() {
            return Err(history_error("early termination"));
        }
        if delivered && clock.phase != Phase::Finished {
            return Err(history_error("postdelivery action"));
        }
        let active = clock.role == Some(v.role);
        let mut successful_pass = false;
        if clock.phase == Phase::AfterAction && active {
            let (token, intent, result, paid) = match v.history.get(cursor) {
                Some(LocalEntry::OwnAction {
                    clock: c,
                    token,
                    intent,
                    result,
                    paid,
                }) if *c == clock => (token, intent, result, *paid),
                _ => return Err(history_error("own action chronology")),
            };
            cursor += 1;
            if *token != v.word.token(clock.round)?
                || *intent != super::controller::own_intent(v.role, token.clone(), cell, cargo)?
            {
                return Err(history_error("own action choice"));
            }
            let cost = match token {
                Token::Giver(GiverToken::Wait) | Token::Receiver(ReceiverToken::Wait) => paid == 0,
                Token::Giver(GiverToken::Pass) => [1, 2, 4].contains(&paid),
                _ => paid == 1,
            };
            if !cost {
                return Err(history_error("own charge"));
            }
            spent = spent
                .checked_add(paid)
                .filter(|x| *x <= 32)
                .ok_or(Error::ArithmeticOverflow)?;
            if *result == ActionOutcome::Success {
                match intent {
                    Intent::Wait => {}
                    Intent::Pass => {
                        if !key {
                            return Err(history_error("Pass without own key"));
                        }
                        successful_pass = true;
                    }
                    Intent::Move(to) => {
                        if v.role == Role::Receiver && !key
                            || v.role == Role::Giver && cell == Cell::O
                        {
                            return Err(history_error("unauthorized move"));
                        }
                        cell = *to;
                    }
                    Intent::Pickup => cargo = true,
                    Intent::Deliver => {
                        cargo = false;
                        delivered = true;
                    }
                }
            } else {
                let valid = match (intent, result) {
                    (
                        Intent::Move(Cell::O),
                        ActionOutcome::Blocked(BlockReason::AlreadyAtOutside),
                    ) if v.role == Role::Giver => cell == Cell::O,
                    (Intent::Move(_), ActionOutcome::Blocked(BlockReason::NotKeyOwner))
                        if v.role == Role::Receiver =>
                    {
                        !key
                    }
                    (Intent::Pass, ActionOutcome::Blocked(BlockReason::TransferDisabled)) => true,
                    (Intent::Pass, ActionOutcome::Blocked(BlockReason::NotKeyOwner)) => !key,
                    (Intent::Pass, ActionOutcome::Blocked(BlockReason::NotColocated)) => key,
                    _ => false,
                };
                if !valid {
                    return Err(history_error("own blocked result"));
                }
            }
        }
        let mut received = false;
        if let Some(LocalEntry::Ownership(receipt)) = v.history.get(cursor) {
            receipt.validate()?;
            if receipt.clock != clock
                || receipt.before != key
                || receipt.counterpart != peer_id
                || receipt.after != (v.role == Role::Receiver)
                || v.role == Role::Giver && !successful_pass
            {
                return Err(history_error("own ownership receipt"));
            }
            key = receipt.after;
            cursor += 1;
            received = true;
        }
        if successful_pass && !received {
            return Err(history_error("missing sender receipt"));
        }
        let should_sense = clock.phase == Phase::Initial
            || active && matches!(clock.phase, Phase::BeforeAction | Phase::AfterAction);
        if should_sense {
            let o = match v.history.get(cursor) {
                Some(LocalEntry::Observed(o)) if o.clock == clock => o,
                _ => return Err(history_error("own sensing chronology")),
            };
            cursor += 1;
            o.validate()?;
            let (observed_cell, observed) = own_image(o, v.role)?;
            if observed_cell != cell
                || observed.id != own_id
                || observed.cargo != cargo
                || observed.key != key
                || !o
                    .cells
                    .iter()
                    .map(|c| c.cell)
                    .eq(visible_cells(cell).iter().copied())
            {
                return Err(history_error("own sense disagreement"));
            }
            if o.cells
                .iter()
                .flat_map(|c| &c.occupants)
                .any(|a| a.role == peer_role && a.id != peer_id)
            {
                return Err(history_error("changed peer identity"));
            }
            last = Some(o);
        }
        if clock.phase == Phase::Finished {
            let term = termination.ok_or_else(|| history_error("missing termination"))?;
            if term == Termination::Deadline && clock.round != 6
                || v.role == Role::Receiver && (term == Termination::Delivered) != delivered
            {
                return Err(history_error("termination mismatch"));
            }
            let (r, n) = match v.history.get(cursor) {
                Some(LocalEntry::Settlement {
                    clock: c,
                    reward,
                    net,
                }) if *c == clock => (*reward, *net),
                _ => return Err(history_error("settlement chronology")),
            };
            cursor += 1;
            if term == Termination::Deadline && r != 0
                || term == Termination::Delivered
                    && !if v.role == Role::Giver {
                        [0, 3, 6].contains(&r)
                    } else {
                        [6, 9, 12].contains(&r)
                    }
                || n != i16::from(r) - i16::from(spent)
            {
                return Err(history_error("own settlement"));
            }
            reward = r;
        }
    }
    if cursor != v.history.len()
        || last != Some(&v.last_observation)
        || v.own_cell != cell
        || v.own_cargo != cargo
        || v.own_key != key
        || v.credits != 32 - spent
        || v.own_reward != reward
        || v.own_net != i16::from(reward) - i16::from(spent)
    {
        return Err(history_error("View does not match own history"));
    }
    Ok(())
}
