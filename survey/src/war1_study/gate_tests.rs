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
fn scientific_collection_requires_channel_before_output_even_for_internal_authorization() {
    let f = gate_fixture();
    let authorized =
        gate::fixture(&std::path::Path::new(source::ROOT).join(source::MANIFEST_PATH)).unwrap();
    let before = construction_count();
    let error = super::archive::collect(authorized, &f.output, &f.output.join("missing-socket"))
        .unwrap_err();
    assert!(error.detail.contains("monitor"));
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

#[test]
fn source_inventory_refuses_aggregate_excess_before_archive_reads() {
    let hash = wire::sha256(b"construction_test");
    let packet = gate::SourceInventory {
        schema: "war1-study-source-inventory-v1".into(),
        compiled_inputs_sha256: hash.clone(),
        entries: (0..5)
            .map(|i| gate::Artifact {
                path: format!("source/{i}.rs"),
                sha256: hash.clone(),
                bytes: 16 * 1024 * 1024,
            })
            .collect(),
    };
    assert!(packet.validate(&hash).is_err());
}
#[test]
fn construction_test_full_preflight_reaches_every_evidence_branch() {
    construction_test_full_evidence_mutations();
}
// All artifacts and Git history below are private construction_test synthetic evidence.
struct FullEvidence {
    root: PathBuf,
    executable: PathBuf,
    frozen_hash: String,
    frozen_paths: Vec<String>,
    d: gate::Declaration,
    a: gate::Acceptance,
    e: gate::Execution,
    build: serde_json::Value,
    numeric: serde_json::Value,
    resources: serde_json::Value,
    inventory: serde_json::Value,
    exported: serde_json::Value,
}
impl FullEvidence {
    fn save_bytes(&self, name: &str, bytes: &[u8]) -> gate::Artifact {
        let path = self.root.join("receipts").join(name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, bytes).unwrap();
        gate::Artifact {
            path: path.to_str().unwrap().into(),
            sha256: wire::sha256(bytes),
            bytes: bytes.len() as u64,
        }
    }
    fn save(&self, name: &str, value: &impl serde::Serialize) -> gate::Artifact {
        self.save_bytes(name, &serde_json::to_vec(value).unwrap())
    }
    fn git(&self, args: &[&str]) {
        let result = std::process::Command::new("git")
            .current_dir(&self.root)
            .args(args)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    fn new() -> Self {
        let root = std::env::temp_dir().canonicalize().unwrap().join(format!(
            "war1-construction_test-full-evidence-{}",
            std::process::id()
        ));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        std::fs::create_dir_all(&root).unwrap();
        for path in source::inputs::FILES
            .iter()
            .copied()
            .chain(source::inputs::literal_paths().unwrap())
            .chain([
                "crates/sugarscape-core/src/lib.rs",
                "survey/src/war1_study/mod.rs",
                "survey/war1_study/contracts.py",
                source::MANIFEST_PATH,
            ])
        {
            let dest = root.join(path);
            std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
            std::fs::write(
                dest,
                if path == source::MANIFEST_PATH {
                    wire::SELECTED
                } else {
                    b"construction_test synthetic source"
                },
            )
            .unwrap();
        }
        for path in source::inputs::KEY_REQUIRED {
            let dest = root.join(path);
            std::fs::create_dir_all(dest.parent().unwrap()).unwrap();
            std::fs::copy(std::path::Path::new(source::ROOT).join(path), dest).unwrap();
        }
        let entries = source::inputs::read(&root, source::MANIFEST_PATH).unwrap();
        let frozen_hash = source::inputs::fingerprint(&entries, source::MANIFEST_PATH).unwrap();
        let inventory = serde_json::json!({"schema":"war1-study-source-inventory-v1","compiled_inputs_sha256":frozen_hash,"entries":entries.iter().map(|(path,bytes)|gate::Artifact{path:path.clone(),sha256:wire::sha256(bytes),bytes:bytes.len() as u64}).collect::<Vec<_>>()});
        let (d, a, e, _, _) = synthetic_receipts();
        let executable = root.join("construction_test-native-binary");
        std::fs::write(&executable, b"construction_test synthetic executable bytes").unwrap();
        let mut f = Self {
            root,
            executable,
            frozen_hash,
            frozen_paths: entries.iter().map(|e| e.0.clone()).collect(),
            d,
            a,
            e,
            build: serde_json::Value::Null,
            numeric: serde_json::Value::Null,
            resources: serde_json::Value::Null,
            inventory,
            exported: serde_json::Value::Null,
        };
        f.git(&["init", "--quiet"]);
        f.git(&["add", "."]);
        f.git(&[
            "-c",
            "user.name=construction_test synthetic",
            "-c",
            "user.email=construction_test@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "construction_test synthetic source baseline",
        ]);
        let m = wire::decode_manifest(wire::SELECTED).unwrap();
        f.d.manifest = f.save_bytes("manifest.json", wire::SELECTED);
        f.d.binary = gate::Artifact {
            path: f.executable.to_str().unwrap().into(),
            sha256: wire::sha256(&std::fs::read(&f.executable).unwrap()),
            bytes: std::fs::metadata(&f.executable).unwrap().len(),
        };
        f.d.identities = gate::Identities {
            protocol_sha256: m.protocol_sha256.clone(),
            proposal_sha256: m.proposal_sha256.clone(),
            resolved_roster_sha256: m.roster_sha256.clone(),
            manifest_sha256: wire::sha256(wire::SELECTED),
            compiled_inputs_sha256: f.frozen_hash.clone(),
            binary_sha256: f.d.binary.sha256.clone(),
            source_inventory_sha256: String::new(),
        };
        f.build = serde_json::json!({"schema":"war1-study-build-v1","identities":f.d.identities,"selected_path":source::MANIFEST_PATH,"selected_purpose":"construction_fixture","binary":f.d.binary,"rustc":source::TOOLCHAIN,"cargo":"construction_test synthetic cargo","platform":"construction_test synthetic platform","features":source::FEATURES,"flags":source::FLAGS.iter().map(|(a,b)|[*a,*b]).collect::<Vec<_>>(),"build_command":["construction_test synthetic build"],"inventory_before":f.inventory["entries"],"inventory_after":f.inventory["entries"]});
        f.exported =
            serde_json::to_value(super::numerics::export_numeric_inputs(&m).unwrap()).unwrap();
        f.exported["compiled_inputs_sha256"] = f.frozen_hash.clone().into();
        let packet = f.save_bytes("oracle.json", b"construction_test synthetic oracle packet");
        f.numeric = serde_json::json!({"schema":"war1-study-numeric-v1","manifest_sha256":f.d.identities.manifest_sha256,"compiled_inputs_sha256":f.frozen_hash,"binary_sha256":f.d.binary.sha256,"numeric_export":packet,"oracle_packet":packet,"certificate_valid":true,"mass_valid":true,"width_valid":true,"ulp_valid":true,"ctmc_valid":true,"unresolved_findings":[],"oracle_ids":wire::approved_oracle_ids().unwrap()});
        let proof =
            super::dose_proof::proof(&f.d.identities.manifest_sha256, &f.frozen_hash).unwrap();
        f.numeric["dose_proof"] = serde_json::to_value(f.save("dose-proof.json", &proof)).unwrap();
        let forecast = f.save_bytes("forecast.json", b"construction_test synthetic forecast");
        let probe = f.save_bytes("probe.json", b"construction_test synthetic probe");
        f.resources = serde_json::json!({"schema":"war1-study-resources-v1","manifest_sha256":f.d.identities.manifest_sha256,"compiled_inputs_sha256":f.frozen_hash,"binary_sha256":f.d.binary.sha256,"budgets":f.d.budgets,"census":f.d.census,"reserved_bytes":super::io::FAILURE_RESERVE+1,"failure_reserve_bytes":super::io::FAILURE_RESERVE,"schema_bounds_valid":true,"aggregate_proof_valid":true,"forecast_valid":true,"forecast_method":"construction_test synthetic method","forecast_inputs":forecast,"forecast_wall_seconds":1.0,"forecast_peak_rss_bytes":1,"fixture_probes":[{"artifact":probe,"command":["construction_test synthetic probe"],"compiled_inputs_sha256":f.frozen_hash,"binary_sha256":f.d.binary.sha256,"wall_seconds":1.0,"peak_rss_bytes":1,"fixture_purpose":"construction_fixture"}],"unavailable":[]});
        f
    }
    fn relink(&mut self) {
        self.d.source_inventory = self.save("inventory.json", &self.inventory);
        self.d.identities.source_inventory_sha256 = self.d.source_inventory.sha256.clone();
        self.build["identities"] = serde_json::to_value(&self.d.identities).unwrap();
        self.d.build = self.save("build.json", &self.build);
        self.numeric["numeric_export"] =
            serde_json::to_value(self.save("export.json", &self.exported)).unwrap();
        self.d.numeric = self.save("numeric.json", &self.numeric);
        self.d.resources = self.save("resources.json", &self.resources);
        let declaration = self.save("declaration.json", &self.d);
        self.a.declaration_sha256 = declaration.sha256.clone();
        self.a.build_sha256 = self.d.build.sha256.clone();
        self.a.numeric_sha256 = self.d.numeric.sha256.clone();
        self.a.resources_sha256 = self.d.resources.sha256.clone();
        self.a.binary_sha256 = self.d.identities.binary_sha256.clone();
        self.a.manifest_sha256 = self.d.identities.manifest_sha256.clone();
        self.e.declaration_sha256 = declaration.sha256;
        self.e.acceptance_sha256 = self.save("acceptance.json", &self.a).sha256;
        self.e.binary_sha256 = self.d.identities.binary_sha256.clone();
        self.e.manifest_sha256 = self.d.identities.manifest_sha256.clone();
        self.e.compiled_inputs_sha256 = self.d.identities.compiled_inputs_sha256.clone();
        self.e.source_inventory_sha256 = self.d.identities.source_inventory_sha256.clone();
        self.save("execution.json", &self.e);
    }
    fn run(&self) -> Result<gate::AuthorizedCampaign, String> {
        gate::construction_test_preflight(
            source::VerificationContext::construction_test(
                &self.root,
                &self.frozen_hash,
                &self.frozen_paths,
                &self.executable,
            ),
            &self.root.join("receipts/declaration.json"),
            &self.root.join("receipts/acceptance.json"),
            &self.root.join("receipts/execution.json"),
        )
    }
}
fn construction_test_full_evidence_mutations() {
    let mut f = FullEvidence::new();
    f.relink();
    let before = construction_count();
    f.run().unwrap();
    assert_eq!(construction_count(), before);
    assert!(!f.root.join("output").exists());
    let baseline = (
        f.d.clone(),
        f.build.clone(),
        f.numeric.clone(),
        f.resources.clone(),
        f.inventory.clone(),
        f.exported.clone(),
    );
    for (kind, expected) in [
        ("protocol", "source/binary/protocol/manifest"),
        ("proposal", "source/binary/protocol/manifest"),
        ("roster", "source/binary/protocol/manifest"),
        ("manifest_identity", "source/binary/protocol/manifest"),
        ("source_identity", "source/binary/protocol/manifest"),
        ("binary_identity", "source/binary/protocol/manifest"),
        ("manifest_bytes", "manifest/inventory descriptor"),
        ("binary_bytes", "actual executable"),
        ("inventory", "frozen source inventory/path"),
        ("build", "invalid build evidence"),
        ("numeric", "numeric readiness"),
        ("dose_missing", "dose_proof"),
        ("dose_changed", "dose proof differs"),
        ("dose_extra", "unknown field"),
        ("dose_omitted_arm", "dose proof differs"),
        ("export", "numeric export differs"),
        ("resources", "resource readiness"),
        ("probe", "invalid fixture resource probe"),
    ] {
        (
            f.d,
            f.build,
            f.numeric,
            f.resources,
            f.inventory,
            f.exported,
        ) = baseline.clone();
        match kind {
            "protocol" => f.d.identities.protocol_sha256 = "0".repeat(64),
            "proposal" => f.d.identities.proposal_sha256 = "0".repeat(64),
            "roster" => f.d.identities.resolved_roster_sha256 = "0".repeat(64),
            "manifest_identity" => f.d.identities.manifest_sha256 = "0".repeat(64),
            "source_identity" => f.d.identities.compiled_inputs_sha256 = "0".repeat(64),
            "binary_identity" => f.d.identities.binary_sha256 = "0".repeat(64),
            "manifest_bytes" => {
                f.d.manifest =
                    f.save_bytes("wrong-manifest.json", b"construction_test wrong manifest")
            }
            "binary_bytes" => {
                f.d.binary = f.save_bytes("wrong-binary", b"construction_test wrong binary")
            }
            "inventory" => f.inventory["entries"][0]["sha256"] = "0".repeat(64).into(),
            "build" => f.build["rustc"] = "construction_test wrong compiler".into(),
            "numeric" => f.numeric["ulp_valid"] = false.into(),
            "dose_missing" => {
                f.numeric.as_object_mut().unwrap().remove("dose_proof");
            }
            "export" => f.exported["states"][0]["targets"][0]["q_numerator"] = 0.into(),
            "dose_changed" | "dose_extra" | "dose_omitted_arm" => {
                let mut proof = serde_json::to_value(
                    super::dose_proof::proof(&f.d.identities.manifest_sha256, &f.frozen_hash)
                        .unwrap(),
                )
                .unwrap();
                match kind {
                    "dose_changed" => proof["positive_dose_extrema"][0] = "0/1".into(),
                    "dose_extra" => proof["valid"] = true.into(),
                    _ => {
                        proof["arms"].as_array_mut().unwrap().pop();
                    }
                }
                f.numeric["dose_proof"] =
                    serde_json::to_value(f.save("wrong-dose.json", &proof)).unwrap();
            }
            "resources" => f.resources["forecast_valid"] = false.into(),
            "probe" => f.resources["fixture_probes"][0]["fixture_purpose"] = "science".into(),
            _ => unreachable!(),
        }
        f.relink();
        let error = f.run().err().expect(kind);
        assert!(error.contains(expected), "{kind}: {error}");
        assert_eq!(construction_count(), before, "{kind}");
        assert!(!f.root.join("output").exists(), "{kind}");
    }
    (
        f.d,
        f.build,
        f.numeric,
        f.resources,
        f.inventory,
        f.exported,
    ) = baseline;
    f.relink();
    for name in [
        "execution.json",
        "acceptance.json",
        "declaration.json",
        "build.json",
        "numeric.json",
        "resources.json",
    ] {
        let path = f.root.join("receipts").join(name);
        let bytes = std::fs::read(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
        let error = f.run().err().expect(name);
        if name == "execution.json" {
            assert!(error.contains("execution receipt"), "{error}");
        }
        if name == "acceptance.json" {
            assert!(error.contains("readiness receipt"), "{error}");
        }
        assert_eq!(construction_count(), before, "{name}");
        assert!(!f.root.join("output").exists(), "{name}");
        std::fs::write(path, bytes).unwrap();
    }
    let map = f.root.join("crates/sugarscape-core/assets/sugar-map.txt");
    let map_bytes = std::fs::read(&map).unwrap();
    std::fs::write(&map, b"construction_test changed map").unwrap();
    let error = f.run().err().unwrap();
    assert!(error.contains("source inputs changed"), "{error}");
    assert_eq!(construction_count(), before);
    assert!(!f.root.join("output").exists());
    std::fs::write(map, map_bytes).unwrap();
    let path = "crates/sugarscape-core/src/lib.rs";
    let original = std::fs::read(f.root.join(path)).unwrap();
    std::fs::write(f.root.join(path), b"changed source").unwrap();
    let error = f.run().err().unwrap();
    assert!(error.contains("source inputs changed"), "{error}");
    f.git(&["add", path]);
    std::fs::write(f.root.join(path), original).unwrap();
    let error = f.run().err().unwrap();
    assert!(error.contains("source-clean requirement"), "{error}");
    assert_eq!(construction_count(), before);
    assert!(!f.root.join("output").exists());
    std::fs::remove_dir_all(f.root).unwrap();
}
