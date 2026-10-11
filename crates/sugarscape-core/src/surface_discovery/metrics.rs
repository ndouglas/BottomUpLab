//! Exact summaries of matched actual outcomes, separate from Agent priors.
use super::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PairedComparison {
    pub investigator: Role,
    pub environment: Environment,
    pub ids: Ids,
    pub left_policy: PolicyKind,
    pub right_policy: PolicyKind,
    /// Left minus right, evaluated on each of the same 256 actual sequences.
    pub difference: Aggregate,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActualMechanismWeight {
    pub model: Mechanism,
    pub weight: Probability,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UniformMechanismSummary {
    pub protocol: Protocol,
    /// An outer distribution of actual mechanisms, never an Agent's OwnPrior.
    /// For KnownRaw, this is a mixture of four separately informed point controls.
    pub actual_mechanism_distribution: [ActualMechanismWeight; 4],
    pub aggregate: Aggregate,
}

fn indexed(panel: &Panel) -> Result<BTreeMap<u16, &Episode>, Error> {
    panel.protocol.validate()?;
    if panel.episodes.len() != 256 {
        return Err(Error::InvalidReport(
            "summary requires all 256 sequences".into(),
        ));
    }
    let mut episodes = BTreeMap::new();
    for episode in &panel.episodes {
        if episode.sequence >= 256 || episodes.insert(episode.sequence, episode).is_some() {
            return Err(Error::InvalidReport(
                "summary requires unique sequences 0..255".into(),
            ));
        }
    }
    Ok(episodes)
}
#[derive(Default)]
struct Totals {
    rows: u64,
    failed: u64,
    spent: [i64; 2],
    reward: [i64; 2],
    net: [i64; 2],
}
fn add(left: i64, right: i64) -> Result<i64, Error> {
    left.checked_add(right).ok_or(Error::ArithmeticOverflow)
}
fn subtract(left: i64, right: i64) -> Result<i64, Error> {
    left.checked_sub(right).ok_or(Error::ArithmeticOverflow)
}
fn terminal(value: Option<i64>) -> Result<i64, Error> {
    value.ok_or_else(|| Error::InvalidReport("complete episode lacks terminal metric".into()))
}
impl Totals {
    fn record(&mut self, left: &Episode, right: Option<&Episode>) -> Result<(), Error> {
        self.rows += 1;
        let failed = left.failure.is_some() || right.is_some_and(|e| e.failure.is_some());
        self.failed += u64::from(failed);
        for i in 0..2 {
            let a = &left.metrics[i];
            let b = right.map(|e| &e.metrics[i]);
            self.spent[i] = add(
                self.spent[i],
                i64::from(a.spent) - b.map_or(0, |m| i64::from(m.spent)),
            )?;
            if !failed {
                self.reward[i] = add(
                    self.reward[i],
                    subtract(
                        terminal(a.reward)?,
                        b.map(|m| terminal(m.reward)).transpose()?.unwrap_or(0),
                    )?,
                )?;
                self.net[i] = add(
                    self.net[i],
                    subtract(
                        terminal(a.net)?,
                        b.map(|m| terminal(m.net)).transpose()?.unwrap_or(0),
                    )?,
                )?;
            }
        }
        Ok(())
    }
    fn finish(self) -> Result<Aggregate, Error> {
        let fraction =
            |n| -> Result<ScoreFraction, Error> { Ok(ScoreFraction::new(n, self.rows)?) };
        let spent = [fraction(self.spent[0])?, fraction(self.spent[1])?];
        let (reward, net, group_net) = if self.failed == 0 {
            (
                [
                    Some(fraction(self.reward[0])?),
                    Some(fraction(self.reward[1])?),
                ],
                [Some(fraction(self.net[0])?), Some(fraction(self.net[1])?)],
                Some(fraction(add(self.net[0], self.net[1])?)?),
            )
        } else {
            ([None, None], [None, None], None)
        };
        Ok(Aggregate {
            valid_mass: Probability::new(self.rows - self.failed, self.rows)?,
            failed_mass: Probability::new(self.failed, self.rows)?,
            spent,
            reward,
            net,
            group_net,
        })
    }
}

/// Compare policies on identical actual mechanisms, physical roles and IDs.
/// Public cached aggregates are ignored; means are recomputed from paired rows.
pub fn compare_panels(left: &Panel, right: &Panel) -> Result<PairedComparison, Error> {
    if left.environment != right.environment
        || left.protocol.investigator != right.protocol.investigator
        || left.protocol.ids != right.protocol.ids
    {
        return Err(Error::InvalidReport(
            "paired panels differ in environment, role, or IDs".into(),
        ));
    }
    let a = indexed(left)?;
    let b = indexed(right)?;
    let mut totals = Totals::default();
    for (sequence, episode) in a {
        totals.record(episode, Some(b[&sequence]))?;
    }
    Ok(PairedComparison {
        investigator: left.protocol.investigator,
        environment: left.environment,
        ids: left.protocol.ids.clone(),
        left_policy: left.protocol.policy,
        right_policy: right.protocol.policy,
        difference: totals.finish()?,
    })
}

/// Equal-weight actual-mechanism summary. No cross-role or cross-policy pooling.
pub fn uniform_actual_mechanism_summary(
    panels: &[Panel],
) -> Result<UniformMechanismSummary, Error> {
    if panels.len() != 4 {
        return Err(Error::InvalidReport(
            "uniform mechanism summary requires four catalog panels".into(),
        ));
    }
    let protocol = &panels[0].protocol;
    let mut seen = [false; 4];
    let mut totals = Totals::default();
    for panel in panels {
        if &panel.protocol != protocol {
            return Err(Error::InvalidReport(
                "uniform mechanism summary requires one policy, role, and ID set".into(),
            ));
        }
        let Environment::InFamily(model) = panel.environment else {
            return Err(Error::InvalidReport(
                "DataFlip is outside the actual catalog mixture".into(),
            ));
        };
        if std::mem::replace(&mut seen[model.index()], true) {
            return Err(Error::InvalidReport(
                "duplicate actual mechanism in summary".into(),
            ));
        }
        for episode in indexed(panel)?.into_values() {
            totals.record(episode, None)?;
        }
    }
    Ok(UniformMechanismSummary {
        protocol: protocol.clone(),
        actual_mechanism_distribution: Mechanism::ALL.map(|model| ActualMechanismWeight {
            model,
            weight: Probability::new(1, 4).expect("fixed catalog weight"),
        }),
        aggregate: totals.finish()?,
    })
}

pub(super) fn aggregate_episodes(episodes: &[Episode]) -> Result<Aggregate, Error> {
    validate_sequences(episodes)?;
    let mut totals = Totals::default();
    for episode in episodes {
        totals.record(episode, None)?;
    }
    totals.finish()
}
fn validate_sequences(episodes: &[Episode]) -> Result<(), Error> {
    if episodes.len() != 256 {
        return Err(Error::InvalidReport(
            "summary requires all 256 sequences".into(),
        ));
    }
    let mut seen = [false; 256];
    for episode in episodes {
        if episode.sequence >= 256
            || std::mem::replace(&mut seen[usize::from(episode.sequence)], true)
        {
            return Err(Error::InvalidReport(
                "summary requires unique sequences 0..255".into(),
            ));
        }
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeanCounts {
    pub reads: ScoreFraction,
    pub writes: ScoreFraction,
    pub waits: ScoreFraction,
    pub inspections: ScoreFraction,
    pub attempted_sends: ScoreFraction,
    pub lineage_reads: ScoreFraction,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CountMass {
    pub count: u8,
    pub mass: Probability,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FailureMass {
    pub role: Role,
    pub checkpoint: Checkpoint,
    pub mass: Probability,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiscoveryMass {
    pub model: Mechanism,
    pub source: CatalogCertaintySource,
    pub checkpoint: Checkpoint,
    pub spent: u8,
    pub mass: Probability,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdentificationProjection {
    pub true_model_mass: Option<ScoreFraction>,
    pub uniquely_identified_mass: Option<Probability>,
    pub discoveries: Vec<DiscoveryMass>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalibrationBin {
    pub trial: u8,
    pub probability: Probability,
    pub issued_mass: Probability,
    pub ones_mass: Probability,
}
/// Scientific scalar projection. External artifact receipts belong to the
/// diagnostic provenance layer, not this runtime population calculation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PanelProjection {
    pub unavailable_mass: Probability,
    pub accuracy: [Option<ScoreFraction>; 2],
    pub action_counts: [MeanCounts; 2],
    pub calibration_rounds: Vec<CountMass>,
    pub failures: Vec<FailureMass>,
    pub identification: [IdentificationProjection; 2],
    pub posterior_calibration: [Vec<CalibrationBin>; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleContrast {
    pub investigator: Role,
    pub left_policy: PolicyKind,
    pub right_policy: PolicyKind,
    pub difference: Aggregate,
}

pub fn project_panel(panel: &Panel) -> Result<PanelProjection, Error> {
    let episodes: Vec<_> = indexed(panel)?.into_values().collect();
    project_population(&episodes, panel.environment == Environment::DataFlip)
}

/// Equal-weight actual-mechanism projection, including emitted posterior bins.
/// These are four separately informed compilations for KnownRaw.
pub fn project_uniform_actual_mechanisms(panels: &[Panel]) -> Result<PanelProjection, Error> {
    uniform_actual_mechanism_summary(panels)?;
    let episodes: Vec<_> = panels.iter().flat_map(|p| &p.episodes).collect();
    project_population(&episodes, false)
}

pub fn compare_uniform_actual_mechanisms(
    left: &[Panel],
    right: &[Panel],
) -> Result<RoleContrast, Error> {
    uniform_actual_mechanism_summary(left)?;
    uniform_actual_mechanism_summary(right)?;
    let mut totals = Totals::default();
    for panel in left {
        let other = right
            .iter()
            .find(|p| p.environment == panel.environment)
            .ok_or_else(|| {
                Error::InvalidReport("uniform contrast lacks matched actual mechanism".into())
            })?;
        compare_panels(panel, other)?;
        let paired = indexed(other)?;
        for (sequence, episode) in indexed(panel)? {
            totals.record(episode, Some(paired[&sequence]))?;
        }
    }
    Ok(RoleContrast {
        investigator: left[0].protocol.investigator,
        left_policy: left[0].protocol.policy,
        right_policy: right[0].protocol.policy,
        difference: totals.finish()?,
    })
}

fn wire_key(value: &impl Serialize) -> Result<String, Error> {
    serde_json::to_string(value).map_err(|e| Error::InvalidReport(format!("distribution key: {e}")))
}
fn project_population(episodes: &[&Episode], transfer: bool) -> Result<PanelProjection, Error> {
    let n = episodes.len() as u64;
    let unavailable = episodes
        .iter()
        .filter(|e| {
            e.failure.is_some()
                || e.metrics
                    .iter()
                    .any(|m| m.reward.is_none() || m.net.is_none() || m.correct.is_none())
        })
        .count() as u64;
    let mean = |role: usize, field: fn(&AgentMetrics) -> u8| -> Result<ScoreFraction, Error> {
        let sum = episodes
            .iter()
            .try_fold(0i64, |s, e| add(s, i64::from(field(&e.metrics[role]))))?;
        Ok(ScoreFraction::new(sum, n)?)
    };
    let counts = |role| -> Result<MeanCounts, Error> {
        Ok(MeanCounts {
            reads: mean(role, |m| m.reads)?,
            writes: mean(role, |m| m.writes)?,
            waits: mean(role, |m| m.waits)?,
            inspections: mean(role, |m| m.inspections)?,
            attempted_sends: mean(role, |m| m.attempted_sends)?,
            lineage_reads: mean(role, |m| m.lineage_reads)?,
        })
    };
    let mut accuracy = [None, None];
    if unavailable == 0 {
        for (role, value) in accuracy.iter_mut().enumerate() {
            let correct = episodes.iter().try_fold(0i64, |s, e| {
                add(
                    s,
                    i64::from(
                        e.metrics[role]
                            .correct
                            .expect("checked terminal availability"),
                    ),
                )
            })?;
            *value = Some(ScoreFraction::new(
                correct,
                n.checked_mul(4).ok_or(Error::ArithmeticOverflow)?,
            )?);
        }
    }
    let mut rounds = BTreeMap::<u8, u64>::new();
    let mut failures = BTreeMap::<String, (Role, Checkpoint, u64)>::new();
    for episode in episodes {
        let purchased = episode
            .decisions
            .iter()
            .filter(|d| d.decision.choice == Choice::Continue)
            .count();
        if purchased > 3 {
            return Err(Error::InvalidReport(
                "more than three purchased calibration rounds".into(),
            ));
        }
        *rounds.entry(purchased as u8).or_default() += 1;
        if let Some(failure) = &episode.failure {
            #[derive(Serialize)]
            struct Key {
                role: Role,
                checkpoint: Checkpoint,
            }
            let key = wire_key(&Key {
                role: failure.role,
                checkpoint: failure.prefix.checkpoint,
            })?;
            failures
                .entry(key)
                .or_insert((failure.role, failure.prefix.checkpoint, 0))
                .2 += 1;
        }
    }
    let calibration_rounds = rounds
        .into_iter()
        .map(|(count, total)| {
            Ok(CountMass {
                count,
                mass: Probability::new(total, n)?,
            })
        })
        .collect::<Result<_, Error>>()?;
    let failures = failures
        .into_values()
        .map(|(role, checkpoint, total)| {
            Ok(FailureMass {
                role,
                checkpoint,
                mass: Probability::new(total, n)?,
            })
        })
        .collect::<Result<_, Error>>()?;
    let identification = |role: usize| -> Result<IdentificationProjection, Error> {
        let mut discoveries = BTreeMap::<String, (CatalogDiscovery, u64)>::new();
        for episode in episodes {
            if let Some(discovery) = &episode.catalog_discoveries[role] {
                discoveries
                    .entry(wire_key(discovery)?)
                    .or_insert((discovery.clone(), 0))
                    .1 += 1;
            }
        }
        let discoveries = discoveries
            .into_values()
            .map(|(d, total)| {
                Ok(DiscoveryMass {
                    model: d.model,
                    source: d.source,
                    checkpoint: d.checkpoint,
                    spent: d.spent,
                    mass: Probability::new(total, n)?,
                })
            })
            .collect::<Result<_, Error>>()?;
        let mut result = IdentificationProjection {
            true_model_mass: None,
            uniquely_identified_mass: None,
            discoveries,
        };
        if !transfer
            && episodes.iter().all(|e| {
                e.failure.is_none()
                    && e.model_projections[role].as_ref().is_some_and(|m| {
                        m.true_model_mass.is_some() && m.uniquely_identified.is_some()
                    })
            })
        {
            let mut mass = ScoreFraction::new(0, 1)?;
            let mut unique = 0;
            for episode in episodes {
                let model = episode.model_projections[role]
                    .as_ref()
                    .expect("checked model projection");
                let p = model.true_model_mass.expect("checked model grade");
                mass = mass.checked_add(ScoreFraction::new(
                    i64::try_from(p.numerator).map_err(|_| Error::ArithmeticOverflow)?,
                    p.denominator,
                )?)?;
                unique += u64::from(model.uniquely_identified.expect("checked model grade"));
            }
            result.true_model_mass = Some(mass.checked_mul(ScoreFraction::new(1, n)?)?);
            result.uniquely_identified_mass = Some(Probability::new(unique, n)?);
        }
        Ok(result)
    };
    let bins = |role: usize| -> Result<Vec<CalibrationBin>, Error> {
        let mut grouped = BTreeMap::<(u8, Probability), (u64, u64)>::new();
        for episode in episodes {
            if episode.prediction_beliefs[role].len() != episode.predictions[role].len()
                || episode.predictions[role].len() > 4
            {
                return Err(Error::InvalidReport(
                    "prediction records disagree or exceed four trials".into(),
                ));
            }
            let bits = EpisodeBits::from_index(episode.sequence)?;
            for (trial, belief) in episode.prediction_beliefs[role].iter().enumerate() {
                let p = belief.target.ok_or_else(|| {
                    Error::InvalidReport("issued prediction lacks target belief".into())
                })?;
                let probability = Probability::new(p.numerator, p.denominator)?;
                let (a, b) = bits.0[trial];
                let count = grouped.entry((trial as u8, probability)).or_default();
                count.0 += 1;
                count.1 += u64::from(if role == 0 { b } else { a });
            }
        }
        let mut bins = grouped
            .into_iter()
            .map(|((trial, probability), (issued, ones))| {
                Ok(CalibrationBin {
                    trial,
                    probability,
                    issued_mass: Probability::new(issued, n)?,
                    ones_mass: Probability::new(ones, n)?,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        // Probability's structural Ord is not numeric ordering.
        bins.sort_by(|a, b| {
            a.trial.cmp(&b.trial).then_with(|| {
                (u128::from(a.probability.numerator) * u128::from(b.probability.denominator)).cmp(
                    &(u128::from(b.probability.numerator) * u128::from(a.probability.denominator)),
                )
            })
        });
        Ok(bins)
    };
    Ok(PanelProjection {
        unavailable_mass: Probability::new(unavailable, n)?,
        accuracy,
        action_counts: [counts(0)?, counts(1)?],
        calibration_rounds,
        failures,
        identification: [identification(0)?, identification(1)?],
        posterior_calibration: [bins(0)?, bins(1)?],
    })
}
