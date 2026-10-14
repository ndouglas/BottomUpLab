use super::{
    budget::{Budget, Limits},
    kernel::World,
    types::*,
};
fn history(message: &str) -> Error {
    Error::InvalidHistory(message.into())
}
fn writer_note(view: &View) -> Result<Note, Error> {
    match view.known_request {
        Some(Good::Left) => Ok(Note::NoteLeft),
        Some(Good::Right) => Ok(Note::NoteRight),
        None => Err(Error::InvalidHistory(
            "missing own manifest observation".into(),
        )),
    }
}
pub fn blank_fallback(i: u64, g: u64) -> Result<BlankChoice, Error> {
    let twice_cost = i.checked_mul(2).ok_or(Error::ArithmeticOverflow)?;
    Ok(if twice_cost < g {
        BlankChoice::Inspect
    } else {
        BlankChoice::GuessLeft
    })
}
fn move_to(cell: Cell) -> Primitive {
    Primitive {
        cell: Some(cell),
        ..Primitive::new(PrimitiveKind::Move)
    }
}
fn pickup(good: Good) -> Primitive {
    Primitive {
        good: Some(good),
        ..Primitive::new(PrimitiveKind::Pickup)
    }
}
fn target(view: &View) -> Result<Good, Error> {
    view.known_request
        .ok_or_else(|| history("missing own target observation"))
}
fn decoded(symbol: Note) -> Option<Good> {
    match symbol {
        Note::Blank => None,
        Note::NoteLeft => Some(Good::Left),
        Note::NoteRight => Some(Good::Right),
    }
}
/// Observation claims must be supported by this role's own paid feedback.
fn validate_sources(view: &View) -> Result<(), Error> {
    let slot = match view.role {
        Role::Writer => view.clock.writer_slot,
        Role::Reader => view.clock.reader_slot,
    };
    let maximum = match view.role {
        Role::Writer => 4,
        Role::Reader => 6,
    };
    if slot > maximum || view.history.len() as u64 != slot {
        return Err(history("own history does not match opportunity clock"));
    }
    let oracle = view.role == Role::Reader && view.reader_routine == Some(ReaderRoutine::Oracle);
    let mut expected = if oracle { Some(target(view)?) } else { None };
    let mut read = None;
    for (index, feedback) in view.history.iter().enumerate() {
        if feedback.status != ActionStatus::Applied {
            continue;
        }
        match feedback.primitive.kind {
            PrimitiveKind::InspectManifest => {
                let allowed = match view.role {
                    Role::Writer => {
                        index == 0
                            && matches!(
                                view.writer_program,
                                Some(WriterProgram::ObserveWrite | WriterProgram::ObserveSilent)
                            )
                    }
                    Role::Reader => match view.reader_routine {
                        Some(ReaderRoutine::InspectFirst) => index == 1,
                        Some(ReaderRoutine::ReadAdaptive) => {
                            index == 2
                                && read
                                    .is_some_and(|r: &Feedback| r.read_symbol == Some(Note::Blank))
                                && blank_fallback(
                                    view.public.manifest_inspection_cost,
                                    view.public.own_share,
                                )? == BlankChoice::Inspect
                        }
                        _ => false,
                    },
                };
                if !allowed {
                    return Err(history("unsupported own manifest observation"));
                }
                expected = Some(
                    feedback
                        .observed_request
                        .ok_or_else(|| history("missing own manifest observation"))?,
                );
            }
            PrimitiveKind::ReadNote => {
                if view.role != Role::Reader
                    || view.reader_routine != Some(ReaderRoutine::ReadAdaptive)
                    || index != 0
                {
                    return Err(history("unsupported own note read"));
                }
                let symbol = feedback
                    .read_symbol
                    .ok_or_else(|| history("missing own note observation"))?;
                if let Some(good) = decoded(symbol) {
                    expected = Some(good);
                }
                read = Some(feedback);
            }
            _ => {}
        }
    }
    if view.known_request != expected {
        return Err(history("own target contradicts observation history"));
    }
    match (read, view.note_read.as_ref()) {
        (Some(feedback), Some(record))
            if feedback.read_symbol == Some(record.symbol) && feedback.clock == record.clock => {}
        (None, None) => {}
        _ => return Err(history("own read record contradicts observation history")),
    }
    Ok(())
}
/// Choose a supplied finite program using only the participant's own dated view.
pub fn choose(view: &View) -> Result<Primitive, Error> {
    validate_sources(view)?;
    if view.status.terminal {
        return Err(history("choice after terminal"));
    }
    match view.role {
        Role::Writer => {
            if !view.status.writer_active || view.clock.phase != Phase::Writer {
                return Err(history("inactive Writer choice"));
            }
            let program = view
                .writer_program
                .ok_or_else(|| history("missing own Writer program"))?;
            Ok(match view.clock.writer_slot {
                0 => Primitive::new(if program == WriterProgram::Silent {
                    PrimitiveKind::Wait
                } else {
                    PrimitiveKind::InspectManifest
                }),
                1 => move_to(Cell::O),
                2 => Primitive::new(PrimitiveKind::PassKey),
                3 => {
                    if program == WriterProgram::ObserveWrite {
                        Primitive {
                            note: Some(writer_note(view)?),
                            ..Primitive::new(PrimitiveKind::WriteNote)
                        }
                    } else {
                        Primitive::new(PrimitiveKind::Wait)
                    }
                }
                _ => return Err(history("Writer program exhausted")),
            })
        }
        Role::Reader => {
            if !matches!(view.clock.phase, Phase::Departure | Phase::Reader) {
                return Err(history("Reader choice before departure"));
            }
            let routine = view
                .reader_routine
                .ok_or_else(|| history("missing own Reader routine"))?;
            let slot = view.clock.reader_slot;
            let read = routine == ReaderRoutine::ReadAdaptive;
            if read && slot == 0 {
                return Ok(Primitive::new(PrimitiveKind::ReadNote));
            }
            let inspect = if routine == ReaderRoutine::InspectFirst {
                true
            } else if read {
                let record = view
                    .note_read
                    .as_ref()
                    .ok_or_else(|| history("missing own note read"))?;
                record.symbol == Note::Blank
                    && blank_fallback(view.public.manifest_inspection_cost, view.public.own_share)?
                        == BlankChoice::Inspect
            } else {
                false
            };
            let route_slot = slot
                .checked_sub(u64::from(read))
                .ok_or(Error::ArithmeticOverflow)?;
            Ok(match route_slot {
                0 => move_to(Cell::C),
                1 if inspect => Primitive::new(PrimitiveKind::InspectManifest),
                n if n == 1 + u64::from(inspect) => {
                    pickup(if routine == ReaderRoutine::BlindGuess {
                        Good::Left
                    } else if read {
                        match view
                            .note_read
                            .as_ref()
                            .ok_or_else(|| history("missing own note read"))?
                            .symbol
                        {
                            Note::NoteLeft => Good::Left,
                            Note::NoteRight => Good::Right,
                            Note::Blank if inspect => target(view)?,
                            Note::Blank => Good::Left,
                        }
                    } else {
                        target(view)?
                    })
                }
                n if n == 2 + u64::from(inspect) => move_to(Cell::O),
                n if n == 3 + u64::from(inspect) => Primitive::new(PrimitiveKind::Deliver),
                _ => return Err(history("Reader program exhausted")),
            })
        }
    }
}
pub fn run_episode(config: &Config) -> Result<Episode, Error> {
    run_with_capture(config, |_| Ok(()))
}
pub fn run_with_capture<F: FnMut(&Checkpoint) -> Result<(), Error>>(
    config: &Config,
    mut capture: F,
) -> Result<Episode, Error> {
    let mut budget = Budget::new(&Limits::default())?;
    run_budgeted(config, &mut capture, &mut budget)
}
pub(crate) fn run_budgeted<F: FnMut(&Checkpoint) -> Result<(), Error>>(
    config: &Config,
    capture: &mut F,
    budget: &mut Budget,
) -> Result<Episode, Error> {
    budget.row("episode_rows", 29463)?;
    config.validate()?;
    if budget.limits.checkpoints_per_episode == 0 {
        return Err(Error::ResourceLimit {
            resource: "checkpoints_per_episode".into(),
            limit: 0,
        });
    }
    budget.charge("checkpoints", 1)?;
    budget.charge("map_cell_visits", 2)?;
    let mut world = World::new(config)?;
    let mut checkpoints = Vec::new();
    let mut append = |checkpoint: &Checkpoint| {
        if checkpoints.len() >= 16 {
            return Err(Error::ResourceLimit {
                resource: "checkpoints_per_episode".into(),
                limit: 16,
            });
        }
        capture(checkpoint)?;
        checkpoints.push(checkpoint.clone());
        Ok(())
    };
    append(world.checkpoint())?;
    for _ in 0..4 {
        let action = choose(&world.checkpoint().own[0])?;
        budget.capture(
            world.checkpoint(),
            Some(&action),
            Some(Role::Writer),
            true,
            world.checkpoint().clock.boundary + 1,
        )?;
        world.act_with_capture(Role::Writer, action, &mut append)?;
    }
    budget.capture(
        world.checkpoint(),
        None,
        None,
        true,
        world.checkpoint().clock.boundary + 1,
    )?;
    world.depart_with_capture(&mut append)?;
    while !world.checkpoint().public.terminal && world.checkpoint().clock.reader_slot < 6 {
        let action = choose(&world.checkpoint().own[1])?;
        budget.capture(
            world.checkpoint(),
            Some(&action),
            Some(Role::Reader),
            true,
            world.checkpoint().clock.boundary + 1,
        )?;
        world.act_with_capture(Role::Reader, action, &mut append)?;
    }
    budget.capture(
        world.checkpoint(),
        None,
        None,
        true,
        world.checkpoint().clock.boundary + 1,
    )?;
    world.finish_with_capture(&mut append)?;
    Ok(Episode {
        key: config.key(),
        config: config.clone(),
        checkpoints,
        outcome: world.outcome()?,
    })
}
