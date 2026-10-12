//! Versioned display records. Reproduction, not an imported success flag, is authority.
use super::{
    catalog, normalize_input, run, FieldError, StudyId, MAX_CHECKPOINTS, MAX_EPISODE_BYTES,
    MAX_INPUT_BYTES,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    io::{self, Write},
};

pub const EPISODE_VERSION: u16 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeKind {
    ExperimentEpisode,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Semantics {
    Trajectory,
    ConditionalCase,
    EvidenceSequence,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub index: u32,
    pub clock: Value,
    pub kind: String,
    pub public: Value,
    pub local: BTreeMap<String, Value>,
    pub researcher: Option<Value>,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeRecord {
    pub kind: EpisodeKind,
    pub version: u16,
    pub study: StudyId,
    pub rules_identity: String,
    pub input: Value,
    pub semantics: Semantics,
    pub checkpoints: Vec<Checkpoint>,
    pub payload: Value,
}

/// Count compact JSON without allocating an unbounded intermediate string.
struct CountingWriter {
    count: usize,
    limit: usize,
}
impl Write for CountingWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.limit - self.count {
            return Err(io::Error::other("serialized byte limit exceeded"));
        }
        self.count += bytes.len();
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub(crate) fn serialized_size<T: Serialize>(
    value: &T,
    limit: usize,
    field: &str,
) -> Result<usize, Vec<FieldError>> {
    let mut writer = CountingWriter { count: 0, limit };
    serde_json::to_writer(&mut writer, value).map_err(|e| super::error(field, e.to_string()))?;
    Ok(writer.count)
}
pub(crate) fn bounded_json<T: Serialize>(
    value: &T,
    limit: usize,
    field: &str,
) -> Result<String, Vec<FieldError>> {
    serialized_size(value, limit, field)?;
    serde_json::to_string(value).map_err(|e| super::error(field, e.to_string()))
}
/// Append a display checkpoint with a bounded, consecutive envelope index.
pub(crate) fn push_checkpoint(
    checkpoints: &mut Vec<Checkpoint>,
    mut checkpoint: Checkpoint,
) -> Result<(), Vec<FieldError>> {
    if checkpoints.len() >= MAX_CHECKPOINTS {
        return Err(super::error(
            "checkpoints",
            "episode exceeds 4096 checkpoints",
        ));
    }
    checkpoint.index = checkpoints.len() as u32;
    checkpoints.push(checkpoint);
    Ok(())
}
pub(crate) fn check_bounds(record: &EpisodeRecord) -> Result<(), Vec<FieldError>> {
    if record.checkpoints.len() > MAX_CHECKPOINTS {
        return Err(super::error(
            "checkpoints",
            "episode exceeds 4096 checkpoints",
        ));
    }
    serialized_size(&record.input, MAX_INPUT_BYTES, "input")?;
    serialized_size(record, MAX_EPISODE_BYTES, "episode")?;
    Ok(())
}
pub fn episode_json(record: &EpisodeRecord) -> Result<String, Vec<FieldError>> {
    check_bounds(record)?;
    serde_json::to_string(record).map_err(|e| super::error("episode", e.to_string()))
}
pub fn validate_episode(json: &str) -> Result<EpisodeRecord, Vec<FieldError>> {
    if json.len() > MAX_EPISODE_BYTES {
        return Err(super::error("episode", "episode exceeds 16 MiB"));
    }
    super::gate::duplicate::check(json, "episode")?;
    super::key::duplicate::check(json, "episode")?;
    let saved: EpisodeRecord =
        serde_json::from_str(json).map_err(|e| super::error("episode", e.to_string()))?;
    if saved.version != EPISODE_VERSION {
        return Err(super::error(
            "version",
            "unsupported episode format version",
        ));
    }
    let expected_identity = catalog::rules_identity(saved.study)?;
    let historical_identity = saved.rules_identity != expected_identity
        && historical::accepts(saved.study, &saved.rules_identity)?;
    if saved.rules_identity != expected_identity && !historical_identity {
        if matches!(
            saved.study,
            StudyId::ForagingFixed | StudyId::ForagingPassage | StudyId::ForagingConstruction
        ) {
            if let (Some((saved_source, saved_target)), Some((expected_source, expected_target))) = (
                saved.rules_identity.rsplit_once(":target:"),
                expected_identity.rsplit_once(":target:"),
            ) {
                if saved_source == expected_source && saved_target != expected_target {
                    return Err(super::error("rules_identity", format!("incompatible CPFA target {saved_target:?}; reconstruction requires {expected_target:?}")));
                }
            }
        }
        return Err(super::error(
            "rules_identity",
            "engine source or rules version mismatch",
        ));
    }
    check_bounds(&saved)?;
    let input = normalize_input(&bounded_json(&saved.input, MAX_INPUT_BYTES, "input")?)?;
    if input.study() != saved.study {
        return Err(super::error("study", "input study does not match record"));
    }
    let fresh = run(&input)?;
    if !super::equivalence::records_equivalent(&fresh, &saved, historical_identity) {
        return Err(super::error(
            "episode",
            "saved episode differs from reconstructed episode",
        ));
    }
    Ok(fresh)
}

/// Exact source/metadata binding for the reviewed key-only registration change.
/// The metadata contains source receipts, never its own receipt or a current
/// whole identity. Current adapter identities bind that metadata separately.
pub(super) mod historical {
    use super::*;
    use crate::key_delegation as key;
    const METADATA: &[u8] = include_bytes!("fixtures/key-registration-compatibility.json");
    const GATE_OLD: &str = "gate-assistance:protocol-1:browser-gate-1:native-source-sha256:e774ebf0e4590138c435ebee7fe148f2e22c5c4de214e9feea9b8765549dd2b1:adapter-source-sha256:c7e282aad8c8a437060dd9671a0821d13f95e9456a3e7186a679052224a2ada3:reviewed-reference-sha256:2ac1444ad6378303da9336a2ba74402ef9d9e955a2402cc4edf1c15a6ccab2fa";
    const DISCOVERY_OLD: &str = "surface-discovery:protocol-1:browser-discovery-1:native-source-sha256:bea9272bcd6e9aab412d1aafd36dc9facd0ffffd8b10ccfd7e5bb2aaac9fa055:adapter-source-sha256:d599b998e7387214eb062bb41f6db55e4a8fe549b696068a4c8bc5352c0a402a:reviewed-reference-sha256:d68afd69e5eb977a40beb2cc5d37a91c8def5740ba38dbc9695108cf3dcf5e2c";
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Binding {
        study: StudyId,
        historical_rules_identity: String,
        sources: Vec<key::Receipt>,
    }
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Metadata {
        version: String,
        bindings: Vec<Binding>,
    }
    fn checked<T, E: std::fmt::Display>(value: Result<T, E>) -> Result<T, Vec<FieldError>> {
        value.map_err(|e| super::super::error("rules_identity", e.to_string()))
    }
    fn old(study: StudyId) -> Option<&'static str> {
        match study {
            StudyId::GateAssistance => Some(GATE_OLD),
            StudyId::SurfaceDiscovery => Some(DISCOVERY_OLD),
            _ => None,
        }
    }
    fn append_unique(
        into: &mut Vec<key::Receipt>,
        more: Vec<key::Receipt>,
    ) -> Result<(), Vec<FieldError>> {
        for receipt in more {
            if let Some(existing) = into.iter().find(|r| r.path == receipt.path) {
                if existing != &receipt {
                    return Err(super::super::error(
                        "rules_identity",
                        "conflicting dependency receipts",
                    ));
                }
            } else {
                into.push(receipt);
            }
        }
        Ok(())
    }
    /// Ordered, actual compiled dependencies, with identical repeated paths kept once.
    pub(super) fn sources(study: StudyId) -> Result<Vec<key::Receipt>, Vec<FieldError>> {
        macro_rules! source {
            ($path:literal) => {
                (
                    concat!("crates/sugarscape-core/src/", $path),
                    include_bytes!(concat!("../", $path)).as_slice(),
                )
            };
        }
        let mut receipts = match study {
            StudyId::GateAssistance => checked(key::compiled_receipts(
                crate::gate_assistance::SOURCE_INPUTS,
            ))?,
            StudyId::SurfaceDiscovery => {
                checked(crate::surface_discovery::compiled_source_identity())?
                    .0
                    .into_iter()
                    .map(|r| key::Receipt {
                        path: r.path,
                        bytes: r.bytes,
                        sha256: r.sha256,
                    })
                    .collect()
            }
            _ => {
                return Err(super::super::error(
                    "rules_identity",
                    "study has no historical registration binding",
                ))
            }
        };
        let adapter = match study {
            StudyId::GateAssistance => vec![
                source!("browser_experiments/gate/mod.rs"),
                source!("browser_experiments/gate/projection.rs"),
                source!("browser_experiments/gate/duplicate.rs"),
            ],
            StudyId::SurfaceDiscovery => vec![
                source!("browser_experiments/discovery/mod.rs"),
                source!("browser_experiments/discovery/projection.rs"),
            ],
            _ => unreachable!(),
        };
        append_unique(&mut receipts, checked(key::compiled_receipts(&adapter))?)?;
        append_unique(
            &mut receipts,
            checked(key::compiled_receipts(&[
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
                source!("browser_experiments/key/mod.rs"),
                source!("browser_experiments/key/projection.rs"),
                source!("browser_experiments/key/duplicate.rs"),
            ]))?,
        )?;
        append_unique(
            &mut receipts,
            checked(key::compiled_receipts(key::SOURCE_INPUTS))?,
        )?;
        append_unique(
            &mut receipts,
            checked(key::compiled_receipts(key::REFERENCE_INPUTS))?,
        )?;
        Ok(receipts)
    }
    fn metadata(bytes: &[u8]) -> Result<Metadata, Vec<FieldError>> {
        let metadata: Metadata = checked(serde_json::from_slice(bytes))?;
        if metadata.version != "key-registration-compatibility-v1"
            || metadata.bindings.len() != 2
            || checked(serde_json::to_vec(&metadata))? != bytes
        {
            return Err(super::super::error(
                "rules_identity",
                "invalid historical registration metadata",
            ));
        }
        for (binding, study) in metadata
            .bindings
            .iter()
            .zip([StudyId::GateAssistance, StudyId::SurfaceDiscovery])
        {
            if binding.study != study
                || Some(binding.historical_rules_identity.as_str()) != old(study)
                || binding.sources != sources(study)?
            {
                return Err(super::super::error(
                    "rules_identity",
                    "unreviewed historical registration source or metadata",
                ));
            }
        }
        Ok(metadata)
    }
    /// Preserve the historical identity grammar and native/reference components;
    /// replace the adapter component with the complete dependency receipt digest.
    pub(in crate::browser_experiments) fn current(
        study: StudyId,
        base: String,
    ) -> Result<String, Vec<FieldError>> {
        if old(study).is_none() {
            return Ok(base);
        }
        let mut receipts = sources(study)?;
        append_unique(&mut receipts, checked(key::compiled_receipts(&[("crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json", METADATA)]))?)?;
        let adapter = checked(key::receipt_identity(&receipts))?;
        let (native, rest) = base
            .split_once(":adapter-source-sha256:")
            .ok_or_else(|| super::super::error("rules_identity", "missing adapter component"))?;
        let (_, references) = rest
            .split_once(":reviewed-reference-sha256:")
            .ok_or_else(|| super::super::error("rules_identity", "missing reference component"))?;
        Ok(format!(
            "{native}:adapter-source-sha256:{adapter}:reviewed-reference-sha256:{references}"
        ))
    }
    pub(super) fn accepts(study: StudyId, saved: &str) -> Result<bool, Vec<FieldError>> {
        if old(study) != Some(saved) {
            return Ok(false);
        }
        let _ = metadata(METADATA)?;
        Ok(true)
    }
    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn metadata_is_exact_and_every_source_is_authenticated() {
            assert!(metadata(METADATA).is_ok());
            let mut changed = METADATA.to_vec();
            changed.push(b' ');
            assert!(metadata(&changed).is_err());
            for (binding, source) in [(0, 0), (0, 30), (1, 0), (1, 30)] {
                let mut changed: Metadata = serde_json::from_slice(METADATA).unwrap();
                changed.bindings[binding].sources[source].sha256 = "0".repeat(64);
                assert!(metadata(&serde_json::to_vec(&changed).unwrap()).is_err());
            }
            let mut changed: Metadata = serde_json::from_slice(METADATA).unwrap();
            changed.bindings[0].sources.pop();
            assert!(metadata(&serde_json::to_vec(&changed).unwrap()).is_err());
            let mut changed: Metadata = serde_json::from_slice(METADATA).unwrap();
            changed.bindings[0].historical_rules_identity = "unknown".into();
            assert!(metadata(&serde_json::to_vec(&changed).unwrap()).is_err());
            assert!(!accepts(StudyId::GateAssistance, "unknown").unwrap());
            assert!(!accepts(StudyId::SurfaceDiscovery, GATE_OLD).unwrap());
            assert!(!accepts(StudyId::KeyDelegation, GATE_OLD).unwrap());
        }
    }
}
