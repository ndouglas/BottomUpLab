//! Recompute the selected closure and actual executable before any collection.
use super::{gate::CampaignBinding, io, wire};
use std::path::Path;
#[path = "../../build_support/war1_study_source_identity.rs"]
pub(super) mod inputs;
include!(concat!(env!("OUT_DIR"), "/war1_study_stamp.rs"));
pub fn verify() -> Result<CampaignBinding, String> {
    let entries = inputs::read(Path::new(ROOT), MANIFEST_PATH)?;
    let paths: Vec<_> = entries.iter().map(|e| e.0.as_str()).collect();
    if paths != INPUT_PATHS || inputs::fingerprint(entries, MANIFEST_PATH)? != INPUTS_SHA256 {
        return Err("study source inputs changed since compilation; rebuild required".into());
    }
    let bytes = io::read_input(&Path::new(ROOT).join(MANIFEST_PATH))?;
    if bytes != wire::SELECTED || wire::sha256(&bytes) != MANIFEST_SHA256 {
        return Err("selected manifest changed since compilation".into());
    }
    let executable = std::env::current_exe().map_err(|e| e.to_string())?;
    let (executable_sha256, _) = io::hash_file(&executable, 1024 * 1024 * 1024)?;
    Ok(CampaignBinding {
        manifest_sha256: MANIFEST_SHA256.into(),
        compiled_inputs_sha256: INPUTS_SHA256.into(),
        executable_sha256,
        declaration_sha256: None,
        acceptance_sha256: None,
        execution_approval_sha256: None,
    })
}
