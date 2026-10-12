//! Raw embedded source and reference receipts; no generated self receipt.
use super::*;
wire_object!(Receipt {pub path:String,pub bytes:u64,pub sha256:String});
pub const SOURCE_INPUTS: &[(&str, &[u8])] = &[
    (
        "crates/sugarscape-core/src/key_delegation/mod.rs",
        include_bytes!("mod.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/types.rs",
        include_bytes!("types.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/validation.rs",
        include_bytes!("validation.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/controller.rs",
        include_bytes!("controller.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/observation.rs",
        include_bytes!("observation.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/kernel.rs",
        include_bytes!("kernel.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/canonical.rs",
        include_bytes!("canonical.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/sha256.rs",
        include_bytes!("sha256.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/budget.rs",
        include_bytes!("budget.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/game.rs",
        include_bytes!("game.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/metrics.rs",
        include_bytes!("metrics.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/diagnostics.rs",
        include_bytes!("diagnostics.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/source_inputs.rs",
        include_bytes!("source_inputs.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/protocol.rs",
        include_bytes!("protocol.rs"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/references.rs",
        include_bytes!("references.rs"),
    ),
    ("Cargo.toml", include_bytes!("../../../../Cargo.toml")),
    (
        "crates/sugarscape-core/Cargo.toml",
        include_bytes!("../../Cargo.toml"),
    ),
    ("Cargo.lock", include_bytes!("../../../../Cargo.lock")),
];
pub const REFERENCE_INPUTS: &[(&str, &[u8])] = &[
    (
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/kernel-reference.json",
        include_bytes!("tests/fixtures/kernel-reference.json"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/game-reference.json",
        include_bytes!("tests/fixtures/game-reference.json"),
    ),
    (
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/limits-reference.json",
        include_bytes!("tests/fixtures/limits-reference.json"),
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
            Ok(Receipt {
                path: (*path).into(),
                bytes: u64::try_from(bytes.len()).map_err(|_| Error::ArithmeticOverflow)?,
                sha256: super::sha256::sha256(bytes),
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
