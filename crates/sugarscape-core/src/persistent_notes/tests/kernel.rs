use super::super::kernel::World;
use super::super::*;
fn initial_checkpoint(config: &Config) -> Result<Checkpoint, Error> {
    Ok(World::new(config)?.checkpoint().clone())
}
#[test]
fn initial_role_custody_and_information() {
    let c = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Right);
    let p = initial_checkpoint(&c).unwrap();
    assert_eq!(p.own[Role::Reader.index()].known_request, None);
    assert_eq!(p.physical.key_owner, Role::Writer);
    assert_eq!(p.physical.field, Note::Blank);
}
#[test]
fn every_sealed_raw_transaction_matches() {
    let fixture: serde_json::Value =
        serde_json::from_slice(super::super::source_inputs::REFERENCE_INPUTS[1].1).unwrap();
    for w in fixture["payload"]["witnesses"].as_array().unwrap() {
        let config: Config = serde_json::from_value(w["config"].clone()).unwrap();
        let mut world = World::new_private(&config).unwrap();
        let mut n = 0;
        let mut replay_final = None;
        for op in w["operations"].as_array().unwrap() {
            if op["kind"] == "CloneReplay" {
                let mut replay = world.clone();
                for tail in w["operations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .skip_while(|x| *x != op)
                    .skip(1)
                {
                    replay_operation(&mut replay, tail).unwrap();
                }
                replay_final = Some(replay);
                continue;
            }
            let before = world.clone();
            match replay_operation(&mut world, op) {
                Ok(()) => {
                    assert_eq!(
                        serde_json::to_value(world.checkpoint()).unwrap(),
                        w["expected"][n],
                        "{} operation {}",
                        w["name"],
                        n
                    );
                    n += 1;
                }
                Err(e) => {
                    assert_eq!(world, before);
                    assert!(!w["expected_error"].is_null(), "unexpected error {e:?}");
                    assert_eq!(e.kind(), w["expected_error"]["kind"].as_str().unwrap());
                    break;
                }
            }
        }
        assert_eq!(n, w["expected"].as_array().unwrap().len());
        if let Some(replay) = replay_final {
            assert_eq!(world, replay);
        }
    }
}
fn replay_operation(world: &mut World, op: &serde_json::Value) -> Result<(), Error> {
    match op["kind"].as_str().unwrap() {
        "Act" => world.act(
            serde_json::from_value(op["actor"].clone()).unwrap(),
            serde_json::from_value(op["primitive"].clone()).unwrap(),
        ),
        "Depart" => world.depart(),
        "FinishDeadline" => world.finish(),
        "ResetField" => world.reset_field(),
        other => panic!("unsupported {other}"),
    }
}
#[test]
fn callback_failure_keeps_transaction_unchanged() {
    let c = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Left);
    let mut w = World::new(&c).unwrap();
    let before = w.clone();
    assert!(w
        .act_with_capture(
            Role::Writer,
            Primitive::new(PrimitiveKind::InspectManifest),
            &mut |_| Err(Error::Io("capture failed".into()))
        )
        .is_err());
    assert_eq!(w, before);
}
#[test]
fn departure_and_deadline_callback_failures_are_atomic() {
    let c = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Left);
    let mut w = World::new(&c).unwrap();
    for _ in 0..4 {
        w.act(Role::Writer, Primitive::new(PrimitiveKind::Wait))
            .unwrap();
    }
    let before = w.clone();
    assert!(w
        .depart_with_capture(&mut |_| Err(Error::Io("departure capture".into())))
        .is_err());
    assert_eq!(w, before);
    w.depart().unwrap();
    for _ in 0..6 {
        w.act(Role::Reader, Primitive::new(PrimitiveKind::Wait))
            .unwrap();
    }
    let before = w.clone();
    assert!(w
        .finish_with_capture(&mut |_| Err(Error::Io("final capture".into())))
        .is_err());
    assert_eq!(w, before);
    w.finish().unwrap();
    let o = w.outcome().unwrap();
    assert!(o.deadline);
    assert!(!o.completed);
    assert_eq!(o.spent, [0, 0]);
}
#[test]
fn impossible_insufficient_credit_attempt_preserves_entire_world() {
    let mut c = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Left);
    c.stratum.acquisition_cost = 33;
    let mut w = World::new_private(&c).unwrap();
    let before = w.clone();
    assert_eq!(
        w.act(Role::Writer, Primitive::new(PrimitiveKind::InspectManifest)),
        Err(Error::InvalidHistory("insufficient credits".into()))
    );
    assert_eq!(w, before);
}
#[test]
fn original_departed_writer_entitlement_is_paid_once() {
    let mut c = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Right);
    c.stratum.writer_share = 6;
    let mut w = World::new(&c).unwrap();
    w.act(Role::Writer, Primitive::new(PrimitiveKind::InspectManifest))
        .unwrap();
    let mut movement = Primitive::new(PrimitiveKind::Move);
    movement.cell = Some(Cell::O);
    w.act(Role::Writer, movement).unwrap();
    w.act(Role::Writer, Primitive::new(PrimitiveKind::PassKey))
        .unwrap();
    let mut write = Primitive::new(PrimitiveKind::WriteNote);
    write.note = Some(Note::NoteRight);
    w.act(Role::Writer, write).unwrap();
    w.depart().unwrap();
    let departed = w.checkpoint().own[0].clone();
    let mut movement = Primitive::new(PrimitiveKind::Move);
    movement.cell = Some(Cell::C);
    w.act(Role::Reader, movement).unwrap();
    let mut pickup = Primitive::new(PrimitiveKind::Pickup);
    pickup.good = Some(Good::Right);
    w.act(Role::Reader, pickup).unwrap();
    let mut movement = Primitive::new(PrimitiveKind::Move);
    movement.cell = Some(Cell::O);
    w.act(Role::Reader, movement).unwrap();
    w.act(Role::Reader, Primitive::new(PrimitiveKind::Deliver))
        .unwrap();
    w.finish().unwrap();
    let outcome = w.outcome().unwrap();
    assert_eq!(outcome.reward, [6, 6]);
    assert_eq!(outcome.spent, [4, 4]);
    assert_eq!(outcome.net, [2, 2]);
    assert!(outcome.note_survived);
    let writer = &w.checkpoint().own[0];
    assert_eq!(writer.history, departed.history);
    assert_eq!(writer.map, departed.map);
    assert_eq!(writer.credits, departed.credits);
    assert_eq!(writer.spent, departed.spent);
    assert_eq!(writer.cell, None);
    let before = w.clone();
    assert!(w
        .act(Role::Reader, Primitive::new(PrimitiveKind::Deliver))
        .is_err());
    assert_eq!(w, before);
    assert!(w.finish().is_err());
    assert_eq!(w, before);
}

#[test]
fn checked_charge_overflow_leaves_both_accounts_unchanged() {
    let mut credits = 1;
    let mut spent = u64::MAX;
    assert_eq!(
        super::super::kernel::charge(&mut credits, &mut spent, 1),
        Err(Error::ArithmeticOverflow)
    );
    assert_eq!((credits, spent), (1, u64::MAX));
}

#[test]
fn unlocked_public_cell_moves_are_real_and_invalid_moves_are_atomic() {
    let c = Config::example(Arm::Retained, ReaderRoutine::ReadAdaptive, Good::Left);
    let mut w = World::new(&c).unwrap();
    for _ in 0..4 {
        w.act(Role::Writer, Primitive::new(PrimitiveKind::Wait))
            .unwrap();
    }
    w.depart().unwrap();
    let mut movement = Primitive::new(PrimitiveKind::Move);
    movement.cell = Some(Cell::P);
    w.act(Role::Reader, movement.clone()).unwrap();
    let before = w.clone();
    assert!(w.act(Role::Reader, movement.clone()).is_err());
    assert_eq!(w, before);
    movement.cell = Some(Cell::C);
    assert!(w.act(Role::Reader, movement.clone()).is_err());
    assert_eq!(w, before);
    movement.cell = Some(Cell::O);
    w.act(Role::Reader, movement.clone()).unwrap();
    movement.cell = Some(Cell::C);
    w.act(Role::Reader, movement).unwrap();
    let p = w.checkpoint();
    assert_eq!(p.own[1].spent, 3);
    assert_eq!(
        p.own[1].history.last().unwrap().block_reason,
        Some(BlockReason::NoKey)
    );
    assert_eq!(p.physical.key_owner, Role::Writer);
    assert!(!p.public.writer_active);
}
#[test]
fn wrong_delivery_remains_a_completed_zero_reward_outcome() {
    let f: serde_json::Value =
        serde_json::from_slice(super::super::source_inputs::REFERENCE_INPUTS[1].1).unwrap();
    let witness = f["payload"]["witnesses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|w| w["name"] == "wrong-pickup-delivery-double")
        .unwrap();
    let config: Config = serde_json::from_value(witness["config"].clone()).unwrap();
    let mut w = World::new(&config).unwrap();
    for op in witness["operations"].as_array().unwrap().iter().take(9) {
        replay_operation(&mut w, op).unwrap();
    }
    w.finish().unwrap();
    let o = w.outcome().unwrap();
    assert!(o.completed);
    assert!(!o.correct);
    assert!(!o.deadline);
    assert_eq!(o.reward, [0, 0]);
    assert_eq!(o.selected_good, Some(Good::Right));
    assert_eq!(o.net, [-4, -4]);
    assert_eq!(o.group_net, -8);
    assert_eq!(o.costs.delivery, [0, 1]);
    assert_eq!(o.counts.attempted, [4, 4]);
}
