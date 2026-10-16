//! One selected native Key episode, with a public-stratum compile per request.
macro_rules! identity_bytes {
    ($p:expr) => {{
        #[cfg(not(target_arch = "wasm32"))]
        {
            include_bytes!($p).as_slice()
        }
        #[cfg(target_arch = "wasm32")]
        {
            b"".as_slice()
        }
    }};
}
pub(crate) mod duplicate;
mod projection;
use super::{
    error, record, EpisodeKind, EpisodeRecord, FieldError, Input, Semantics, StudyDescriptor,
    StudyFamily, StudyId, EPISODE_VERSION, MAX_EPISODE_BYTES, MAX_INPUT_BYTES,
};
use crate::key_delegation as g;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::{json, Value};
type Result<T> = std::result::Result<T, Vec<FieldError>>;
fn checked<T, E: std::fmt::Display>(v: std::result::Result<T, E>) -> Result<T> {
    v.map_err(|e| error("key_delegation", e.to_string()))
}
/// Absent choice fields are None; a present null is never a selector/profile.
pub(crate) fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> std::result::Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}
pub(crate) fn validate_input(input: &Input) -> Result<()> {
    let Input::KeyDelegation {
        stratum,
        selector,
        profile,
        ids,
    } = input
    else {
        return Err(error("study", "expected key delegation"));
    };
    checked(stratum.validate())?;
    checked(ids.validate())?;
    if selector.is_some() == profile.is_some() {
        return Err(error(
            "key_delegation",
            "provide exactly one selector or raw profile",
        ));
    }
    if let Some(p) = profile {
        checked(p.validate())?;
    }
    Ok(())
}
pub(crate) fn descriptor() -> StudyDescriptor {
    StudyDescriptor {
        id: StudyId::KeyDelegation, family: StudyFamily::Key,
        title: "Paid key delegation".into(),
        supplied: "Two supplied roles and words, one carrier-only key, three cells, six rounds and 32 credits per Agent.".into(),
        question: "When is paid key transfer team-beneficial, and when is it privately stable?".into(),
        default_input: json!({"study":"key_delegation","stratum":{"transfer_enabled":true,"transfer_cost":1,"giver_share":3,"distance":0},"selector":"SuppliedHandoff","ids":{"giver":"Agent-A","receiver":"Agent-B"}}),
        controls: json!({"transfer_enabled":[false,true],"transfer_cost":[1,2,4],"giver_share":[0,3,6],"distance":[0,1],"selectors":["SoloAttempt","SuppliedHandoff","TeamOptimal","NashLex","NashBestGroup","InitiallyEquipped"],"default_label":"Supplied minimal handoff","initially_equipped_label":"Changed initial ownership; main-game classification unavailable","profile":{"giver":{"min":0,"max":728},"receiver":{"min":0,"max":63}}}),
    }
}
pub(crate) fn rules_identity() -> Result<String> {
    let native = checked(g::source_identity())?;
    macro_rules! source {
        ($path:literal) => {
            (
                concat!("crates/sugarscape-core/src/", $path),
                identity_bytes!(concat!("../../", $path)),
            )
        };
    }
    let adapter = checked(g::compiled_bytes_identity(&[
        source!("browser_experiments/key/mod.rs"),
        source!("browser_experiments/key/projection.rs"),
        source!("browser_experiments/key/duplicate.rs"),
        source!("browser_experiments/mod.rs"),
        source!("browser_experiments/input.rs"),
        source!("browser_experiments/catalog.rs"),
        source!("browser_experiments/record.rs"),
        source!("browser_experiments/equivalence.rs"),
        source!("browser_experiments/wire.rs"),
        source!("browser_experiments/recorded.rs"),
        source!("config.rs"),
        source!("lib.rs"),
        source!("browser_experiments/gate/duplicate.rs"),
        source!("browser_experiments/notes/duplicate.rs"),
        source!("browser_experiments/fixtures/key-registration-compatibility.json"),
    ]))?;
    let references = checked(g::reference_identity())?;
    Ok(format!("key-delegation:protocol-1:browser-key-1:native-source-sha256:{native}:adapter-source-sha256:{adapter}:reviewed-reference-sha256:{references}"))
}
fn selection(table: &g::GameTable, selector: g::Selector) -> Result<g::Selection> {
    checked(g::resolve(table, selector))?.ok_or_else(|| {
        error(
            "key_delegation",
            format!(
                "selector {selector:?} unavailable for {:?}: no pure equilibrium",
                table.stratum()
            ),
        )
    })
}

#[derive(Serialize)]
struct Payload<'a> {
    episode: &'a g::Episode,
    classification: Option<&'a g::Classification>,
    classification_scope: g::ClassificationScope,
    compiler_stats: &'a g::CompileStats,
    trace_hash: String,
}
pub(crate) fn run(input: &Input) -> Result<EpisodeRecord> {
    record::serialized_size(input, MAX_INPUT_BYTES, "input")?;
    validate_input(input)?;
    let Input::KeyDelegation {
        stratum,
        selector,
        profile,
        ids,
    } = input
    else {
        return Err(error("study", "expected key delegation"));
    };
    let mut result = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: StudyId::KeyDelegation,
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
    let table = checked(g::compile(
        stratum.clone(),
        &checked(g::Limits::registered())?,
    ))?;
    let selected = match (selector, profile) {
        (Some(s), None) => selection(&table, *s)?,
        (None, Some(p)) => g::Selection {
            initial_owner: g::InitialOwner::Giver,
            profile: *p,
            classification: Some(checked(g::classify(&table, *p))?),
        },
        _ => {
            return Err(error(
                "key_delegation",
                "provide exactly one selector or raw profile",
            ))
        }
    };
    let profile = selected.profile;
    let classification = selected.classification;
    let classification_scope = match selected.initial_owner {
        g::InitialOwner::Giver => g::ClassificationScope::Main,
        g::InitialOwner::Receiver => g::ClassificationScope::ChangedInitialControl,
    };
    let stats = table.stats().clone();
    drop(table);
    let config = checked(g::Config::new(
        stratum.clone(),
        ids.clone(),
        selected.initial_owner,
    ))?;
    let episode = checked(g::run_with_capture(&config, profile, &mut |point| {
        let checkpoint = projection::checkpoint(
            point,
            ids,
            profile,
            classification.as_ref(),
            classification_scope,
            result.checkpoints.len(),
            &retention,
        )?;
        retention.push(&mut result.checkpoints, checkpoint)
    }))?;
    let payload = Payload {
        episode: &episode,
        classification: classification.as_ref(),
        classification_scope,
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
    fn ordered_raw_receipt_codec_matches_independent_known_answer() {
        let inputs: &[(&str, &[u8])] = &[("empty", b""), ("unicode", "雪\n".as_bytes())];
        assert_eq!(
            g::compiled_bytes_identity(inputs).unwrap(),
            "21f608974747bcddbe465b716260a2392aee7a51b12ed892477fdfc0dae32e80"
        );
        assert!(g::compiled_bytes_identity(&[("same", b"a"), ("same", b"a")]).is_err());
    }
}
