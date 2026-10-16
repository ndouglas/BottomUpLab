//! Raw embedded source and reference receipts; no generated self receipt.
macro_rules! identity_bytes {
    ($p:expr) => {{
        #[cfg(not(target_arch = "wasm32"))]
        {
            include_bytes!($p).as_slice()
        }
        #[cfg(target_arch = "wasm32")]
        {
            b"".as_slice()
        }
    }};
}
use super::*;
wire_object!(Receipt {pub path:String,pub bytes:u64,pub sha256:String});
pub const SOURCE_INPUTS: &[(&str, &[u8])] = &[
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
    (
        "crates/sugarscape-core/src/key_delegation/mod.rs",
        identity_bytes!("mod.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/types.rs",
        identity_bytes!("types.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/validation.rs",
        identity_bytes!("validation.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/controller.rs",
        identity_bytes!("controller.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/observation.rs",
        identity_bytes!("observation.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/kernel.rs",
        identity_bytes!("kernel.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/canonical.rs",
        identity_bytes!("canonical.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/sha256.rs",
        identity_bytes!("sha256.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/budget.rs",
        identity_bytes!("budget.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/game.rs",
        identity_bytes!("game.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/metrics.rs",
        identity_bytes!("metrics.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/diagnostics.rs",
        identity_bytes!("diagnostics.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/source_inputs.rs",
        identity_bytes!("source_inputs.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/protocol.rs",
        identity_bytes!("protocol.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/references.rs",
        identity_bytes!("references.rs"),
    ),
    ("Cargo.toml", identity_bytes!("../../../../Cargo.toml")),
    (
        "crates/sugarscape-core/Cargo.toml",
        identity_bytes!("../../Cargo.toml"),
    ),
    ("Cargo.lock", identity_bytes!("../../../../Cargo.lock")),
];
pub const REFERENCE_INPUTS: &[(&str, &[u8])] = &[
    (
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/kernel-reference.json",
        identity_bytes!("tests/fixtures/kernel-reference.json"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/game-reference.json",
        identity_bytes!("tests/fixtures/game-reference.json"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/limits-reference.json",
        identity_bytes!("tests/fixtures/limits-reference.json"),
    ),
];
impl Receipt {
    pub fn validate(&self) -> Result<(), Error> {
        if self.path.is_empty()
            || !self.path.is_ascii()
            || self
                .path
                .bytes()
                .any(|b| b < 32 || b == 127 || b == b'\\' || b == b'"')
            || self
                .path
                .split('/')
                .any(|c| c.is_empty() || c == "." || c == "..")
            || self.sha256.len() != 64
            || !self
                .sha256
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(Error::InvalidReport(
                "invalid receipt path or SHA-256".into(),
            ));
        }
        Ok(())
    }
}
pub(crate) fn receipts(inputs: &[(&str, &[u8])]) -> Result<Vec<Receipt>, Error> {
    inputs
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
        .collect()
}
pub(crate) fn identity(receipts: &[Receipt]) -> Result<String, Error> {
    let mut paths = std::collections::BTreeSet::new();
    for r in receipts {
        r.validate()?;
        if !paths.insert(&r.path) {
            return Err(Error::InvalidReport("duplicate receipt path".into()));
        }
    }
    Ok(super::sha256::sha256(
        &serde_json::to_vec(receipts).map_err(|e| Error::InvalidReport(e.to_string()))?,
    ))
}
pub fn source_receipts() -> Result<Vec<Receipt>, Error> {
    receipts(SOURCE_INPUTS)
}
pub fn reference_receipts() -> Result<Vec<Receipt>, Error> {
    receipts(REFERENCE_INPUTS)
}
pub fn source_identity() -> Result<String, Error> {
    identity(&source_receipts()?)
}
pub fn reference_identity() -> Result<String, Error> {
    identity(&reference_receipts()?)
}

/// Ordered raw-byte receipts, shared with the selected browser boundary.
pub(crate) fn compiled_bytes_identity(inputs: &[(&str, &[u8])]) -> Result<String, Error> {
    identity(&receipts(inputs)?)
}
