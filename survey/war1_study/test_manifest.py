"""Manifest behavior tests; all seed draws are deterministic synthetic bytes."""

from copy import deepcopy
import hashlib
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch

from survey.war1_study import manifest

ROOT = Path(__file__).resolve().parents[2]


class ManifestTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.fixture = manifest.make_fixture(ROOT)
        cls.science = manifest.sample_manifest(
            ROOT, lambda n: (7).to_bytes(8, "little")
        )

    def reject(self, raw):
        with self.assertRaises(ValueError):
            manifest.validate_manifest(json.dumps(raw).encode(), ROOT)

    def test_with_replacement_retains_equal_seed_values(self):
        parsed = manifest.validate_manifest(self.science, ROOT)
        self.assertEqual(
            (len(parsed.cases), len({c.key for c in parsed.cases})), (52224, 52224)
        )
        self.assertEqual({c.seed for c in parsed.cases}, {7})

    def test_draws_eight_bytes_once_per_key_and_hashes_actual_bytes(self):
        sizes = []

        def entropy(n):
            sizes.append(n)
            return ((len(sizes) % 3) + 7).to_bytes(8, "little")

        raw = json.loads(manifest.sample_manifest(ROOT, entropy))
        self.assertEqual(sizes, [8] * 52224)
        blocks = b"".join(c["seed"].to_bytes(8, "little") for c in raw["cases"])
        self.assertEqual(
            raw["sampler"]["seed_bytes_sha256"], hashlib.sha256(blocks).hexdigest()
        )
        self.assertEqual(raw["sampler"]["bytes_drawn"], 417792)

    def test_bad_entropy_fails_without_retry(self):
        for value in (b"", b"1234567", b"123456789", bytearray(8), None):
            calls = []

            def entropy(n):
                calls.append(n)
                return value

            with self.subTest(value=value), self.assertRaises(ValueError):
                manifest.sample_manifest(ROOT, entropy)
            self.assertEqual(calls, [8])

    def test_command_limits_are_refused_before_drawing(self):
        calls = []

        def entropy(n):
            calls.append(n)
            return b"\x00" * 8

        for command in ((), ["prepare"], ("x" * 1025,), ("é" * 513,), ("x",) * 33):
            with self.subTest(command=command), self.assertRaises(ValueError):
                manifest.sample_manifest(ROOT, entropy, command=command)
        self.assertEqual(calls, [])

    def test_command_exact_utf8_bounds_are_accepted(self):
        command=("é"*512,)*32
        data=manifest.sample_manifest(ROOT,lambda n:b"\x07"*8,command=command)
        self.assertEqual(manifest.validate_manifest(data,ROOT).raw["sampler"]["command"],list(command))

    def test_sampler_version_is_bounded_and_must_be_supported(self):
        raw=json.loads(self.science)
        for version in ("v"*129, "v"*128, 1):
            raw["sampler"]["version"]=version
            self.reject(raw)

    def test_manifest_size_gate_runs_before_seed_list_allocation_and_draws(self):
        calls = []

        def entropy(n):
            calls.append(n)
            return b"\x00" * 8

        with patch.object(manifest, "METADATA_LIMIT", 100000):
            with self.assertRaises(ValueError):
                manifest.sample_manifest(ROOT, entropy)
        self.assertEqual(calls, [])

    def test_full_population_census_and_work_bound(self):
        value = manifest.proposal(ROOT)
        arms = manifest.validate_manifest(self.science, ROOT).arms
        self.assertEqual(len(arms), 66)
        self.assertEqual(sum(a["replicates"] for a in arms), 52224)
        self.assertEqual(
            sum(a["replicates"] * a["config"]["max_steps"] for a in arms), 17563648
        )
        self.assertEqual(
            sum(
                a["replicates"]
                * a["config"]["max_steps"]
                * (a["config"]["blue"] + a["config"]["red"])
                for a in arms
            ),
            2054160384,
        )
        self.assertEqual(
            {
                g: (
                    sum(a["group"] == g for a in arms),
                    sum(a["replicates"] for a in arms if a["group"] == g),
                )
                for g in ("core", "scale", "mirror", "oracle")
            },
            {
                "core": (36, 18432),
                "scale": (8, 4096),
                "mirror": (10, 5120),
                "oracle": (12, 24576),
            },
        )
        self.assertEqual(
            max(a["config"]["blue"] + a["config"]["red"] for a in arms), 768
        )
        self.assertEqual(
            max(
                a["config"]["max_steps"] * (a["config"]["blue"] + a["config"]["red"])
                for a in arms
            ),
            393216,
        )
        self.assertEqual(
            (
                len(value["contrast_groups"]),
                len(value["cdf_comparisons"]),
                len(value["scalar_endpoint_ids"]),
                len(value["cdf_endpoints"]),
            ),
            (121, 46, 14, 3),
        )
        self.assertEqual(value["observation_times"], [1, 2, 4, 8])
        self.assertEqual(
            value["budgets"],
            {
                "compact_outcome_bytes": 32768,
                "attempt_start_bytes": 1024,
                "audit_frame_bytes": 32768,
                "diagnostic_replicate_index": 0,
                "archive_uncompressed_bytes": 4294967296,
                "rss_bytes": 2147483648,
                "wall_seconds": 43200,
                "workers": 1,
            },
        )
        self.assertLess(len(self.science), 16 * 1024 * 1024)

    def test_profile_balance_mirrors_steps_and_replication(self):
        arms = manifest.proposal(ROOT)["arms"]
        lookup = {a["arm_id"]: a for a in arms}
        for arm in arms:
            c = arm["config"]
            self.assertEqual(c["dt"] * c["max_steps"], 8)
            self.assertTrue(0 <= c["blue"] <= 4096 and 0 <= c["red"] <= 4096)
            self.assertTrue(1 <= c["max_steps"] <= 1000000)
            self.assertLessEqual((c["blue"] + c["red"]) * c["max_steps"], 64000000)
            self.assertLessEqual(max(c["blue_rate"], c["red_rate"]) * c["dt"], 0.1)
            self.assertEqual(
                [int(t / c["dt"]) for t in (1, 2, 4, 8)],
                {
                    128: [16, 32, 64, 128],
                    256: [32, 64, 128, 256],
                    512: [64, 128, 256, 512],
                }[c["max_steps"]],
            )
            self.assertEqual(
                arm["replicates"], 2048 if arm["group"] == "oracle" else 512
            )
            if arm["mirror"]:
                orig = lookup[
                    arm["arm_id"].replace("mirror.", "core.", 1).removesuffix(".mirror")
                ]["config"]
                self.assertEqual(
                    (c["blue"], c["red"], c["blue_rate"], c["red_rate"]),
                    (orig["red"], orig["blue"], orig["red_rate"], orig["blue_rate"]),
                )
            elif arm["profile"] == "NC":
                self.assertEqual(c["blue"] * 16, c["red"] * 17)
            elif arm["profile"] == "L":
                self.assertEqual(c["blue_rate"] * c["blue"], c["red_rate"] * c["red"])
            elif arm["profile"] == "S":
                self.assertEqual(
                    c["blue_rate"] * c["blue"] ** 2, c["red_rate"] * c["red"] ** 2
                )

    def test_fixture_has_separate_eight_arm_sixteen_key_population(self):
        parsed = manifest.validate_manifest(self.fixture, ROOT)
        self.assertEqual(parsed.purpose, "construction_fixture")
        self.assertEqual((len(parsed.arms), len(parsed.cases)), (8, 16))
        self.assertEqual([c.seed for c in parsed.cases], [7, 8] * 8)
        self.assertTrue(all(a["arm_id"].startswith("fixture.") for a in parsed.arms))
        self.assertEqual(
            {(a["config"]["blue"], a["config"]["red"]) for a in parsed.arms},
            {(1, 1), (2, 1), (64, 64), (512, 256)},
        )
        self.assertEqual(parsed.sha256, hashlib.sha256(self.fixture).hexdigest())
        self.assertEqual(parsed.raw_bytes, self.fixture)

    def test_validated_contract_cannot_be_mutated(self):
        parsed = manifest.validate_manifest(self.fixture, ROOT)
        with self.assertRaises((TypeError, AttributeError)):
            parsed.arms[0]["config"]["blue"] = 99
        with self.assertRaises((TypeError, AttributeError)):
            parsed.raw["cases"].pop()

    def test_u64_max_is_accepted(self):
        data = manifest.sample_manifest(ROOT, lambda n: b"\xff" * 8)
        self.assertEqual(
            {c.seed for c in manifest.validate_manifest(data, ROOT).cases}, {2**64 - 1}
        )

    def test_bad_seed_types_and_bounds_are_refused(self):
        for seed in (True, -1, 1.5, 7.0, 2**64):
            raw = json.loads(self.fixture)
            raw["cases"][0]["seed"] = seed
            with self.subTest(seed=seed):
                self.reject(raw)

    def test_manifest_contract_mutations_are_refused(self):
        raw = json.loads(self.fixture)
        changes = [
            ("schema", "wrong"),
            ("purpose", "wrong"),
            ("protocol_sha256", "0" * 64),
            ("proposal_sha256", "0" * 64),
            ("roster_sha256", "0" * 64),
        ]
        for field, value in changes:
            changed = deepcopy(raw)
            changed[field] = value
            with self.subTest(field=field):
                self.reject(changed)
        for field, value in [
            ("profile", "wrong"),
            ("replicates", 1),
            ("scale", True),
            ("mirror", 0),
            ("unknown", 1),
        ]:
            changed = deepcopy(raw)
            changed["arms"][0][field] = value
            with self.subTest(arm_field=field):
                self.reject(changed)
        for field, value in [
            ("blue_rate", 10**400),
            ("blue_rate", 0.5),
            ("dt", 0.03125),
            ("max_steps", 256),
            ("blue", True),
            ("geometry", "wrong"),
            ("extra", 1),
        ]:
            changed = deepcopy(raw)
            changed["arms"][0]["config"][field] = value
            with self.subTest(config_field=field):
                self.reject(changed)
        for field, value in [
            ("ordinal", True),
            ("ordinal", 1),
            ("arm_id", "unregistered"),
            ("replicate_index", 1),
            ("extra", 1),
        ]:
            changed = deepcopy(raw)
            changed["cases"][0][field] = value
            with self.subTest(case_field=field):
                self.reject(changed)
        for field, value in [
            ("bytes_drawn", True),
            ("bytes_drawn", 1),
            ("method", "wrong"),
            ("seed_bytes_sha256", "0" * 64),
            ("command", []),
            ("version", "wrong"),
            ("extra", 1),
        ]:
            changed = deepcopy(raw)
            changed["sampler"][field] = value
            with self.subTest(sampler_field=field):
                self.reject(changed)
        changed = deepcopy(raw)
        changed["extra"] = 1
        self.reject(changed)

    def test_duplicate_missing_reordered_keys_and_arms_are_refused(self):
        raw = json.loads(self.fixture)
        for field in ("cases", "arms"):
            for mode in ("duplicate", "missing", "reverse"):
                changed = deepcopy(raw)
                if mode == "duplicate":
                    changed[field][1] = deepcopy(changed[field][0])
                if mode == "missing":
                    changed[field].pop()
                if mode == "reverse":
                    changed[field].reverse()
                with self.subTest(field=field, mode=mode):
                    self.reject(changed)

    def test_edited_roster_protocol_matrix_or_cap_is_not_trusted(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            for path in (manifest.PROPOSAL, manifest.PROTOCOL):
                (root / path).parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / path, root / path)
            original = (root / manifest.PROPOSAL).read_bytes()
            for mutate in ("profile", "matrix", "cap", "subset"):
                raw = json.loads(original)
                if mutate == "profile":
                    raw["arms"][0]["profile"] = "changed"
                if mutate == "matrix":
                    raw["contrast_groups"].pop()
                if mutate == "cap":
                    raw["budgets"]["workers"] = 2
                if mutate == "subset":
                    raw["arms"].pop()
                (root / manifest.PROPOSAL).write_text(json.dumps(raw))
                with self.subTest(mutate=mutate), self.assertRaises(ValueError):
                    manifest.proposal(root)
            (root / manifest.PROPOSAL).write_bytes(original)
            with (root / manifest.PROTOCOL).open("ab") as file:
                file.write(b"changed")
            with self.assertRaises(ValueError):
                manifest.make_fixture(root)

    def test_checked_in_fixture_is_canonical_factory_output(self):
        self.assertEqual(
            (
                ROOT / "survey/war1_study/fixtures/construction-manifest.json"
            ).read_bytes(),
            self.fixture,
        )
