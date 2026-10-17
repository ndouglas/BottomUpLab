//! Current engineering closure parity only; never launches an engine or study.
#[allow(dead_code)]
#[path = "../build_support/war1_study_source_identity.rs"]
mod inputs;
use sha2::{Digest, Sha256};
use std::{path::Path, process::Command};

#[test]
fn python_and_native_current_closures_match_every_raw_entry() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let native = inputs::read(root, inputs::DEFAULT_MANIFEST).unwrap();
    let output = Command::new("python3")
        .current_dir(root)
        .args(["-c", "import json; from pathlib import Path; from survey.war1_study.registration import source_inventory, DEFAULT_MANIFEST; print(json.dumps(source_inventory(Path.cwd(), DEFAULT_MANIFEST)))"])
        .output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let python: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        python["compiled_inputs_sha256"],
        inputs::fingerprint(&native, inputs::DEFAULT_MANIFEST).unwrap()
    );
    let expected: Vec<_> = native
        .iter()
        .map(|(path, bytes)| {
            serde_json::json!({
                "path": path, "bytes": bytes.len(), "sha256": format!("{:x}", Sha256::digest(bytes))
            })
        })
        .collect();
    assert_eq!(python["entries"], serde_json::json!(expected));
    for required in [
        "crates/sugarscape-core/build_support/compact_receipts.rs",
        "docs/superpowers/protocols/2026-10-14-paid-persistent-notes.md",
    ] {
        assert!(native.iter().any(|(path, _)| path == required));
        let mut changed = native.clone();
        changed
            .iter_mut()
            .find(|(path, _)| path == required)
            .unwrap()
            .1
            .push(b' ');
        assert_ne!(
            inputs::fingerprint(&native, inputs::DEFAULT_MANIFEST).unwrap(),
            inputs::fingerprint(&changed, inputs::DEFAULT_MANIFEST).unwrap()
        );
        changed.retain(|(path, _)| path != required);
        assert!(inputs::fingerprint(&changed, inputs::DEFAULT_MANIFEST).is_err());
    }
    assert!(!native.iter().any(|(path, _)| path
        == "crates/sugarscape-core/src/persistent_notes/tests/fixtures/evaluation-reference.json"));
    assert!(root
        .join(
            "crates/sugarscape-core/src/persistent_notes/tests/fixtures/evaluation-reference.json"
        )
        .is_file());
}
