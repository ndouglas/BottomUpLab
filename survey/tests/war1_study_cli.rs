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

#[test]
fn metadata_reports_actual_binary_and_build_inputs_without_collection() {
    let result = run(&["metadata"]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(value["schema"], "war1-study-metadata-v1");
    assert_eq!(value["selected_purpose"], "construction_fixture");
    assert_eq!(value["registered_histories"], 0);
    assert_eq!(
        value["inventory"]["compiled_inputs_sha256"],
        value["binding"]["compiled_inputs_sha256"]
    );
    assert_eq!(value["flags"].as_array().unwrap().len(), 7);
}

#[test]
fn parent_wait4_peak_can_veto_a_complete_candidate() {
    let code = r#"
from pathlib import Path
import tempfile
import sys
from types import SimpleNamespace
from unittest.mock import patch
from survey.war1_study import supervisor
root=Path.cwd()
with tempfile.TemporaryDirectory(prefix='construction_test_peak_',dir=Path('/tmp').resolve()) as temp:
    out=Path(temp)/'archive'
    command=(sys.argv[1],'fixture','--manifest',str(root/'survey/war1_study/fixtures/construction-manifest.json'),'--out',str(out))
    original=supervisor.os.wait4
    def unsampled_burst(pid, flags):
        waited,status,usage=original(pid,flags)
        if waited:
            usage=SimpleNamespace(ru_maxrss=2147483649 if sys.platform=='darwin' else 2097153)
        return waited,status,usage
    with patch.object(supervisor.os,'wait4',side_effect=unsampled_burst):
        result=supervisor.supervise(command,out,43200,2147483648)
    assert result['schema']=='war1-study-supervisor-failure-v1',result
    assert any('RSS' in e for e in result['errors']),result
    assert out.joinpath('candidate-index.json').is_file()
    assert not out.joinpath('success.json').exists()
    assert result['semantic_steps'] is None
    assert result['observed_steps'] is None
    assert result['final_checkpoint'] is None
    assert result['reason']=='unknown_after_termination'
"#;
    let result = Command::new("python3")
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
        .args(["-c", code, env!("CARGO_BIN_EXE_war1-study")])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn compiled_fixture_refuses_full_synthetic_scientific_manifest_before_output() {
    let code = r#"
from pathlib import Path
import subprocess,sys,tempfile
from survey.war1_study.manifest import sample_manifest,validate_manifest
root=Path.cwd()
with tempfile.TemporaryDirectory(prefix='construction_test_science_refusal_') as temp:
    directory=Path(temp).resolve()
    raw=sample_manifest(root,lambda n: (7).to_bytes(n,'little'),command=('construction_test injected scientific-form inputs only',))
    assert len(validate_manifest(raw,root).cases)==52224
    manifest=directory/'synthetic.json'; manifest.write_bytes(raw)
    output=directory/'must-not-exist'
    result=subprocess.run((sys.argv[1],'fixture','--manifest',str(manifest),'--out',str(output)),capture_output=True)
    assert result.returncode==2,(result.returncode,result.stderr)
    assert not output.exists()
"#;
    let result = Command::new("python3")
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
        .args(["-c", code, env!("CARGO_BIN_EXE_war1-study")])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn merged_core_literal_audit_matches_native_inventory() {
    let code = r#"
from pathlib import Path
import json,subprocess,sys
from survey.war1_study.registration import audit_literal_inputs,source_inventory
root=Path.cwd()
metadata=json.loads(subprocess.run((sys.argv[1],'metadata'),check=True,capture_output=True).stdout)
assert len(audit_literal_inputs(root))==253
assert source_inventory(root,metadata['selected_path'])==metadata['inventory']
"#;
    let result = Command::new("python3")
        .current_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap())
        .args(["-c", code, env!("CARGO_BIN_EXE_war1-study")])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
