//! Compact authoritative records; cached check flags are never evidence.
pub(super) mod hash;
mod protocol;
pub(super) mod references;
use super::*;
pub use protocol::{registered_settings, DIAGNOSTIC_VERSION, PROTOCOL_VERSION};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrefixRef {
    pub history: u64,
    pub entry_count: u64,
    pub checkpoint: Checkpoint,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactDecision {
    pub prefix: PrefixRef,
    pub decision: Decision,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactFailure {
    pub role: Role,
    pub prefix: PrefixRef,
    pub last_supported: Option<Belief>,
    pub spent: [u8; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactEpisode {
    pub sequence: u16,
    pub histories: [u64; 2],
    pub decisions: Vec<CompactDecision>,
    pub metrics: [AgentMetrics; 2],
    pub predictions: [Vec<bool>; 2],
    pub prediction_beliefs: [Vec<Belief>; 2],
    pub privileged: Vec<PrivilegedStep>,
    pub failure: Option<CompactFailure>,
    pub model_projections: [Option<ModelProjection>; 2],
    pub catalog_discoveries: [Option<CatalogDiscovery>; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompactPanel {
    pub protocol: Protocol,
    pub environment: Environment,
    pub episodes: Vec<CompactEpisode>,
    pub aggregate: Aggregate,
    pub projection: PanelProjection,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArtifactReceipt {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompilationStats {
    pub protocol: Protocol,
    pub prior: OwnPrior,
    pub stats: SearchStats,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub name: String,
    pub passed: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticReport {
    pub version: String,
    pub protocol_version: u16,
    pub supplied_structure: Vec<String>,
    pub source_hash: String,
    pub sources: Vec<ArtifactReceipt>,
    pub reference_artifacts: Vec<ArtifactReceipt>,
    pub histories: Vec<Prefix>,
    pub panels: Vec<CompactPanel>,
    pub comparisons: Vec<PairedComparison>,
    pub role_expectations: Vec<UniformMechanismSummary>,
    pub role_projections: Vec<PanelProjection>,
    pub role_contrasts: Vec<RoleContrast>,
    pub compilations: Vec<CompilationStats>,
    pub checks: Vec<Check>,
    pub passed: bool,
}
fn invalid(message: impl Into<String>) -> Error {
    Error::InvalidReport(message.into())
}
fn json_value(value: &impl Serialize) -> Result<serde_json::Value, Error> {
    serde_json::to_value(value).map_err(|e| invalid(format!("record serialization: {e}")))
}
pub(super) fn content_hash(value: &impl Serialize) -> Result<String, Error> {
    let mut digest = hash::Digest::default();
    serde_json::to_writer(&mut digest, value)
        .map_err(|e| invalid(format!("hash serialization: {e}")))?;
    Ok(digest.finish())
}

/// Current compiled dependencies, distinct from retained historical measurements.
pub(crate) fn compiled_source_identity() -> Result<(Vec<ArtifactReceipt>, String), Error> {
    let sources = protocol::sources();
    let digest = content_hash(&sources)?;
    Ok((sources, digest))
}
/// Hash a declared compiled-byte closure through the same canonical receipt codec.
pub(crate) fn compiled_bytes_identity(inputs: &[(&str, &[u8])]) -> Result<String, Error> {
    let sources: Vec<_> = inputs
        .iter()
        .map(|(path, bytes)| ArtifactReceipt {
            path: (*path).into(),
            bytes: bytes.len() as u64,
            sha256: hash::sha256(bytes),
        })
        .collect();
    content_hash(&sources)
}
/// Declared source files may use generated raw receipts on the browser target.
pub(crate) fn compiled_file_identity(inputs: &[(&str, &[u8])]) -> Result<String, Error> {
    let sources: Vec<_> = inputs
        .iter()
        .map(|(path, bytes)| {
            let (bytes, sha256) = crate::compact_receipts::raw(path, bytes).map_err(invalid)?;
            Ok(ArtifactReceipt {
                path: (*path).into(),
                bytes,
                sha256,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    content_hash(&sources)
}

impl PrefixRef {
    pub fn reconstruct(&self, histories: &[Prefix]) -> Result<Prefix, Error> {
        let index = usize::try_from(self.history).map_err(|_| Error::ArithmeticOverflow)?;
        let count = usize::try_from(self.entry_count).map_err(|_| Error::ArithmeticOverflow)?;
        let history = histories
            .get(index)
            .ok_or_else(|| invalid("history index out of range"))?;
        let entries = history
            .entries
            .get(..count)
            .ok_or_else(|| invalid("prefix entry count exceeds history"))?;
        let prefix = Prefix {
            role: history.role,
            ids: history.ids.clone(),
            own_prior: history.own_prior,
            checkpoint: self.checkpoint,
            entries: entries.to_vec(),
        };
        prefix.validate()?;
        Ok(prefix)
    }
}
#[derive(Default)]
pub(super) struct Library {
    pub histories: Vec<Prefix>,
    indexes: std::collections::BTreeMap<Prefix, u64>,
}
impl Library {
    fn intern(&mut self, prefix: &Prefix) -> Result<u64, Error> {
        prefix.validate()?;
        if let Some(index) = self.indexes.get(prefix) {
            return Ok(*index);
        }
        let index = u64::try_from(self.histories.len()).map_err(|_| Error::ArithmeticOverflow)?;
        self.histories.push(prefix.clone());
        self.indexes.insert(prefix.clone(), index);
        Ok(index)
    }
    pub fn compact(&mut self, episode: &Episode) -> Result<CompactEpisode, Error> {
        EpisodeBits::from_index(episode.sequence)?;
        let histories = [
            self.intern(&episode.histories[0])?,
            self.intern(&episode.histories[1])?,
        ];
        let reference = |prefix: &Prefix| -> Result<PrefixRef, Error> {
            let history = &episode.histories[prefix.role.index()];
            if history.ids != prefix.ids
                || history.own_prior != prefix.own_prior
                || !history.entries.starts_with(&prefix.entries)
            {
                return Err(invalid("prefix is not an owned terminal-history prefix"));
            }
            prefix.validate()?;
            Ok(PrefixRef {
                history: histories[prefix.role.index()],
                entry_count: prefix.entries.len() as u64,
                checkpoint: prefix.checkpoint,
            })
        };
        Ok(CompactEpisode {
            sequence: episode.sequence,
            histories,
            decisions: episode
                .decisions
                .iter()
                .map(|d| {
                    Ok(CompactDecision {
                        prefix: reference(&d.prefix)?,
                        decision: d.decision.clone(),
                    })
                })
                .collect::<Result<_, Error>>()?,
            failure: episode
                .failure
                .as_ref()
                .map(|f| {
                    Ok::<_, Error>(CompactFailure {
                        role: f.role,
                        prefix: reference(&f.prefix)?,
                        last_supported: f.last_supported.clone(),
                        spent: f.spent,
                    })
                })
                .transpose()?,
            metrics: episode.metrics.clone(),
            predictions: episode.predictions.clone(),
            prediction_beliefs: episode.prediction_beliefs.clone(),
            privileged: episode.privileged.clone(),
            model_projections: episode.model_projections.clone(),
            catalog_discoveries: episode.catalog_discoveries.clone(),
        })
    }
}
/// Lossless reconstruction, including all alternatives and first-failure observations.
/// This checks representation, not current-source scientific authenticity.
pub fn reconstruct_episode(
    report: &DiagnosticReport,
    setting: usize,
    sequence: u16,
) -> Result<Episode, Error> {
    EpisodeBits::from_index(sequence)?;
    let panel = report
        .panels
        .get(setting)
        .ok_or_else(|| invalid("setting index out of range"))?;
    panel.protocol.validate()?;
    let row = panel
        .episodes
        .get(usize::from(sequence))
        .ok_or_else(|| invalid("sequence row missing"))?;
    if row.sequence != sequence {
        return Err(invalid("sequence order mismatch"));
    }
    let history = |role: Role| -> Result<Prefix, Error> {
        let index =
            usize::try_from(row.histories[role.index()]).map_err(|_| Error::ArithmeticOverflow)?;
        let prefix = report
            .histories
            .get(index)
            .ok_or_else(|| invalid("episode history index out of range"))?
            .clone();
        panel.protocol.validate_prefix(&prefix)?;
        if prefix.role != role {
            return Err(invalid("history has wrong physical owner"));
        }
        Ok(prefix)
    };
    let histories = [history(Role::A)?, history(Role::B)?];
    let prefix = |reference: &PrefixRef, role: Role| -> Result<Prefix, Error> {
        if reference.history != row.histories[role.index()] {
            return Err(invalid("prefix references another owned history"));
        }
        let prefix = reference.reconstruct(&report.histories)?;
        panel.protocol.validate_prefix(&prefix)?;
        if prefix.role != role {
            return Err(invalid("prefix owner mismatch"));
        }
        Ok(prefix)
    };
    let episode = Episode {
        sequence,
        histories,
        decisions: row
            .decisions
            .iter()
            .map(|d| {
                Ok(DecisionTrace {
                    prefix: prefix(&d.prefix, panel.protocol.investigator)?,
                    decision: d.decision.clone(),
                })
            })
            .collect::<Result<_, Error>>()?,
        failure: row
            .failure
            .as_ref()
            .map(|f| {
                Ok::<_, Error>(Failure {
                    role: f.role,
                    prefix: prefix(&f.prefix, f.role)?,
                    last_supported: f.last_supported.clone(),
                    spent: f.spent,
                })
            })
            .transpose()?,
        metrics: row.metrics.clone(),
        predictions: row.predictions.clone(),
        prediction_beliefs: row.prediction_beliefs.clone(),
        privileged: row.privileged.clone(),
        model_projections: row.model_projections.clone(),
        catalog_discoveries: row.catalog_discoveries.clone(),
    };
    // Typed decoding checks canonical rationals and every nested enum/clock, including
    // records supplied programmatically rather than through Deserialize.
    serde_json::from_value(json_value(&episode)?)
        .map_err(|e| invalid(format!("reconstructed episode: {e}")))
}
fn reconstruct_panel(report: &DiagnosticReport, index: usize) -> Result<Panel, Error> {
    let row = report
        .panels
        .get(index)
        .ok_or_else(|| invalid("setting index out of range"))?;
    if row.episodes.len() != 256 {
        return Err(invalid("panel must contain 256 sequences"));
    }
    Ok(Panel {
        protocol: row.protocol.clone(),
        environment: row.environment,
        aggregate: row.aggregate.clone(),
        episodes: (0..256)
            .map(|sequence| reconstruct_episode(report, index, sequence))
            .collect::<Result<_, _>>()?,
    })
}
fn check_names() -> Vec<Check> {
    [
        "ordered_registered_census",
        "complete_current_source_episodes",
        "exact_independent_projections_and_witnesses",
        "sealed_reference_receipt_binding",
        "complete_paired_and_uniform_summaries",
        "cumulative_compilation_accounting",
        "canonical_compact_histories",
    ]
    .map(|name| Check {
        name: name.into(),
        passed: false,
    })
    .to_vec()
}
pub(super) fn new_report(reference: &references::Reference) -> Result<DiagnosticReport, Error> {
    let sources = protocol::sources();
    Ok(DiagnosticReport {
        version: DIAGNOSTIC_VERSION.into(),
        protocol_version: PROTOCOL_VERSION,
        supplied_structure: protocol::supplied_structure(),
        source_hash: content_hash(&sources)?,
        sources,
        reference_artifacts: reference.artifacts.clone(),
        histories: vec![],
        panels: vec![],
        comparisons: vec![],
        role_expectations: vec![],
        role_projections: vec![],
        role_contrasts: vec![],
        compilations: vec![],
        checks: check_names(),
        passed: false,
    })
}
/// One complete evaluation per declared setting. Each compilation owns one Replay
/// across all its applicable environments; it is released before the next owner.
pub fn diagnose(limits: &Limits) -> Result<DiagnosticReport, Error> {
    limits.validate()?;
    let reference = references::Reference::read()?;
    let settings = registered_settings();
    let mut report = new_report(&reference)?;
    let mut library = Library::default();
    let mut panels: Vec<Option<CompactPanel>> = vec![None; settings.len()];
    let mut matches_reference = true;
    for (protocol, prior) in protocol::compilations(&settings)? {
        let policy = CompiledPolicy::build(&protocol, prior, limits)?;
        let replay = Replay::build(&protocol, &policy, limits)?;
        for (index, (p, environment)) in settings.iter().enumerate() {
            if *p != protocol || protocol::own_prior(p, *environment)? != prior {
                continue;
            }
            let panel = evaluate_panel(p, *environment, &policy, &replay)?;
            matches_reference &= reference.panel_matches(index, &panel)?;
            panels[index] = Some(CompactPanel {
                protocol: p.clone(),
                environment: *environment,
                aggregate: panel.aggregate.clone(),
                projection: project_panel(&panel)?,
                episodes: panel
                    .episodes
                    .iter()
                    .map(|e| library.compact(e))
                    .collect::<Result<_, _>>()?,
            });
        }
        report.compilations.push(CompilationStats {
            protocol,
            prior,
            stats: replay.stats(),
        });
    }
    report.histories = library.histories;
    report.panels = panels
        .into_iter()
        .map(|p| p.ok_or_else(|| invalid("census setting not evaluated")))
        .collect::<Result<_, _>>()?;
    fill_summaries(&mut report)?;
    matches_reference &= reference.summaries_match(&report)?;
    // Fresh execution produced and checked this payload. The checked emitter still
    // performs an independent fresh logical revalidation with new budget owners.
    for check in &mut report.checks {
        check.passed = matches_reference;
    }
    report.passed = matches_reference;
    Ok(report)
}
struct Summaries {
    comparisons: Vec<PairedComparison>,
    expectations: Vec<UniformMechanismSummary>,
    projections: Vec<PanelProjection>,
    contrasts: Vec<RoleContrast>,
}
fn fill_summaries(report: &mut DiagnosticReport) -> Result<(), Error> {
    let rows = compute_summaries(report)?;
    report.comparisons = rows.comparisons;
    report.role_expectations = rows.expectations;
    report.role_projections = rows.projections;
    report.role_contrasts = rows.contrasts;
    Ok(())
}
fn compute_summaries(report: &DiagnosticReport) -> Result<Summaries, Error> {
    let find = |role: Role, policy: PolicyKind, environment: Environment| -> Result<usize, Error> {
        report
            .panels
            .iter()
            .position(|p| {
                p.protocol.investigator == role
                    && p.protocol.policy == policy
                    && p.environment == environment
            })
            .ok_or_else(|| invalid("summary endpoint missing"))
    };
    let pairs = [
        (PolicyKind::RawAdaptive, PolicyKind::SuppliedAdaptive),
        (PolicyKind::RawAdaptive, PolicyKind::NoProbe),
        (PolicyKind::SuppliedAdaptive, PolicyKind::NoProbe),
    ];
    let mut comparisons = Vec::new();
    let mut expectations = Vec::new();
    let mut projections = Vec::new();
    let mut contrasts = Vec::new();
    for model in Mechanism::ALL {
        for role in [Role::A, Role::B] {
            for (left, right) in pairs {
                comparisons.push(compare_panels(
                    &reconstruct_panel(report, find(role, left, Environment::InFamily(model))?)?,
                    &reconstruct_panel(report, find(role, right, Environment::InFamily(model))?)?,
                )?);
            }
        }
    }
    let group = |role, policy| -> Result<Vec<Panel>, Error> {
        Mechanism::ALL
            .into_iter()
            .map(|model| {
                reconstruct_panel(report, find(role, policy, Environment::InFamily(model))?)
            })
            .collect()
    };
    for role in [Role::A, Role::B] {
        for policy in [
            PolicyKind::RawAdaptive,
            PolicyKind::SuppliedAdaptive,
            PolicyKind::NoProbe,
            PolicyKind::KnownRaw,
            PolicyKind::InspectOnly,
        ] {
            let panels = group(role, policy)?;
            expectations.push(uniform_actual_mechanism_summary(&panels)?);
            projections.push(project_uniform_actual_mechanisms(&panels)?);
        }
        for (left, right) in pairs {
            contrasts.push(compare_uniform_actual_mechanisms(
                &group(role, left)?,
                &group(role, right)?,
            )?);
        }
    }
    Ok(Summaries {
        comparisons,
        expectations,
        projections,
        contrasts,
    })
}
/// Bounded row seam used by tests with independently reviewed complete witnesses.
/// Computational errors remain errors; unequal scientific data returns false.
pub(super) fn episode_matches_current(
    report: &DiagnosticReport,
    index: usize,
    sequence: u16,
    policy: &CompiledPolicy,
    replay: &Replay,
) -> Result<bool, Error> {
    let episode = reconstruct_episode(report, index, sequence)?;
    let row = &report.panels[index];
    let expected = run_episode(&row.protocol, row.environment, sequence, policy, replay)?;
    Ok(episode == expected)
}
fn payload_integrity(
    report: &DiagnosticReport,
    limits: &Limits,
    reference: &references::Reference,
) -> Result<bool, Error> {
    limits.validate()?;
    let sources = protocol::sources();
    if report.version != DIAGNOSTIC_VERSION
        || report.protocol_version != PROTOCOL_VERSION
        || report.supplied_structure != protocol::supplied_structure()
        || report.reference_artifacts != reference.artifacts
        || report
            .checks
            .iter()
            .map(|c| &c.name)
            .ne(check_names().iter().map(|c| &c.name))
    {
        return Ok(false);
    }
    if report.sources != sources || report.source_hash != content_hash(&sources)? {
        return Err(Error::InvalidProtocol(
            "diagnostic compiled source identity mismatch".into(),
        ));
    }
    let settings = registered_settings();
    if report.panels.len() != settings.len() {
        return Ok(false);
    }
    for (row, (protocol, environment)) in report.panels.iter().zip(&settings) {
        if row.protocol != *protocol
            || row.environment != *environment
            || row.episodes.len() != 256
            || row
                .episodes
                .iter()
                .enumerate()
                .any(|(index, e)| usize::from(e.sequence) != index)
        {
            return Ok(false);
        }
    }
    // Every scientific row is compared to a canonical recomputation below; episode
    // decoding also checks nested rationals for programmatic callers. Avoid creating
    // another complete serialized report just to validate its representation.
    let mut library = Library::default();
    let mut stats = Vec::new();
    for (protocol, prior) in protocol::compilations(&settings)? {
        let policy = CompiledPolicy::build(&protocol, prior, limits)?;
        let replay = Replay::build(&protocol, &policy, limits)?;
        for (index, (p, environment)) in settings.iter().enumerate() {
            if *p != protocol || protocol::own_prior(p, *environment)? != prior {
                continue;
            }
            let panel = reconstruct_panel(report, index)?;
            for (sequence, episode) in panel.episodes.iter().enumerate() {
                if !episode_matches_current(report, index, sequence as u16, &policy, &replay)?
                    || library.compact(episode)? != report.panels[index].episodes[sequence]
                {
                    return Ok(false);
                }
            }
            if super::metrics::aggregate_episodes(&panel.episodes)? != panel.aggregate
                || project_panel(&panel)? != report.panels[index].projection
                || !reference.panel_matches(index, &panel)?
            {
                return Ok(false);
            }
        }
        stats.push(CompilationStats {
            protocol,
            prior,
            stats: replay.stats(),
        });
    }
    if library.histories != report.histories
        || stats != report.compilations
        || !reference.summaries_match(report)?
    {
        return Ok(false);
    }
    let summaries = compute_summaries(report)?;
    Ok(summaries.comparisons == report.comparisons
        && summaries.expectations == report.role_expectations
        && summaries.projections == report.role_projections
        && summaries.contrasts == report.role_contrasts)
}
/// Check the complete current payload. Cached passed/check flags are ignored.
/// Scientific inequalities fail validation; malformed representations and
/// source/protocol/resource errors preserve context rather than becoming outcomes.
pub fn report_integrity(report: &DiagnosticReport, limits: &Limits) -> Result<bool, Error> {
    let reference = references::Reference::read()?;
    payload_integrity(report, limits, &reference)
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SummarySetting {
    pub setting: u8,
    pub protocol: Protocol,
    pub environment: Environment,
    pub aggregate: Aggregate,
    pub projection: PanelProjection,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    pub version: String,
    pub report_hash: String,
    pub source_hash: String,
    pub reference_artifacts: Vec<ArtifactReceipt>,
    pub settings: Vec<SummarySetting>,
    pub comparisons: Vec<PairedComparison>,
    pub role_expectations: Vec<UniformMechanismSummary>,
    pub role_projections: Vec<PanelProjection>,
    pub role_contrasts: Vec<RoleContrast>,
    pub compilations: Vec<CompilationStats>,
}
/// Export only after fresh payload validation. The report hash covers exactly its
/// compact serde JSON bytes, without a trailing newline; Summary stays outside it.
pub fn summary_json(report: &DiagnosticReport) -> Result<String, Error> {
    if !report_integrity(report, &Limits::registered()?)? {
        return Err(invalid("cannot summarize an invalid diagnostic report"));
    }
    let summary = build_summary(report)?;
    serde_json::to_string(&summary).map_err(|e| invalid(format!("summary serialization: {e}")))
}
pub(super) fn build_summary(report: &DiagnosticReport) -> Result<Summary, Error> {
    Ok(Summary {
        version: "surface-discovery-summary-v1".into(),
        report_hash: content_hash(report)?,
        source_hash: report.source_hash.clone(),
        reference_artifacts: report.reference_artifacts.clone(),
        settings: report
            .panels
            .iter()
            .enumerate()
            .map(|(index, p)| SummarySetting {
                setting: index as u8,
                protocol: p.protocol.clone(),
                environment: p.environment,
                aggregate: p.aggregate.clone(),
                projection: p.projection.clone(),
            })
            .collect(),
        comparisons: report.comparisons.clone(),
        role_expectations: report.role_expectations.clone(),
        role_projections: report.role_projections.clone(),
        role_contrasts: report.role_contrasts.clone(),
        compilations: report.compilations.clone(),
    })
}
