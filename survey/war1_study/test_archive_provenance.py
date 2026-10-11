"""Synthetic retained receipt chain: no review, permission or scientific histories."""

from copy import deepcopy
import hashlib
from pathlib import Path
import tempfile
import unittest

from survey.war1_study import archive_provenance as provenance, test_fixtures as fixtures
from survey.war1_study.io import canonical_bytes, strict_json, METADATA_LIMIT
from survey.war1_study.oracle import prepare_oracles
from survey.war1_study import test_oracle


def synthetic_packet(root, manifest):
    """Private temporary test packet with explicit synthetic actor identities."""
    fixtures.save_campaign(root, manifest, stop=0)
    (root / "provenance/fixture-probes").mkdir(parents=True)

    def artifact(path, value):
        data = canonical_bytes(value)
        (root / path).write_bytes(data)
        return dict(
            path="/synthetic-construction-unreadable-original/" + path,
            sha256=hashlib.sha256(data).hexdigest(),
            bytes=len(data),
        )

    def load(path):
        return strict_json((root / path).read_bytes(), METADATA_LIMIT)

    header = load("header.json")
    binding = header["binding"]
    inventory = load("source-inventory.json")
    source = artifact("provenance/source-inventory.json", inventory)
    ids = dict(
        protocol_sha256=manifest.raw["protocol_sha256"],
        proposal_sha256=manifest.raw["proposal_sha256"],
        manifest_sha256=manifest.sha256,
        compiled_inputs_sha256=binding["compiled_inputs_sha256"],
        source_inventory_sha256=source["sha256"],
        resolved_roster_sha256=manifest.raw["roster_sha256"],
        binary_sha256=binding["executable_sha256"],
    )
    binary = dict(
        path="/synthetic-construction-binary", sha256=ids["binary_sha256"], bytes=1
    )
    build = artifact(
        "provenance/build.json",
        dict(
            schema="war1-study-build-v1",
            identities=ids,
            selected_path="survey/war1_study/manifests/synthetic-construction.json",
            selected_purpose=manifest.purpose,
            binary=binary,
            rustc="synthetic-construction",
            cargo="synthetic-construction",
            platform="synthetic-construction",
            features=["CARGO_FEATURE_WAR_STUDIES"],
            flags=[
                [k, ""]
                for k in (
                    "RUSTFLAGS",
                    "CARGO_ENCODED_RUSTFLAGS",
                    "TARGET",
                    "HOST",
                    "PROFILE",
                    "OPT_LEVEL",
                    "DEBUG",
                )
            ],
            build_command=["synthetic-construction"],
            inventory_before=inventory["entries"],
            inventory_after=inventory["entries"],
        ),
    )
    test_oracle.PreparedOracleTests.setUpClass()
    export = deepcopy(test_oracle.PreparedOracleTests.export)
    export.update(
        purpose=manifest.purpose,
        manifest_sha256=manifest.sha256,
        binding_state="source_bound",
        compiled_inputs_sha256=ids["compiled_inputs_sha256"],
    )
    oracles = prepare_oracles(manifest, export)
    numeric_export = artifact("provenance/numeric-export.json", export)
    packet = artifact(
        "provenance/oracle-packet.json",
        provenance.oracle_packet(manifest, ids["compiled_inputs_sha256"], oracles),
    )
    common = {
        key: ids[key]
        for key in ("manifest_sha256", "compiled_inputs_sha256", "binary_sha256")
    }
    numeric = artifact(
        "provenance/numeric.json",
        dict(
            schema="war1-study-numeric-v1",
            **common,
            numeric_export=numeric_export,
            oracle_packet=packet,
            certificate_valid=True,
            mass_valid=True,
            width_valid=True,
            ulp_valid=True,
            ctmc_valid=True,
            unresolved_findings=[],
            oracle_ids=[o.arm_id for o in oracles]
        ),
    )
    forecast = artifact(
        "provenance/resource-forecast-inputs.json",
        dict(evidence_label="synthetic_construction_only"),
    )
    probe = artifact(
        "provenance/fixture-probes/000000.json",
        dict(evidence_label="synthetic_construction_only"),
    )
    resources = artifact(
        "provenance/resources.json",
        dict(
            schema="war1-study-resources-v1",
            **common,
            budgets=provenance.LIMITS,
            census=provenance.POPULATION,
            reserved_bytes=4294967296,
            failure_reserve_bytes=1048576,
            schema_bounds_valid=True,
            aggregate_proof_valid=True,
            forecast_valid=True,
            forecast_method="synthetic_construction_only",
            forecast_inputs=forecast,
            forecast_wall_seconds=1,
            forecast_peak_rss_bytes=1024,
            fixture_probes=[
                dict(
                    artifact=probe,
                    command=["synthetic-construction"],
                    compiled_inputs_sha256=ids["compiled_inputs_sha256"],
                    binary_sha256=ids["binary_sha256"],
                    wall_seconds=1,
                    peak_rss_bytes=1024,
                    fixture_purpose="construction_fixture",
                )
            ],
            unavailable=[]
        ),
    )
    declared_manifest = dict(
        path="/synthetic-construction-manifest",
        sha256=manifest.sha256,
        bytes=len(manifest.raw_bytes),
    )
    declaration = artifact(
        "provenance/declaration.json",
        dict(
            schema="war1-study-declaration-v1",
            purpose="prospective_frozen_not_blind",
            identities=ids,
            build=build,
            numeric=numeric,
            resources=resources,
            source_inventory=source,
            manifest=declared_manifest,
            binary=binary,
            normative_frozen_revision="1" * 40,
            declaration_revision="2" * 40,
            registered_histories=0,
            census=provenance.POPULATION,
            budgets=provenance.LIMITS,
        ),
    )
    acceptance = artifact(
        "provenance/acceptance.json",
        dict(
            schema="war1-study-readiness-v1",
            declaration_sha256=declaration["sha256"],
            reviewer_identity="synthetic construction reviewer; no real review",
            reviewer_role="independent_reviewer",
            decision="accepted",
            registered_histories=0,
            open_required_findings=[],
            build_sha256=build["sha256"],
            numeric_sha256=numeric["sha256"],
            resources_sha256=resources["sha256"],
            binary_sha256=ids["binary_sha256"],
            manifest_sha256=manifest.sha256,
        ),
    )
    execution = artifact(
        "provenance/execution.json",
        dict(
            schema="war1-study-execution-v1",
            declaration_sha256=declaration["sha256"],
            acceptance_sha256=acceptance["sha256"],
            owner_identity="synthetic construction owner; no real authority",
            action="execute_exact_population",
            census=provenance.POPULATION,
            budgets=provenance.LIMITS,
            authorization_quote="SYNTHETIC TEST ONLY: never actual authorization",
            authorized_at="synthetic-construction",
            compiled_inputs_sha256=ids["compiled_inputs_sha256"],
            source_inventory_sha256=ids["source_inventory_sha256"],
            binary_sha256=ids["binary_sha256"],
            manifest_sha256=manifest.sha256,
        ),
    )
    binding.update(
        declaration_sha256=declaration["sha256"],
        acceptance_sha256=acceptance["sha256"],
        execution_approval_sha256=execution["sha256"],
    )
    artifact("header.json", header)
    return header, oracles


class RetainedReceiptTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.temp.cleanup)
        cls.root = Path(cls.temp.name).resolve()
        cls.manifest = fixtures.synthetic_manifest()
        cls.header, cls.oracles = synthetic_packet(cls.root, cls.manifest)

    def test_complete_retained_links_recompute_all_twelve_oracles(self):
        result = provenance.validate_retained_receipts(
            self.root, self.header, self.manifest
        )
        self.assertEqual(
            provenance.oracle_rows(result), provenance.oracle_rows(self.oracles)
        )

    def test_native_inventory_integer_tokens_are_checked_before_semantic_equality(self):
        path = self.root / "source-inventory.json"
        raw = path.read_bytes()
        original = strict_json(raw, METADATA_LIMIT)
        try:
            for value in (float(original["entries"][0]["bytes"]), True):
                with self.subTest(value=value):
                    bad = deepcopy(original)
                    bad["entries"][0]["bytes"] = value
                    path.write_bytes(canonical_bytes(bad))
                    with self.assertRaisesRegex(ValueError, "integer"):
                        provenance.validate_retained_receipts(
                            self.root, self.header, self.manifest
                        )
        finally:
            path.write_bytes(raw)

    def test_missing_monitor_cannot_turn_synthetic_receipts_into_primary_inference(
        self,
    ):
        available, reason = provenance.inference_provenance(
            self.root, self.header, self.manifest, None
        )
        self.assertFalse(available)
        self.assertIn("unavailable", reason)

    def test_external_artifact_path_is_not_followed_and_tampered_copy_is_rejected(self):
        path = self.root / "provenance/build.json"
        raw = path.read_bytes()
        try:
            path.write_bytes(b"{}\n")
            with self.assertRaisesRegex(ValueError, "retained artifact digest"):
                provenance.validate_retained_receipts(
                    self.root, self.header, self.manifest
                )
        finally:
            path.write_bytes(raw)

    def test_numeric_packet_keeps_exact_rationals_and_required_nullable_ctmc(self):
        packet = provenance.oracle_packet(
            self.manifest,
            self.header["binding"]["compiled_inputs_sha256"],
            self.oracles,
        )
        two = next(o for o in packet["oracles"] if "2v1" in o["arm_id"])
        self.assertIsNone(two["ctmc_one_vs_one"])
        self.assertEqual(set(two["scalars"]), set(self.oracles[0].scalars))
        self.assertEqual(
            two["cdfs"]["duration.U_over_H"][-1]["point"],
            {"numerator": "1", "denominator": "1"},
        )

    def test_full_synthetic_population_reports_tiny_rejections_and_all_ctmc_margins(
        self,
    ):
        from survey.war1_study import analysis, archive
        from unittest.mock import patch

        arms = {a["arm_id"]: a for a in self.manifest.arms}

        def rows():
            for case in self.manifest.cases:
                yield fixtures.synthetic_row(
                    arms[case.key.arm_id], case, self.manifest.sha256
                )

        saved = archive.ArchiveRead(
            {
                "evidence_label": "synthetic_construction_only",
                "validation": {
                    "primary_available": True,
                    "registered": False,
                    "reason": None,
                },
            },
            rows,
            {"planned": 52224, "complete_valid": 52224},
            (),
            True,
        )
        with patch(
            "subprocess.Popen", side_effect=AssertionError("simulator forbidden")
        ), patch(
            "os.urandom", side_effect=AssertionError("scientific entropy forbidden")
        ):
            result = analysis.analyze(self.manifest, saved, self.oracles)
        cdfs = [r for r in result.oracle_checks if r["kind"] == "cdf"]
        self.assertEqual(len(cdfs), 36)
        self.assertTrue(
            all(
                r["verdict"] == "incompatible_with_ideal_word_law"
                for r in cdfs
                if r["endpoint_id"] == "duration.U_over_H"
            )
        )
        margins = [r for r in result.oracle_checks if r["kind"] == "ctmc_one_vs_one"]
        self.assertEqual(len(margins), 30)
        self.assertTrue(all(r["criterion"] is not None for r in margins))
        self.assertTrue(
            all(r["criterion"]["passes"] for r in margins if r["required_at_this_dt"])
        )
        self.assertTrue(all(not r["registered"] for r in result.scalars))


class SuccessIntegerTests(unittest.TestCase):
    def test_success_nested_integer_tokens_are_strict_before_preparation(self):
        from types import SimpleNamespace
        from unittest.mock import patch

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            header = dict(mode="prospective_seed_population", binding={})
            census = dict(
                planned=16,
                attempted=16,
                complete_valid=16,
                failed=0,
                invalid=0,
                partial=0,
                pending=0,
                unstarted=0,
            )
            receipt = dict(sha256="a" * 64, bytes=123, acknowledged_cases=16)
            candidate = dict(census=census, receipt=receipt)
            monitor = dict(
                schema="war1-study-monitor-result-v1",
                **header,
                child_pid=123,
                exit_code=0,
                platform="linux",
                peak_method="os.wait4.ru_maxrss",
                wall_seconds=1.25,
                peak_rss_bytes=1024,
                wall_limit_seconds=43200,
                rss_limit_bytes=2147483648,
                campaign_limit_bytes=4294967296,
                source_valid=True,
                archive_valid=True
            )

            def artifact(path, value):
                data = canonical_bytes(value)
                (root / path).write_bytes(data)
                return dict(
                    path=path, bytes=len(data), sha256=hashlib.sha256(data).hexdigest()
                )

            success = dict(
                schema="war1-study-success-v1",
                **header,
                candidate=artifact("candidate-index.json", candidate),
                receipt=receipt,
                census=census,
                monitor=artifact("monitor.json", monitor)
            )
            m = SimpleNamespace(purpose=header["mode"], cases=tuple(range(16)))
            # Real success, artifact and monitor validation; expensive independent
            # numerical receipt recomputation is outside this schema regression.
            with patch.object(
                provenance, "validate_retained_receipts", return_value=()
            ):
                (root / "success.json").write_bytes(canonical_bytes(success))
                self.assertEqual(
                    provenance.inference_provenance(root, dict(header), m, candidate),
                    (True, None),
                )
                for location, key, value in [
                    ("receipt", "acknowledged_cases", 16.0),
                    ("census", "failed", 0.0),
                    ("census", "failed", False),
                ]:
                    with self.subTest(location=location, value=value):
                        bad = deepcopy(success)
                        bad[location][key] = value
                        (root / "success.json").write_bytes(canonical_bytes(bad))
                        available, reason = provenance.inference_provenance(
                            root, dict(header), m, candidate
                        )
                        self.assertFalse(available)
                        self.assertIn("integer", reason)


class InventoryIntegerTests(unittest.TestCase):
    def test_inventory_bytes_bool_and_integral_float_never_equal_integer(self):
        packet = dict(
            schema="war1-study-source-inventory-v1",
            compiled_inputs_sha256="a" * 64,
            entries=[dict(path="survey/war1_study/probe.py", sha256="b" * 64, bytes=1)],
        )
        provenance.validate_inventory(packet)
        for value in (1.0, True):
            with self.subTest(value=value), self.assertRaisesRegex(
                ValueError, "integer"
            ):
                bad = deepcopy(packet)
                bad["entries"][0]["bytes"] = value
                provenance.validate_inventory(bad)
