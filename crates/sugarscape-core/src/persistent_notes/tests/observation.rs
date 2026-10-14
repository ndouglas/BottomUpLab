use super::super::kernel::World;
use super::super::*;
#[test]
fn initial_reader_view_hides_request_and_arm() {
    for routine in [
        ReaderRoutine::BlindGuess,
        ReaderRoutine::InspectFirst,
        ReaderRoutine::ReadAdaptive,
    ] {
        let mut baseline = None;
        for arm in [
            Arm::Retained,
            Arm::Erased,
            Arm::NoNote,
            Arm::NoAcquireNoNote,
        ] {
            for request in [Good::Left, Good::Right] {
                let c = Config::example(arm, routine, request);
                let w = World::new(&c).unwrap();
                let value = serde_json::to_value(&w.checkpoint().own[1]).unwrap();
                if let Some(ref initial) = baseline {
                    assert_eq!(&value, initial);
                } else {
                    baseline = Some(value);
                }
            }
        }
    }
}
#[test]
fn oracle_is_explicit_privileged_exception() {
    for request in [Good::Left, Good::Right] {
        let c = Config::example(Arm::Retained, ReaderRoutine::Oracle, request);
        assert_eq!(
            World::new(&c).unwrap().checkpoint().own[1].known_request,
            Some(request)
        );
    }
}
