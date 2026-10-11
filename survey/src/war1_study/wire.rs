//! Task 2 accepts only the checked-in engineering construction input.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sugarscape_core::war::config::EngagementConfig;

pub const LIMIT: usize = 16 * 1024 * 1024;
pub const FIXTURE: &[u8] = include_bytes!("../../war1_study/fixtures/construction-manifest.json");
pub const PROPOSAL: &[u8] =
    include_bytes!("../../../docs/superpowers/specs/2026-10-10-war-1-measurement-proposal.json");
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: String,
    pub purpose: String,
    pub protocol_sha256: String,
    pub proposal_sha256: String,
    pub roster_sha256: String,
    pub sampler: Sampler,
    pub arms: Vec<Arm>,
    pub cases: Vec<Case>,
}
#[derive(Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sampler {
    pub method: String,
    pub bytes_drawn: u64,
    pub seed_bytes_sha256: String,
    pub command: Vec<String>,
    pub version: String,
}
#[derive(Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arm {
    pub arm_id: String,
    pub group: String,
    pub profile: String,
    pub scale: u32,
    pub mirror: bool,
    pub config: EngagementConfig,
    pub replicates: u32,
    pub seed_namespace: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Key {
    pub ordinal: u64,
    pub arm_id: String,
    pub replicate_index: u32,
}
#[derive(Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub ordinal: u64,
    pub arm_id: String,
    pub replicate_index: u32,
    pub seed: u64,
}
impl Case {
    pub fn key(&self) -> Key {
        Key {
            ordinal: self.ordinal,
            arm_id: self.arm_id.clone(),
            replicate_index: self.replicate_index,
        }
    }
}
pub fn decode_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    if bytes.len() > LIMIT {
        return Err("manifest exceeds 16 MiB".into());
    }
    // Compute the raw identity before resolving anything. Byte equality rejects
    // duplicate keys, unknown fields, noncanonical types and every other mutation.
    let digest = sha256(bytes);
    if bytes != FIXTURE {
        return Err(format!("engineering fixture identity mismatch: {digest}"));
    }
    let manifest: Manifest = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    for arm in &manifest.arms {
        arm.config
            .validate()
            .map_err(|e| format!("invalid fixture config: {e:?}"))?;
    }
    Ok(manifest)
}
