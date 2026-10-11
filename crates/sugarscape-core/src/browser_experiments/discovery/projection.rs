//! Display-only reconstruction of authoritative native choices and paid events.
use super::{checked, Result};
use crate::browser_experiments::{error, record, wire::lossless_value, Checkpoint};
use crate::{shared_surface as s, surface_discovery as d};
use serde_json::{json, Value};
use std::collections::BTreeMap;
/// The last actual paid Read is an observation, never a claim about current storage.
fn opaque_surface<'a>(entries: impl Iterator<Item = &'a s::LocalEntry>) -> Result<Value> {
    let mut last = None;
    for entry in entries {
        if let s::LocalEntry::Action(event) = entry {
            if let s::Outcome::Read(symbol) = event.outcome {
                last = Some(
                    json!({"symbol":lossless_value(&symbol)?,"observed_at":lossless_value(&s::Checkpoint::AfterSlot(event.position))?}),
                );
            }
        }
    }
    Ok(json!({"kind":"opaque_surface","last_observation":last,"current_state":"unobserved"}))
}
/// This world only projects already-authorized original events. Reads on discarded
/// clones expose physical fields to Researcher without adding costs or history.
struct PhysicalDisplay {
    world: s::World,
    environment: s::Environment,
    phase: Option<s::Phase>,
    credits: [u8; 2],
    last_after: Option<s::Position>,
}
impl PhysicalDisplay {
    fn new(environment: s::Environment, ids: &s::Ids) -> Self {
        Self {
            world: s::World::new(environment, ids.clone()),
            environment,
            phase: None,
            credits: [48; 2],
            last_after: None,
        }
    }
    fn leave_after(&mut self) {
        if self.last_after.take().is_some_and(|p| p.slot == 4) {
            self.world.finish_round();
        }
    }
    fn reset(&mut self, phase: s::Phase) {
        if self.phase != Some(phase) {
            self.world.reset(phase);
            self.phase = Some(phase);
        }
    }
    fn event(&mut self, role: s::Role, event: &s::Event, bits: &s::EpisodeBits) -> Result<()> {
        let target = match event.position.phase {
            s::Phase::Calibration => None,
            s::Phase::Live { trial } => Some(if role == s::Role::A {
                bits.0[usize::from(trial)].1
            } else {
                bits.0[usize::from(trial)].0
            }),
        };
        let actual = checked(self.world.apply(
            role,
            event.position,
            &event.action,
            target,
            self.credits[role.index()],
        ))?;
        if actual != *event {
            return Err(error(
                "surface",
                "display replay event differs from original evaluator",
            ));
        }
        self.credits[role.index()] = event.credits_after;
        self.last_after = Some(event.position);
        Ok(())
    }
    fn snapshot(&self) -> Result<Value> {
        let mut fields = Vec::new();
        for role in [s::Role::A, s::Role::B] {
            let position = s::Position {
                phase: self.phase.unwrap_or(s::Phase::Calibration),
                round: 1,
                slot: if role == s::Role::A { 1 } else { 2 },
            };
            let probe =
                checked(
                    self.world
                        .clone()
                        .apply(role, position, &s::Action::Read, None, 48),
                )?;
            let s::Outcome::Read(symbol) = probe.outcome else {
                return Err(error("surface", "display probe was not a read"));
            };
            fields.push(json!({"visible_to":role,"symbol":symbol,"lineage":lossless_value(&self.world.read_lineage(role))?}));
        }
        Ok(
            json!({"label":"Researcher","environment":self.environment,"topology":if self.environment==s::Environment::InFamily(s::Mechanism::PrivatePersistent) {"private_fields"} else {"shared_field"},"fields":fields,"credits":self.credits.map(|v|v.to_string())}),
        )
    }
}

pub(super) fn project(
    protocol: &d::Protocol,
    environment: d::Environment,
    policy: &d::CompiledPolicy,
    replay: &d::Replay,
    episode: &d::Episode,
    initial_bytes: usize,
) -> Result<Vec<Checkpoint>> {
    let bits = checked(d::EpisodeBits::from_index(episode.sequence))?;
    let nominal = match environment {
        d::Environment::InFamily(m) => m,
        d::Environment::DataFlip => d::Mechanism::SharedPersistent,
    };
    let mut priors = [d::OwnPrior::PointMass(nominal); 2];
    priors[protocol.investigator.index()] = policy.own_prior();
    let mut state = checked(d::EpisodeState::new(
        protocol,
        environment,
        bits.clone(),
        priors,
    ))?;
    let mut physical = PhysicalDisplay::new(environment, &protocol.ids);
    let mut retained_bytes = initial_bytes;
    let mut out = Vec::new();
    let mut selected = false;
    let mut decisions = 0;
    let mut events = 0;
    let mut last_supported: [Option<d::Belief>; 2] = [None, None];
    let mut discoveries: [Option<d::CatalogDiscovery>; 2] = [None, None];
    let mut last_decision: Option<&d::Decision> = None;
    loop {
        let own = state.prefix(protocol.investigator);
        let clock = own.checkpoint;
        // Raw action and live routine boundaries are owned selections. The
        // public slot clock does not announce which private choice was made.
        let public_clock = match clock {
            d::Checkpoint::Boundary(d::Boundary::Calibration(p)) => d::Checkpoint::BeforeSlot(p),
            d::Checkpoint::Boundary(b @ d::Boundary::Trial { .. }) => {
                d::Checkpoint::BeforeSlot(checked(b.trial_position(protocol.investigator))?)
            }
            c => c,
        };
        let mut local = BTreeMap::new();
        let mut first_failure = None;
        for role in [d::Role::A, d::Role::B] {
            let i = role.index();
            let prefix = state.prefix(role);
            let not_checked = first_failure.is_some();
            let inferred = if not_checked {
                None
            } else {
                let result = match replay.infer(prefix) {
                    Err(d::Error::UnsupportedHistory { .. }) if role == protocol.investigator => {
                        policy.infer(prefix)
                    }
                    other => other,
                };
                match result {
                    Ok(b) => {
                        last_supported[i] = Some(b.clone());
                        Some(b)
                    }
                    Err(d::Error::UnsupportedHistory { prefix: p }) => {
                        let failure = episode.failure.as_ref().ok_or_else(|| {
                            error(
                                "surface_discovery",
                                "display inferred an unrecorded unsupported history",
                            )
                        })?;
                        if failure.role != role
                            || failure.prefix != p
                            || failure.last_supported != last_supported[i]
                        {
                            return Err(error(
                                "surface_discovery",
                                "display failure differs from native first failure",
                            ));
                        }
                        first_failure = Some(role);
                        None
                    }
                    Err(e) => return Err(error("surface_discovery", e.to_string())),
                }
            };
            if discoveries[i].is_none() {
                if let Some(b) = inferred.as_ref() {
                    if let Some(model) = d::Mechanism::ALL
                        .into_iter()
                        .find(|m| b.models[m.index()].numerator == b.models[m.index()].denominator)
                    {
                        discoveries[i] = Some(d::CatalogDiscovery {
                            model,
                            source: if prefix.entries.is_empty()
                                && matches!(prefix.own_prior, d::OwnPrior::PointMass(_))
                            {
                                d::CatalogCertaintySource::Supplied
                            } else {
                                d::CatalogCertaintySource::Observed
                            },
                            checkpoint: prefix.checkpoint,
                            spent: 48 - state.credits(role),
                        });
                    }
                }
            }
            let bit = prefix.entries.iter().rev().find_map(|e| match e {
                d::Entry::Physical(s::LocalEntry::PrivateBit { bit, .. }) => Some(*bit),
                _ => None,
            });
            let decision = if role == protocol.investigator && selected {
                last_decision
            } else {
                None
            };
            let prediction = if let d::Checkpoint::Prediction { trial } = prefix.checkpoint {
                episode.predictions[i].get(usize::from(trial)).copied()
            } else {
                None
            };
            local.insert(protocol.ids.agents[i].clone(),json!({
                "prefix":lossless_value(prefix)?,"belief":lossless_value(&inferred)?,"last_supported_belief":if inferred.is_none() {lossless_value(&last_supported[i])?} else {Value::Null},
                "credits":state.credits(role).to_string(),"private_bit":bit,
                "surface":opaque_surface(prefix.entries.iter().filter_map(|e|if let d::Entry::Physical(e)=e {Some(e)} else {None}))?,
                "decision":lossless_value(&decision)?,"decision_label":decision.map(|_|"Expected remaining investigator net task utility under the declared continuation policy"),
                "prediction":prediction,"catalog_discovery":lossless_value(&discoveries[i])?,
                "inference_status":if not_checked {"not_checked_after_peer_failure"} else if inferred.is_some() {"supported"} else {"unsupported"},
            }));
        }
        let stopped = first_failure.is_some() || clock == d::Checkpoint::Finished;
        let probe_stop = own.entries.iter().find_map(|e| {
            if let d::Entry::PublicProbeStop { completed } = e {
                Some(completed.to_string())
            } else {
                None
            }
        });
        let kind = if selected {
            "after_choice"
        } else {
            match clock {
                d::Checkpoint::Boundary(_) => "before_choice",
                d::Checkpoint::BeforeSlot(_) => "before_slot",
                d::Checkpoint::AfterSlot(_) => "after_slot",
                d::Checkpoint::Prediction { .. } => "prediction",
                d::Checkpoint::Finished => "finished",
            }
        };
        let mut research = physical.snapshot()?;
        research["event"] = if let d::Checkpoint::AfterSlot(p) = clock {
            lossless_value(&episode.privileged.iter().find(|e| e.event.position == p))?
        } else {
            Value::Null
        };
        if stopped {
            research["failure"] = lossless_value(&episode.failure)?;
            research["metrics"] = lossless_value(&episode.metrics)?;
            research["model_projections"] = lossless_value(&episode.model_projections)?;
        }
        let checkpoint = Checkpoint {
            index: 0,
            clock: lossless_value(&public_clock)?,
            kind: kind.into(),
            public: json!({"reset_phase":lossless_value(&physical.phase)?,"probe_stop":probe_stop,"status":if stopped {"stopped"} else {"running"}}),
            local,
            researcher: Some(research),
        };
        retain_checkpoint(&mut out, &mut retained_bytes, checkpoint)?;
        if stopped {
            if [state.prefix(d::Role::A), state.prefix(d::Role::B)] != episode.histories.each_ref()
                || events != episode.privileged.len()
                || decisions != episode.decisions.len()
                || discoveries != episode.catalog_discoveries
            {
                return Err(error(
                    "surface_discovery",
                    "display differs from complete native terminal records",
                ));
            }
            break;
        }
        if state.next_boundary().is_some() {
            let trace = episode
                .decisions
                .get(decisions)
                .ok_or_else(|| error("surface_discovery", "missing native decision"))?;
            if trace.prefix != *own {
                return Err(error(
                    "surface_discovery",
                    "native decision prefix differs from display",
                ));
            }
            checked(d::select_choice(&mut state, trace.decision.choice))?;
            decisions += 1;
            selected = true;
            last_decision = Some(&trace.decision);
            continue;
        }
        physical.leave_after();
        let step = checked(d::advance_one(&mut state))?;
        let phase = state.prefix(d::Role::A).entries.iter().rev().find_map(|e| {
            if let d::Entry::Physical(s::LocalEntry::Reset { phase }) = e {
                Some(*phase)
            } else {
                None
            }
        });
        if let Some(phase) = phase {
            physical.reset(phase);
        }
        for (role, event) in step.events {
            let original = episode.privileged.get(events).ok_or_else(|| {
                error(
                    "surface_discovery",
                    "display advanced past native paid termination",
                )
            })?;
            if original.role != role || original.event != event {
                return Err(error(
                    "surface_discovery",
                    "display paid event differs from native",
                ));
            }
            physical.event(role, &event, &bits)?;
            events += 1;
        }
        selected = false;
        last_decision = None;
    }
    Ok(out)
}

/// Count the final indexed component and comma before retaining it. Rejection
/// leaves both the retained vector and its exact serialized byte count intact.
fn retain_checkpoint(
    out: &mut Vec<Checkpoint>,
    bytes: &mut usize,
    mut checkpoint: Checkpoint,
) -> Result<()> {
    if out.len() >= crate::browser_experiments::MAX_CHECKPOINTS {
        return Err(error("checkpoints", "episode exceeds 4096 checkpoints"));
    }
    checkpoint.index = out.len() as u32;
    let size = record::serialized_size(
        &checkpoint,
        crate::browser_experiments::MAX_EPISODE_BYTES,
        "checkpoints",
    )?;
    let next = bytes
        .checked_add(size)
        .and_then(|n| n.checked_add(usize::from(!out.is_empty())))
        .ok_or_else(|| error("episode", "checkpoint retention byte count overflow"))?;
    if next > crate::browser_experiments::MAX_EPISODE_BYTES {
        return Err(error("episode", "checkpoint retention exceeds 16 MiB"));
    }
    record::push_checkpoint(out, checkpoint)?;
    *bytes = next;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovery_early_record_byte_exhaustion_rejects_before_retention() {
        let mut out = Vec::new();
        let mut bytes = crate::browser_experiments::MAX_EPISODE_BYTES - 1;
        let checkpoint = Checkpoint {
            index: 0,
            clock: Value::Null,
            kind: "bounded".into(),
            public: Value::Null,
            local: BTreeMap::new(),
            researcher: None,
        };
        let errors = retain_checkpoint(&mut out, &mut bytes, checkpoint).unwrap_err();
        assert_eq!(errors[0].field, "episode");
        assert!(out.is_empty());
        assert_eq!(bytes, crate::browser_experiments::MAX_EPISODE_BYTES - 1);
    }
}
