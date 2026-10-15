use super::source_identity::*;
fn entries() -> Vec<(String, Vec<u8>)> {
    FILES
        .iter()
        .map(|p| ((*p).into(), vec![1]))
        .chain(REQUIRED_SOURCES.iter().map(|p| ((*p).into(), vec![2])))
        .collect()
}
#[test]
fn stamp_is_order_independent_and_sensitive_to_bytes() {
    let mut files = entries();
    let stamp = fingerprint(files.clone().into_iter().map(Ok)).unwrap();
    files.reverse();
    assert_eq!(
        stamp,
        fingerprint(files.clone().into_iter().map(Ok)).unwrap()
    );
    files[0].1.push(4);
    assert_ne!(stamp, fingerprint(files.into_iter().map(Ok)).unwrap());
}
#[test]
fn stamp_rejects_duplicates_missing_and_unselected_inputs() {
    let mut files = entries();
    files.push(files[0].clone());
    assert!(fingerprint(files.into_iter().map(Ok)).is_err());
    let mut files = entries();
    files.remove(0);
    assert!(fingerprint(files.into_iter().map(Ok)).is_err());
    let mut files = entries();
    files.push(("../outside".into(), vec![]));
    assert!(fingerprint(files.into_iter().map(Ok)).is_err());
}

#[test]
fn stamp_requires_the_local_runner_source_files() {
    for source in [
        "survey/src/war1/mod.rs",
        "survey/src/war1/cli.rs",
        "survey/src/war1/io.rs",
        "survey/src/war1/wire.rs",
    ] {
        let entries = entries()
            .into_iter()
            .filter(|(path, _)| path != source)
            .map(Ok);
        assert!(fingerprint(entries).is_err(), "missing {source}");
    }
}
#[cfg(unix)]
#[test]
fn source_reader_rejects_symlinks() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let root = parent.join(format!("war1-source-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir_all(root.join("crates/sugarscape-core")).unwrap();
    std::os::unix::fs::symlink(&parent, root.join("crates/sugarscape-core/src")).unwrap();
    assert!(read(&root).unwrap_err().contains("symlink"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn runtime_binding_refuses_changed_source_bytes() {
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-binding-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let inputs = entries();
    for (path, bytes) in &inputs {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, bytes).unwrap();
    }
    let compiled = fingerprint(inputs.into_iter().map(Ok)).unwrap();
    verify(&root, &compiled).unwrap();
    std::fs::write(root.join("survey/src/war1/cli.rs"), b"changed after build").unwrap();
    let result = verify(&root, &compiled);
    std::fs::remove_dir_all(root).unwrap();
    assert!(result.unwrap_err().contains("changed since compilation"));
}

const MAP: &str = "crates/sugarscape-core/assets/sugar-map.txt";
const AMENDMENT: &str = "docs/superpowers/specs/2026-10-10-war-1-engineering-amendment.md";

// Removing either explicit dependency must reject an incomplete stamp.
#[test]
fn stamp_requires_embedded_map_and_engineering_amendment() {
    for source in [MAP, AMENDMENT] {
        let inputs = entries()
            .into_iter()
            .filter(|(path, _)| path != source)
            .map(Ok);
        assert!(
            fingerprint(inputs).is_err(),
            "missing {source} was accepted"
        );
    }
}

// Exercise the real reader/verifier against changed, absent and linked assets.
#[test]
fn embedded_map_changes_and_missing_map_invalidate_compiled_binding() {
    let root = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-map-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    for (path, bytes) in entries() {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, bytes).unwrap();
    }
    let map = root.join(MAP);
    std::fs::create_dir_all(map.parent().unwrap()).unwrap();
    std::fs::write(&map, b"original map").unwrap();
    let compiled = fingerprint(read(&root).unwrap().into_iter().map(Ok)).unwrap();
    verify(&root, &compiled).unwrap();
    std::fs::write(&map, b"changed map").unwrap();
    let changed = verify(&root, &compiled);
    std::fs::remove_file(&map).unwrap();
    let missing = verify(&root, &compiled);
    #[cfg(unix)]
    let linked = {
        std::os::unix::fs::symlink(root.join("Cargo.toml"), &map).unwrap();
        verify(&root, &compiled)
    };
    std::fs::remove_dir_all(root).unwrap();
    assert!(changed.unwrap_err().contains("changed since compilation"));
    assert!(missing.unwrap_err().contains("sugar-map.txt"));
    #[cfg(unix)]
    assert!(linked.unwrap_err().contains("symlink"));
}

#[test]
fn source_reader_binds_gate_host_inputs_and_rejects_missing_files() {
    let root = std::env::temp_dir().join(format!("war1-gate-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    for (path, bytes) in entries() {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, bytes).unwrap();
    }
    let added = [
        "crates/sugarscape-core/build.rs",
        "crates/sugarscape-core/build_support/surface_discovery.rs",
        "crates/sugarscape-core/src/gate_assistance/source_inputs.rs",
        "crates/sugarscape-core/src/gate_assistance/tests/fixtures/kernel-reference.json",
        "crates/sugarscape-core/src/gate_assistance/tests/fixtures/game-reference.json",
        "crates/sugarscape-core/src/gate_assistance/tests/fixtures/limits-reference.json",
    ];
    for path in added {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, b"original").unwrap();
    }
    let original = fingerprint(read(&root).unwrap().into_iter().map(Ok)).unwrap();
    for path in added {
        let file = root.join(path);
        std::fs::write(&file, b"changed").unwrap();
        assert!(verify(&root, &original).is_err(), "unbound {path}");
        std::fs::remove_file(&file).unwrap();
        assert!(read(&root).is_err(), "missing {path}");
        std::fs::write(file, b"original").unwrap();
    }
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn key_source_closure_requires_references() {
    for path in [
        "crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json",
        "crates/sugarscape-core/src/key_delegation/source_inputs.rs",
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/kernel-reference.json",
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/game-reference.json",
        "crates/sugarscape-core/src/key_delegation/tests/fixtures/limits-reference.json",
    ] {
        let inputs = entries().into_iter().filter(|(p, _)| p != path).map(Ok);
        assert!(fingerprint(inputs).is_err(), "missing {path}");
    }
}
#[test]
fn persistent_notes_protocol_is_a_bound_current_dependency() {
    let path = "docs/superpowers/protocols/2026-10-14-paid-persistent-notes.md";
    assert!(FILES.contains(&path));
    let original = entries();
    let stamp = fingerprint(original.clone().into_iter().map(Ok)).unwrap();
    let mut changed = original;
    changed
        .iter_mut()
        .find(|(p, _)| p == path)
        .unwrap()
        .1
        .push(b' ');
    assert_ne!(stamp, fingerprint(changed.into_iter().map(Ok)).unwrap());
}
