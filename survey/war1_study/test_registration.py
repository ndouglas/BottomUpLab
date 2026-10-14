"""Construction-only preparation boundaries; entropy is always injected."""
import hashlib
import re
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from survey.war1_study import registration
from survey.war1_study.io import read_regular, METADATA_LIMIT
from survey.war1_study.manifest import validate_manifest

ROOT = Path(__file__).resolve().parents[2]


class RegistrationTests(unittest.TestCase):
    def test_prepare_preserves_duplicate_draws_and_names_raw_digest(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            out = Path(temp).resolve() / "packet"
            with patch("survey.war1_study.preparation.validate_preseed", return_value={}), patch.object(registration.os, "urandom", return_value=(7).to_bytes(8, "little")) as entropy:
                path = registration.prepare(ROOT, out)
            raw = read_regular(path, METADATA_LIMIT)
            manifest = validate_manifest(raw, ROOT)
            self.assertEqual(entropy.call_count, 52224)
            self.assertEqual({c.seed for c in manifest.cases}, {7})
            self.assertEqual(path.name, "frozen-" + hashlib.sha256(raw).hexdigest() + ".json")
            with patch.object(registration.os, "urandom", side_effect=AssertionError("no second sampling")):
                with self.assertRaises(ValueError):
                    registration.prepare(ROOT, out)

    def test_missing_preseed_gate_draws_nothing(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            with patch.object(registration.os, "urandom", side_effect=AssertionError("entropy forbidden")):
                with self.assertRaises(ValueError):
                    registration.prepare(ROOT, Path(temp).resolve() / "packet")

    def test_forged_or_stale_preseed_gate_draws_nothing(self):
        from survey.war1_study import preparation
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            root = Path(temp).resolve()
            artifact = root / "stale.json"
            artifact.write_bytes(b"{}\n")
            gates = [dict(schema="forged", approved=True),
                     dict(schema="war1-study-preseed-resources-v1", purpose="construction_fixture", registered_histories=0,
                          resources=dict(path=str(artifact), sha256="0"*64, bytes=3), forecast_inputs={})]
            for gate in gates:
                with self.subTest(gate=gate), patch.object(preparation, "load", return_value=gate), patch.object(registration.os, "urandom", side_effect=AssertionError("entropy forbidden")) as entropy:
                    with self.assertRaises(ValueError):
                        registration.prepare(ROOT, root / "packet")
                    entropy.assert_not_called()
                    self.assertFalse((root / "packet").exists())

    def test_source_walk_refuses_queued_directory_growth(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            root = Path(temp).resolve()
            tree = root / "tree"
            tree.mkdir()
            for i in range(4097):
                (tree / str(i)).mkdir()
            with self.assertRaisesRegex(ValueError, "source directory bound"):
                list(registration._source_files(root, "tree"))

    def test_key_compatibility_metadata_is_a_required_source(self):
        from survey.war1_study import archive_provenance
        path = "crates/sugarscape-core/src/browser_experiments/fixtures/key-registration-compatibility.json"
        self.assertIn(path, archive_provenance.SOURCE_REQUIRED)
        inventory = registration.source_inventory(ROOT, registration.DEFAULT_MANIFEST)
        self.assertEqual(sum(e["path"] == path for e in inventory["entries"]), 1)

    def test_literal_inventory_matches_all_core_compile_time_includes(self):
        paths = registration.audit_literal_inputs(ROOT)
        self.assertEqual(len(paths), 247)
        self.assertIn("data/anasazi/Map.txt", paths)
        self.assertIn("docs/examples/burrow/access-explore.json", paths)

    def test_literal_source_macros_resolve_closed_paths_and_reject_forgery(self):
        """Only the three checked concrete macro bodies and literal calls resolve."""
        import shutil
        definitions = (
            "crates/sugarscape-core/src/browser_experiments/record.rs",
            "crates/sugarscape-core/src/browser_experiments/discovery/mod.rs",
            "crates/sugarscape-core/src/surface_discovery/diagnostics/protocol.rs",
        )
        for original in definitions:
            with self.subTest(definition=original), tempfile.TemporaryDirectory(prefix="construction_test_include_") as temp:
                root = Path(temp).resolve()
                # Use actual source bytes; mutation is confined to this temp tree.
                shutil.copytree(ROOT/"crates/sugarscape-core", root/"crates/sugarscape-core")
                for manifest in ("Cargo.toml", "Cargo.lock"):
                    shutil.copyfile(ROOT/manifest, root/manifest)
                (root/registration.LITERAL_INVENTORY).parent.mkdir(parents=True)
                (root/registration.LITERAL_INVENTORY).write_bytes((ROOT/registration.LITERAL_INVENTORY).read_bytes())
                self.assertEqual(len(registration.audit_literal_inputs(root)), 247)
                path = root/original
                text = path.read_text()
                call = next(re.finditer(r'source!\("([^"\n]+)"\)', text)).group(1)
                prefix = "../" if original.endswith("/record.rs") else "../../"
                replacements = (
                    text.replace('$path:literal', '$path:expr'),
                    text.replace('"'+prefix+'", $path', '"../../../", $path'),
                    text.replace('$path)).', '$other)).'),
                    text.replace('source!("'+call+'")', 'source!(concat!("'+call+'"))'),
                    text.replace('source!("'+call+'")', 'source!("../Cargo.toml")'),
                    text.replace('source!("'+call+'")', 'source!("/tmp/forged")'),
                    text.replace('source!("'+call+'")', 'source!("missing.rs")'),
                    text.replace('source!("'+call+'")', 'source!("shared_surface//mod.rs")'),
                    text.replace('source!("'+call+'")', 'source!("shared_surface/./mod.rs")'),
                    text.replace('source!("'+call+'")', 'source!("'+'x'*4097+'")'),
                    text + '\ninclude_bytes!(env!("FORGED"));\n',
                    text.replace('source!("'+call+'")', 'source/**/!("../forged")'),
                    text + '\ninclude_bytes/**/!("../forged");\n',
                )
                for index, forged in enumerate(replacements):
                    with self.subTest(forgery=index):
                        self.assertNotEqual(forged, text)
                        path.write_text(forged)
                        with self.assertRaises((ValueError, OSError)):
                            registration.audit_literal_inputs(root)
                path.write_text(text)
                link = root/"crates/sugarscape-core/src"/call
                link.unlink()
                link.symlink_to(ROOT/"crates/sugarscape-core/src"/call)
                with self.assertRaises(ValueError):
                    registration.audit_literal_inputs(root)

    def test_key_manifest_includes_are_exact_mandatory_paths(self):
        import shutil
        with tempfile.TemporaryDirectory(prefix="construction_test_key_include_") as temp:
            root = Path(temp).resolve()
            shutil.copytree(ROOT/"crates/sugarscape-core", root/"crates/sugarscape-core")
            for manifest in ("Cargo.toml", "Cargo.lock"):
                shutil.copyfile(ROOT/manifest, root/manifest)
            (root/registration.LITERAL_INVENTORY).parent.mkdir(parents=True)
            (root/registration.LITERAL_INVENTORY).write_bytes((ROOT/registration.LITERAL_INVENTORY).read_bytes())
            self.assertEqual(len(registration.audit_literal_inputs(root)), 247)
            path = root/"crates/sugarscape-core/src/key_delegation/source_inputs.rs"
            original = path.read_text()
            (root/"unregistered.bin").write_bytes(b"undeclared asset")
            path.write_text(original.replace('../../../../Cargo.toml', '../../../../unregistered.bin'))
            with self.assertRaisesRegex(ValueError, "literal inventory"):
                registration.audit_literal_inputs(root)
            path.write_text(original)
            for manifest in ("Cargo.toml", "Cargo.lock", "crates/sugarscape-core/Cargo.toml"):
                target = root/manifest
                raw = target.read_bytes()
                target.unlink()
                with self.subTest(missing=manifest), self.assertRaises((ValueError, OSError)):
                    registration.audit_literal_inputs(root)
                target.write_bytes(raw)

    def test_gate_host_inputs_are_mandatory(self):
        for path in ("crates/sugarscape-core/build.rs", "crates/sugarscape-core/build_support/surface_discovery.rs"):
            with self.subTest(path=path):
                self.assertIn(path, registration.SOURCE_REQUIRED)

    def test_gate_generated_include_accepts_only_the_exact_bound_output(self):
        import shutil
        with tempfile.TemporaryDirectory(prefix="construction_test_gate_include_") as temp:
            root = Path(temp).resolve()
            shutil.copytree(ROOT/"crates/sugarscape-core", root/"crates/sugarscape-core")
            for manifest in ("Cargo.toml", "Cargo.lock"):
                shutil.copyfile(ROOT/manifest, root/manifest)
            (root/registration.LITERAL_INVENTORY).parent.mkdir(parents=True)
            (root/registration.LITERAL_INVENTORY).write_bytes((ROOT/registration.LITERAL_INVENTORY).read_bytes())
            self.assertEqual(len(registration.audit_literal_inputs(root)), 247)
            path = root/"crates/sugarscape-core/src/gate_assistance/diagnostics.rs"
            original = path.read_text()
            expression = 'include!(concat!(env!("OUT_DIR"), "/gate_source_receipts.rs"))'
            for replacement in (expression.replace("OUT_DIR", "OTHER_DIR"),
                                expression.replace("gate_source_receipts.rs", "forged.rs"),
                                expression.replace("/gate_source", "/../gate_source"),
                                expression + ";" + expression):
                with self.subTest(replacement=replacement):
                    path.write_text(original.replace(expression, replacement))
                    with self.assertRaises(ValueError):
                        registration.audit_literal_inputs(root)
            path.write_text(original)
            for name in ("build.rs", "build_support/surface_discovery.rs"):
                target = root/"crates/sugarscape-core"/name
                raw = target.read_bytes()
                target.unlink()
                with self.subTest(missing=name), self.assertRaises((ValueError, OSError)):
                    registration.audit_literal_inputs(root)
                target.write_bytes(raw)

    def test_inventory_matches_native_domain_and_excludes_receipts(self):
        selected = "survey/war1_study/fixtures/construction-manifest.json"
        inventory = registration.source_inventory(ROOT, selected)
        paths = {a["path"] for a in inventory["entries"]}
        self.assertIn("survey/war1_study/registration.py", paths)
        self.assertNotIn("survey/war1_study/manifests/not-selected.json", paths)
        self.assertEqual(len(inventory["compiled_inputs_sha256"]), 64)


class SyntheticPacketWorkflowTests(unittest.TestCase):
    def test_freeze_build_declare_and_all_required_mutations(self):
        """Real Git and receipt validators; only executable metadata is synthetic."""
        import os
        import subprocess
        from copy import deepcopy
        from contextlib import ExitStack
        from survey.war1_study import preparation, archive_provenance as provenance, manifest as manifests
        from survey.war1_study import test_oracle
        from survey.war1_study.oracle import prepare_oracles
        from survey.war1_study.test_preparation import synthetic_resources
        with tempfile.TemporaryDirectory(prefix="construction_test_registration_") as temp:
            root = Path(temp).resolve()
            literals = provenance.literal_paths((ROOT/provenance.LITERAL_INVENTORY).read_bytes())
            for relative in set(provenance.SOURCE_REQUIRED) | set(literals) | {registration.DEFAULT_MANIFEST}:
                target = root/relative
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes((ROOT/relative).read_bytes())
            core = root/"crates/sugarscape-core/src/lib.rs"
            core.write_text("// construction_test synthetic include closure; never compiled\n" + "\n".join(
                'const _: &str = include_str!("' + os.path.relpath(root/path, core.parent) + '");' for path in literals))
            raw = manifests.sample_manifest(root, lambda n: (7).to_bytes(n, "little"), command=("construction_test injected seeds only",))
            selected = "survey/war1_study/manifests/construction_test_synthetic.json"
            manifest_path = root/selected
            manifest_path.parent.mkdir(parents=True)
            manifest_path.write_bytes(raw)
            m = manifests.validate_manifest(raw, root)
            def git(*args):
                return subprocess.run(("git", "-C", str(root), *args), check=True, capture_output=True).stdout.decode().strip()
            git("init")
            git("config", "user.name", "construction_test synthetic author")
            git("config", "user.email", "construction-test@example.invalid")
            git("add", ".")
            git("commit", "-m", "construction_test normative fixture")
            frozen_revision = git("rev-parse", "HEAD")
            packet = root/"survey/war1_study/preparation/construction_test_packet"
            packet.mkdir(parents=True)
            inventory = registration.freeze(root, manifest_path, packet/"source-inventory.json")
            binary = packet/"construction_test_scientific_binary"
            binary.write_bytes(b"construction_test synthetic executable; never launched")
            flags = [[k, "release" if k == "PROFILE" else ""] for k in (
                "RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "TARGET", "HOST", "PROFILE", "OPT_LEVEL", "DEBUG")]
            def metadata(purpose, path, inv, digest, manifest_hash):
                return dict(schema="war1-study-metadata-v1", root=str(root), selected_path=path, selected_purpose=purpose,
                    registered_histories=0, reference_calls_checked=264, inventory=inv, rustc="construction_test rustc",
                    cargo="construction_test cargo", platform="construction_test platform", features=["CARGO_FEATURE_WAR_STUDIES"], flags=flags,
                    binding=dict(manifest_sha256=manifest_hash, compiled_inputs_sha256=inv["compiled_inputs_sha256"], executable_sha256=digest,
                        declaration_sha256=None, acceptance_sha256=None, execution_approval_sha256=None))
            science = metadata(m.purpose, selected, inventory, registration.artifact(binary)["sha256"], m.sha256)
            fixture_inventory = registration.source_inventory(root, registration.DEFAULT_MANIFEST)
            fixture = metadata("construction_fixture", registration.DEFAULT_MANIFEST, fixture_inventory,
                hashlib.sha256(b"construction_test fake executable, never launched").hexdigest(),
                hashlib.sha256((root/registration.DEFAULT_MANIFEST).read_bytes()).hexdigest())
            fixture, resources, inputs, probe = synthetic_resources(packet/"resource-evidence", science, fixture)
            def native(path):
                return deepcopy(science if Path(path) == binary else fixture)
            with ExitStack() as stack:
                stack.enter_context(patch.object(registration, "native_metadata", side_effect=native))
                stack.enter_context(patch.object(preparation, "native_metadata", side_effect=native))
                build = registration.build_receipt(root, m, inventory, binary,
                    ("construction_test", "synthetic build; no native process"), packet/"build.json")
                test_oracle.PreparedOracleTests.setUpClass()
                export = deepcopy(test_oracle.PreparedOracleTests.export)
                export.update(purpose=m.purpose, manifest_sha256=m.sha256, binding_state="source_bound", compiled_inputs_sha256=inventory["compiled_inputs_sha256"])
                registration.save(packet/"numeric-export.json", export)
                oracles = prepare_oracles(m, export)
                from survey.war1_study.dose_proof import dose_proof
                registration.save(packet/"dose-proof.json", dose_proof(root, m.sha256, inventory["compiled_inputs_sha256"]))
                registration.save(packet/"oracle-packet.json", provenance.oracle_packet(m, inventory["compiled_inputs_sha256"], oracles))
                numeric = dict(schema="war1-study-numeric-v1", manifest_sha256=m.sha256,
                    compiled_inputs_sha256=inventory["compiled_inputs_sha256"], binary_sha256=build["binary"]["sha256"],
                    dose_proof=registration.artifact(packet/"dose-proof.json"),
                    numeric_export=registration.artifact(packet/"numeric-export.json"), oracle_packet=registration.artifact(packet/"oracle-packet.json"),
                    certificate_valid=True, mass_valid=True, width_valid=True, ulp_valid=True, ctmc_valid=True,
                    unresolved_findings=[], oracle_ids=[o.arm_id for o in oracles])
                registration.save(packet/"numeric.json", numeric)
                registration.save(packet/"resources.json", resources)
                git("add", ".")
                git("commit", "-m", "construction_test excluded preparation receipts")
                declaration_revision = git("rev-parse", "HEAD")
                declaration = registration.declare(build, numeric, resources, packet/"declaration.json")
                self.assertEqual(declaration["binary"], registration.artifact(binary))
                self.assertEqual(declaration["normative_frozen_revision"], frozen_revision)
                self.assertEqual(declaration["declaration_revision"], declaration_revision)
                self.assertNotEqual(frozen_revision, declaration_revision)
                self.assertEqual(declaration["registered_histories"], 0)
                self.assertFalse(any("preparation/" in entry["path"] for entry in inventory["entries"]))
                self.assertIn(selected, {entry["path"] for entry in inventory["entries"]})
                for name, path in (("source", core), ("map", root/"crates/sugarscape-core/assets/sugar-map.txt"),
                                   ("manifest", manifest_path), ("binary", binary)):
                    with self.subTest(mutation=name):
                        original = path.read_bytes()
                        path.write_bytes(original+b"\nconstruction_test mutation")
                        try:
                            output = packet/(name+"-declaration.json")
                            with self.assertRaises(ValueError):
                                registration.declare(build, numeric, resources, output)
                            self.assertFalse(output.exists())
                            with self.assertRaises(ValueError):
                                registration.build_receipt(root, m, inventory, binary, ("construction_test",), packet/(name+"-build.json"))
                            self.assertFalse((packet/(name+"-build.json")).exists())
                        finally:
                            path.write_bytes(original)
                for name in ("missing_certificate", "missing_forecast", "wrong_numeric_envelope", "missing_dose", "changed_dose"):
                    with self.subTest(mutation=name):
                        bad_numeric, bad_resources = deepcopy(numeric), deepcopy(resources)
                        changed = {}
                        if name == "missing_certificate":
                            bad_numeric.pop("certificate_valid")
                        elif name == "missing_forecast":
                            bad_resources.pop("forecast_inputs")
                        elif name == "missing_dose":
                            bad_numeric.pop("dose_proof")
                        elif name == "changed_dose":
                            target = packet/"dose-proof.json"
                            changed[target] = target.read_bytes()
                            bad = registration.load(target)
                            bad["arms"].pop()
                            target.unlink()
                            registration.save(target, bad)
                            bad_numeric["dose_proof"] = registration.artifact(target)
                        else:
                            target = packet/"numeric-export.json"
                            changed[target] = target.read_bytes()
                            target.unlink()
                            registration.save(target, dict(export, purpose="construction_fixture"))
                            bad_numeric["numeric_export"] = registration.artifact(target)
                        for target, value in ((packet/"numeric.json", bad_numeric), (packet/"resources.json", bad_resources)):
                            changed[target] = target.read_bytes()
                            target.unlink()
                            registration.save(target, value)
                        try:
                            output = packet/(name+"-declaration.json")
                            with self.assertRaises((ValueError, KeyError)):
                                registration.declare(build, bad_numeric, bad_resources, output)
                            self.assertFalse(output.exists())
                        finally:
                            for path, original in changed.items():
                                path.write_bytes(original)


if __name__ == "__main__":
    unittest.main()
