//! Immutable observation of the existing transactional finite engine.
use super::{archive, wire};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sugarscape_core::war::{
    config::{EngagementConfig, Geometry, Side},
    reference_at, Checkpoint, Ending, Engagement, Frame, ReferenceRegime,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub schema: String,
    pub key: wire::Key,
    pub seed: u64,
    pub case_input_sha256: String,
    pub manifest_sha256: String,
    pub completed_steps: u64,
    pub observed_steps: u64,
    pub active_steps: u64,
    pub calendar_time: f64,
    pub active_time: f64,
    pub initial_counts: [u32; 2],
    pub final_counts: [u32; 2],
    pub ending: Ending,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,4>")]
    pub observations: Vec<Observation>,
    pub casualty_totals: [u32; 2],
    pub contributed_exposure_totals: [f64; 2],
    #[serde(deserialize_with = "checkpoint_from_wire")]
    pub final_checkpoint: Checkpoint,
    pub transcript_sha256: String,
    pub audit_descriptor: Option<AuditDescriptor>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub step: u64,
    pub counts: [u32; 2],
    pub origin: String,
    pub reason: Option<String>,
    pub reference: Option<ReferenceWire>,
    pub reference_error: Option<ReferenceErrorWire>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceWire {
    pub requested_time: f64,
    pub evaluated_time: f64,
    pub forces: [f64; 2],
    pub regime: String,
    pub extinction_time: Option<f64>,
    pub at_boundary: bool,
    pub survivor: Option<Side>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceErrorWire {
    pub field: String,
    pub detail: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuditDescriptor {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    pub acknowledged_steps: u64,
}
#[derive(Debug, Serialize)]
pub struct CaseFailure {
    pub kind: String,
    pub detail: String,
    pub key: wire::Key,
    pub attempted_step: Option<u64>,
    pub semantic_steps: u64,
    pub observed_steps: u64,
    pub durable_audit_steps: u64,
    pub final_checkpoint: Option<Checkpoint>,
    pub acknowledged_archive_prefix: Option<archive::ArchiveReceipt>,
}
pub trait FrameSink {
    fn initial(&mut self, _: &Checkpoint) -> Result<(), String> {
        Ok(())
    }
    fn frame(&mut self, frame: &Frame) -> Result<(), String>;
    fn terminal(&mut self, _: &Checkpoint, _: u64) -> Result<(), String> {
        Ok(())
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CaseInput {
    pub config: EngagementConfig,
    pub seed: u64,
}
pub(super) fn semantic_bytes(value: &impl Serialize) -> Result<Vec<u8>, String> {
    super::io::encode(value, 32 * 1024, false)
}
pub(super) fn hash_part(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}
#[cfg(test)]
thread_local! { static CONSTRUCTIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(super) fn construction_count() -> usize {
    CONSTRUCTIONS.with(|n| n.get())
}

/// Internal boundary: only the authorized archive path and fixed-fixture tests call this.
#[allow(clippy::result_large_err)] // Frozen typed failure API retains the actual checkpoint.
pub(super) fn execute_case(
    case: &wire::Case,
    arm: &wire::Arm,
    audit: Option<&mut dyn FrameSink>,
) -> Result<Outcome, CaseFailure> {
    execute_monitored(case, arm, audit, &mut || Ok(()))
}
#[allow(clippy::result_large_err)] // Same typed failure contract as execute_case.
pub(super) fn execute_monitored(
    case: &wire::Case,
    arm: &wire::Arm,
    mut audit: Option<&mut dyn FrameSink>,
    stop: &mut dyn FnMut() -> Result<(), String>,
) -> Result<Outcome, CaseFailure> {
    let c = &arm.config;
    let mut observed = 0;
    let mut durable = 0;
    let failure = |kind: &str,
                   detail: String,
                   checkpoint: Option<Checkpoint>,
                   observed_steps,
                   durable_audit_steps| {
        let semantic_steps = checkpoint.as_ref().map_or(0, |p| p.step);
        CaseFailure {
            kind: kind.into(),
            detail: bounded_detail(detail),
            key: case.key(),
            attempted_step: if kind == "kernel" {
                Some(semantic_steps + 1)
            } else {
                None
            },
            semantic_steps,
            observed_steps,
            durable_audit_steps,
            final_checkpoint: checkpoint,
            acknowledged_archive_prefix: None,
        }
    };
    if c.blue as u64 + c.red as u64 > 768
        || c.max_steps > 512
        || ![7, 8].contains(&case.seed)
        || !arm.arm_id.starts_with("fixture.")
    {
        return Err(failure(
            "bounds",
            "case exceeds approved population/step/fixture bounds".into(),
            None,
            0,
            0,
        ));
    }
    stop().map_err(|e| failure("resource_stop", e, None, 0, 0))?;
    #[cfg(test)]
    CONSTRUCTIONS.with(|n| n.set(n.get() + 1));
    let mut engine = Engagement::new(c.clone(), case.seed)
        .map_err(|e| failure("construction", format!("{e:?}"), None, 0, 0))?;
    let mut checkpoint = engine.checkpoint();
    let input = semantic_bytes(&CaseInput {
        config: c.clone(),
        seed: case.seed,
    })
    .map_err(|e| failure("serialization", e, Some(checkpoint.clone()), 0, 0))?;
    let mut transcript = Sha256::new();
    transcript.update(b"war1-study-transcript-v1\0");
    hash_part(&mut transcript, &input);
    hash_part(
        &mut transcript,
        &semantic_bytes(&checkpoint)
            .map_err(|e| failure("serialization", e, Some(checkpoint.clone()), 0, 0))?,
    );
    if let Some(sink) = audit.as_deref_mut() {
        sink.initial(&checkpoint)
            .map_err(|e| failure("audit_initial", e, Some(checkpoint.clone()), 0, 0))?;
    }
    let mut state = Accounting::new(c);
    let requested = [1.0, 2.0, 4.0, 8.0].map(|t| (t / c.dt) as u64);
    let mut observations = Vec::with_capacity(4);
    while engine.ending().is_none() {
        stop().map_err(|e| {
            failure(
                "resource_stop",
                e,
                Some(engine.checkpoint()),
                observed,
                durable,
            )
        })?;
        let frame = engine
            .step()
            .map_err(|e| {
                failure(
                    "kernel",
                    format!("{}: {}", e.issue.field, e.issue.detail),
                    Some(engine.checkpoint()),
                    observed,
                    durable,
                )
            })?
            .ok_or_else(|| {
                failure(
                    "kernel",
                    "missing unended frame".into(),
                    Some(engine.checkpoint()),
                    observed,
                    durable,
                )
            })?;
        checkpoint = engine.checkpoint();
        state.accept(c, &frame).map_err(|e| {
            failure(
                "observation",
                e,
                Some(checkpoint.clone()),
                observed,
                durable,
            )
        })?;
        validate_checkpoint(&checkpoint)
            .map_err(|e| failure("checkpoint", e, Some(checkpoint.clone()), observed, durable))?;
        let bytes = semantic_bytes(&frame).map_err(|e| {
            failure(
                "serialization",
                e,
                Some(checkpoint.clone()),
                observed,
                durable,
            )
        })?;
        hash_part(&mut transcript, &bytes);
        observed += 1;
        if requested.contains(&frame.step) {
            observations.push(observation(c, frame.step, frame.survivors, None));
        }
        if let Some(sink) = audit.as_deref_mut() {
            sink.frame(&frame).map_err(|e| {
                let mut f = failure(
                    "audit_frame",
                    e,
                    Some(checkpoint.clone()),
                    observed,
                    durable,
                );
                f.attempted_step = Some(frame.step);
                f
            })?;
            durable += 1;
        }
    }
    stop().map_err(|e| {
        failure(
            "resource_stop",
            e,
            Some(checkpoint.clone()),
            observed,
            durable,
        )
    })?;
    for step in requested {
        if !observations.iter().any(|o| o.step == step) {
            observations.push(observation(
                c,
                step,
                engine.counts(),
                Some(format!(
                    "run ended at step {} ({:?}); future frame unavailable",
                    checkpoint.step,
                    engine.ending().unwrap().reason
                )),
            ));
        }
    }
    observations.sort_by_key(|o| o.step);
    hash_part(
        &mut transcript,
        &semantic_bytes(&checkpoint).map_err(|e| {
            failure(
                "serialization",
                e,
                Some(checkpoint.clone()),
                observed,
                durable,
            )
        })?,
    );
    if let Some(sink) = audit {
        sink.terminal(&checkpoint, observed).map_err(|e| {
            failure(
                "audit_terminal",
                e,
                Some(checkpoint.clone()),
                observed,
                durable,
            )
        })?;
    }
    Ok(Outcome {
        schema: "war1-study-outcome-v1".into(),
        key: case.key(),
        seed: case.seed,
        case_input_sha256: wire::sha256(&input),
        manifest_sha256: wire::sha256(wire::SELECTED),
        completed_steps: checkpoint.step,
        observed_steps: observed,
        active_steps: checkpoint.active_steps,
        calendar_time: checkpoint.step as f64 * c.dt,
        active_time: checkpoint.active_steps as f64 * c.dt,
        initial_counts: [c.blue, c.red],
        final_counts: engine.counts(),
        ending: engine.ending().unwrap().clone(),
        observations,
        casualty_totals: state.casualties,
        contributed_exposure_totals: state.exposure,
        final_checkpoint: checkpoint,
        transcript_sha256: format!("{:x}", transcript.finalize()),
        audit_descriptor: None,
    })
}
fn bounded_detail(mut value: String) -> String {
    if value.len() > 4096 {
        let mut end = 4096;
        while !value.is_char_boundary(end) {
            end -= 1;
        }
        value.truncate(end);
    }
    value
}
fn observation(
    c: &EngagementConfig,
    step: u64,
    counts: [u32; 2],
    reason: Option<String>,
) -> Observation {
    let (reference, reference_error) = match reference_at(c, step as f64 * c.dt) {
        Ok(p) => (
            Some(ReferenceWire {
                requested_time: p.requested_time,
                evaluated_time: p.evaluated_time,
                forces: p.forces,
                regime: match p.regime {
                    ReferenceRegime::InitialExtinction => "initial_extinction",
                    ReferenceRegime::FiniteExtinction => "finite_extinction",
                    ReferenceRegime::Asymptotic => "asymptotic",
                    ReferenceRegime::RateZero => "rate_zero",
                }
                .into(),
                extinction_time: p.extinction_time,
                at_boundary: p.at_boundary,
                survivor: p.survivor,
            }),
            None,
        ),
        Err(e) => (
            None,
            Some(ReferenceErrorWire {
                field: e.field,
                detail: bounded_detail(e.detail),
            }),
        ),
    };
    Observation {
        step,
        counts,
        origin: if reason.is_some() {
            "derived_absorbing"
        } else {
            "native"
        }
        .into(),
        reason,
        reference,
        reference_error,
    }
}
pub(super) fn expected_ending(c: &EngagementConfig, counts: [u32; 2], step: u64) -> Option<Ending> {
    use sugarscape_core::war::EndReason;
    let (reason, winner, censored) = match counts {
        [0, 0] => (EndReason::DoubleExtinction, None, false),
        [0, _] => (EndReason::OneSideExtinction, Some(Side::Red), false),
        [_, 0] => (EndReason::OneSideExtinction, Some(Side::Blue), false),
        _ if c.blue_rate == 0.0 && c.red_rate == 0.0 => (EndReason::RateZero, None, false),
        _ if step == c.max_steps => (EndReason::Horizon, None, true),
        _ => return None,
    };
    Some(Ending {
        reason,
        winner,
        censored,
    })
}
pub(super) fn validate_checkpoint(p: &Checkpoint) -> Result<(), String> {
    if p.schema != sugarscape_core::war::config::CHECKPOINT_SCHEMA {
        return Err("checkpoint schema mismatch".into());
    }
    p.config
        .validate()
        .map_err(|e| format!("checkpoint config: {e:?}"))?;
    if p.config.blue as u64 + p.config.red as u64 > 768 || p.config.max_steps > 512 {
        return Err("checkpoint population bound".into());
    }
    if p.ending
        != expected_ending(
            &p.config,
            [p.blue_alive.len() as u32, p.red_alive.len() as u32],
            p.step,
        )
    {
        return Err("checkpoint ending mismatch".into());
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct State {
        state: u128,
    }
    for rng in [&p.contact_rng, &p.casualty_rng] {
        if rng.len() > 128 {
            return Err("RNG text bound".into());
        }
        let s: State = serde_json::from_str(rng).map_err(|e| e.to_string())?;
        if s.state & 1 == 0 {
            return Err("invalid even PCG state".into());
        }
    }
    if p.active_steps != p.step || p.step > p.config.max_steps {
        return Err("checkpoint clocks".into());
    }
    for (ids, lo, hi) in [
        (&p.blue_alive, 0, p.config.blue),
        (&p.red_alive, p.config.blue, p.config.blue + p.config.red),
    ] {
        if ids.len() > 768
            || ids.iter().any(|i| *i < lo || *i >= hi)
            || ids.windows(2).any(|w| w[0] >= w[1])
        {
            return Err("checkpoint ordered ID bounds".into());
        }
    }
    Ok(())
}
pub(super) struct Accounting {
    counts: [u32; 2],
    step: u64,
    alive: Vec<bool>,
    pub casualties: [u32; 2],
    pub exposure: [f64; 2],
}
impl Accounting {
    pub(super) fn new(c: &EngagementConfig) -> Self {
        Self {
            counts: [c.blue, c.red],
            step: 0,
            alive: vec![true; (c.blue + c.red) as usize],
            casualties: [0; 2],
            exposure: [0.0; 2],
        }
    }
    pub(super) fn accept(&mut self, c: &EngagementConfig, f: &Frame) -> Result<(), String> {
        if f.step != self.step + 1
            || f.start != self.counts
            || f.active_steps != f.step
            || f.calendar_time != f.step as f64 * c.dt
            || f.active_time != f.active_steps as f64 * c.dt
        {
            return Err("frame clock/start mismatch".into());
        }
        let sources = if c.geometry == Geometry::DuelContact {
            [f.start[0].min(f.start[1]); 2]
        } else {
            f.start
        };
        if f.ending != expected_ending(c, f.survivors, f.step)
            || f.step > c.max_steps
            || f.exposure.contact_pairs
                != if c.geometry == Geometry::DuelContact {
                    sources[0] as u64
                } else {
                    f.start[0] as u64 * f.start[1] as u64
                }
        {
            return Err("frame ending/contact mismatch".into());
        }
        let mut losses = [0; 2];
        let mut ids = std::collections::BTreeSet::new();
        for dead in &f.casualties {
            let side = usize::from(dead.side == Side::Red);
            if dead.id as usize >= self.alive.len()
                || !self.alive[dead.id as usize]
                || !ids.insert(dead.id)
                || (side == 0) != (dead.id < c.blue)
            {
                return Err("duplicate/out-of-range casualty ID".into());
            }
            losses[side] += 1;
        }
        for side in 0..2 {
            let rate = [c.blue_rate, c.red_rate][side] * f64::from(sources[side]);
            if f.survivors[side].checked_add(losses[side]) != Some(f.start[side])
                || losses[side] > f.exposure.exposed[side]
                || f.exposure.exposed[side] > f.start[side]
                || f.exposure.contributed_rate[side] != rate
                || f.exposure.integrated[side] != rate * c.dt
                || !rate.is_finite()
                || !f.exposure.integrated[side].is_finite()
                || !(self.exposure[side] + f.exposure.integrated[side]).is_finite()
            {
                return Err("frame population/exposure accounting".into());
            }
            if let Some(p) = &f.exposure.target_probability[side] {
                if [p.hazard, p.dose, p.ideal, p.realized]
                    .iter()
                    .any(|x| !x.is_finite() || *x < 0.0)
                    || p.ideal > 1.0
                    || p.realized > 1.0
                {
                    return Err("invalid probability".into());
                }
            }
        }
        for id in ids {
            self.alive[id as usize] = false;
        }
        for (side, loss) in losses.into_iter().enumerate() {
            self.casualties[side] += loss;
            self.exposure[side] += f.exposure.integrated[side];
        }
        self.counts = f.survivors;
        self.step = f.step;
        Ok(())
    }
}

// Local strict readers preserve the frozen core DTOs while bounding actor arrays
// and PCG strings before allocating their typed payloads.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckpointWire {
    schema: String,
    config: EngagementConfig,
    seed: u64,
    step: u64,
    active_steps: u64,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,768>")]
    blue_alive: Vec<u32>,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,768>")]
    red_alive: Vec<u32>,
    #[serde(deserialize_with = "wire::bounded_text::<_,128>")]
    contact_rng: String,
    #[serde(deserialize_with = "wire::bounded_text::<_,128>")]
    casualty_rng: String,
    ending: Option<Ending>,
}
fn checkpoint_from_wire<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Checkpoint, D::Error> {
    let w = CheckpointWire::deserialize(deserializer)?;
    let p = Checkpoint {
        schema: w.schema,
        config: w.config,
        seed: w.seed,
        step: w.step,
        active_steps: w.active_steps,
        blue_alive: w.blue_alive,
        red_alive: w.red_alive,
        contact_rng: w.contact_rng,
        casualty_rng: w.casualty_rng,
        ending: w.ending,
    };
    validate_checkpoint(&p).map_err(serde::de::Error::custom)?;
    Ok(p)
}
pub(super) fn read_checkpoint(text: &str) -> Result<Checkpoint, String> {
    wire::reject_numeric_underflow(text.as_bytes())?;
    let mut d = serde_json::Deserializer::from_str(text);
    let checkpoint = checkpoint_from_wire(&mut d).map_err(|e| e.to_string())?;
    d.end().map_err(|e| e.to_string())?;
    Ok(checkpoint)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FrameWire {
    step: u64,
    start: [u32; 2],
    survivors: [u32; 2],
    active_steps: u64,
    calendar_time: f64,
    active_time: f64,
    exposure: sugarscape_core::war::Exposure,
    #[serde(deserialize_with = "wire::bounded_vec::<_,_,768>")]
    casualties: Vec<sugarscape_core::war::Casualty>,
    ending: Option<Ending>,
}
pub(super) fn read_frame(text: &str) -> Result<Frame, String> {
    wire::reject_numeric_underflow(text.as_bytes())?;
    let w: FrameWire = serde_json::from_str(text).map_err(|e| e.to_string())?;
    Ok(Frame {
        step: w.step,
        start: w.start,
        survivors: w.survivors,
        active_steps: w.active_steps,
        calendar_time: w.calendar_time,
        active_time: w.active_time,
        exposure: w.exposure,
        casualties: w.casualties,
        ending: w.ending,
    })
}
