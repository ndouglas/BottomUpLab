use super::{archive, source};
#[test]
fn fixture_archive_has_exact_population_candidate_and_no_success_marker() {
    let parent = std::env::temp_dir().canonicalize().unwrap();
    let output = parent.join(format!("war1-construction-archive-{}", std::process::id()));
    if output.exists() {
        std::fs::remove_dir_all(&output).unwrap();
    }
    let receipt = archive::collect_fixture(
        &std::path::Path::new(source::ROOT).join(source::MANIFEST_PATH),
        &output,
    )
    .unwrap();
    assert_eq!(receipt.acknowledged_cases, 16);
    assert!(output.join("candidate-index.json").exists());
    assert!(!output.join("success.json").exists());
    archive::construction_test_validate_saved(&output).unwrap();
    let header_path = output.join("header.json");
    let original = std::fs::read(&header_path).unwrap();
    for (key, value) in [
        ("schema", "unsupported"),
        ("mode", "prospective_seed_population"),
    ] {
        let mut header: serde_json::Value = serde_json::from_slice(&original).unwrap();
        header[key] = value.into();
        std::fs::write(&header_path, serde_json::to_vec(&header).unwrap()).unwrap();
        let error = archive::construction_test_validate_saved(&output).unwrap_err();
        assert!(error.contains("header/manifest"), "{error}");
    }
    std::fs::write(header_path, original).unwrap();
    assert!(archive::collect_fixture(
        &std::path::Path::new(source::ROOT).join(source::MANIFEST_PATH),
        &output
    )
    .is_err());
    std::fs::remove_dir_all(output).unwrap();
}
#[test]
fn injected_durable_start_and_outcome_failures_preserve_prefix_and_clocks() {
    use super::{
        gate_tests::construction_count,
        io_tests::{set_faults, Effect},
    };
    for (event, nth, effect) in [
        ("write", 1, Effect::Partial),
        ("flush", 1, Effect::Fail),
        ("sync", 1, Effect::Fail),
        ("write", 2, Effect::Partial),
        ("flush", 2, Effect::Fail),
        ("sync", 2, Effect::Fail),
    ] {
        let out = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "war1-injection-{}-{event}-{nth}",
            std::process::id()
        ));
        if out.exists() {
            std::fs::remove_dir_all(&out).unwrap();
        }
        set_faults(vec![("blocks/block-000000.jsonl", event, nth, effect)]);
        let before = construction_count();
        let failure = archive::collect_fixture(
            &std::path::Path::new(source::ROOT).join(source::MANIFEST_PATH),
            &out,
        )
        .unwrap_err();
        set_faults(vec![]);
        assert_eq!(failure.census.attempted, 1);
        assert_eq!(failure.census.partial, 1);
        assert_eq!(failure.census.pending, 0);
        assert_eq!(failure.census.unstarted, 15);
        assert_eq!(failure.receipt.acknowledged_cases, 0);
        assert_eq!(construction_count() - before, usize::from(nth == 2));
        let last = failure.case_failure.unwrap();
        assert_eq!(
            last.semantic_steps,
            last.final_checkpoint.as_ref().map_or(0, |p| p.step)
        );
        if nth == 1 {
            assert_eq!(last.semantic_steps, 0);
            assert_eq!(failure.receipt.bytes, 0);
        } else {
            assert!(last.semantic_steps > 0);
            assert_eq!(last.semantic_steps, last.observed_steps);
            assert!(failure.receipt.bytes > 0);
        }
        assert!(!out.join("candidate-index.json").exists());
        assert!(!out.join("success.json").exists());
        // The construction producer invokes the saved-only reader, never the
        // reverse. Exercise bytes left by actual native write/flush/sync faults.
        let checked = std::process::Command::new("python3")
            .current_dir(source::ROOT)
            .arg("-c")
            .arg(
                r#"
from pathlib import Path
import sys
from survey.war1_study import archive, manifest
out = Path(sys.argv[1])
m = manifest.validate_manifest((out / 'manifest.json').read_bytes(), Path.cwd())
saved = archive.read_campaign(out, m)
assert saved.census['complete_valid'] == 0, saved.census
assert saved.census['partial'] == 1, saved.census
assert saved.census['unstarted'] == 15, saved.census
assert list(saved.outcomes()) == [], saved.census
errors = saved.header['validation']['errors']
assert not any(e.startswith('failure:') for e in errors), errors
"#,
            )
            .arg(&out)
            .output()
            .expect("python3 must be available for saved-reader integration");
        assert!(
            checked.status.success(),
            "saved reader failed for {event}/{nth}: stdout={} stderr={}",
            String::from_utf8_lossy(&checked.stdout),
            String::from_utf8_lossy(&checked.stderr)
        );
        std::fs::remove_dir_all(out).unwrap();
    }
}
#[test]
fn final_marker_parent_sync_failure_revokes_candidate() {
    use super::io_tests::{set_faults, Effect};
    let out = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-revoke-{}", std::process::id()));
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    set_faults(vec![(
        "candidate-index.json",
        "parent_sync",
        1,
        Effect::Fail,
    )]);
    let f = archive::collect_fixture(
        &std::path::Path::new(source::ROOT).join(source::MANIFEST_PATH),
        &out,
    )
    .unwrap_err();
    set_faults(vec![]);
    assert_eq!(f.census.complete_valid, 16);
    assert_eq!(f.receipt.acknowledged_cases, 16);
    assert!(!out.join("candidate-index.json").exists());
    assert!(!out.join("success.json").exists());
    assert!(out.join("failure.json").exists());
    std::fs::remove_dir_all(out).unwrap();
}
#[test]
fn failure_receipt_failure_remains_visible_with_original_prefix() {
    use super::io_tests::{set_faults, Effect};
    let out = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-double-failure-{}", std::process::id()));
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    set_faults(vec![
        ("blocks/block-000000.jsonl", "write", 1, Effect::Partial),
        ("failure.json", "sync", 1, Effect::Fail),
    ]);
    let f = archive::collect_fixture(
        &std::path::Path::new(source::ROOT).join(source::MANIFEST_PATH),
        &out,
    )
    .unwrap_err();
    set_faults(vec![]);
    assert!(f.detail.contains("failure receipt also failed"));
    assert_eq!(f.receipt.bytes, 0);
    assert_eq!(f.census.partial, 1);
    assert!(f.case_failure.is_some());
    std::fs::remove_dir_all(out).unwrap();
}
#[test]
fn audited_frame_sync_failure_retains_real_settlement_without_acknowledgment() {
    use super::io_tests::{set_faults, Effect};
    let out = std::env::temp_dir()
        .canonicalize()
        .unwrap()
        .join(format!("war1-audit-failure-{}", std::process::id()));
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    set_faults(vec![("audits/0.jsonl", "sync", 3, Effect::Fail)]);
    let f = archive::collect_fixture(
        &std::path::Path::new(source::ROOT).join(source::MANIFEST_PATH),
        &out,
    )
    .unwrap_err();
    set_faults(vec![]);
    let last = f.case_failure.unwrap();
    assert_eq!(
        (
            last.semantic_steps,
            last.observed_steps,
            last.durable_audit_steps
        ),
        (1, 1, 0)
    );
    assert_eq!(last.final_checkpoint.unwrap().step, 1);
    assert_eq!(f.census.partial, 1);
    assert_eq!(f.census.failed, 0);
    std::fs::remove_dir_all(out).unwrap();
}
