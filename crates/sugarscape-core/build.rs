//! Deterministic raw receipts for the shared Gate input inventory.
#[path = "build_support/compact_receipts.rs"]
mod compact_receipts;
#[path = "src/gate_assistance/source_inputs.rs"]
mod source_inputs;
#[path = "build_support/surface_discovery.rs"]
mod surface_discovery;
use std::{collections::BTreeSet, fmt::Write as _, path::PathBuf};

fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && path.is_ascii()
        && !path.starts_with('/')
        && path
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"/_-.".contains(&b))
        && path.split('/').all(|p| !matches!(p, "" | "." | ".."))
}
fn receipt(path: &str, bytes: &[u8]) -> Result<String, String> {
    let len = u64::try_from(bytes.len()).map_err(|_| format!("{path}: byte length overflow"))?;
    Ok(format!(
        "{{\"path\":\"{path}\",\"bytes\":{len},\"sha256\":\"{}\"}}",
        surface_discovery::sha256(bytes)
    ))
}
fn generate(name: &str, inputs: &[(&str, &[u8])], out: &mut String) -> Result<(), String> {
    let mut paths = BTreeSet::new();
    let mut receipts = Vec::new();
    writeln!(
        out,
        "pub const {name}_RECEIPTS: &[(&str,u64,&str,&str)] = &["
    )
    .map_err(|e| e.to_string())?;
    for &(path, bytes) in inputs {
        if !valid_path(path) || !paths.insert(path) {
            return Err(format!("invalid/duplicate Gate input {path:?}"));
        }
        let raw = receipt(path, bytes)?;
        let identity = surface_discovery::sha256(format!("[{raw}]").as_bytes());
        writeln!(
            out,
            "({path:?},{},{:?},{identity:?}),",
            bytes.len(),
            surface_discovery::sha256(bytes)
        )
        .map_err(|e| e.to_string())?;
        receipts.push(raw);
        let root = PathBuf::from(
            std::env::var_os("CARGO_MANIFEST_DIR").ok_or("missing CARGO_MANIFEST_DIR")?,
        )
        .join("../..");
        println!("cargo:rerun-if-changed={}", root.join(path).display());
    }
    out.push_str("];\n");
    let body = format!("[{}]", receipts.join(","));
    let outer = surface_discovery::sha256(body.as_bytes());
    let metadata = surface_discovery::sha256(
        format!(
            "[{}]",
            receipt("gate-receipt-metadata-v1", body.as_bytes())?
        )
        .as_bytes(),
    );
    writeln!(out,"pub const {name}_IDENTITY: &str = {outer:?};\npub const {name}_METADATA_IDENTITY: &str = {metadata:?};").map_err(|e|e.to_string())?;
    Ok(())
}
fn main() {
    let result = (|| -> Result<(), String> {
        compact_receipts::generate(
            &PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").ok_or("missing manifest dir")?)
                .join("../..")
                .canonicalize()
                .map_err(|e| e.to_string())?,
            &PathBuf::from(std::env::var_os("OUT_DIR").ok_or("missing output dir")?),
        )?;
        let mut out =
            String::from("// Derived from the shared inventory; excluded from its own inputs.\n");
        generate("SOURCE", source_inputs::SOURCE_INPUTS, &mut out)?;
        generate("REFERENCE", source_inputs::REFERENCE_INPUTS, &mut out)?;
        let path = PathBuf::from(std::env::var_os("OUT_DIR").ok_or("missing OUT_DIR")?)
            .join("gate_source_receipts.rs");
        std::fs::write(&path, out).map_err(|e| format!("{}: {e}", path.display()))
    })();
    if let Err(e) = result {
        panic!("Gate receipt generation failed: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unchanged_sha_matches_independent_standard_vectors() {
        for (input, expected) in [
            (
                b"".as_slice(),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
            (
                b"abc".as_slice(),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            ),
        ] {
            assert_eq!(surface_discovery::sha256(input), expected);
        }
    }
    #[test]
    fn path_labels_reject_noncanonical_and_json_special_bytes() {
        for path in [
            "", "/a", "a//b", "a/./b", "../a", "a/../b", "a\\b", "a\"b", "a\nb", "café",
        ] {
            assert!(!valid_path(path), "{path:?}");
        }
        assert!(valid_path("crates/sugarscape-core/build.rs"));
    }
    #[test]
    fn generated_inventory_rejects_bad_and_duplicate_paths() {
        assert!(generate("TEST", &[("../bad", b"x")], &mut String::new()).is_err());
        assert!(generate("TEST", &[("a", b"x"), ("a", b"y")], &mut String::new()).is_err());
    }
}
