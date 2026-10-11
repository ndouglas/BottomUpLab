#![cfg(feature = "war-studies")]
use std::{path::PathBuf, process::Command};
fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_war1-study"))
        .args(args)
        .output()
        .unwrap()
}
fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("war1_study/fixtures/construction-manifest.json")
}
#[test]
fn fixture_validation_is_engine_free_and_collect_is_unavailable() {
    assert!(
        run(&["validate", "--manifest", fixture().to_str().unwrap()])
            .status
            .success()
    );
    assert_eq!(run(&["collect"]).status.code(), Some(2));
}
#[test]
fn numerical_export_is_source_bound_and_create_only() {
    let dir = std::env::temp_dir().join(format!("war1-numerics-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let dir = dir.canonicalize().unwrap();
    let output = dir.join("numbers.json");
    let input = fixture();
    let args = [
        "numerics",
        "--manifest",
        input.to_str().unwrap(),
        "--out",
        output.to_str().unwrap(),
    ];
    assert!(run(&args).status.success());
    let data = std::fs::read(&output).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&data).unwrap();
    assert_eq!(value["binding_state"], "source_bound");
    assert_eq!(value["compiled_inputs_sha256"].as_str().unwrap().len(), 64);
    assert_eq!(value["states"].as_array().unwrap().len(), 18);
    assert_eq!(run(&args).status.code(), Some(1));
    assert_eq!(std::fs::read(&output).unwrap(), data);
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn validation_refuses_mutations_nonregular_inputs_and_symlinks() {
    let dir = std::env::temp_dir().join(format!("war1-invalid-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let dir = dir.canonicalize().unwrap();
    let original = std::fs::read_to_string(fixture()).unwrap();
    for (index, bytes) in [
        original.replacen("\"seed\":7", "\"seed\":true", 1),
        original.replacen("\"blue\":1", "\"blue\":1,\"blue\":1", 1),
        original.replacen("\"dt\":0.015625", "\"dt\":1e-999", 1),
        original.replacen("\"schema\":", "\"unknown\":1,\"schema\":", 1),
    ]
    .iter()
    .enumerate()
    {
        let path = dir.join(format!("bad-{index}.json"));
        std::fs::write(&path, bytes).unwrap();
        assert!(!run(&["validate", "--manifest", path.to_str().unwrap()])
            .status
            .success());
    }
    assert!(!run(&["validate", "--manifest", dir.to_str().unwrap()])
        .status
        .success());
    #[cfg(unix)]
    {
        let link = dir.join("linked.json");
        std::os::unix::fs::symlink(fixture(), &link).unwrap();
        assert!(!run(&["validate", "--manifest", link.to_str().unwrap()])
            .status
            .success());
    }
    assert_eq!(
        run(&["numerics", "--out", "x", "--manifest", "y"])
            .status
            .code(),
        Some(2)
    );
    assert!(run(&["--help"]).status.success());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_execution_is_refused_before_output_creation() {
    let output = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-no-execution-{}", std::process::id()));
    let result = run(&[
        "collect",
        "--declaration",
        "missing-declaration",
        "--acceptance",
        "missing-acceptance",
        "--execution",
        "missing-execution",
        "--out",
        output.to_str().unwrap(),
    ]);
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("execution receipt"));
    assert!(!output.exists());
}
