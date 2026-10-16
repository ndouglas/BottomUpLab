//! Host-only raw receipts for the selected browser identity boundaries.
use std::{fmt::Write as _, path::Path};
pub fn entry(path: &str, bytes: &[u8]) -> String {
    format!(
        "({path:?}, {}, {:?}),\n",
        bytes.len(),
        super::surface_discovery::sha256(bytes)
    )
}
fn visit(root: &Path, path: &Path, output: &mut String) -> Result<(), String> {
    println!("cargo:rerun-if-changed={}", path.display());
    if path.is_dir() {
        let mut paths = std::fs::read_dir(path)
            .map_err(|e| e.to_string())?
            .map(|e| e.map(|e| e.path()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;
        paths.sort();
        for path in paths {
            visit(root, &path, output)?;
        }
    } else {
        let relative = path
            .strip_prefix(root)
            .map_err(|e| e.to_string())?
            .to_str()
            .ok_or("non-UTF8 input")?;
        let bytes = std::fs::read(path).map_err(|e| format!("{relative}: {e}"))?;
        output.push_str(&entry(relative, &bytes));
    }
    Ok(())
}
pub fn generate(root: &Path, out: &Path) -> Result<(), String> {
    let mut output = String::from("pub const RAW_RECEIPTS: &[(&str,u64,&str)] = &[\n");
    // Closed, deliberately bounded source families. Native raw inventory order remains authoritative.
    for path in [
        "Cargo.toml",
        "Cargo.lock",
        "crates/sugarscape-core/Cargo.toml",
        "crates/sugarscape-core/build.rs",
        "crates/sugarscape-core/build_support",
        "crates/sugarscape-core/src/compact_receipts.rs",
        "crates/sugarscape-core/src/lib.rs",
        "crates/sugarscape-core/src/config.rs",
        "crates/sugarscape-core/src/persistent_notes",
        "crates/sugarscape-core/src/key_delegation",
        "crates/sugarscape-core/src/gate_assistance",
        "crates/sugarscape-core/src/surface_discovery",
        "crates/sugarscape-core/src/shared_surface",
        "crates/sugarscape-core/src/browser_experiments",
        "docs/superpowers/protocols/2026-10-14-paid-persistent-notes.md",
    ] {
        visit(root, &root.join(path), &mut output)?;
    }
    writeln!(output, "]; ").map_err(|e| e.to_string())?;
    std::fs::write(out.join("compact_raw_receipts.rs"), output).map_err(|e| e.to_string())?;
    for (name, fields) in [
        (
            "strategic",
            &[
                "transformation_version",
                "identity",
                "provenance",
                "versions",
                "environments",
                "policies",
            ][..],
        ),
        (
            "strategy",
            &[
                "transformation_version",
                "identity",
                "provenance",
                "version",
                "environments",
                "catalogs",
            ][..],
        ),
        (
            "audit",
            &[
                "transformation_version",
                "identity",
                "provenance",
                "version",
                "environments",
                "policy_provenance",
                "controller_snapshots",
                "targeted_audits",
                "control_evaluations",
                "nominal_evaluations",
                "cross_target_evaluations",
                "bound_checks",
            ][..],
        ),
    ] {
        let raw = std::fs::read(root.join(format!(
            "crates/sugarscape-core/src/browser_experiments/fixtures/{name}-recorded.json"
        )))
        .map_err(|e| e.to_string())?;
        let original: serde_json::Value =
            serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
        let mut projection = serde_json::Map::new();
        for field in fields {
            projection.insert(
                (*field).to_owned(),
                original
                    .get(field)
                    .ok_or_else(|| format!("{name}: missing {field}"))?
                    .clone(),
            );
        }
        std::fs::write(
            out.join(format!("{name}-selected.json")),
            serde_json::to_vec(&projection).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}
