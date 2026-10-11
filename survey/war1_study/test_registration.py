"""Construction-only preparation boundaries; entropy is always injected."""
import hashlib
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

    def test_literal_inventory_matches_all_core_compile_time_includes(self):
        paths = registration.audit_literal_inputs(ROOT)
        self.assertEqual(len(paths), 229)
        self.assertIn("data/anasazi/Map.txt", paths)
        self.assertIn("docs/examples/burrow/access-explore.json", paths)

    def test_inventory_matches_native_domain_and_excludes_receipts(self):
        selected = "survey/war1_study/fixtures/construction-manifest.json"
        inventory = registration.source_inventory(ROOT, selected)
        paths = {a["path"] for a in inventory["entries"]}
        self.assertIn("survey/war1_study/registration.py", paths)
        self.assertNotIn("survey/war1_study/manifests/not-selected.json", paths)
        self.assertEqual(len(inventory["compiled_inputs_sha256"]), 64)


if __name__ == "__main__":
    unittest.main()
