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
    let mut initial_by_routine = std::collections::BTreeMap::new();
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
            let initial = initial_by_routine
                .entry(routine)
                .or_insert_with(|| left.checkpoints[0].own[1].clone());
            assert_eq!(
                initial, &left.checkpoints[0].own[1],
                "initial arm leak: {arm:?}"
            );
            assert_eq!(
                initial, &right.checkpoints[0].own[1],
                "initial request leak: {arm:?}"
            );
            // Oracle is the labeled supplied-truth exception and is excluded above.
            // Reward feedback after settlement may reveal success legitimately.
            for (a, b) in left.checkpoints.iter().zip(&right.checkpoints) {
                let own = &a.own[1];
                if own.known_request.is_some()
                    || own
                        .note_read
                        .as_ref()
                        .is_some_and(|read| read.symbol != Note::Blank)
                {
                    break;
                }
                let mut own = own.clone();
                let mut peer_request = b.own[1].clone();
                if a.public.terminal {
                    // Only settled reward and reward feedback are permitted to differ.
                    // A terminal known_request, map or other hidden-truth leak still fails.
                    own.reward = 0;
                    peer_request.reward = 0;
                    for feedback in &mut own.history {
                        feedback.reward = 0;
                    }
                    for feedback in &mut peer_request.history {
                        feedback.reward = 0;
                    }
                }
                assert_eq!(
                    own, peer_request,
                    "{arm:?}/{routine:?}/{}",
                    a.clock.boundary
                );
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
