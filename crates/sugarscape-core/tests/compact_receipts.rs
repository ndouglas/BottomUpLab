#[path = "../build_support/compact_receipts.rs"]
mod compact_receipts;
#[path = "../build_support/surface_discovery.rs"]
mod surface_discovery;
#[test]
fn raw_receipts_change_when_input_changes() {
    assert_ne!(
        compact_receipts::entry("a", b"a"),
        compact_receipts::entry("a", b"b")
    );
    assert!(compact_receipts::entry("a", b"abc")
        .contains("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"));
}
#[test]
fn selected_projections_preserve_every_consumed_field_exactly() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let out = std::env::temp_dir().join(format!("selected-receipts-{}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    compact_receipts::generate(&root.canonicalize().unwrap(), &out).unwrap();
    for (name, required) in [
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
        let full: serde_json::Value = serde_json::from_slice(
            &std::fs::read(root.join(format!(
                "crates/sugarscape-core/src/browser_experiments/fixtures/{name}-recorded.json"
            )))
            .unwrap(),
        )
        .unwrap();
        let selected: serde_json::Value = serde_json::from_slice(
            &std::fs::read(out.join(format!("{name}-selected.json"))).unwrap(),
        )
        .unwrap();
        let actual_keys: std::collections::BTreeSet<_> = selected
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            actual_keys,
            required.iter().copied().collect(),
            "{name} required fields"
        );
        for (field, value) in selected.as_object().unwrap() {
            assert_eq!(value, &full[field], "{name}/{field}");
        }
        assert_eq!(selected["provenance"], full["provenance"]);
    }
    std::fs::remove_dir_all(out).unwrap();
}
