use super::{observer, wire};
use sugarscape_core::war::{Engagement, Frame};
#[derive(Default)]
struct Frames(Vec<Frame>);
impl observer::FrameSink for Frames {
    fn frame(&mut self, frame: &Frame) -> Result<(), String> {
        self.0.push(frame.clone());
        Ok(())
    }
}
#[test]
fn fixed_fixture_observer_preserves_every_frame_rng_and_clock() {
    let manifest = wire::decode_manifest(wire::FIXTURE).unwrap();
    for case in &manifest.cases {
        let arm = manifest
            .arms
            .iter()
            .find(|a| a.arm_id == case.arm_id)
            .unwrap();
        let mut direct = Engagement::new(arm.config.clone(), case.seed).unwrap();
        let mut expected = vec![];
        while let Some(frame) = direct.step().unwrap() {
            expected.push(frame);
        }
        let mut captured = Frames::default();
        let outcome = observer::execute_case(case, arm, Some(&mut captured)).unwrap();
        assert_eq!(captured.0, expected);
        assert_eq!(outcome.final_checkpoint, direct.checkpoint());
        assert_eq!(
            outcome.calendar_time,
            direct.checkpoint().step as f64 * arm.config.dt
        );
        assert_eq!(
            outcome.active_time,
            direct.checkpoint().active_steps as f64 * arm.config.dt
        );
        let plain = observer::execute_case(case, arm, None).unwrap();
        assert_eq!(plain.transcript_sha256, outcome.transcript_sha256);
        assert_eq!(plain.observations.len(), 4);
        for observation in plain.observations {
            if observation.step > plain.completed_steps {
                assert_eq!(observation.origin, "derived_absorbing");
                assert_eq!(observation.counts, plain.final_counts);
            }
        }
    }
}
#[test]
fn failed_audit_keeps_semantic_and_observed_clocks_distinct() {
    struct Fail;
    impl observer::FrameSink for Fail {
        fn frame(&mut self, _: &Frame) -> Result<(), String> {
            Err("injected sync failure".into())
        }
    }
    let manifest = wire::decode_manifest(wire::FIXTURE).unwrap();
    let failure =
        observer::execute_case(&manifest.cases[0], &manifest.arms[0], Some(&mut Fail)).unwrap_err();
    assert_eq!(
        (
            failure.semantic_steps,
            failure.observed_steps,
            failure.durable_audit_steps
        ),
        (1, 1, 0)
    );
    assert_eq!(failure.final_checkpoint.unwrap().step, 1);
}
#[test]
fn compact_observer_matches_existing_runner_and_sums_source_exposure() {
    use sugarscape_core::war::{run_to, Capture, ObservedFrame, RunPayload, StudyInput};
    let manifest = wire::decode_manifest(wire::FIXTURE).unwrap();
    for case in &manifest.cases {
        let arm = manifest
            .arms
            .iter()
            .find(|a| a.arm_id == case.arm_id)
            .unwrap();
        let mut frames = vec![];
        let summary = run_to(
            &StudyInput::ReciprocalGraph {
                config: arm.config.clone(),
                seed: case.seed,
            },
            &Capture {
                retain_steps: vec![64, 128, 256, 512],
            },
            &mut |record| {
                if let RunPayload::Observed {
                    frame: ObservedFrame::Graph { frame, .. },
                } = &record.payload
                {
                    frames.push(frame.clone());
                }
                Ok(())
            },
        )
        .unwrap();
        let mut captured = Frames::default();
        let o = observer::execute_case(case, arm, Some(&mut captured)).unwrap();
        assert_eq!(captured.0, frames);
        assert_eq!(summary.completed_steps, o.completed_steps);
        assert_eq!(summary.ending, Some(o.ending));
        let mut sums = [0.0; 2];
        let mut ids = std::collections::BTreeSet::new();
        for f in frames {
            for (side, sum) in sums.iter_mut().enumerate() {
                *sum += f.exposure.integrated[side];
            }
            for d in f.casualties {
                assert!(ids.insert(d.id));
            }
        }
        assert_eq!(sums, o.contributed_exposure_totals);
    }
}
#[test]
fn stop_after_one_frame_keeps_actual_checkpoint_and_is_not_horizon() {
    let manifest = wire::decode_manifest(wire::FIXTURE).unwrap();
    let mut polls = 0;
    let failure =
        observer::execute_monitored(&manifest.cases[0], &manifest.arms[0], None, &mut || {
            polls += 1;
            if polls == 3 {
                Err("supervisor resource stop".into())
            } else {
                Ok(())
            }
        })
        .unwrap_err();
    assert_eq!(failure.kind, "resource_stop");
    assert_eq!(failure.semantic_steps, 1);
    assert_eq!(failure.observed_steps, 1);
    assert_eq!(failure.durable_audit_steps, 0);
    assert!(failure.final_checkpoint.unwrap().ending.is_none());
}
#[test]
fn altered_frame_ending_and_contact_accounting_are_rejected() {
    let manifest = wire::decode_manifest(wire::FIXTURE).unwrap();
    let c = &manifest.arms[0].config;
    let mut e = Engagement::new(c.clone(), 7).unwrap();
    let f = e.step().unwrap().unwrap();
    let mut wrong = f.clone();
    wrong.ending = Some(sugarscape_core::war::Ending {
        reason: sugarscape_core::war::EndReason::Horizon,
        winner: None,
        censored: true,
    });
    assert!(observer::Accounting::new(c).accept(c, &wrong).is_err());
    let mut wrong = f;
    wrong.exposure.contact_pairs += 1;
    assert!(observer::Accounting::new(c).accept(c, &wrong).is_err());
}
#[test]
fn initial_endings_and_last_step_extinction_keep_native_precedence() {
    let manifest = wire::decode_manifest(wire::FIXTURE).unwrap();
    let case = &manifest.cases[0];
    let mut arm = manifest.arms[0].clone();
    for (blue, red, rate) in [(0, 1, 1.0), (1, 1, 0.0)] {
        arm.config.blue = blue;
        arm.config.red = red;
        arm.config.blue_rate = rate;
        arm.config.red_rate = rate;
        let o = observer::execute_case(case, &arm, None).unwrap();
        assert_eq!(o.completed_steps, 0);
        assert_eq!(o.active_time, 0.0);
        assert!(o
            .observations
            .iter()
            .all(|o| o.origin == "derived_absorbing"));
    }
    arm = manifest.arms[0].clone();
    let baseline = observer::execute_case(case, &arm, None).unwrap();
    assert!(!baseline.ending.censored);
    arm.config.max_steps = baseline.completed_steps;
    let final_step = observer::execute_case(case, &arm, None).unwrap();
    assert_eq!(final_step.completed_steps, arm.config.max_steps);
    assert!(!final_step.ending.censored);
    assert_eq!(final_step.ending, baseline.ending);
}
#[test]
fn initial_and_terminal_sink_failures_keep_true_clocks() {
    struct Sink {
        fail_initial: bool,
    }
    impl observer::FrameSink for Sink {
        fn initial(&mut self, _: &sugarscape_core::war::Checkpoint) -> Result<(), String> {
            if self.fail_initial {
                Err("initial failure".into())
            } else {
                Ok(())
            }
        }
        fn frame(&mut self, _: &Frame) -> Result<(), String> {
            Ok(())
        }
        fn terminal(&mut self, _: &sugarscape_core::war::Checkpoint, _: u64) -> Result<(), String> {
            Err("terminal failure".into())
        }
    }
    let m = wire::decode_manifest(wire::FIXTURE).unwrap();
    for initial in [true, false] {
        let f = observer::execute_case(
            &m.cases[0],
            &m.arms[0],
            Some(&mut Sink {
                fail_initial: initial,
            }),
        )
        .unwrap_err();
        assert_eq!(f.semantic_steps, f.observed_steps);
        assert_eq!(f.observed_steps, f.durable_audit_steps);
        assert_eq!(f.semantic_steps == 0, initial);
        assert_eq!(f.final_checkpoint.unwrap().step, f.semantic_steps);
    }
}
#[test]
fn strict_saved_payload_reader_rejects_nonzero_underflow_and_oversized_rng() {
    let m = wire::decode_manifest(wire::FIXTURE).unwrap();
    let mut engine = Engagement::new(m.arms[0].config.clone(), 7).unwrap();
    let frame = engine.step().unwrap().unwrap();
    let text = String::from_utf8(observer::semantic_bytes(&frame).unwrap()).unwrap();
    let literal =
        serde_json::to_string(&frame.exposure.target_probability[0].as_ref().unwrap().ideal)
            .unwrap();
    let bad = text.replacen(&format!("\"ideal\":{literal}"), "\"ideal\":1e-999", 1);
    assert!(observer::read_frame(&bad).is_err());
    let mut checkpoint = engine.checkpoint();
    checkpoint.contact_rng = "x".repeat(129);
    let bad = serde_json::to_string(&checkpoint).unwrap();
    assert!(observer::read_checkpoint(&bad).is_err());
}

#[test]
fn saved_outcome_checkpoint_and_frame_require_nullable_keys() {
    let m = wire::decode_manifest(wire::FIXTURE).unwrap();
    let case = &m.cases[0];
    let arm = m.arms.iter().find(|a| a.arm_id == case.arm_id).unwrap();
    let outcome = observer::execute_case(case, arm, None).unwrap();
    let original = serde_json::to_value(&outcome).unwrap();
    assert!(serde_json::from_value::<observer::Outcome>(original.clone()).is_ok());
    for pointer in [
        "/audit_descriptor",
        "/ending/winner",
        "/final_checkpoint/ending/winner",
    ] {
        let mut bad = original.clone();
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        bad.pointer_mut(parent)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(key);
        assert!(
            serde_json::from_value::<observer::Outcome>(bad).is_err(),
            "{pointer}"
        );
    }
    let mut engine = Engagement::new(arm.config.clone(), case.seed).unwrap();
    let initial = serde_json::to_string(&engine.checkpoint()).unwrap();
    assert!(initial.contains("\"ending\":null"));
    assert!(observer::read_checkpoint(&initial).is_ok());
    assert!(observer::read_checkpoint(&initial.replace(",\"ending\":null", "")).is_err());
    let frame = serde_json::to_string(&engine.step().unwrap().unwrap()).unwrap();
    assert!(frame.contains("\"ending\":null"));
    assert!(observer::read_frame(&frame).is_ok());
    assert!(observer::read_frame(&frame.replace(",\"ending\":null", "")).is_err());
}
