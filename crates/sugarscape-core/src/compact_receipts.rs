//! Compact build receipts are used only where raw bytes are identity preimages.
#[cfg(any(target_arch = "wasm32", test))]
mod generated {
    include!(concat!(env!("OUT_DIR"), "/compact_raw_receipts.rs"));
}
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn lookup(path: &str) -> Result<(u64, String), String> {
    generated::RAW_RECEIPTS
        .iter()
        .find(|r| r.0 == path)
        .map(|r| (r.1, r.2.to_owned()))
        .ok_or_else(|| format!("missing generated raw receipt: {path}"))
}
pub(crate) fn raw(path: &str, bytes: &[u8]) -> Result<(u64, String), String> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = bytes;
        lookup(path)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = path;
        Ok((
            bytes.len() as u64,
            crate::surface_discovery::raw_sha256(bytes),
        ))
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn generated_receipts_match_independent_raw_files_and_reject_unknown() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for &(path, length, digest) in super::generated::RAW_RECEIPTS {
            let bytes = std::fs::read(root.join(path)).unwrap();
            assert_eq!(
                super::raw(path, &bytes).unwrap(),
                (length, digest.to_owned()),
                "{path}"
            );
        }
        assert!(super::lookup("unknown-input").is_err());
    }
}
