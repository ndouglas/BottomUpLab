//! Separately selected W1-A closure. Final receipts and outputs are never inputs.
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    path::{Component, Path},
};
pub const DEFAULT_MANIFEST: &str = "survey/war1_study/fixtures/construction-manifest.json";
pub const DIRECTORIES: &[&str] = &["crates/sugarscape-core/src", "survey/src/war1_study"];
pub const FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "crates/sugarscape-core/Cargo.toml",
    "crates/sugarscape-core/assets/sugar-map.txt",
    "survey/Cargo.toml",
    "survey/Cargo.lock",
    "survey/build.rs",
    "survey/src/bin/war1_study.rs",
    "survey/build_support/war1_study_source_identity.rs",
    "docs/superpowers/specs/2026-10-10-war-1-measurement-design.md",
    "docs/superpowers/specs/2026-10-10-war-1-measurement-proposal.json",
    "docs/superpowers/plans/2026-10-10-war-1-measurement.md",
    "docs/superpowers/specs/2026-10-09-war-1-engagement-design.md",
    "docs/superpowers/specs/2026-10-10-war-1-engineering-amendment.md",
    "docs/superpowers/plans/2026-10-09-war-1-engagement.md",
];
pub fn valid_relative(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && path
            .split('/')
            .all(|p| !p.is_empty() && p != "." && p != "..")
        && Path::new(path)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
}
pub fn validate_selection(path: &str) -> Result<(), String> {
    if !valid_relative(path)
        || (path != DEFAULT_MANIFEST
            && !(path.starts_with("survey/war1_study/manifests/") && path.ends_with(".json")))
    {
        return Err("WAR1_STUDY_MANIFEST must select one canonical manifest under survey/war1_study/manifests/".into());
    }
    Ok(())
}
fn selected(path: &str, manifest: &str) -> bool {
    valid_relative(path)
        && (path == manifest
            || FILES.contains(&path)
            || DIRECTORIES
                .iter()
                .any(|d| path.starts_with(&format!("{d}/")))
            || (path.starts_with("survey/war1_study/") && path.ends_with(".py")))
}
pub fn fingerprint(mut entries: Vec<(String, Vec<u8>)>, manifest: &str) -> Result<String, String> {
    validate_selection(manifest)?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let mut seen = BTreeSet::new();
    let mut hash = Sha256::new();
    hash.update(b"war1-study-compiled-inputs-v1\0");
    for (path, bytes) in entries {
        if !selected(&path, manifest) || !seen.insert(path.clone()) {
            return Err(format!("invalid/duplicate study input {path}"));
        }
        hash.update((path.len() as u64).to_be_bytes());
        hash.update(path.as_bytes());
        hash.update((bytes.len() as u64).to_be_bytes());
        hash.update(bytes);
    }
    for path in FILES.iter().copied().chain([
        manifest,
        "crates/sugarscape-core/src/lib.rs",
        "survey/src/war1_study/mod.rs",
        "survey/war1_study/contracts.py",
    ]) {
        if !seen.contains(path) {
            return Err(format!("missing study input {path}"));
        }
    }
    Ok(format!("{:x}", hash.finalize()))
}
pub fn read(root: &Path, manifest: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    validate_selection(manifest)?;
    fn visit(root: &Path, relative: &str, out: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
        let mut prefix = root.to_path_buf();
        for part in Path::new(relative).components() {
            prefix.push(part);
            let m =
                fs::symlink_metadata(&prefix).map_err(|e| format!("{}: {e}", prefix.display()))?;
            if m.file_type().is_symlink() {
                return Err(format!("symlink study input {}", prefix.display()));
            }
        }
        let path = root.join(relative);
        let m = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if m.is_dir() {
            for item in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let item = item.map_err(|e| e.to_string())?;
                let name = item
                    .file_name()
                    .into_string()
                    .map_err(|_| "non-UTF8 study input")?;
                visit(root, &format!("{relative}/{name}"), out)?;
            }
        } else if m.is_file() && m.len() <= 16 * 1024 * 1024 {
            out.push((relative.into(), fs::read(path).map_err(|e| e.to_string())?));
        } else {
            return Err(format!("nonregular/oversized study input {relative}"));
        }
        Ok(())
    }
    let mut entries = vec![];
    for path in DIRECTORIES.iter().chain(FILES).copied().chain([manifest]) {
        visit(root, path, &mut entries)?;
    }
    // Python sources only; do not recurse through manifests, receipts, probes or outputs.
    for item in fs::read_dir(root.join("survey/war1_study")).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        let name = item
            .file_name()
            .into_string()
            .map_err(|_| "non-UTF8 Python path")?;
        if name.ends_with(".py") {
            visit(root, &format!("survey/war1_study/{name}"), &mut entries)?;
        }
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    fingerprint(entries.clone(), manifest)?;
    Ok(entries)
}
