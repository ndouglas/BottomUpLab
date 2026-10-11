import unittest
from survey.war1_study import archive, manifest, test_fixtures as fixtures


class OutcomeTests(unittest.TestCase):
    def setUp(self):
        self.manifest = manifest.validate_manifest(
            manifest.make_fixture(fixtures.ROOT), fixtures.ROOT
        )
        self.row = fixtures.outcome(
            ending="one_side_extinction", step=1, winner="blue", censored=False
        )

    def test_valid_native_shaped_construction_row(self):
        archive.validate_outcome(self.row, self.manifest)

    def test_compact_binding_clocks_rng_and_capture_are_checked(self):
        from copy import deepcopy

        changes = [
            ("seed", 9),
            ("completed_steps", 2),
            ("case_input_sha256", "f" * 64),
            ("calendar_time", 8),
            ("observed_steps", 0),
            ("casualty_totals", [0, 0]),
            ("transcript_sha256", "z" * 64),
        ]
        for key, value in changes:
            with self.subTest(key=key), self.assertRaises(ValueError):
                row = deepcopy(self.row)
                row[key] = value
                archive.validate_outcome(row, self.manifest)
        for text in [
            '{"state":2}',
            '{"state":true}',
            '{"state":340282366920938463463374607431768211456}',
            '{"state":1.0}',
            '{"state":1,"state":3}',
        ]:
            with self.subTest(rng=text), self.assertRaises(ValueError):
                row = deepcopy(self.row)
                row["final_checkpoint"]["contact_rng"] = text
                archive.validate_outcome(row, self.manifest)
        row = deepcopy(self.row)
        row["observations"][0]["origin"] = "native"
        with self.assertRaises(ValueError):
            archive.validate_outcome(row, self.manifest)


class SavedArchiveTests(unittest.TestCase):
    def setUp(self):
        import tempfile
        from pathlib import Path

        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.manifest = manifest.validate_manifest(
            manifest.make_fixture(fixtures.ROOT), fixtures.ROOT
        )
        fixtures.save_campaign(self.root, self.manifest)

    def test_valid_census_is_reiterable_with_bounded_audit_descriptors(self):
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertTrue(saved.complete)
        self.assertEqual(saved.census["complete_valid"], 16)
        self.assertEqual(len(saved.audits), 8)
        self.assertEqual([r["seed"] for r in saved.outcomes()], [7, 8] * 8)
        self.assertEqual(len(list(saved.outcomes())), 16)

    def test_changed_bytes_after_validation_are_rejected_on_iteration(self):
        saved = archive.read_campaign(self.root, self.manifest)
        path = self.root / "blocks/block-000000.jsonl"
        path.write_bytes(path.read_bytes().replace(b'"seed":7', b'"seed":9', 1))
        with self.assertRaises(ValueError):
            list(saved.outcomes())

    def test_forged_audit_bytes_never_count_as_complete(self):
        path = self.root / "audits/0.jsonl"
        path.write_bytes(path.read_bytes() + b"{}\n")
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertFalse(saved.complete)
        self.assertGreater(saved.census["invalid"], 0)

    def test_missing_block_preserves_planned_denominator(self):
        (self.root / "blocks/block-000000.jsonl").unlink()
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertFalse(saved.complete)
        self.assertEqual(saved.census["planned"], 16)

    def test_source_bytes_are_matched_before_inference(self):
        path = self.root / "source/000000.bin"
        path.write_bytes(b"forged")
        with self.assertRaises(ValueError):
            archive.read_campaign(self.root, self.manifest)


class ProvenanceTests(unittest.TestCase):
    def test_monitor_requires_actual_child_peak_and_exact_limits(self):
        from survey.war1_study import archive_provenance as p

        header = dict(mode="prospective_seed_population", binding={})
        monitor = dict(
            schema="war1-study-monitor-result-v1",
            mode=header["mode"],
            binding={},
            child_pid=123,
            exit_code=0,
            platform="darwin",
            peak_method="os.wait4.ru_maxrss",
            wall_seconds=1.25,
            peak_rss_bytes=1024,
            wall_limit_seconds=43200,
            rss_limit_bytes=2147483648,
            campaign_limit_bytes=4294967296,
            source_valid=True,
            archive_valid=True,
        )
        p.validate_monitor(monitor, header)
        for k, v in [
            ("peak_rss_bytes", 0),
            ("exit_code", 1),
            ("wall_seconds", 43201),
            ("child_pid", True),
            ("peak_method", "polled_peak"),
            ("source_valid", 1),
            ("rss_limit_bytes", 2147483649),
            ("wall_seconds", float("nan")),
        ]:
            with self.subTest(field=k, value=v), self.assertRaises(ValueError):
                p.validate_monitor({**monitor, k: v}, header)


class InterruptedArchiveTests(unittest.TestCase):
    setUp = SavedArchiveTests.setUp

    def test_open_block_start_is_partial_and_unattempted_keys_stay_unstarted(self):
        path = self.root / "blocks/block-000000.jsonl"
        path.write_bytes(path.read_bytes().splitlines(keepends=True)[0])
        (self.root / "blocks/block-000000.descriptor.json").unlink()
        (self.root / "candidate-index.json").unlink()
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertEqual(
            (
                saved.census["attempted"],
                saved.census["partial"],
                saved.census["unstarted"],
                saved.census["invalid"],
            ),
            (1, 1, 15, 0),
        )

    def test_torn_outcome_keeps_started_case_partial(self):
        path = self.root / "blocks/block-000000.jsonl"
        first = path.read_bytes().splitlines(keepends=True)[0]
        path.write_bytes(first + b'{"kind":"outcome"')
        (self.root / "blocks/block-000000.descriptor.json").unlink()
        (self.root / "candidate-index.json").unlink()
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertEqual((saved.census["partial"], saved.census["unstarted"]), (1, 15))


class FailureReceiptTests(unittest.TestCase):
    setUp = SavedArchiveTests.setUp

    def prepare_failure(self):
        import hashlib
        from survey.war1_study.io import canonical_bytes

        path = self.root / "blocks/block-000000.jsonl"
        raw = path.read_bytes().splitlines(keepends=True)[0]
        path.write_bytes(raw)
        (self.root / "blocks/block-000000.descriptor.json").unlink()
        (self.root / "candidate-index.json").unlink()
        census = dict(
            planned=16,
            attempted=1,
            complete_valid=0,
            failed=0,
            invalid=0,
            partial=1,
            pending=0,
            unstarted=15,
        )
        receipt = dict(
            sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw), acknowledged_cases=0
        )
        case = self.manifest.cases[0]
        c = self.manifest.arms[0]["config"]
        failure = dict(
            detail="synthetic resource interruption",
            census=census,
            receipt=receipt,
            case_failure=dict(
                kind="resource_stop",
                detail="synthetic interruption",
                key=dict(ordinal=0, arm_id=case.key.arm_id, replicate_index=0),
                attempted_step=None,
                semantic_steps=0,
                observed_steps=0,
                durable_audit_steps=0,
                final_checkpoint=fixtures.checkpoint(c, 7, 0, [1, 1], None),
                acknowledged_archive_prefix=receipt,
            ),
        )
        (self.root / "failure.json").write_bytes(canonical_bytes(failure))
        return failure, census

    def test_failure_receipt_retains_actual_checkpoint_without_becoming_horizon(self):
        from survey.war1_study.io import canonical_bytes

        failure, census = self.prepare_failure()
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertEqual(saved.census, census)
        self.assertIn("failure_receipt_present", saved.header["validation"]["errors"])
        failure["case_failure"]["observed_steps"] = 2
        (self.root / "failure.json").write_bytes(canonical_bytes(failure))
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertTrue(
            any("failure clocks" in e for e in saved.header["validation"]["errors"])
        )


class TornStartTests(unittest.TestCase):
    setUp = SavedArchiveTests.setUp

    def test_torn_start_is_partial_at_expected_key_not_an_unstarted_run(self):
        (self.root / "blocks/block-000000.jsonl").write_bytes(b'{"kind":"start"')
        (self.root / "blocks/block-000000.descriptor.json").unlink()
        (self.root / "candidate-index.json").unlink()
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertEqual(
            (
                saved.census["attempted"],
                saved.census["partial"],
                saved.census["unstarted"],
            ),
            (1, 1, 15),
        )


class ReboundTamperTests(unittest.TestCase):
    setUp = SavedArchiveTests.setUp

    def rewrite(self, rows):
        import hashlib
        from survey.war1_study.io import canonical_bytes, strict_json, METADATA_LIMIT

        block = self.root / "blocks/block-000000.jsonl"
        raw = b"".join(canonical_bytes(row) for row in rows)
        block.write_bytes(raw)
        path = self.root / "blocks/block-000000.descriptor.json"
        d = strict_json(path.read_bytes(), METADATA_LIMIT)
        d.update(sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw))
        path.write_bytes(canonical_bytes(d))
        path = self.root / "candidate-index.json"
        candidate = strict_json(path.read_bytes(), METADATA_LIMIT)
        candidate["blocks"] = [d]
        candidate["receipt"].update(sha256=d["sha256"], bytes=d["bytes"])
        path.write_bytes(canonical_bytes(candidate))

    def rows(self):
        from survey.war1_study.io import strict_json

        return [
            strict_json(line, 32768)
            for line in (self.root / "blocks/block-000000.jsonl")
            .read_bytes()
            .splitlines()
        ]

    def test_wrong_config_cannot_hide_behind_recomputed_block_hash(self):
        rows = self.rows()
        rows[3]["outcome"]["final_checkpoint"]["config"]["dt"] = 0.03125
        self.rewrite(rows)
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertFalse(saved.complete)
        self.assertEqual(saved.census["invalid"], 1)

    def test_duplicate_key_is_rejected_even_when_all_file_hashes_are_rebound(self):
        rows = self.rows()
        rows[2] = rows[0]
        rows[3] = rows[1]
        self.rewrite(rows)
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertFalse(saved.complete)
        self.assertTrue(
            any(
                "duplicate start key" in e for e in saved.header["validation"]["errors"]
            )
        )

    def test_raw_semantic_spelling_is_hashed_instead_of_python_reserialization(self):
        import hashlib
        from survey.war1_study.io import canonical_bytes, strict_json

        path = self.root / "audits/0.jsonl"
        audit = [strict_json(line, 32768) for line in path.read_bytes().splitlines()]
        audit[1]["checkpoint_json"] = " " + audit[1]["checkpoint_json"]
        raw = b"".join(canonical_bytes(row) for row in audit)
        path.write_bytes(raw)
        rows = self.rows()
        rows[1]["outcome"]["audit_descriptor"].update(
            sha256=hashlib.sha256(raw).hexdigest(), bytes=len(raw)
        )
        self.rewrite(rows)
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertTrue(
            any(
                "semantic transcript hash" in e
                for e in saved.header["validation"]["errors"]
            )
        )


class FailedStartReceiptTests(unittest.TestCase):
    setUp = SavedArchiveTests.setUp
    prepare_failure = FailureReceiptTests.prepare_failure

    def test_failure_before_any_start_bytes_still_counts_one_partial_attempt(self):
        import hashlib
        from survey.war1_study.io import canonical_bytes, strict_json, METADATA_LIMIT

        # Use the existing valid failure fixture then move the failure to the
        # durable-start boundary, before any start bytes were acknowledged.
        self.prepare_failure()
        path = self.root / "failure.json"
        f = strict_json(path.read_bytes(), METADATA_LIMIT)
        receipt = {
            "sha256": hashlib.sha256(b"").hexdigest(),
            "bytes": 0,
            "acknowledged_cases": 0,
        }
        f["receipt"] = receipt
        f["case_failure"].update(
            kind="archive_start",
            observed_steps=0,
            final_checkpoint=None,
            acknowledged_archive_prefix=receipt,
        )
        (self.root / "blocks/block-000000.jsonl").write_bytes(b"")
        path.write_bytes(canonical_bytes(f))
        saved = archive.read_campaign(self.root, self.manifest)
        self.assertEqual(
            (
                saved.census["attempted"],
                saved.census["partial"],
                saved.census["unstarted"],
            ),
            (1, 1, 15),
        )
