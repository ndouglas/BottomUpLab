//! Explicit raw input binding. Receipts never embed their generated digest.
use crate::identity_bytes::identity_bytes;
// Static tables preserve one raw allocation across code-generation units and consumers.
use super::{canonical, sha256, Error};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceIdentity {
    pub domain: String,
    pub receipts: Vec<Receipt>,
    pub sha256: String,
}
macro_rules! input {
    ($p:literal) => {
        (
            concat!("crates/sugarscape-core/src/persistent_notes/", $p),
            identity_bytes!($p),
        )
    };
}
pub(crate) static SOURCE_INPUTS: &[(&str, &[u8])] = &[
    (
        "crates/sugarscape-core/src/identity_bytes.rs",
        identity_bytes!("../identity_bytes.rs"),
    ),
    (
        "crates/sugarscape-core/src/surface_discovery/mod.rs",
        identity_bytes!("../surface_discovery/mod.rs"),
    ),
    (
        "crates/sugarscape-core/build_support/compact_receipts.rs",
        identity_bytes!("../../build_support/compact_receipts.rs"),
    ),
    (
        "crates/sugarscape-core/src/compact_receipts.rs",
        identity_bytes!("../../src/compact_receipts.rs"),
    ),
    (
        "crates/sugarscape-core/build.rs",
        identity_bytes!("../../build.rs"),
    ),
    (
        "crates/sugarscape-core/build_support/surface_discovery.rs",
        identity_bytes!("../../build_support/surface_discovery.rs"),
    ),
    (
        "crates/sugarscape-core/src/surface_discovery/diagnostics/hash.rs",
        identity_bytes!("../../src/surface_discovery/diagnostics/hash.rs"),
    ),
    ("Cargo.lock", identity_bytes!("../../../../Cargo.lock")),
    ("Cargo.toml", identity_bytes!("../../../../Cargo.toml")),
    (
        "crates/sugarscape-core/Cargo.toml",
        identity_bytes!("../../Cargo.toml"),
    ),
    input!("budget.rs"),
    input!("canonical.rs"),
    input!("controller.rs"),
    input!("diagnostics.rs"),
    input!("evaluation.rs"),
    input!("kernel.rs"),
    input!("metrics.rs"),
    input!("mod.rs"),
    input!("observation.rs"),
    input!("protocol.rs"),
    input!("references.rs"),
    input!("sha256.rs"),
    input!("source_inputs.rs"),
    input!("types.rs"),
    input!("validation.rs"),
    (
        "docs/superpowers/protocols/2026-10-14-paid-persistent-notes.md",
        super::protocol::PROTOCOL_BYTES,
    ),
];
pub(crate) static REFERENCE_INPUTS: &[(&str, &[u8])] = &[
    input!("tests/fixtures/evaluation-reference.json"),
    input!("tests/fixtures/kernel-reference.json"),
    input!("tests/fixtures/limits-reference.json"),
];
fn identity(domain: &str, inputs: &[(&str, &[u8])]) -> Result<SourceIdentity, Error> {
    let mut receipts: Vec<_> = inputs
        .iter()
        .map(|(path, bytes)| {
            let (bytes, sha256) =
                crate::compact_receipts::raw(path, bytes).map_err(Error::InvalidReport)?;
            Ok(Receipt {
                path: (*path).into(),
                bytes,
                sha256,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;
    receipts.sort_by(|a, b| a.path.cmp(&b.path));
    #[derive(Serialize)]
    struct Payload<'a> {
        domain: &'a str,
        receipts: &'a [Receipt],
    }
    let mut hash = sha256::Sha256::new();
    canonical::write_record(
        &mut hash,
        &Payload {
            domain,
            receipts: &receipts,
        },
        65536,
    )?;
    Ok(SourceIdentity {
        domain: domain.into(),
        receipts,
        sha256: hash.finish(),
    })
}
pub(crate) fn source() -> Result<SourceIdentity, Error> {
    static VALUE: OnceLock<Result<SourceIdentity, Error>> = OnceLock::new();
    VALUE
        .get_or_init(|| identity("persistent-notes-source-v1", SOURCE_INPUTS))
        .clone()
}
pub(crate) fn reference() -> Result<SourceIdentity, Error> {
    static VALUE: OnceLock<Result<SourceIdentity, Error>> = OnceLock::new();
    VALUE
        .get_or_init(|| identity("persistent-notes-reference-v1", REFERENCE_INPUTS))
        .clone()
}
pub fn source_identity() -> Result<String, Error> {
    Ok(source()?.sha256)
}
pub fn reference_identity() -> Result<String, Error> {
    Ok(reference()?.sha256)
}
