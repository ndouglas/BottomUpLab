//! One selected native Gate episode, with a public-stratum compile per request.
use crate::identity_bytes::identity_bytes;
pub(crate) mod duplicate;
mod projection;
use super::{
    error, record, EpisodeKind, EpisodeRecord, FieldError, Input, Semantics, StudyDescriptor,
    StudyFamily, StudyId, EPISODE_VERSION, MAX_EPISODE_BYTES, MAX_INPUT_BYTES,
};
use crate::gate_assistance as g;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
type Result<T> = std::result::Result<T, Vec<FieldError>>;
fn checked<T, E: std::fmt::Display>(v: std::result::Result<T, E>) -> Result<T> {
    v.map_err(|e| error("gate_assistance", e.to_string()))
}
/// Absent choice fields are None; a present null is never a selector/profile.
pub(crate) fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
pub(crate) fn validate_input(input: &Input) -> Result<()> {
    let Input::GateAssistance {
        stratum,
        selector,
        profile,
        ids,
    } = input
    else {
        return Err(error("study", "expected gate assistance"));
    };
    checked(stratum.validate())?;
    checked(ids.validate())?;
    if selector.is_some() == profile.is_some() {
        return Err(error(
            "gate_assistance",
            "provide exactly one selector or raw profile",
        ));
    }
    if let Some(p) = profile {
        checked(p.validate())?;
    }
    Ok(())
}
pub(crate) fn descriptor() -> StudyDescriptor {
    StudyDescriptor {id:StudyId::GateAssistance,family:StudyFamily::Gate,title:"Paid gate assistance".into(),supplied:"Two supplied roles and schedules, a three-cell graph, six rounds and 32 credits per Agent.".into(),question:"When is paid assistance team-beneficial, and when is it privately stable?".into(),default_input:json!({"study":"gate_assistance","stratum":{"holding_cost":1,"holder_share":3,"distance":0},"selector":"SuppliedHelp","ids":{"holder":"Agent-A","retriever":"Agent-B"}}),controls:json!({"holding_cost":[1,2,4],"holder_share":[0,3,6],"distance":[0,1],"selectors":["SoloAttempt","SuppliedHelp","TeamOptimal","NashLex","NashBestGroup"],"default_label":"Supplied minimal assistance","profile":{"holder":{"min":0,"max":728},"retriever":{"min":0,"max":63}}})}
}
pub(crate) fn rules_identity() -> Result<String> {
    let native = checked(crate::surface_discovery::compiled_file_identity(
        g::SOURCE_INPUTS,
    ))?;
    macro_rules! source {
        ($path:literal) => {
            (
                concat!("crates/sugarscape-core/src/", $path),
                identity_bytes!(concat!("../../", $path)),
            )
        };
    }
    let adapter = checked(crate::surface_discovery::compiled_file_identity(&[
        source!("browser_experiments/gate/mod.rs"),
        source!("browser_experiments/gate/projection.rs"),
        source!("browser_experiments/gate/duplicate.rs"),
        source!("browser_experiments/mod.rs"),
        source!("browser_experiments/input.rs"),
        source!("browser_experiments/catalog.rs"),
        source!("browser_experiments/record.rs"),
        source!("browser_experiments/equivalence.rs"),
        source!("browser_experiments/wire.rs"),
        source!("config.rs"),
        source!("lib.rs"),
    ]))?;
    let references = checked(crate::surface_discovery::compiled_file_identity(
        g::REFERENCE_INPUTS,
    ))?;
    Ok(format!("gate-assistance:protocol-1:browser-gate-1:native-source-sha256:{native}:adapter-source-sha256:{adapter}:reviewed-reference-sha256:{references}"))
}
fn selected_profile(table: &g::GameTable, selector: g::Selector) -> Result<g::Profile> {
    g::resolve(table, selector).ok_or_else(|| {
        error(
            "gate_assistance",
            format!(
                "selector {selector:?} unavailable for {:?}: no pure equilibrium",
                table.stratum
            ),
        )
    })
}

#[derive(Serialize)]
struct Payload<'a> {
    episode: &'a g::Episode,
    classification: &'a g::Classification,
    compiler_stats: &'a g::CompileStats,
    trace_hash: String,
}
pub(crate) fn run(input: &Input) -> Result<EpisodeRecord> {
    record::serialized_size(input, MAX_INPUT_BYTES, "input")?;
    validate_input(input)?;
    let Input::GateAssistance {
        stratum,
        selector,
        profile,
        ids,
    } = input
    else {
        return Err(error("study", "expected gate assistance"));
    };
    let mut result = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: StudyId::GateAssistance,
        rules_identity: rules_identity()?,
        input: checked(serde_json::to_value(input))?,
        semantics: Semantics::Trajectory,
        checkpoints: vec![],
        payload: Value::Null,
    };
    let mut retention = projection::Retention {
        bytes: record::serialized_size(&result, MAX_EPISODE_BYTES, "episode")?,
        limit: MAX_EPISODE_BYTES,
    };
    // Names never enter compile or its payoff-table key. No cross-request cache.
    let table = checked(g::compile(stratum, &checked(g::Limits::registered())?))?;
    let profile = match (selector, profile) {
        (Some(s), None) => selected_profile(&table, *s)?,
        (None, Some(p)) => *p,
        _ => {
            return Err(error(
                "gate_assistance",
                "provide exactly one selector or raw profile",
            ))
        }
    };
    let classification = checked(g::classify(&table, profile))?;
    let stats = table.stats.clone();
    drop(table);
    let config = g::Config {
        stratum: stratum.clone(),
        ids: ids.clone(),
    };
    let episode = checked(g::run_with_capture(&config, profile, &mut |point| {
        let checkpoint = projection::checkpoint(
            point,
            ids,
            profile,
            &classification,
            result.checkpoints.len(),
            &retention,
        )?;
        retention.push(&mut result.checkpoints, checkpoint)
    }))?;
    let payload = Payload {
        episode: &episode,
        classification: &classification,
        compiler_stats: &stats,
        trace_hash: checked(g::trace_hash(&episode))?,
    };
    let value = checked(retention.value(&payload))?;
    let payload_bytes = checked(retention.size(&value))?;
    checked(retention.reserve(payload_bytes.saturating_sub(4)))?; // replace null
    result.payload = value;
    record::check_bounds(&result)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_unavailable_nash_selectors_report_selector_and_stratum() {
        // Synthetic resolver seam only: no registered stratum is claimed to lack a pure profile.
        let table = g::GameTable {
            stratum: g::Stratum::new(1, 3, 0).unwrap(),
            rows: vec![],
            analysis: g::Analysis {
                holder_best: vec![],
                retriever_best: vec![],
                pure: vec![],
                team: g::Profile::new(0, 0).unwrap(),
                lex_pure: None,
                best_group_pure: None,
                solo_best: 0,
            },
            stats: g::CompileStats {
                profiles: 0,
                transitions: 0,
                peak_bytes: 0,
            },
        };
        for selector in [g::Selector::NashLex, g::Selector::NashBestGroup] {
            assert_eq!(
                selected_profile(&table, selector).unwrap_err(),
                vec![FieldError::new(
                    "gate_assistance",
                    format!(
                        "selector {selector:?} unavailable for {:?}: no pure equilibrium",
                        table.stratum
                    ),
                )],
            );
        }
    }
}
