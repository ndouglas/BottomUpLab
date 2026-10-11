"""Synthetic monitor and schema proof boundaries; no scientific engines."""
import unittest
from pathlib import Path
import tempfile
from unittest.mock import patch
from survey.war1_study import supervisor, resource_proof


class MonitorTests(unittest.TestCase):
    def test_wait4_units_are_individual_and_platform_specific(self):
        self.assertEqual(supervisor.peak_bytes(4096, "darwin"), 4096)
        self.assertEqual(supervisor.peak_bytes(4096, "linux"), 4194304)
        with self.assertRaises(ValueError):
            supervisor.peak_bytes(1, "unsupported")
        with self.assertRaises(ValueError):
            supervisor.peak_bytes(0, "darwin")

    def test_unsampled_final_peak_invalidates_candidate(self):
        with self.assertRaisesRegex(ValueError, "RSS"):
            supervisor.check_limits(1.0, 2147483649, 43200, 2147483648)

    def test_unknown_or_negative_samples_fail_closed(self):
        for sample in (None, -1, True):
            with self.assertRaises(ValueError):
                supervisor.check_limits(1, sample, 43200, 2147483648)

    def test_parent_publication_requires_reserved_space_including_logs(self):
        with tempfile.TemporaryDirectory(prefix="construction_test_") as temp:
            root = Path(temp).resolve()
            (root / "tail").write_bytes(b"x" * 100)
            with patch.object(supervisor, "CAMPAIGN_LIMIT", 100 + supervisor.PARENT_RESERVE - 1):
                with self.assertRaisesRegex(ValueError, "publication reserve"):
                    supervisor.publication_budget(root)

    def test_proof_accounts_full_population_and_all_metadata(self):
        proof = resource_proof.schema_proof()
        self.assertEqual(proof["starts_outcomes_bytes"], 1764753408)
        self.assertEqual(proof["audit_bytes"], 778240000)
        self.assertEqual(proof["reserved_bytes"], 3616735232)
        self.assertLessEqual(proof["audit_frame_bytes"], 28672)
        self.assertLessEqual(proof["outcome_bytes"], 32768)
        self.assertLessEqual(proof["start_bytes"], 1024)
        self.assertLessEqual(proof["metadata_reserved_bytes"], 1073741824)

    def test_forecast_requires_measured_per_case_constructor_and_terminal_cost(self):
        rows = [dict(actor_steps=1, kernel_seconds_per_actor_step=1e-8,
                     projection_seconds_per_step=1e-8, start_outcome_seconds=1e-8,
                     audit_seconds_per_frame=1e-8) for _ in range(16)]
        synthetic = dict(source_seconds=1, archive_seconds_per_byte=1e-9, peak_rss_bytes=1000000)
        with self.assertRaisesRegex(ValueError, "missing measured construction cost"):
            resource_proof.forecast(rows, synthetic)

    def test_unavailable_or_excess_forecast_blocks_readiness(self):
        for rows in ([], [{"actor_steps": 0}], [{"kernel_seconds_per_actor_step": 1.0}]):
            with self.assertRaises(ValueError):
                resource_proof.forecast(rows, {})


if __name__ == "__main__":
    unittest.main()
