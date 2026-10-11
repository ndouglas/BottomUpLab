"""Real Python command surface; invalid inputs cannot create outputs."""
from pathlib import Path
import subprocess
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
            with patch.object(registration, "prepare", side_effect=output_failure):
                result = __main__.main(["prepare", "--root", str(ROOT), "--out-dir", str(destination)])
            self.assertEqual(result, 1)
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
            self.assertEqual(__main__.main(argv), 2)
            self.assertFalse(destination.parent.exists())

    def test_prepare_missing_resource_proof_refuses_before_output(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            output = Path(temp).resolve() / "must-not-exist"
            result = subprocess.run((sys.executable, "-m", "survey.war1_study", "prepare", "--root", str(ROOT), "--out-dir", str(output)), cwd=ROOT, capture_output=True)
            self.assertEqual(result.returncode, 2)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
