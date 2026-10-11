//! Separately selected W1-A closure. Final receipts and outputs are never inputs.
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
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
pub const MAX_ENTRIES: usize = 4096;
pub const MAX_FILE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_SOURCE_BYTES: usize = 64 * 1024 * 1024;
pub fn fingerprint(entries: &[(String, Vec<u8>)], manifest: &str) -> Result<String, String> {
    validate_selection(manifest)?;
    if entries.len() > MAX_ENTRIES {
        return Err("source entry limit exceeded".into());
    }
    let mut total = 0usize;
    for (_, bytes) in entries {
        if bytes.len() > MAX_FILE_BYTES {
            return Err("source per-file byte limit exceeded".into());
        }
        total = total
            .checked_add(bytes.len())
            .ok_or("source byte overflow")?;
        if total > MAX_SOURCE_BYTES {
            return Err("source aggregate byte limit exceeded".into());
        }
    }
    let mut ordered: Vec<_> = entries.iter().collect();
    ordered.sort_by(|a, b| a.0.cmp(&b.0));
    let mut seen = BTreeSet::new();
    let mut hash = Sha256::new();
    hash.update(b"war1-study-compiled-inputs-v1\0");
    for (path, bytes) in ordered {
        if !selected(path, manifest) || !seen.insert(path.as_str()) {
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
/// At most limit retained bytes; the growth sentinel stays on the stack.
pub fn read_bounded_source(reader: &mut impl Read, limit: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let remaining = limit - bytes.len();
        let capacity = remaining.min(buffer.len());
        if capacity == 0 {
            let mut sentinel = [0];
            return if reader.read(&mut sentinel).map_err(|e| e.to_string())? == 0 {
                Ok(bytes)
            } else {
                Err("source grew beyond bounded byte budget".into())
            };
        }
        let n = reader
            .read(&mut buffer[..capacity])
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(bytes);
        }
        bytes
            .try_reserve_exact(n)
            .map_err(|e| format!("bounded source allocation: {e}"))?;
        bytes.extend_from_slice(&buffer[..n]);
    }
}
struct Loader {
    entries: Vec<(String, Vec<u8>)>,
    bytes: usize,
    directories: usize,
    max_entries: usize,
    max_file: usize,
    max_bytes: usize,
}
impl Loader {
    fn visit(&mut self, root: &Path, relative: &str) -> Result<(), String> {
        if relative.len() > 4096 {
            return Err("source path length limit exceeded".into());
        }
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
            self.directories += 1;
            if self.directories > MAX_ENTRIES {
                return Err("source directory limit exceeded".into());
            }
            for item in fs::read_dir(&path).map_err(|e| e.to_string())? {
                let item = item.map_err(|e| e.to_string())?;
                let name = item
                    .file_name()
                    .into_string()
                    .map_err(|_| "non-UTF8 study input")?;
                self.visit(root, &format!("{relative}/{name}"))?;
            }
        } else if m.is_file() {
            if self.entries.len() >= self.max_entries {
                return Err("source entry limit exceeded".into());
            }
            if m.len() > self.max_file as u64 {
                return Err("source per-file byte limit exceeded".into());
            }
            let remaining = self
                .max_bytes
                .checked_sub(self.bytes)
                .ok_or("source aggregate byte limit exceeded")?;
            if m.len() > remaining as u64 {
                return Err("source aggregate byte limit exceeded".into());
            }
            let mut file = fs::File::open(&path).map_err(|e| e.to_string())?;
            if !file.metadata().map_err(|e| e.to_string())?.is_file() {
                return Err("nonregular source handle".into());
            }
            let bytes = read_bounded_source(&mut file, self.max_file.min(remaining))?;
            self.bytes += bytes.len();
            self.entries.push((relative.into(), bytes));
        } else {
            return Err(format!("nonregular study input {relative}"));
        }
        Ok(())
    }
}
fn read_limits(
    root: &Path,
    manifest: &str,
    max_entries: usize,
    max_file: usize,
    max_bytes: usize,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    validate_selection(manifest)?;
    let mut loader = Loader {
        entries: vec![],
        bytes: 0,
        directories: 0,
        max_entries,
        max_file,
        max_bytes,
    };
    for path in DIRECTORIES.iter().chain(FILES).copied().chain([manifest]) {
        loader.visit(root, path)?;
    }
    for item in fs::read_dir(root.join("survey/war1_study")).map_err(|e| e.to_string())? {
        let item = item.map_err(|e| e.to_string())?;
        let name = item
            .file_name()
            .into_string()
            .map_err(|_| "non-UTF8 Python path")?;
        if name.ends_with(".py") {
            loader.visit(root, &format!("survey/war1_study/{name}"))?;
        }
    }
    loader.entries.sort_by(|a, b| a.0.cmp(&b.0));
    fingerprint(&loader.entries, manifest)?;
    Ok(loader.entries)
}
pub fn read(root: &Path, manifest: &str) -> Result<Vec<(String, Vec<u8>)>, String> {
    read_limits(
        root,
        manifest,
        MAX_ENTRIES,
        MAX_FILE_BYTES,
        MAX_SOURCE_BYTES,
    )
}
#[cfg(test)]
pub fn read_with_test_limits(
    root: &Path,
    manifest: &str,
    entries: usize,
    file: usize,
    total: usize,
) -> Result<Vec<(String, Vec<u8>)>, String> {
    read_limits(root, manifest, entries, file, total)
}

/// Build-only committed selection check; kept here to exercise the actual subprocess reader.
#[allow(dead_code)]
pub fn read_committed_manifest(root: &Path, selected: &str) -> Result<Vec<u8>, String> {
    let mut child = std::process::Command::new("git")
        .current_dir(root)
        .args(["show", &format!("HEAD:{selected}")])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    let mut stdout = child.stdout.take().expect("piped Git stdout");
    let bytes = read_bounded_source(&mut stdout, MAX_FILE_BYTES);
    drop(stdout);
    if bytes.is_err() {
        let _ = child.kill();
    }
    // Always reap, including overflow/read failure. Closing stdout also unblocks Git.
    let status = child
        .wait()
        .map_err(|e| format!("reap committed-manifest child: {e}"))?;
    let bytes = bytes?;
    if !status.success() {
        return Err(format!("cannot read committed selected manifest: {status}"));
    }
    Ok(bytes)
}
