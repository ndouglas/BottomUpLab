"""Real Python command surface; invalid inputs cannot create outputs."""
from pathlib import Path
import subprocess
from contextlib import redirect_stderr
from io import StringIO
import sys
import tempfile
import unittest
from unittest.mock import patch
from survey.war1_study import __main__, registration

ROOT = Path(__file__).resolve().parents[2]


class CliTests(unittest.TestCase):
    def test_help_and_unknown_approval_flag(self):
        help_result = subprocess.run((sys.executable, "-m", "survey.war1_study", "--help"), cwd=ROOT, capture_output=True)
        self.assertEqual(help_result.returncode, 0, help_result.stderr)
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            output = Path(temp).resolve() / "must-not-exist"
            result = subprocess.run((sys.executable, "-m", "survey.war1_study", "prepare", "--root", str(ROOT), "--out-dir", str(output), "--approved"), cwd=ROOT, capture_output=True)
            self.assertEqual(result.returncode, 2)
            self.assertFalse(output.exists())

    def test_synchronized_output_failure_is_execution_exit_one(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            destination = Path(temp).resolve() / "exists.json"
            destination.write_bytes(b"already exists")
            def output_failure(*args):
                registration.save(destination, {})
            diagnostic = StringIO()
            with patch.object(registration, "prepare", side_effect=output_failure), redirect_stderr(diagnostic):
                result = __main__.main(["prepare", "--root", str(ROOT), "--out-dir", str(destination)])
            self.assertEqual(result, 1)
            self.assertIn("output/execution", diagnostic.getvalue())
            self.assertEqual(destination.read_bytes(), b"already exists")

    def test_invalid_declaration_packet_does_not_stage_output(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            root = Path(temp).resolve()
            for name in ("build", "numeric", "resources", "source-inventory", "freeze-context"):
                (root / (name + ".json")).write_bytes(b"{}\n")
            destination = root / "new-packet/declaration.json"
            argv = ["declare", "--out", str(destination)]
            for name in ("build", "numeric", "resources"):
                argv += ["--" + name, str(root / (name + ".json"))]
            diagnostic = StringIO()
            with redirect_stderr(diagnostic):
                self.assertEqual(__main__.main(argv), 2)
            self.assertIn("freeze context", diagnostic.getvalue())
            self.assertFalse(destination.parent.exists())

    def test_real_saved_analysis_cli_creates_complete_construction_report(self):
        from copy import deepcopy
        from survey.war1_study import test_fixtures, test_oracle, manifest, archive_provenance
        from survey.war1_study.oracle import prepare_oracles
        with tempfile.TemporaryDirectory(prefix="construction_test_analysis_cli_") as temp:
            root = Path(temp).resolve()
            campaign = root/"campaign"
            m = manifest.validate_manifest(manifest.make_fixture(ROOT), ROOT)
            test_fixtures.save_campaign(campaign, m)
            manifest_path = root/"construction-manifest.json"
            manifest_path.write_bytes(m.raw_bytes)
            binding = registration.load(campaign/"header.json")["binding"]
            test_oracle.PreparedOracleTests.setUpClass()
            export = deepcopy(test_oracle.PreparedOracleTests.export)
            export.update(binding_state="source_bound", compiled_inputs_sha256=binding["compiled_inputs_sha256"])
            registration.save(root/"numeric-export.json", export)
            registration.save(root/"oracle-packet.json", archive_provenance.oracle_packet(m, binding["compiled_inputs_sha256"], prepare_oracles(m, export)))
            output = root/"analysis"
            result = subprocess.run((sys.executable, "-m", "survey.war1_study", "analyze", "--campaign", str(campaign),
                "--manifest", str(manifest_path), "--oracles", str(root/"oracle-packet.json"), "--out", str(output)), cwd=ROOT, capture_output=True)
            self.assertEqual(result.returncode, 0, result.stderr.decode())
            self.assertEqual({p.name for p in output.iterdir()}, {"summary.json", "scalars.json", "cdfs.json", "contrasts.json", "cdf_comparisons.json", "oracle_checks.json"})
            summary = registration.load(output/"summary.json")
            self.assertIs(summary["primary_available"], False)
            self.assertEqual(summary["census"]["complete_valid"], 16)

    def test_prepare_missing_resource_proof_refuses_before_output(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            output = Path(temp).resolve() / "must-not-exist"
            result = subprocess.run((sys.executable, "-m", "survey.war1_study", "prepare", "--root", str(ROOT), "--out-dir", str(output)), cwd=ROOT, capture_output=True)
            self.assertEqual(result.returncode, 2)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
