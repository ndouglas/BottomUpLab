//! Bounded single episodes from the native surface-discovery controller.
use crate::identity_bytes::identity_bytes;
mod projection;
use super::{
    error, record, wire::lossless_value, EpisodeKind, EpisodeRecord, FieldError, Input, Semantics,
    StudyDescriptor, StudyFamily, StudyId, EPISODE_VERSION,
};
use crate::surface_discovery as d;
use serde_json::json;
type Result<T> = std::result::Result<T, Vec<FieldError>>;
fn checked<T, E: std::fmt::Display>(value: std::result::Result<T, E>) -> Result<T> {
    value.map_err(|e| error("surface_discovery", e.to_string()))
}
fn prior(protocol: &d::Protocol, environment: d::Environment) -> d::OwnPrior {
    match (protocol.policy, environment) {
        (d::PolicyKind::KnownRaw, d::Environment::InFamily(model)) => d::OwnPrior::PointMass(model),
        _ => d::OwnPrior::Uniform,
    }
}
pub fn validate_input(input: &Input) -> Result<()> {
    let Input::SurfaceDiscovery {
        protocol,
        environment,
        sequence,
    } = input
    else {
        return Err(error("study", "expected surface discovery"));
    };
    checked(protocol.validate())?;
    checked(d::EpisodeBits::from_index(*sequence))?;
    if protocol.policy == d::PolicyKind::KnownRaw && *environment == d::Environment::DataFlip {
        return Err(error(
            "protocol",
            "KnownRaw requires a matching in-family mechanism",
        ));
    }
    Ok(())
}
pub fn descriptor() -> StudyDescriptor {
    let settings: Vec<_> = d::registered_settings()
        .into_iter()
        .map(|(protocol, environment)| json!({"protocol":protocol,"environment":environment}))
        .collect();
    StudyDescriptor { id:StudyId::SurfaceDiscovery,family:StudyFamily::Surface,title:"Surface experiment discovery".into(),supplied:"Four supplied catalog mechanisms, a fixed informed helper, seven supplied symbols, exact priors, and paid physical operations.".into(),question:"Does selecting owned calibration actions improve downstream investigator utility relative to purchasing the supplied recipe?".into(),default_input:json!({"study":"surface_discovery","protocol":d::Protocol::new(d::Role::A,d::PolicyKind::RawAdaptive),"environment":d::Environment::InFamily(d::Mechanism::SharedPersistent),"sequence":0}),controls:json!({"settings":settings,"sequence":{"min":0,"max":255},"ids":"Opaque identifiers may be consistently renamed"}) }
}
pub fn rules_identity() -> Result<String> {
    let (_, native) = checked(d::compiled_source_identity())?;
    macro_rules! source {
        ($path:literal) => {
            (
                concat!("crates/sugarscape-core/src/", $path),
                identity_bytes!(concat!("../../", $path)),
            )
        };
    }
    let adapter = checked(d::compiled_file_identity(&[
        source!("browser_experiments/discovery/mod.rs"),
        source!("browser_experiments/discovery/projection.rs"),
        source!("browser_experiments/mod.rs"),
        source!("browser_experiments/input.rs"),
        source!("browser_experiments/catalog.rs"),
        source!("browser_experiments/record.rs"),
        source!("browser_experiments/wire.rs"),
        source!("browser_experiments/equivalence.rs"),
    ]))?;
    let references = checked(d::compiled_file_identity(&[
        source!("surface_discovery/tests/fixtures/planning-reference.json"),
        source!("surface_discovery/tests/fixtures/evaluation-reference.json"),
        source!("surface_discovery/tests/fixtures/limits-reference.json"),
    ]))?;
    Ok(format!("surface-discovery:protocol-1:browser-discovery-1:native-source-sha256:{native}:adapter-source-sha256:{adapter}:reviewed-reference-sha256:{references}"))
}
pub fn run(input: &Input) -> Result<EpisodeRecord> {
    let limits = checked(d::Limits::registered())?;
    run_with_limits(input, &limits)
}
fn run_with_limits(input: &Input, limits: &d::Limits) -> Result<EpisodeRecord> {
    record::serialized_size(input, super::MAX_INPUT_BYTES, "input")?;
    validate_input(input)?;
    let Input::SurfaceDiscovery {
        protocol,
        environment,
        sequence,
    } = input
    else {
        return Err(error("study", "expected surface discovery"));
    };
    let policy = checked(d::CompiledPolicy::build(
        protocol,
        prior(protocol, *environment),
        limits,
    ))?;
    let replay = checked(d::Replay::build(protocol, &policy, limits))?;
    let episode = checked(d::run_episode(
        protocol,
        *environment,
        *sequence,
        &policy,
        &replay,
    ))?;
    // Count before constructing the exact wire payload. Integer quotes can
    // increase native JSON by at most a factor of three. The native horizon
    // and input bound limit this single component independently of retention.
    record::serialized_size(&episode, super::MAX_EPISODE_BYTES / 3, "payload")?;
    let mut result = EpisodeRecord {
        kind: EpisodeKind::ExperimentEpisode,
        version: EPISODE_VERSION,
        study: StudyId::SurfaceDiscovery,
        rules_identity: rules_identity()?,
        input: checked(serde_json::to_value(input))?,
        semantics: Semantics::Trajectory,
        checkpoints: Vec::new(),
        payload: lossless_value(&episode)?,
    };
    let initial_bytes = record::serialized_size(&result, super::MAX_EPISODE_BYTES, "episode")?;
    result.checkpoints = projection::project(
        protocol,
        *environment,
        &policy,
        &replay,
        &episode,
        initial_bytes,
    )?;
    record::check_bounds(&result)?;
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn discovery_early_native_resource_rejection_propagates_without_episode() {
        let input = super::super::normalize_input(&descriptor().default_input.to_string()).unwrap();
        let limits = d::Limits {
            states: 1,
            branches: 1,
            allocation_bytes: 1,
        };
        let errors = run_with_limits(&input, &limits).unwrap_err();
        assert_eq!(errors[0].field, "surface_discovery");
        assert!(errors[0]
            .message
            .contains("allocation_bytes limit 1 exhausted"));
    }
}
