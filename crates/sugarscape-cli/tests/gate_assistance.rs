use std::process::Command;
#[test]
fn diagnose_help_exposes_no_scientific_settings() {
    let out = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
        .args(["gate-assistance", "diagnose", "--help"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let help = String::from_utf8(out.stdout).unwrap();
    assert!(help.contains("gate-assistance diagnose"));
    assert!(!help.contains("--seed"));
}
#[test]
fn diagnose_rejects_scientific_flags_and_extra_arguments() {
    for extra in ["--seed", "--holding-cost", "--limits", "--out", "extra"] {
        let out = Command::new(env!("CARGO_BIN_EXE_sugarscape"))
            .args(["gate-assistance", "diagnose", extra])
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
    }
}
