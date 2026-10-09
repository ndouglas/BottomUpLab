//! Fully resolved immutable registration and explicit saved historical identities.
use super::{
    scenario, sha256, wire::WireManifest, CollectionMode, Geometry, Panel, Regime, RunKey,
};
use serde::Serialize;
use sugarscape_core::foraging::construction as core;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Condition {
    pub id: String,
    pub panel: Panel,
    pub geometry: Geometry,
    pub regime: Regime,
    pub setup: core::Setup,
    pub options: core::RunOptions,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct Manifest {
    pub schema: String,
    pub version: u32,
    pub status: String,
    pub execution_authorized: bool,
    pub protocol: String,
    pub conditions: Vec<Condition>,
    pub construction_seeds: Vec<u64>,
    pub scientific_seeds: Vec<u64>,
    pub raw_record_limit: u64,
    pub raw_total_limit: u64,
    pub metadata_limit: u64,
}

pub(super) fn candidate() -> Result<Manifest, String> {
    let mut conditions = Vec::new();
    for (panel, panel_name) in [(Panel::Route, "route"), (Panel::Access, "access")] {
        for (geometry, geometry_name) in [
            (Geometry::Straight, "straight"),
            (Geometry::Detour, "detour"),
            (Geometry::Twisting, "twisting"),
        ] {
            for (regime, regime_name) in [
                (Regime::Paid, "paid"),
                (Regime::Protected, "protected"),
                (Regime::AlreadyOpen, "already_open"),
            ] {
                if panel == Panel::Access && regime == Regime::AlreadyOpen {
                    continue;
                }
                conditions.push(Condition {
                    id: format!("{panel_name}.{geometry_name}.{regime_name}"),
                    panel,
                    geometry,
                    regime,
                    setup: scenario::build(panel, geometry, regime)?,
                    options: core::RunOptions {
                        ticks: 512,
                        sample_every: 128,
                        snapshots: true,
                    },
                });
            }
        }
    }
    Ok(Manifest {
        schema: "foraging-shortcut-manifest-v1".into(),
        version: 1,
        status: "registered".into(),
        execution_authorized: false,
        protocol:
            "docs/superpowers/specs/2026-10-08-foraging-5-scientific-registration-proposal.md"
                .into(),
        conditions,
        construction_seeds: vec![7, 8],
        scientific_seeds: (10001..=10040).collect(),
        raw_record_limit: 4 * 1024 * 1024,
        raw_total_limit: 1024 * 1024 * 1024,
        metadata_limit: 4 * 1024 * 1024,
    })
}

pub(super) fn manifest_bytes() -> Result<Vec<u8>, String> {
    canonical_bytes(&candidate()?)
}

pub(super) fn canonical_bytes(manifest: &Manifest) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|e| format!("serialize shortcut manifest: {e}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

// The reader bounds untrusted metadata before it can select an identity's limits.
pub(super) const SAVED_METADATA_LIMIT: u64 = 4 * 1024 * 1024;

pub(super) fn wire_manifest(manifest: &Manifest) -> Result<WireManifest, String> {
    serde_json::from_slice(&serde_json::to_vec(manifest).map_err(|e| e.to_string())?)
        .map_err(|e| format!("manifest wire representation: {e}"))
}

/// Only these two complete identities are supported; history never authorizes collection.
pub(super) fn resolve_saved(wire: &WireManifest, digest: &str) -> Result<Manifest, String> {
    let active = candidate()?;
    let mut historical = active.clone();
    historical.status = "draft".into();
    historical.protocol =
        "docs/superpowers/specs/2026-10-07-foraging-5-shortcut-comparison-design.md".into();
    for (manifest, known_digest) in [
        (
            active,
            "a96d3a678676b581a385880cf8f4fd80d0b737bcd0557b76ec9e77267a07d759",
        ),
        (
            historical,
            "8e532eaf779b970d4bd9a2550fc1634f89abe63a00b7d05d3134f04a4a2deb62",
        ),
    ] {
        if *wire == wire_manifest(&manifest)?
            && digest == known_digest
            && sha256(&canonical_bytes(&manifest)?) == known_digest
        {
            return Ok(manifest);
        }
    }
    Err("unknown complete saved manifest identity or manifest_sha256 mismatch".into())
}

pub(super) fn condition<'a>(manifest: &'a Manifest, id: &str) -> Result<&'a Condition, String> {
    manifest
        .conditions
        .iter()
        .find(|c| c.id == id)
        .ok_or_else(|| format!("unknown shortcut condition: {id}"))
}

pub(super) fn expected_keys(manifest: &Manifest, mode: CollectionMode) -> Vec<RunKey> {
    let seeds = match mode {
        CollectionMode::Construction => &manifest.construction_seeds,
        CollectionMode::Scientific => &manifest.scientific_seeds,
    };
    manifest
        .conditions
        .iter()
        .flat_map(|c| {
            seeds.iter().map(move |&seed| RunKey {
                condition: c.id.clone(),
                seed,
            })
        })
        .collect()
}

pub(super) fn authorize(manifest: &Manifest, mode: CollectionMode) -> Result<(), String> {
    if mode == CollectionMode::Scientific
        && (!manifest.execution_authorized || manifest.status != "registered")
    {
        return Err(format!("scientific shortcut execution requires an authorized registered manifest; current status {}, execution approval pending", manifest.status));
    }
    Ok(())
}
