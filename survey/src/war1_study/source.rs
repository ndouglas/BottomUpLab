//! Recompute the selected closure and actual executable before any collection.
use super::{gate::CampaignBinding, io, wire};
use std::path::Path;
#[path = "../../build_support/war1_study_source_identity.rs"]
pub(super) mod inputs;
include!(concat!(env!("OUT_DIR"), "/war1_study_stamp.rs"));
// Fields are private; production can only construct the compiled context.
pub(super) struct VerificationContext {
    root: std::path::PathBuf,
    inputs_sha256: String,
    paths: Vec<String>,
    executable: std::path::PathBuf,
}
impl VerificationContext {
    pub(super) fn compiled() -> Result<Self, String> {
        Ok(Self {
            root: ROOT.into(),
            inputs_sha256: INPUTS_SHA256.into(),
            paths: INPUT_PATHS.iter().map(|s| s.to_string()).collect(),
            executable: std::env::current_exe().map_err(|e| e.to_string())?,
        })
    }
    #[cfg(test)]
    pub(super) fn construction_test(
        root: &Path,
        inputs_sha256: &str,
        paths: &[String],
        executable: &Path,
    ) -> Self {
        Self {
            root: root.into(),
            inputs_sha256: inputs_sha256.into(),
            paths: paths.into(),
            executable: executable.into(),
        }
    }
    pub(super) fn root(&self) -> &Path {
        &self.root
    }
    pub(super) fn verify(&self) -> Result<CampaignBinding, String> {
        let entries = inputs::read(&self.root, MANIFEST_PATH)?;
        let paths: Vec<_> = entries.iter().map(|e| &e.0).collect();
        if paths != self.paths.iter().collect::<Vec<_>>()
            || inputs::fingerprint(&entries, MANIFEST_PATH)? != self.inputs_sha256
        {
            return Err("study source inputs changed since compilation; rebuild required".into());
        }
        let bytes = io::read_input(&self.root.join(MANIFEST_PATH))?;
        if bytes != wire::SELECTED || wire::sha256(&bytes) != MANIFEST_SHA256 {
            return Err("selected manifest changed since compilation".into());
        }
        let (executable_sha256, _) = io::hash_file(&self.executable, 1024 * 1024 * 1024)?;
        Ok(CampaignBinding {
            manifest_sha256: MANIFEST_SHA256.into(),
            compiled_inputs_sha256: self.inputs_sha256.clone(),
            executable_sha256,
            declaration_sha256: None,
            acceptance_sha256: None,
            execution_approval_sha256: None,
        })
    }
}
pub fn verify() -> Result<CampaignBinding, String> {
    VerificationContext::compiled()?.verify()
}
