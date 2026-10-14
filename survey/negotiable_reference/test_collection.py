"""Collector behavior at the private fake-child boundary; no survey execution."""

import copy
import hashlib
import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import unittest

from survey.negotiable_reference import collection as c, reference as r

ROOT = Path(__file__).resolve().parents[2]
FIXTURES = ROOT / "crates/sugarscape-core/src/negotiable/tests/fixtures"
SOURCE = "dede454af44b33817fcf14cbd8688633abfcd1c901e744c95c4ad676a6debf70"


def report():
    rows = []
    hashes = {k: hashlib.sha256((FIXTURES / f"{k}-reference.json").read_bytes()).hexdigest()
              for k in r.ROW_COUNTS}
    for kind in r.ROW_COUNTS:
        record = json.loads((FIXTURES / f"{kind}-reference.json").read_bytes())
        record.update(scope="finite_exact_foundation", binding={
            "main_pdf_sha256": r.SOURCE_HASHES["main_pdf"],
            "supplement_pdf_sha256": r.SOURCE_HASHES["supplement_pdf"],
            "reference_sha256": hashes,
            "settings_sha256": hashlib.sha256(r.canonical_bytes(record["settings"])).hexdigest(),
            "compiled_source_sha256": SOURCE})
        rows.append(dict(id=f"negotiable.{kind}", item="negotiable", source="Book",
                         citation="finite source", text="finite check", verdict="Holds",
                         measured=json.dumps(record), detail="finite foundation", seconds=0.1))
    return rows


class ValidationTests(unittest.TestCase):
    def test_exact_census_is_retained(self):
        result = c.validate_report(json.dumps(report()).encode(), SOURCE)
        self.assertEqual(result["row_counts"], {"cake": 225, "pareto": 9, "priority": 4599})

    def test_timing_differences_do_not_change_canonical_math(self):
        rows = report()
        first = c.validate_report(json.dumps(rows).encode(), SOURCE)
        rows[0]["seconds"] = 1.2
        self.assertEqual(first, c.validate_report(json.dumps(rows).encode(), SOURCE))

    def test_duplicate_claim_is_rejected(self):
        rows = report()
        rows[1] = rows[0]
        with self.assertRaisesRegex(ValueError, "claim"):
            c.validate_report(json.dumps(rows).encode(), SOURCE)

    def test_unknown_claim_is_rejected(self):
        rows = report()
        rows[1]["id"] = "negotiable.extra"
        with self.assertRaisesRegex(ValueError, "claim"):
            c.validate_report(json.dumps(rows).encode(), SOURCE)

    def test_missing_row_is_rejected(self):
        rows = report()
        rec = json.loads(rows[2]["measured"])
        rec["rows"].pop()
        rows[2]["measured"] = json.dumps(rec)
        with self.assertRaisesRegex(ValueError, "4599"):
            c.validate_report(json.dumps(rows).encode(), SOURCE)

    def test_unreduced_fraction_is_rejected(self):
        rows = report()
        rec = json.loads(rows[0]["measured"])
        rec["rows"][0]["weight"] = {"num": "0", "den": "2"}
        rows[0]["measured"] = json.dumps(rec)
        with self.assertRaisesRegex(ValueError, "reduced"):
            c.validate_report(json.dumps(rows).encode(), SOURCE)

    def test_binding_all_references_and_source_are_checked(self):
        for field in ("compiled_source_sha256", "settings_sha256", "main_pdf_sha256", "reference_sha256"):
            with self.subTest(field=field):
                rows = report()
                rec = json.loads(rows[0]["measured"])
                rec["binding"][field] = "wrong"
                rows[0]["measured"] = json.dumps(rec)
                with self.assertRaisesRegex(ValueError, "binding"):
                    c.validate_report(json.dumps(rows).encode(), SOURCE)

    def test_error_is_never_collection_acceptance(self):
        rows = report()
        rows[1].update(verdict="Error", measured="")
        result = c.validate_report(json.dumps(rows).encode(), SOURCE)
        self.assertEqual((result["complete"], result["verdicts"]["negotiable.pareto"]), (False, "Error"))

    def test_fails_is_retained_with_gap_without_false_pass(self):
        rows = report()
        rows[0].update(verdict="Fails", measured="")
        result = c.validate_report(json.dumps(rows).encode(), SOURCE)
        self.assertEqual((result["complete"], result["gaps"]), (False, ["negotiable.cake"]))
        self.assertEqual(result["verdicts"]["negotiable.cake"], "Fails")

    def test_empty_holds_and_unknown_verdict_are_rejected(self):
        for changes in ({"measured": ""}, {"verdict": "Pass"}, {"seconds": True}):
            rows = report()
            rows[0].update(changes)
            with self.assertRaises(ValueError):
                c.validate_report(json.dumps(rows).encode(), SOURCE)

    def test_malformed_metadata_and_huge_timing_are_validation_errors(self):
        for changes in ({"source": []}, {"seconds": 10**1000}):
            rows = report()
            rows[0].update(changes)
            with self.assertRaises(ValueError):
                c.validate_report(json.dumps(rows).encode(), SOURCE)

    def test_duplicate_json_and_truncated_json_are_rejected(self):
        for data in (b'[{"id":"a","id":"b"}]', b'[{', b'[NaN]'):
            with self.assertRaises(ValueError):
                c.validate_report(data, SOURCE)

    def test_packing_counts_both_raw_copies_and_escaping(self):
        pack = c.packing_certificate({"inputs": ["a" * 1000]})
        self.assertGreater(pack["outer_report_bytes"], 1524291)
        self.assertGreaterEqual(pack["total_bytes"], sum(pack["components"].values()))
        self.assertLess(pack["total_bytes"], 32 * 1024 * 1024)


class ProcessTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.legacy = self.root / "legacy.json"
        self.archives = self.root / "collection"
        self.payload = self.root / "payload.json"
        self.payload.write_text(json.dumps(report()))
        self.input = self.root / "input"
        self.input.write_text("source identity")
        self.binary = self.root / "binary"
        self.binary.write_text("binary identity")

    def run_child(self, body, **caps):
        fake = self.root / "fake.py"
        fake.write_text("import os, sys, time, pathlib, signal, subprocess\n" + body)
        command = [sys.executable, str(fake)]
        def snapshot():
            return {"source_sha256": SOURCE, "input_sha256": hashlib.sha256(self.input.read_bytes()).hexdigest(),
                    "binary_sha256": hashlib.sha256(self.binary.read_bytes()).hexdigest()}
        return c._collect_pair(command, self.legacy, self.archives, snapshot,
                               c._Limits(**caps))

    def success_body(self):
        return f"pathlib.Path({str(self.legacy)!r}).write_bytes(pathlib.Path({str(self.payload)!r}).read_bytes())\nprint('raw timing', time.monotonic())\n"

    def status(self):
        return json.loads((self.archives / "first/status.json").read_bytes())

    def test_first_repeat_are_archived_once_and_staging_released(self):
        result = self.run_child(self.success_body())
        self.assertTrue(result["canonical_equal"])
        self.assertFalse(self.legacy.exists())
        self.assertNotEqual((self.archives / "first/stdout.log").read_bytes(),
                            (self.archives / "repeat/stdout.log").read_bytes())
        self.assertEqual(self.status()["state"], "accepted")
        with self.assertRaises(FileExistsError):
            self.run_child(self.success_body())

    def test_existing_legacy_and_symlink_are_refused_without_attempt(self):
        for symlink in (False, True):
            if symlink:
                self.legacy.symlink_to(self.input)
            else:
                self.legacy.write_text("preserve")
            with self.assertRaises(FileExistsError):
                self.run_child(self.success_body())
            self.assertFalse(self.archives.exists())
            self.legacy.unlink()

    def test_competing_ownership_is_refused(self):
        self.legacy.with_name(self.legacy.name + ".collection-lock").mkdir()
        with self.assertRaises(FileExistsError):
            self.run_child(self.success_body())
        self.assertFalse(self.archives.exists())

    def test_symlink_archive_parent_is_refused_without_execution(self):
        target = self.root / "target"
        target.mkdir()
        self.archives.symlink_to(target, target_is_directory=True)
        with self.assertRaises(ValueError):
            self.run_child(self.success_body())
        self.assertEqual(list(target.iterdir()), [])

    def test_binary_change_during_child_stops(self):
        self.run_child(self.success_body() + f"pathlib.Path({str(self.binary)!r}).write_text('changed')\n")
        self.assertIn("identity", self.status()["error"])
        self.assertFalse((self.archives / "repeat").exists())

    def test_nonzero_exit_retains_failure_and_stops_repeat(self):
        self.run_child("print('failed raw'); sys.exit(7)\n")
        self.assertEqual(self.status()["returncode"], 7)
        self.assertFalse((self.archives / "repeat").exists())
        self.assertTrue(self.legacy.with_name(self.legacy.name + ".collection-lock").exists())

    def test_malformed_partial_and_duplicate_reports_stop(self):
        self.run_child(f"pathlib.Path({str(self.legacy)!r}).write_text('[{{')\n")
        self.assertEqual(self.status()["state"], "failed")
        self.assertFalse((self.archives / "repeat").exists())

    def test_claim_failure_gap_retains_typed_outcome_and_stops(self):
        rows = report()
        rows[0].update(verdict="Fails", measured="")
        self.payload.write_text(json.dumps(rows))
        self.run_child(self.success_body())
        self.assertEqual(self.status()["validation"]["verdicts"]["negotiable.cake"], "Fails")
        self.assertFalse((self.archives / "repeat").exists())

    def test_source_or_binary_change_during_child_stops(self):
        self.run_child(self.success_body() + f"pathlib.Path({str(self.input)!r}).write_text('changed')\n")
        self.assertIn("identity", self.status()["error"])
        self.assertFalse((self.archives / "repeat").exists())

    def test_timeout_kills_process_group_and_retains_logs(self):
        marker = self.root / "escaped"
        child = f"import time,pathlib;time.sleep(0.8);pathlib.Path({str(marker)!r}).write_text('bad')"
        self.run_child(f"subprocess.Popen([sys.executable, '-c', {child!r}]); print('started', flush=True); time.sleep(10)\n", seconds=0.15)
        self.assertIn("elapsed", self.status()["error"])
        import time
        time.sleep(0.9)
        self.assertFalse(marker.exists())

    def test_lingering_descendant_is_failure_even_if_parent_exits(self):
        self.run_child(self.success_body() + "subprocess.Popen([sys.executable, '-c', 'import time;time.sleep(10)'], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)\n")
        self.assertIn("descendant", self.status()["error"])

    def test_output_overflow_retains_bounded_failure(self):
        self.run_child("os.write(1, b'x' * 1000000); time.sleep(10)\n", total_bytes=128 * 1024)
        self.assertIn("bytes", self.status()["error"])
        self.assertLessEqual(sum(p.stat().st_size for p in (self.archives / 'first').iterdir()), 128 * 1024)

    def test_report_file_overflow_is_bounded(self):
        self.run_child(f"pathlib.Path({str(self.legacy)!r}).write_bytes(b'x' * 1000000)\n", total_bytes=128 * 1024)
        self.assertEqual(self.status()["state"], "failed")
        self.assertLessEqual(sum(p.stat().st_size for p in (self.archives / 'first').iterdir()), 128 * 1024)

    def test_peak_rss_breach_is_not_accepted(self):
        self.run_child("x = bytearray(80 * 1024 * 1024); time.sleep(2)\n", rss_bytes=40 * 1024 * 1024)
        self.assertIn("RSS", self.status()["error"])
        self.assertFalse((self.archives / "repeat").exists())

    def test_child_symlink_report_is_refused_and_preserved(self):
        self.run_child(f"pathlib.Path({str(self.legacy)!r}).unlink(); pathlib.Path({str(self.legacy)!r}).symlink_to({str(self.payload)!r})\n")
        self.assertEqual(self.status()["state"], "failed")
        self.assertTrue(self.legacy.is_symlink())


if __name__ == "__main__":
    unittest.main()
