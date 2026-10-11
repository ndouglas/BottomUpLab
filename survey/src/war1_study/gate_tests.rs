//! Synthetic syntax/link tests only. None represents an actual review or owner action.
use super::{gate, observer, source, wire};
use std::path::PathBuf;
pub struct GateFixture {
    pub declaration: PathBuf,
    pub acceptance: PathBuf,
    pub missing_execution: PathBuf,
    pub output: PathBuf,
}
pub fn construction_count() -> usize {
    observer::construction_count()
}
fn synthetic_receipts() -> (
    gate::Declaration,
    gate::Acceptance,
    gate::Execution,
    Vec<u8>,
    Vec<u8>,
) {
    let hash = wire::sha256(b"construction_test synthetic non-scientific artifact");
    let descriptor = gate::Artifact {
        path: "construction_test-synthetic-artifact".into(),
        sha256: hash.clone(),
        bytes: 51,
    };
    let identities = gate::Identities {
        protocol_sha256: hash.clone(),
        proposal_sha256: hash.clone(),
        manifest_sha256: hash.clone(),
        compiled_inputs_sha256: hash.clone(),
        source_inventory_sha256: hash.clone(),
        resolved_roster_sha256: hash.clone(),
        binary_sha256: hash.clone(),
    };
    let d = gate::Declaration {
        schema: "war1-study-declaration-v1".into(),
        purpose: "prospective_frozen_not_blind".into(),
        identities,
        build: descriptor.clone(),
        numeric: descriptor.clone(),
        resources: descriptor.clone(),
        source_inventory: descriptor.clone(),
        manifest: descriptor.clone(),
        binary: descriptor,
        normative_frozen_revision: "1".repeat(40),
        declaration_revision: "2".repeat(40),
        registered_histories: 0,
        census: gate::Population::required(),
        budgets: gate::Limits::required(),
    };
    let db = serde_json::to_vec(&d).unwrap();
    let a = gate::Acceptance {
        schema: "war1-study-readiness-v1".into(),
        declaration_sha256: wire::sha256(&db),
        reviewer_identity: "construction_test synthetic reviewer".into(),
        reviewer_role: "independent_reviewer".into(),
        decision: "accepted".into(),
        registered_histories: 0,
        open_required_findings: vec![],
        build_sha256: hash.clone(),
        numeric_sha256: hash.clone(),
        resources_sha256: hash.clone(),
        binary_sha256: hash.clone(),
        manifest_sha256: hash.clone(),
    };
    let ab = serde_json::to_vec(&a).unwrap();
    let e = gate::Execution {
        schema: "war1-study-execution-v1".into(),
        declaration_sha256: wire::sha256(&db),
        acceptance_sha256: wire::sha256(&ab),
        owner_identity: "construction_test synthetic owner".into(),
        action: "execute_exact_population".into(),
        census: gate::Population::required(),
        budgets: gate::Limits::required(),
        authorization_quote:
            "construction_test synthetic syntax only; no scientific execution authorized".into(),
        authorized_at: "construction_test synthetic time".into(),
        compiled_inputs_sha256: hash.clone(),
        source_inventory_sha256: hash.clone(),
        binary_sha256: hash.clone(),
        manifest_sha256: hash,
    };
    (d, a, e, db, ab)
}
pub fn gate_fixture() -> GateFixture {
    let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
        "war1-construction_test-synthetic-receipts-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let declaration = root.join("declaration.json");
    let acceptance = root.join("acceptance.json");
    let (_, _, _, db, ab) = synthetic_receipts();
    std::fs::write(&declaration, db).unwrap();
    std::fs::write(&acceptance, ab).unwrap();
    GateFixture {
        declaration,
        acceptance,
        missing_execution: root.join("missing-execution.json"),
        output: root.join("must-not-exist"),
    }
}
#[test]
fn valid_looking_declaration_without_execution_receipt_constructs_nothing() {
    let fixture = gate_fixture();
    let before = construction_count();
    let error = gate::preflight(
        &fixture.declaration,
        &fixture.acceptance,
        &fixture.missing_execution,
    )
    .err()
    .unwrap();
    assert!(error.contains("execution receipt"), "{error}");
    assert_eq!(construction_count(), before);
    assert!(!fixture.output.exists());
}
#[test]
fn receipt_link_validator_refuses_unbound_approval_and_prior_histories() {
    let (d, a, e, db, ab) = synthetic_receipts();
    gate::validate_receipt_links(&d, &a, &e, &db, &ab).unwrap();
    for kind in 0..9 {
        let mut d = d.clone();
        let mut a = a.clone();
        let mut e = e.clone();
        match kind {
            0 => e.action = "approved".into(),
            1 => a.decision = "ready".into(),
            2 => d.registered_histories = 1,
            3 => a.registered_histories = 1,
            4 => a.reviewer_identity = e.owner_identity.clone(),
            5 => e.binary_sha256 = "0".repeat(64),
            6 => e.compiled_inputs_sha256 = "0".repeat(64),
            7 => e.manifest_sha256 = "0".repeat(64),
            _ => a.schema = "unknown".into(),
        };
        assert!(
            gate::validate_receipt_links(&d, &a, &e, &db, &ab).is_err(),
            "mutation {kind}"
        );
    }
}
#[test]
fn complete_synthetic_receipt_set_still_cannot_activate_compiled_fixture() {
    let f = gate_fixture();
    let (_, _, e, _, _) = synthetic_receipts();
    let execution = f
        .missing_execution
        .with_file_name("synthetic-execution.json");
    std::fs::write(&execution, serde_json::to_vec(&e).unwrap()).unwrap();
    let before = construction_count();
    let error = gate::preflight(&f.declaration, &f.acceptance, &execution)
        .err()
        .unwrap();
    assert!(error.contains("construction fixture"), "{error}");
    assert_eq!(construction_count(), before);
    assert!(!f.output.exists());
}
#[test]
fn scientific_collection_is_unready_before_output_even_for_internal_authorization() {
    let f = gate_fixture();
    let authorized =
        gate::fixture(&std::path::Path::new(source::ROOT).join(source::MANIFEST_PATH)).unwrap();
    let before = construction_count();
    let error = super::archive::collect(authorized, &f.output).unwrap_err();
    assert!(error.detail.contains("supervisor"));
    assert_eq!(construction_count(), before);
    assert!(!f.output.exists());
}
#[test]
fn source_inventory_requires_object_schema_digest_and_ordered_entries() {
    let hash = wire::sha256(b"construction_test inventory");
    let entries = vec![gate::Artifact {
        path: "Cargo.toml".into(),
        sha256: hash.clone(),
        bytes: 1,
    }];
    let packet = gate::SourceInventory {
        schema: "war1-study-source-inventory-v1".into(),
        compiled_inputs_sha256: hash.clone(),
        entries: entries.clone(),
    };
    packet.validate(&hash).unwrap();
    assert!(packet.validate(&"0".repeat(64)).is_err());
    assert!(serde_json::from_slice::<gate::SourceInventory>(
        &serde_json::to_vec(&entries).unwrap()
    )
    .is_err());
    let mut bad = packet;
    bad.entries.push(entries[0].clone());
    assert!(bad.validate(&hash).is_err());
}
