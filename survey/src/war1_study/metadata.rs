//! Pure, source-verified build metadata. Never constructs an Engagement.
use super::{gate, io, source, wire};
use serde::Serialize;
use std::path::Path;
#[derive(Serialize)]
pub struct Metadata {
    schema: &'static str,
    root: &'static str,
    selected_path: &'static str,
    selected_purpose: &'static str,
    registered_histories: u64,
    binding: gate::CampaignBinding,
    inventory: gate::SourceInventory,
    rustc: &'static str,
    cargo: &'static str,
    platform: &'static str,
    features: &'static [&'static str],
    flags: &'static [(&'static str, &'static str)],
    reference_calls_checked: usize,
}
pub fn export() -> Result<Metadata, String> {
    let binding = source::verify()?;
    let proposal: serde_json::Value =
        serde_json::from_slice(wire::PROPOSAL).map_err(|e| e.to_string())?;
    let arms: Vec<wire::Arm> =
        serde_json::from_value(proposal["arms"].clone()).map_err(|e| e.to_string())?;
    for arm in &arms {
        for time in [1.0, 2.0, 4.0, 8.0] {
            if let Err(error) = sugarscape_core::war::reference_at(&arm.config, time) {
                for text in [&error.field, &error.detail] {
                    if text.len() > 512 || !text.bytes().all(|b| (32..=126).contains(&b)) {
                        return Err("pure reference error vocabulary exceeds encoded proof".into());
                    }
                }
            }
        }
    }
    let entries = source::inputs::read(Path::new(source::ROOT), source::MANIFEST_PATH)?;
    let inventory = gate::SourceInventory {
        schema: "war1-study-source-inventory-v1".into(),
        compiled_inputs_sha256: binding.compiled_inputs_sha256.clone(),
        entries: entries
            .iter()
            .map(|(path, bytes)| gate::Artifact {
                path: path.clone(),
                sha256: wire::sha256(bytes),
                bytes: bytes.len() as u64,
            })
            .collect(),
    };
    Ok(Metadata {
        schema: "war1-study-metadata-v1",
        root: source::ROOT,
        selected_path: source::MANIFEST_PATH,
        selected_purpose: source::PURPOSE,
        registered_histories: 0,
        binding,
        inventory,
        rustc: source::TOOLCHAIN,
        cargo: source::CARGO_VERSION,
        platform: std::env::consts::OS,
        features: source::FEATURES,
        flags: source::FLAGS,
        reference_calls_checked: arms.len() * 4,
    })
}
pub fn bytes() -> Result<Vec<u8>, String> {
    io::encode(&export()?, io::METADATA_LIMIT, true)
}
