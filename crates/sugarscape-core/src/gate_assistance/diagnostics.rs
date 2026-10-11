//! Fresh exhaustive integrity, separate from operational errors and saved flags.
use super::{
    canonical::canonical_bytes,
    game::SELECTORS,
    protocol::CHECK_NAMES,
    references::{decode, References},
    *,
};
use serde::Serialize;
mod generated {
    include!(concat!(env!("OUT_DIR"), "/gate_source_receipts.rs"));
}
pub(super) fn identity(inputs: &[(&str, &[u8])]) -> Result<String, Error> {
    crate::surface_discovery::compiled_bytes_identity(inputs)
        .map_err(|e| Error::InvalidReport(format!("Gate identity: {e}")))
}
fn json<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    serde_json::to_vec(value).map_err(|e| Error::InvalidReport(format!("Gate serialization: {e}")))
}
fn receipts(
    inputs: &[(&str, &[u8])],
    metadata: &[(&str, u64, &str, &str)],
    outer: &str,
    metadata_identity: &str,
) -> Result<Vec<Receipt>, Error> {
    if inputs.len() != metadata.len() {
        return Err(Error::InvalidReport("generated receipt count".into()));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut result = Vec::new();
    for (&(path, bytes), &(label, len, sha, one)) in inputs.iter().zip(metadata) {
        if path != label
            || !seen.insert(path)
            || u64::try_from(bytes.len()).map_err(|_| Error::ArithmeticOverflow)? != len
            || !hash_valid(sha)
            || identity(&[(path, bytes)])? != one
        {
            return Err(Error::InvalidReport(format!(
                "generated receipt mismatch {path}"
            )));
        }
        result.push(Receipt {
            path: path.into(),
            bytes: len,
            sha256: sha.into(),
        });
    }
    if identity(inputs)? != outer
        || identity(&[("gate-receipt-metadata-v1", &json(&result)?)])? != metadata_identity
    {
        return Err(Error::InvalidReport(
            "generated receipt aggregate mismatch".into(),
        ));
    }
    Ok(result)
}
pub(super) fn empty_report(limits: &Limits) -> Result<Report, Error> {
    Ok(Report {
        version: "gate-assistance-diagnostic-v1".into(),
        source_hash: identity(SOURCE_INPUTS)?,
        source_receipts: receipts(
            SOURCE_INPUTS,
            generated::SOURCE_RECEIPTS,
            generated::SOURCE_IDENTITY,
            generated::SOURCE_METADATA_IDENTITY,
        )?,
        reference_receipts: receipts(
            REFERENCE_INPUTS,
            generated::REFERENCE_RECEIPTS,
            generated::REFERENCE_IDENTITY,
            generated::REFERENCE_METADATA_IDENTITY,
        )?,
        limits: limits.clone(),
        panels: vec![],
        contrasts: vec![],
        checks: vec![],
        passed: false,
    })
}
pub(super) fn validate_identity(report: &Report) -> Result<(), Error> {
    let current = empty_report(&report.limits)?;
    if report.source_hash != current.source_hash
        || report.source_receipts != current.source_receipts
        || report.reference_receipts != current.reference_receipts
    {
        return Err(Error::InvalidReport(
            "stale or forged source/reference identity".into(),
        ));
    }
    Ok(())
}
fn hash_valid(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub(super) fn controls(table: &GameTable) -> Result<Vec<ControlRecord>, Error> {
    SELECTORS
        .into_iter()
        .map(|selector| {
            let profile = resolve(table, selector);
            Ok(ControlRecord {
                selector,
                profile,
                episode: profile
                    .map(|p| {
                        run_episode(
                            &Config {
                                stratum: table.stratum.clone(),
                                ids: Ids::default(),
                            },
                            p,
                        )
                    })
                    .transpose()?,
                classification: profile.map(|p| classify(table, p)).transpose()?,
            })
        })
        .collect()
}
fn panel(table: GameTable) -> Result<Panel, Error> {
    let controls = controls(&table)?;
    Ok(Panel {
        stratum: table.stratum,
        rows: table.rows,
        analysis: table.analysis,
        controls,
        stats: table.stats,
    })
}
fn table(panel: &Panel) -> GameTable {
    GameTable {
        stratum: panel.stratum.clone(),
        rows: panel.rows.clone(),
        analysis: panel.analysis.clone(),
        stats: panel.stats.clone(),
    }
}
pub(super) fn shape(report: &Report) -> Result<bool, Error> {
    if report.checks.len() != 10
        || report
            .checks
            .iter()
            .zip(CHECK_NAMES)
            .any(|(check, name)| check.name != name)
        || report.version != "gate-assistance-diagnostic-v1"
        || report.panels.len() != 18
    {
        return Ok(false);
    }
    for (p, s) in report.panels.iter().zip(registered_strata()) {
        if p.stratum != s
            || p.rows.len() != 46656
            || p.analysis.holder_best.len() != 64
            || p.analysis.retriever_best.len() != 729
            || p.controls.len() != 5
        {
            return Ok(false);
        }
        let mut previous = None;
        for profile in &p.analysis.pure {
            profile.validate()?;
            if previous.is_some_and(|old| old >= *profile) {
                return Ok(false);
            }
            previous = Some(*profile);
        }
        p.analysis.team.validate()?;
        for profile in [p.analysis.lex_pure, p.analysis.best_group_pure]
            .into_iter()
            .flatten()
        {
            profile.validate()?;
        }
        for row in &p.rows {
            row.outcome.validate()?;
            if !hash_valid(&row.trace_hash) {
                return Err(Error::InvalidReport("malformed row trace hash".into()));
            }
        }
        for (c, selector) in p.controls.iter().zip(SELECTORS) {
            if c.selector != selector {
                return Ok(false);
            }
            match (&c.profile, &c.episode, &c.classification) {
                (Some(profile), Some(episode), Some(classification)) => {
                    profile.validate()?;
                    episode.validate()?;
                    classification.outcome.validate()?;
                    if episode.profile != *profile
                        || episode.config.stratum != p.stratum
                        || episode.config.ids != Ids::default()
                        || episode.outcome != classification.outcome
                    {
                        return Ok(false);
                    }
                }
                (None, None, None)
                    if matches!(selector, Selector::NashLex | Selector::NashBestGroup)
                        && p.analysis.pure.is_empty() => {}
                _ => return Ok(false),
            }
        }
    }
    Ok(true)
}
fn checks(flags: [bool; 10], details: [String; 10]) -> Vec<Check> {
    CHECK_NAMES
        .into_iter()
        .zip(flags)
        .zip(details)
        .map(|((name, passed), detail)| Check {
            name: name.into(),
            passed,
            detail,
        })
        .collect()
}
fn details() -> [String; 10] {
    std::array::from_fn(|i| {
        format!(
            "{}: 18 ordered strata, 46656 raw pairs each (839808 total), 5 ordered controls each",
            CHECK_NAMES[i]
        )
    })
}
fn failure_checks() -> Vec<Check> {
    checks(
        [false; 10],
        std::array::from_fn(|_| {
            "incomplete or malformed registered census; no exhaustive acceptance".into()
        }),
    )
}
// One seam shared by fresh production and retained-payload verification.
pub(super) fn compare_panel(retained: &Panel, current: &GameTable) -> Result<[bool; 6], Error> {
    Ok([
        retained.rows == current.rows,
        retained.analysis.holder_best == current.analysis.holder_best
            && retained.analysis.retriever_best == current.analysis.retriever_best,
        retained.analysis.pure == current.analysis.pure,
        retained.analysis.team == current.analysis.team
            && retained.analysis.lex_pure == current.analysis.lex_pure
            && retained.analysis.best_group_pure == current.analysis.best_group_pure
            && retained.analysis.solo_best == current.analysis.solo_best,
        retained.controls == controls(current)?,
        retained.stats == current.stats,
    ])
}
fn compare_references(
    index: usize,
    p: &Panel,
    current: &GameTable,
    refs: &References,
    flags: &mut [bool; 10],
) -> Result<(), Error> {
    let g = &refs.game["panels"][index];
    let expected: Analysis = decode(&g["analysis"], "analysis")?;
    let equality = compare_panel(p, current)?;
    flags[1] &= equality[0];
    flags[2] &= equality[0];
    flags[3] &= equality[4];
    flags[4] &= equality[1];
    flags[5] &= equality[2];
    flags[6] &= equality[3];
    let rows = refs.row_identity(index, &current.rows)?;
    flags[1] &= rows;
    flags[2] &= rows;
    flags[4] &= current.analysis.holder_best == expected.holder_best
        && current.analysis.retriever_best == expected.retriever_best;
    flags[5] &= current.analysis.pure == expected.pure;
    flags[6] &= current.analysis.team == expected.team
        && current.analysis.lex_pure == expected.lex_pure
        && current.analysis.best_group_pure == expected.best_group_pure
        && current.analysis.solo_best == expected.solo_best;
    flags[6] &= equilibrium_summary(current)?
        == decode::<EquilibriumSummary>(&g["equilibrium"], "equilibrium")?;
    let expected_controls = g["controls"]
        .as_array()
        .ok_or_else(|| Error::InvalidReport("reference controls".into()))?;
    if expected_controls.len() != 5 {
        return Err(Error::InvalidReport("reference control count".into()));
    }
    for (c, e) in controls(current)?.iter().zip(expected_controls) {
        flags[3] &= c.selector == decode::<Selector>(&e["selector"], "control selector")?
            && c.profile == decode::<Option<Profile>>(&e["profile"], "control profile")?
            && c.classification
                == decode::<Option<Classification>>(
                    &e["classification"],
                    "control classification",
                )?;
        let hash = c.episode.as_ref().map(trace_hash).transpose()?;
        flags[3] &= hash == decode::<Option<String>>(&e["trace_hash"], "control trace hash")?;
    }
    flags[8] &= equality[5]
        && current.stats
            == decode::<CompileStats>(&refs.limits["panels"][index]["stats"], "stats")?;
    Ok(())
}
fn reward_invariance(panels: &[Panel]) -> Result<bool, Error> {
    for lower in panels {
        for upper in panels {
            if lower.stratum.holding_cost() != upper.stratum.holding_cost()
                || lower.stratum.distance() != upper.stratum.distance()
                || lower.stratum.holder_share() >= upper.stratum.holder_share()
            {
                continue;
            }
            let delta =
                i16::from(upper.stratum.holder_share()) - i16::from(lower.stratum.holder_share());
            for (l, u) in lower.rows.iter().zip(&upper.rows) {
                let mut expected = l.outcome.clone();
                if expected.delivery_round.is_some() {
                    expected.reward = [
                        upper.stratum.holder_share(),
                        12 - upper.stratum.holder_share(),
                    ];
                    expected.net = [
                        expected.net[0]
                            .checked_add(delta)
                            .ok_or(Error::ArithmeticOverflow)?,
                        expected.net[1]
                            .checked_sub(delta)
                            .ok_or(Error::ArithmeticOverflow)?,
                    ];
                }
                if expected != u.outcome {
                    return Ok(false);
                }
            }
        }
    }
    Ok(true)
}
fn finish_checks(
    report: &Report,
    flags: &mut [bool; 10],
    settings: &[SummaryStratum],
    refs: &References,
) -> Result<Vec<Check>, Error> {
    let (outcomes, hashes) = refs.witnesses()?;
    flags[1] &= outcomes;
    flags[2] &= hashes;
    flags[7] &= reward_invariance(&report.panels)?;
    flags[6] &= report.contrasts == contrasts(settings)?;
    Ok(checks(*flags, details()))
}
/// Compile under supplied per-stratum bounds; caps above registration cannot redefine the experiment.
pub fn diagnose(limits: &Limits) -> Result<Report, Error> {
    let registered = Limits::registered()?;
    if limits.profiles > registered.profiles
        || limits.transitions > registered.transitions
        || limits.allocation_bytes > registered.allocation_bytes
    {
        return Err(Error::InvalidInput("limits exceed registration".into()));
    }
    let mut report = empty_report(&registered)?;
    let refs = References::load()?;
    let mut flags = [true; 10];
    let mut settings = Vec::new();
    for (i, s) in registered_strata().iter().enumerate() {
        let current = compile(s, limits)?;
        settings.push(summarize_stratum(&current)?);
        let p = panel(current.clone())?;
        compare_references(i, &p, &current, &refs, &mut flags)?;
        report.panels.push(p);
    }
    report.contrasts = contrasts(&settings)?;
    report.checks = finish_checks(&report, &mut flags, &settings, &refs)?;
    report.passed = report.checks.iter().all(|c| c.passed);
    Ok(report)
}
fn recompute_checks(report: &Report, limits: &Limits) -> Result<Vec<Check>, Error> {
    validate_identity(report)?;
    let registered = Limits::registered()?;
    if report.limits != registered
        || limits.profiles > registered.profiles
        || limits.transitions > registered.transitions
        || limits.allocation_bytes > registered.allocation_bytes
    {
        return Err(Error::InvalidReport(
            "report limits differ from registration".into(),
        ));
    }
    if !shape(report)? {
        return Ok(failure_checks());
    }
    let refs = References::load()?;
    let mut flags = [true; 10];
    let mut settings = Vec::new();
    for (i, p) in report.panels.iter().enumerate() {
        let current = compile(&p.stratum, limits)?;
        compare_references(i, p, &current, &refs, &mut flags)?;
        settings.push(summarize_stratum(&current)?);
    }
    finish_checks(report, &mut flags, &settings, &refs)
}
pub fn report_integrity(report: &Report, limits: &Limits) -> Result<bool, Error> {
    let checks = recompute_checks(report, limits)?;
    let passed = checks.iter().all(|c| c.passed);
    Ok(passed && report.passed == passed && report.checks == checks)
}
pub fn reconstruct_episode(
    report: &Report,
    stratum_index: usize,
    p: Profile,
) -> Result<Episode, Error> {
    validate_identity(report)?;
    p.validate()?;
    let panel = report
        .panels
        .get(stratum_index)
        .ok_or_else(|| Error::InvalidReport("stratum index absent".into()))?;
    if registered_strata().get(stratum_index) != Some(&panel.stratum) || panel.rows.len() != 46656 {
        return Err(Error::InvalidReport("reconstruction census shape".into()));
    }
    let current = run_episode(
        &Config {
            stratum: panel.stratum.clone(),
            ids: Ids::default(),
        },
        p,
    )?;
    let row = &panel.rows[usize::from(p.holder) * 64 + usize::from(p.retriever)];
    if row.outcome != current.outcome || row.trace_hash != trace_hash(&current)? {
        return Err(Error::InvalidReport(
            "reconstruction raw row mismatch".into(),
        ));
    }
    Ok(current)
}
pub fn summary_json(report: &Report) -> Result<String, Error> {
    if !report_integrity(report, &Limits::registered()?)? {
        return Err(Error::InvalidReport(
            "summary requires freshly validated report".into(),
        ));
    }
    let settings = report
        .panels
        .iter()
        .map(|p| summarize_stratum(&table(p)))
        .collect::<Result<Vec<_>, _>>()?;
    let report_hash = identity(&[("gate-assistance-report-v1", &canonical_bytes(report)?)])?;
    let summary = Summary {
        version: "gate-assistance-summary-v1".into(),
        report_hash,
        source_hash: report.source_hash.clone(),
        settings,
        contrasts: report.contrasts.clone(),
        limits: report.limits.clone(),
        source_receipts: report.source_receipts.clone(),
        reference_receipts: report.reference_receipts.clone(),
    };
    serde_json::to_string(&summary)
        .map_err(|e| Error::InvalidReport(format!("summary serialization: {e}")))
}
