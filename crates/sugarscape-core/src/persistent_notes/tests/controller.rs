use super::super::*;
#[test]
fn local_blank_tie_and_routes() -> Result<(), Error> {
    assert_eq!(blank_fallback(3, 6)?, BlankChoice::GuessLeft);
    assert_eq!(blank_fallback(2, 6)?, BlankChoice::Inspect);
    assert_eq!(blank_fallback(8, 12)?, BlankChoice::GuessLeft);
    assert_eq!(blank_fallback(u64::MAX, 12), Err(Error::ArithmeticOverflow));
    assert_eq!(registered_strata().len(), 144);
    for request in [Good::Left, Good::Right] {
        let retained = run_episode(&Config::example(
            Arm::Retained,
            ReaderRoutine::ReadAdaptive,
            request,
        ))?;
        let erased = run_episode(&Config::example(
            Arm::Erased,
            ReaderRoutine::ReadAdaptive,
            request,
        ))?;
        assert_eq!(retained.outcome.spent, [4, 5]);
        assert_eq!(erased.outcome.spent, [4, 9]);
        assert_eq!(retained.outcome.net, [-4, 7]);
        assert_eq!(erased.outcome.net, [-4, 3]);
        assert!(retained.outcome.correct && erased.outcome.correct);
        assert_eq!(erased.outcome.counts.attempted, [4, 6]);
        assert_eq!(
            choose(&retained.checkpoints[3].own[0])?.note,
            Some(if request == Good::Left {
                Note::NoteLeft
            } else {
                Note::NoteRight
            })
        );
        let mut own = retained.checkpoints[3].own[0].clone();
        own.known_request = None;
        assert!(matches!(choose(&own), Err(Error::InvalidHistory(_))));
    }
    Ok(())
}

#[test]
fn public_inputs_replay_without_hidden_request() -> Result<(), Error> {
    for arm in [
        Arm::Retained,
        Arm::Erased,
        Arm::NoNote,
        Arm::NoAcquireNoNote,
    ] {
        for routine in [
            ReaderRoutine::BlindGuess,
            ReaderRoutine::InspectFirst,
            ReaderRoutine::ReadAdaptive,
        ] {
            let left = run_episode(&Config::example(arm, routine, Good::Left))?;
            let right = run_episode(&Config::example(arm, routine, Good::Right))?;
            for k in 0..=1 {
                assert_eq!(left.checkpoints[k].own[1], right.checkpoints[k].own[1]);
            }
            for episode in [left, right] {
                for pair in episode.checkpoints.windows(2) {
                    if let Some(event) = &pair[1].event {
                        if let Some(actor) = event.actor {
                            assert_eq!(
                                choose(&pair[0].own[actor.index()])?,
                                event.feedback.as_ref().unwrap().primitive
                            );
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
#[test]
fn blank_high_inspection_guesses_and_wrong_delivery_preserves_writer_ledger() -> Result<(), Error> {
    let mut config = Config::example(Arm::Erased, ReaderRoutine::ReadAdaptive, Good::Right);
    config.stratum.writer_share = 6;
    config.stratum.manifest_inspection_cost = 8;
    let episode = run_episode(&config)?;
    assert!(!episode.outcome.correct);
    assert_eq!(episode.outcome.spent, [4, 5]);
    assert_eq!(episode.outcome.reward, [0, 0]);
    assert_eq!(episode.outcome.net, [-4, -5]);
    let before = &episode.checkpoints[5].own[0];
    let after = &episode.checkpoints.last().unwrap().own[0];
    assert_eq!(before.history, after.history);
    assert_eq!(before.map, after.map);
    assert_eq!(before.spent, after.spent);
    Ok(())
}
#[test]
fn public_capture_failure_returns_no_episode() {
    let config = Config::example(Arm::Retained, ReaderRoutine::Oracle, Good::Right);
    for boundary in [0, 1, 5, 9, 10] {
        let result = run_with_capture(&config, |c| {
            if c.clock.boundary == boundary {
                Err(Error::Io("injected capture failure".into()))
            } else {
                Ok(())
            }
        });
        assert_eq!(result, Err(Error::Io("injected capture failure".into())));
    }
}

#[test]
fn contradictory_own_observation_sources_are_invalid_history() -> Result<(), Error> {
    let episode = run_episode(&Config::example(
        Arm::Retained,
        ReaderRoutine::ReadAdaptive,
        Good::Left,
    ))?;
    let mut reader = episode.checkpoints[7].own[1].clone();
    reader.known_request = Some(Good::Right);
    assert!(matches!(choose(&reader), Err(Error::InvalidHistory(_))));
    let mut writer = episode.checkpoints[3].own[0].clone();
    writer.history[0].observed_request = Some(Good::Right);
    assert!(matches!(choose(&writer), Err(Error::InvalidHistory(_))));
    let blank = run_episode(&Config::example(
        Arm::Erased,
        ReaderRoutine::ReadAdaptive,
        Good::Right,
    ))?;
    let mut reader = blank.checkpoints[7].own[1].clone();
    reader.known_request = Some(Good::Right);
    assert!(matches!(choose(&reader), Err(Error::InvalidHistory(_))));
    Ok(())
}
