use super::types::*;

pub(super) fn writer_program(arm: Arm) -> WriterProgram {
    match arm {
        Arm::Retained | Arm::Erased => WriterProgram::ObserveWrite,
        Arm::NoNote => WriterProgram::ObserveSilent,
        Arm::NoAcquireNoNote => WriterProgram::Silent,
    }
}
pub(super) fn belief(request: Option<Good>) -> [Ratio; 2] {
    match request {
        None => [Ratio::HALF, Ratio::HALF],
        Some(Good::Left) => [Ratio::ONE, Ratio::ZERO],
        Some(Good::Right) => [Ratio::ZERO, Ratio::ONE],
    }
}
pub(super) fn initial_view(
    config: &Config,
    role: Role,
    clock: &Clock,
    status: &PublicStatus,
    physical: &Physical,
) -> View {
    let s = &config.stratum;
    let known_request =
        (role == Role::Reader && config.routine == ReaderRoutine::Oracle).then_some(config.request);
    let a = &physical.participants[role.index()];
    let mut view = View {
        role,
        writer_program: (role == Role::Writer).then_some(writer_program(config.arm)),
        reader_routine: (role == Role::Reader).then_some(config.routine),
        public: PublicSettings {
            acquisition_cost: s.acquisition_cost,
            write_cost: s.write_cost,
            note_read_cost: s.note_read_cost,
            manifest_inspection_cost: s.manifest_inspection_cost,
            own_share: if role == Role::Writer {
                s.writer_share
            } else {
                12 - s.writer_share
            },
            initial_credits: 32,
            writer_opportunities: 4,
            reader_opportunities: 6,
            correct_total_reward: 12,
            request_prior: [Ratio::HALF; 2],
        },
        clock: clock.clone(),
        status: status.clone(),
        cell: a.cell,
        key: a.key,
        cargo: a.cargo,
        credits: 32,
        spent: 0,
        reward: 0,
        known_request,
        belief: belief(known_request),
        map: LocalMap {
            sensed_at: clock.clone(),
            cells: Vec::with_capacity(3),
        },
        note_read: None,
        key_receipt: None,
        history: Vec::with_capacity(if role == Role::Writer { 4 } else { 6 }),
    };
    sense(&mut view, physical, clock);
    view
}
pub(super) fn synchronize(p: &mut Checkpoint) {
    for role in [Role::Writer, Role::Reader] {
        let a = &p.physical.participants[role.index()];
        let v = &mut p.own[role.index()];
        v.clock = p.clock.clone();
        v.status = p.public.clone();
        v.cell = a.cell;
        v.key = a.key;
        v.cargo = a.cargo;
        v.credits = a.credits;
        v.spent = a.spent;
        v.reward = a.reward;
    }
}
pub(super) fn sense(view: &mut View, physical: &Physical, clock: &Clock) {
    let Some(cell) = view.cell else {
        return;
    };
    let occupants: Vec<_> = physical
        .participants
        .iter()
        .filter(|a| a.active && a.cell == Some(cell))
        .map(|a| a.role)
        .collect();
    let key_holder = physical
        .participants
        .iter()
        .find(|a| a.active && a.cell == Some(cell) && a.key)
        .map(|a| a.role);
    let goods = if cell == Cell::C {
        [Good::Left, Good::Right]
            .into_iter()
            .filter(|g| physical.goods[g.index()])
            .collect()
    } else {
        Vec::new()
    };
    let entry = MapCell {
        cell,
        sensed_at: clock.clone(),
        occupants,
        key_holder,
        goods,
    };
    if let Some(old) = view.map.cells.iter_mut().find(|old| old.cell == cell) {
        *old = entry;
    } else {
        view.map.cells.push(entry);
        view.map.cells.sort_by_key(|c| c.cell);
    }
    view.map.sensed_at = clock.clone();
}
