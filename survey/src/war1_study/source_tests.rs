use super::{source, wire};
#[test]
fn selected_manifest_is_source_bound() {
    let binding = source::verify().unwrap();
    assert_eq!(binding.manifest_sha256, wire::sha256(wire::FIXTURE));
    assert_eq!(binding.compiled_inputs_sha256.len(), 64);
    assert_eq!(binding.executable_sha256.len(), 64);
}
#[test]
fn source_fingerprint_changes_on_map_mutation() {
    let root = std::path::Path::new(source::ROOT);
    let original = source::inputs::read(root, source::MANIFEST_PATH).unwrap();
    let mut changed = original.clone();
    changed
        .iter_mut()
        .find(|(p, _)| p.ends_with("sugar-map.txt"))
        .unwrap()
        .1
        .push(b' ');
    assert_ne!(
        source::inputs::fingerprint(&original, source::MANIFEST_PATH).unwrap(),
        source::inputs::fingerprint(&changed, source::MANIFEST_PATH).unwrap()
    );
}
#[test]
fn source_loader_refuses_entry_and_running_byte_excess_before_retention() {
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-source-budget-{}", std::process::id()));
    std::fs::create_dir_all(root.join("crates/sugarscape-core/src")).unwrap();
    for n in 0..3 {
        std::fs::write(
            root.join(format!("crates/sugarscape-core/src/{n}.rs")),
            b"four",
        )
        .unwrap();
    }
    let by_count =
        source::inputs::read_with_test_limits(&root, source::MANIFEST_PATH, 2, 16, 64).unwrap_err();
    assert!(by_count.contains("entry limit"), "{by_count}");
    let by_total =
        source::inputs::read_with_test_limits(&root, source::MANIFEST_PATH, 10, 16, 7).unwrap_err();
    assert!(by_total.contains("aggregate byte limit"), "{by_total}");
    std::fs::remove_dir_all(root).unwrap();
}
#[test]
fn bounded_source_read_refuses_bytes_beyond_observed_metadata_budget() {
    let mut growing = std::io::Cursor::new(b"more than the inspected size");
    assert!(source::inputs::read_bounded_source(&mut growing, 8).is_err());
    assert_eq!(growing.position(), 9);
}

#[test]
fn committed_manifest_output_is_bounded_and_failed_child_is_reaped() {
    let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "war1-construction_test-git-bound-{}",
        std::process::id()
    ));
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    std::fs::create_dir_all(&root).unwrap();
    let git = |args: &[&str]| {
        let result = std::process::Command::new("git")
            .current_dir(&root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    };
    git(&["init", "--quiet"]);
    std::fs::write(root.join("small"), b"construction_test committed bytes").unwrap();
    std::fs::write(
        root.join("oversized"),
        vec![b'x'; source::inputs::MAX_FILE_BYTES + 1],
    )
    .unwrap();
    git(&["add", "."]);
    git(&[
        "-c",
        "user.name=construction_test synthetic",
        "-c",
        "user.email=construction_test@example.invalid",
        "commit",
        "--quiet",
        "-m",
        "construction_test committed byte bound",
    ]);
    assert_eq!(
        source::inputs::read_committed_manifest(&root, "small").unwrap(),
        b"construction_test committed bytes"
    );
    assert!(source::inputs::read_committed_manifest(&root, "oversized").is_err());
    assert!(source::inputs::read_committed_manifest(&root, "missing").is_err());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn gate_build_inputs_are_required_and_each_gate_dependency_changes_the_fingerprint() {
    let root = std::path::Path::new(source::ROOT);
    let entries = source::inputs::read(root, source::MANIFEST_PATH).unwrap();
    let original = source::inputs::fingerprint(&entries, source::MANIFEST_PATH).unwrap();
    for path in [
        "crates/sugarscape-core/build.rs",
        "crates/sugarscape-core/build_support/surface_discovery.rs",
        "crates/sugarscape-core/src/gate_assistance/source_inputs.rs",
        "crates/sugarscape-core/src/gate_assistance/diagnostics.rs",
        "crates/sugarscape-core/src/gate_assistance/tests/fixtures/kernel-reference.json",
        "crates/sugarscape-core/src/gate_assistance/tests/fixtures/game-reference.json",
        "crates/sugarscape-core/src/gate_assistance/tests/fixtures/limits-reference.json",
    ] {
        let index = entries.iter().position(|(p, _)| p == path).expect(path);
        let mut mutated = entries.clone();
        mutated[index].1.push(b' ');
        assert_ne!(
            source::inputs::fingerprint(&mutated, source::MANIFEST_PATH).unwrap(),
            original,
            "{path}"
        );
        if !path.contains("/src/") {
            let mut missing = entries.clone();
            missing.remove(index);
            assert!(source::inputs::fingerprint(&missing, source::MANIFEST_PATH)
                .unwrap_err()
                .contains(path));
        }
    }
}

#[test]
fn source_reader_refuses_missing_gate_host_inputs() {
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-gate-source-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let entries =
        source::inputs::read(std::path::Path::new(source::ROOT), source::MANIFEST_PATH).unwrap();
    for (path, bytes) in &entries {
        let target = root.join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, bytes).unwrap();
    }
    assert_eq!(
        source::inputs::read(&root, source::MANIFEST_PATH).unwrap(),
        entries
    );
    for path in [
        "crates/sugarscape-core/build.rs",
        "crates/sugarscape-core/build_support/surface_discovery.rs",
    ] {
        let target = root.join(path);
        let bytes = std::fs::read(&target).unwrap();
        std::fs::remove_file(&target).unwrap();
        assert!(source::inputs::read(&root, source::MANIFEST_PATH)
            .unwrap_err()
            .contains(path));
        std::fs::write(target, bytes).unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn key_source_closure_requires_references() {
    let root = std::path::Path::new(source::ROOT);
    let entries = source::inputs::read(root, source::MANIFEST_PATH).unwrap();
    let original = source::inputs::fingerprint(&entries, source::MANIFEST_PATH).unwrap();
    for path in [
        "crates/sugarscape-core/src/key_delegation/source_inputs.rs",
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/kernel-reference.json",
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/game-reference.json",
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/limits-reference.json",
    ] {
        let mut changed = entries.clone();
        changed
            .iter_mut()
            .find(|(p, _)| p == path)
            .unwrap()
            .1
            .push(b' ');
        assert_ne!(
            original,
            source::inputs::fingerprint(&changed, source::MANIFEST_PATH).unwrap()
        );
        let missing: Vec<_> = entries.iter().filter(|(p, _)| p != path).cloned().collect();
        assert!(
            source::inputs::fingerprint(&missing, source::MANIFEST_PATH).is_err(),
            "missing {path}"
        );
    }
}
