use super::*;
use crate::shared_surface::Lineage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionTrace {
    pub prefix: Prefix,
    pub decision: Decision,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentMetrics {
    pub spent: u8,
    pub reward: Option<i64>,
    pub net: Option<i64>,
    pub correct: Option<u8>,
    pub reads: u8,
    pub writes: u8,
    pub waits: u8,
    pub inspections: u8,
    pub attempted_sends: u8,
    pub lineage_reads: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Failure {
    pub role: Role,
    pub prefix: Prefix,
    pub last_supported: Option<Belief>,
    pub spent: [u8; 2],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum CatalogCertaintySource {
    Supplied,
    Observed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogDiscovery {
    pub model: Mechanism,
    pub source: CatalogCertaintySource,
    pub checkpoint: Checkpoint,
    pub spent: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelProjection {
    /// First nominal catalog certainty, which is not a true-model grade on DataFlip.
    pub identified_catalog_model: Option<Mechanism>,
    pub identification_source: Option<CatalogCertaintySource>,
    pub final_belief: Belief,
    pub true_model_mass: Option<Probability>,
    pub uniquely_identified: Option<bool>,
    pub first_identification: Option<Checkpoint>,
    pub spent_to_identify: Option<u8>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrivilegedStep {
    pub role: Role,
    pub event: Event,
    pub read_lineage: Option<Lineage>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Episode {
    #[serde(deserialize_with = "deserialize_sequence")]
    pub sequence: u16,
    pub histories: [Prefix; 2],
    pub decisions: Vec<DecisionTrace>,
    pub metrics: [AgentMetrics; 2],
    pub predictions: [Vec<bool>; 2],
    pub prediction_beliefs: [Vec<Belief>; 2],
    pub privileged: Vec<PrivilegedStep>,
    pub failure: Option<Failure>,
    pub model_projections: [Option<ModelProjection>; 2],
    /// Partial nominal discoveries survive unsupported histories.
    pub catalog_discoveries: [Option<CatalogDiscovery>; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Aggregate {
    pub valid_mass: Probability,
    pub failed_mass: Probability,
    pub spent: [ScoreFraction; 2],
    pub reward: [Option<ScoreFraction>; 2],
    pub net: [Option<ScoreFraction>; 2],
    pub group_net: Option<ScoreFraction>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Panel {
    pub protocol: Protocol,
    pub environment: Environment,
    pub episodes: Vec<Episode>,
    pub aggregate: Aggregate,
}
fn deserialize_sequence<'de, D: serde::Deserializer<'de>>(d: D) -> Result<u16, D::Error> {
    let sequence = u16::deserialize(d)?;
    EpisodeBits::from_index(sequence).map_err(serde::de::Error::custom)?;
    Ok(sequence)
}

/// Privileged actual host. Only each own Prefix/Belief reaches the policy API.
/// Every charged observation is checked before any subsequent transition.
pub fn run_episode(
    protocol: &Protocol,
    environment: Environment,
    sequence: u16,
    policy: &CompiledPolicy,
    replay: &Replay,
) -> Result<Episode, Error> {
    replay.validate_policy(protocol, policy)?;
    let bits = EpisodeBits::from_index(sequence)?;
    let nominal = match environment {
        Environment::InFamily(model) => model,
        Environment::DataFlip => Mechanism::SharedPersistent,
    };
    let mut priors = [OwnPrior::PointMass(nominal); 2];
    priors[protocol.investigator.index()] = policy.own_prior();
    let mut state = EpisodeState::new(protocol, environment, bits.clone(), priors)?;
    let reservation = replay.reserve_episode()?;
    let histories = [Role::A, Role::B].map(|r| state.prefix(r).clone());
    let mut episode = Episode {
        sequence,
        histories,
        decisions: Vec::new(),
        metrics: [AgentMetrics::default(), AgentMetrics::default()],
        predictions: [Vec::new(), Vec::new()],
        prediction_beliefs: [Vec::new(), Vec::new()],
        privileged: Vec::new(),
        failure: None,
        model_projections: [None, None],
        catalog_discoveries: [None, None],
    };
    let mut last_supported: [Option<Belief>; 2] = [None, None];
    let mut next = Some(Boundary::Probe { completed: 0 });
    loop {
        replay.checkpoint()?;
        episode.histories = [Role::A, Role::B].map(|r| state.prefix(r).clone());
        // Query independently. Never prune one Agent's candidates using the
        // other Agent's actual prefix, even when their explanations differ.
        for role in [Role::A, Role::B] {
            let i = role.index();
            let prefix = &episode.histories[i];
            episode.metrics[i].spent = 48 - state.credits(role);
            match replay.infer_for_host(prefix, policy) {
                Ok(belief) => {
                    if episode.catalog_discoveries[i].is_none() {
                        if let Some(model) = Mechanism::ALL.into_iter().find(|m| {
                            belief.models[m.index()].numerator
                                == belief.models[m.index()].denominator
                        }) {
                            episode.catalog_discoveries[i] = Some(CatalogDiscovery {
                                model,
                                source: if prefix.entries.is_empty()
                                    && matches!(prefix.own_prior, OwnPrior::PointMass(_))
                                {
                                    CatalogCertaintySource::Supplied
                                } else {
                                    CatalogCertaintySource::Observed
                                },
                                checkpoint: prefix.checkpoint,
                                spent: episode.metrics[i].spent,
                            });
                        }
                    }
                    last_supported[i] = Some(belief);
                }
                Err(Error::UnsupportedHistory { prefix }) => {
                    let spent = [48 - state.credits(Role::A), 48 - state.credits(Role::B)];
                    for (metric, cost) in episode.metrics.iter_mut().zip(spent) {
                        metric.spent = cost;
                    }
                    episode.failure = Some(Failure {
                        role,
                        prefix,
                        last_supported: last_supported[i].clone(),
                        spent,
                    });
                    reservation.retain();
                    return Ok(episode);
                }
                Err(error) => return Err(error),
            }
        }
        let checkpoint = state.prefix(protocol.investigator).checkpoint;
        if checkpoint == Checkpoint::Finished {
            break;
        }
        if let Checkpoint::Prediction { trial } = checkpoint {
            for role in [Role::A, Role::B] {
                let i = role.index();
                let belief = last_supported[i]
                    .as_ref()
                    .expect("supported own prediction");
                if state.prefix(role).checkpoint != (Checkpoint::Prediction { trial }) {
                    return Err(Error::InvalidHistory("prediction clocks disagree".into()));
                }
                episode.predictions[i].push(predict_target(belief)?);
                episode.prediction_beliefs[i].push(belief.clone());
            }
        }
        replay.transition()?;
        if next.is_some() {
            let prefix = state.prefix(protocol.investigator).clone();
            let belief = last_supported[protocol.investigator.index()]
                .clone()
                .expect("supported boundary");
            let decision = replay.decide(policy, &View::from_prefix(prefix.clone(), belief)?)?;
            select_choice(&mut state, decision.choice)?;
            episode.decisions.push(DecisionTrace { prefix, decision });
            // Record the free choice/public stop before entering the next phase.
            next = None;
            continue;
        }
        let step = advance_one(&mut state)?;
        next = step.next;
        for (role, event) in step.events {
            let read_lineage = if event.action == Action::Read {
                state.read_lineage(role)
            } else {
                None
            };
            let metric = &mut episode.metrics[role.index()];
            match event.action {
                Action::Read => {
                    metric.reads += 1;
                    if let (Phase::Live { trial }, Some(lineage)) =
                        (event.position.phase, &read_lineage)
                    {
                        if lineage.writer != role && lineage.task_trial == Some(trial) {
                            metric.lineage_reads += 1;
                        }
                    }
                }
                Action::Write(symbol) => {
                    metric.writes += 1;
                    if matches!(event.position.phase, Phase::Live { .. })
                        && matches!(symbol, Symbol::Data0 | Symbol::Data1)
                    {
                        metric.attempted_sends += 1;
                    }
                }
                Action::Wait => metric.waits += 1,
                Action::InspectOwnTarget => metric.inspections += 1,
            }
            episode.privileged.push(PrivilegedStep {
                role,
                event,
                read_lineage,
            });
        }
    }
    for role in [Role::A, Role::B] {
        let i = role.index();
        let correct = episode.predictions[i]
            .iter()
            .zip(bits.0)
            .filter(|(prediction, (a, b))| **prediction == if role == Role::A { *b } else { *a })
            .count() as u8;
        let reward = i64::from(correct) * 12;
        episode.metrics[i].correct = Some(correct);
        episode.metrics[i].reward = Some(reward);
        episode.metrics[i].net = Some(reward - i64::from(episode.metrics[i].spent));
        let final_belief = last_supported[i]
            .clone()
            .expect("supported terminal history");
        let true_model_mass = match environment {
            Environment::InFamily(model) => Some(final_belief.models[model.index()]),
            Environment::DataFlip => None,
        };
        episode.model_projections[i] = Some(ModelProjection {
            final_belief,
            true_model_mass,
            uniquely_identified: true_model_mass.map(|p| p.numerator == p.denominator),
            identified_catalog_model: episode.catalog_discoveries[i].as_ref().map(|d| d.model),
            identification_source: episode.catalog_discoveries[i].as_ref().map(|d| d.source),
            first_identification: episode.catalog_discoveries[i]
                .as_ref()
                .map(|d| d.checkpoint),
            spent_to_identify: episode.catalog_discoveries[i].as_ref().map(|d| d.spent),
        });
    }
    reservation.retain();
    Ok(episode)
}

pub fn evaluate_panel(
    protocol: &Protocol,
    environment: Environment,
    policy: &CompiledPolicy,
    replay: &Replay,
) -> Result<Panel, Error> {
    let mut episodes = Vec::with_capacity(256);
    for sequence in 0..256 {
        match run_episode(protocol, environment, sequence, policy, replay) {
            Ok(episode) => episodes.push(episode),
            Err(error) => {
                replay.release_episodes(episodes.len())?;
                return Err(error);
            }
        }
    }
    let aggregate = match super::metrics::aggregate_episodes(&episodes) {
        Ok(aggregate) => aggregate,
        Err(error) => {
            replay.release_episodes(episodes.len())?;
            return Err(error);
        }
    };
    Ok(Panel {
        protocol: protocol.clone(),
        environment,
        episodes,
        aggregate,
    })
}

/// Local posterior majority, with a zero prediction at exact ties.
pub fn predict_target(belief: &Belief) -> Result<bool, Error> {
    let target = belief
        .target
        .ok_or_else(|| Error::InvalidHistory("prediction requires issued target".into()))?;
    Ok(target.checked_cmp(Probability::new(1, 2)?)? == std::cmp::Ordering::Greater)
}
