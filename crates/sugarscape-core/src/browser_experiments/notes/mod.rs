//! One selected native paid-notes episode; no exhaustive collector or evaluation.
pub(crate) mod duplicate;
mod projection;
use super::{
    error, record, wire, EpisodeKind, EpisodeRecord, FieldError, Input, Semantics, StudyDescriptor,
    StudyFamily, StudyId, EPISODE_VERSION, MAX_EPISODE_BYTES, MAX_INPUT_BYTES,
};
use crate::persistent_notes as n;
use serde_json::{json, Value};
type Result<T> = std::result::Result<T, Vec<FieldError>>;
fn checked<T, E: std::fmt::Display>(value: std::result::Result<T, E>) -> Result<T> {
    value.map_err(|e| error("persistent_notes", e.to_string()))
}
fn config(input: &Input) -> Result<n::Config> {
    let Input::PersistentNotes {
        stratum,
        arm,
        routine,
        request,
        ids,
    } = input
    else {
        return Err(error("study", "expected persistent notes"));
    };
    Ok(n::Config {
        stratum: stratum.clone(),
        arm: *arm,
        routine: *routine,
        request: *request,
        ids: ids.clone(),
    })
}
pub(crate) fn validate_input(input: &Input) -> Result<()> {
    checked(config(input)?.validate())
}
pub(crate) fn descriptor() -> StudyDescriptor {
    let mut default_input = serde_json::to_value(n::Config::example(
        n::Arm::Retained,
        n::ReaderRoutine::ReadAdaptive,
        n::Good::Left,
    ))
    .expect("static config serializes");
    default_input["study"] = json!("persistent_notes");
    StudyDescriptor{id:StudyId::PersistentNotes,family:StudyFamily::Notes,title:"Paid persistent notes".into(),supplied:"Two goods, one hidden request, four Writer opportunities and at most six Reader opportunities.".into(),question:"When can paid persistent notes substitute for repeated inspection?".into(),default_input,controls:json!({"acquisition_cost":[1,4],"write_cost":[1,2,4,6],"note_read_cost":[1,2],"manifest_inspection_cost":[2,4,8],"writer_share":[0,3,6],"arms":["Retained","Erased","NoNote","NoAcquireNoNote"],"routines":["BlindGuess","InspectFirst","ReadAdaptive","Oracle"],"requests":["Left","Right"],"truth_label":"Researcher setup · hidden request; Oracle is the supplied-truth exception"})}
}
pub(crate) fn rules_identity() -> Result<String> {
    macro_rules! source {
        ($p:literal) => {
            (
                concat!("crates/sugarscape-core/src/", $p),
                include_bytes!(concat!("../../", $p)).as_slice(),
            )
        };
    }
    let adapter = checked(crate::key_delegation::compiled_bytes_identity(&[
        source!("browser_experiments/notes/mod.rs"),
        source!("browser_experiments/recorded.rs"),
        source!("browser_experiments/fixtures/notes-recorded.json"),
        source!("browser_experiments/notes/projection.rs"),
        source!("browser_experiments/notes/duplicate.rs"),
        source!("browser_experiments/mod.rs"),
        source!("browser_experiments/input.rs"),
        source!("browser_experiments/catalog.rs"),
        source!("browser_experiments/record.rs"),
        source!("browser_experiments/equivalence.rs"),
        source!("browser_experiments/wire.rs"),
        source!("browser_experiments/gate/duplicate.rs"),
        source!("browser_experiments/key/duplicate.rs"),
        source!("config.rs"),
        source!("lib.rs"),
    ]))?;
    Ok(format!("persistent-notes:protocol-1:browser-notes-1:native-source-sha256:{}:adapter-source-sha256:{adapter}:reviewed-reference-sha256:{}",checked(n::source_identity())?,checked(n::reference_identity())?))
}
pub(crate) fn run(input: &Input) -> Result<EpisodeRecord> {
    record::serialized_size(input, MAX_INPUT_BYTES, "input")?;
    let config = config(input)?;
    checked(config.validate())?;
    let mut result = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: StudyId::PersistentNotes,
        rules_identity: rules_identity()?,
        input: checked(serde_json::to_value(input))?,
        semantics: Semantics::Trajectory,
        checkpoints: vec![],
        payload: Value::Null,
    };
    let mut retained = record::serialized_size(&result, MAX_EPISODE_BYTES, "episode")?;
    let episode = checked(n::run_with_capture(&config, |point| {
        if result.checkpoints.len() >= 16 {
            return Err(n::Error::InvalidInput(
                "browser notes checkpoint cap".into(),
            ));
        }
        let temporary = record::serialized_size(point, MAX_EPISODE_BYTES / 4, "checkpoint")
            .map_err(|e| n::Error::InvalidInput(format!("browser size: {e:?}")))?;
        if retained
            .checked_add(temporary.saturating_mul(4))
            .is_none_or(|bytes| bytes > MAX_EPISODE_BYTES)
        {
            return Err(n::Error::InvalidInput("browser notes byte cap".into()));
        }
        let candidate = projection::checkpoint(point, &config.ids, result.checkpoints.len())
            .map_err(|e| n::Error::InvalidInput(format!("browser projection: {e:?}")))?;
        // Count the candidate before retaining it; native capture itself is bounded by sealed limits.
        let bytes = record::serialized_size(&candidate, MAX_EPISODE_BYTES, "checkpoint")
            .map_err(|e| n::Error::InvalidInput(format!("browser size: {e:?}")))?;
        retained = retained
            .checked_add(bytes + usize::from(!result.checkpoints.is_empty()))
            .ok_or_else(|| n::Error::InvalidInput("browser notes byte overflow".into()))?;
        if retained > MAX_EPISODE_BYTES {
            return Err(n::Error::InvalidInput("browser notes byte cap".into()));
        }
        result.checkpoints.push(candidate);
        Ok(())
    }))?;
    let bytes = record::serialized_size(&episode, MAX_EPISODE_BYTES / 4, "episode")?;
    if retained
        .checked_add(bytes.saturating_mul(4))
        .is_none_or(|sum| sum > MAX_EPISODE_BYTES)
    {
        return Err(error("episode", "notes payload byte cap"));
    }
    result.payload = wire::lossless_value(&episode)?;
    record::check_bounds(&result)?;
    Ok(result)
}
