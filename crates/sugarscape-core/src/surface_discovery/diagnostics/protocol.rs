use super::*;
pub const DIAGNOSTIC_VERSION: &str = "surface-discovery-diagnostic-v1";
pub const PROTOCOL_VERSION: u16 = 1;
/// Exact registered census, including the separate out-of-catalog transfer panel.
pub fn registered_settings() -> Vec<(Protocol, Environment)> {
    let mut settings = Vec::with_capacity(48);
    for model in Mechanism::ALL {
        for role in [Role::A, Role::B] {
            for policy in [
                PolicyKind::RawAdaptive,
                PolicyKind::SuppliedAdaptive,
                PolicyKind::NoProbe,
                PolicyKind::KnownRaw,
                PolicyKind::InspectOnly,
            ] {
                settings.push((Protocol::new(role, policy), Environment::InFamily(model)));
            }
        }
    }
    for role in [Role::A, Role::B] {
        for policy in [
            PolicyKind::RawAdaptive,
            PolicyKind::SuppliedAdaptive,
            PolicyKind::NoProbe,
            PolicyKind::InspectOnly,
        ] {
            settings.push((Protocol::new(role, policy), Environment::DataFlip));
        }
    }
    settings
}
pub(super) fn supplied_structure() -> Vec<String> {
    [
 "Four supplied catalog mechanisms; DataFlip is outside the catalog and has a nominal SharedPersistent helper.",
 "Seven supplied symbols and opaque IDs; complete own histories and public clock; no peer history or evaluator lineage enters an Agent View.",
 "48 credits per Agent; read/write/wait cost 1; trusted inspection costs 4; correct prediction reward 12.",
 "A/B/B/A paid ownership; up to three calibration rounds and four independent fair-bit pairs; owned raw actions; public free Stop.",
 "Exact expected remaining investigator net task utility; frozen action order and Stop/Inspect tie rules; private selection.",
 "Uniform investigator prior except matching point-informed KnownRaw controls; helper uses its supplied nominal point prior and declared-policy replay.",
 "40 in-family and eight transfer settings; 256 distinct ordered original sequences per setting; all unavailable and partial-cost mass retained.",
 "Nominal catalog certainty distinguishes supplied and observed evidence; DataFlip true-model identification is unavailable.",
 ].map(str::to_owned).to_vec()
}
pub(super) fn own_prior(protocol: &Protocol, environment: Environment) -> Result<OwnPrior, Error> {
    match (protocol.policy, environment) {
        (PolicyKind::KnownRaw, Environment::InFamily(model)) => Ok(OwnPrior::PointMass(model)),
        (PolicyKind::KnownRaw, Environment::DataFlip) => {
            Err(invalid("KnownRaw transfer is unregistered"))
        }
        _ => Ok(OwnPrior::Uniform),
    }
}
pub(super) fn compilations(
    settings: &[(Protocol, Environment)],
) -> Result<Vec<(Protocol, OwnPrior)>, Error> {
    let mut keys = Vec::new();
    for (protocol, environment) in settings {
        let key = (protocol.clone(), own_prior(protocol, *environment)?);
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    Ok(keys)
}
/// Fixed source path order and SHA-256 of the exact compiled dependency bytes.
pub(super) fn sources() -> Vec<ArtifactReceipt> {
    macro_rules! source {
        ($path:literal) => {{
            let path = concat!("crates/sugarscape-core/src/", $path);
            let raw = crate::identity_bytes::identity_bytes!(concat!("../../", $path));
            let (bytes, sha256) =
                crate::compact_receipts::raw(path, raw).expect("compiled source receipt");
            ArtifactReceipt {
                path: path.into(),
                bytes,
                sha256,
            }
        }};
    }
    let mut sources = vec![
        source!("identity_bytes.rs"),
        source!("compact_receipts.rs"),
        source!("gate_assistance/source_inputs.rs"),
        source!("surface_discovery/mod.rs"),
        source!("surface_discovery/types.rs"),
        source!("surface_discovery/kernel.rs"),
        source!("surface_discovery/budget.rs"),
        source!("surface_discovery/belief.rs"),
        source!("surface_discovery/planner.rs"),
        source!("surface_discovery/replay.rs"),
        source!("surface_discovery/evaluation.rs"),
        source!("surface_discovery/metrics.rs"),
        source!("surface_discovery/diagnostics.rs"),
        source!("surface_discovery/diagnostics/protocol.rs"),
        source!("surface_discovery/diagnostics/references.rs"),
        source!("surface_discovery/diagnostics/hash.rs"),
        source!("surface_discovery/tests/fixtures/planning-reference.json"),
        source!("surface_discovery/tests/fixtures/evaluation-reference.json"),
        source!("surface_discovery/tests/fixtures/limits-reference.json"),
        source!("shared_surface/mod.rs"),
        source!("shared_surface/types.rs"),
        source!("shared_surface/protocol.rs"),
        source!("shared_surface/controller.rs"),
        source!("shared_surface/world.rs"),
    ];
    sources.extend(
        crate::gate_assistance::SOURCE_INPUTS
            .iter()
            .filter(|(p, _)| p.contains("/build"))
            .map(|(p, b)| {
                let (bytes, sha256) =
                    crate::compact_receipts::raw(p, b).expect("compiled build receipt");
                ArtifactReceipt {
                    path: (*p).into(),
                    bytes,
                    sha256,
                }
            }),
    );
    sources
}
