import unittest
from survey.war1_study import analysis


class MatrixTests(unittest.TestCase):
    def test_fixed_family_maps_side_swap_exactly(self):
        self.assertEqual(
            analysis.mapped_endpoint("population.blue.t2", "swap_blue_red"),
            "population.red.t2",
        )
        self.assertEqual(
            analysis.mapped_endpoint("ending.horizon", "swap_blue_red"),
            "ending.horizon",
        )


class FullSyntheticTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        from survey.war1_study import test_fixtures as fixtures, archive

        cls.manifest = fixtures.synthetic_manifest()
        arms = {a["arm_id"]: a for a in cls.manifest.arms}

        def rows():
            for case in cls.manifest.cases:
                yield fixtures.synthetic_row(
                    arms[case.key.arm_id], case, cls.manifest.sha256
                )

        cls.archive = archive.ArchiveRead(
            dict(
                evidence_label="synthetic_construction_only",
                validation=dict(primary_available=True, registered=False, reason=None),
            ),
            rows,
            dict(planned=52224, complete_valid=52224),
            (),
            True,
        )
        from unittest.mock import patch

        with patch(
            "os.urandom", side_effect=AssertionError("scientific entropy forbidden")
        ), patch(
            "subprocess.Popen",
            side_effect=AssertionError("simulation subprocess forbidden"),
        ):
            cls.result = analysis.analyze(cls.manifest, cls.archive, ())

    def test_full_planned_family_keeps_every_unavailable_oracle_slot(self):
        r = self.result
        self.assertEqual(
            (len(r.scalars), len(r.cdfs), len(r.contrasts), len(r.cdf_comparisons)),
            (924, 198, 1694, 46),
        )
        self.assertEqual(len(r.oracle_checks), 234)  # 12*(14+3) + 6*5 CTMC margins
        self.assertTrue(
            all(x["reason"] == "oracle_unavailable" for x in r.oracle_checks)
        )

    def test_same_arm_unchanged_variables_cancel_exactly(self):
        rows = [
            r
            for r in self.result.contrasts
            if r["kind"] == "within_arm_symmetry"
            and r["endpoint_id"] == "ending.horizon"
        ]
        self.assertEqual(len(rows), 16)
        self.assertTrue(
            all(
                r["interval"]["lo"] == {"numerator": "0", "denominator": "1"}
                and r["interval"]["hi"] == {"numerator": "0", "denominator": "1"}
                for r in rows
            )
        )

    def test_cdf_all_support_right_continuous_and_atom_at_one(self):
        duration = self.result.cdfs[0]
        self.assertEqual(len(duration["support"]), 128)
        self.assertEqual(
            duration["support"][0]["point"], {"numerator": "1", "denominator": "128"}
        )
        self.assertEqual(
            duration["support"][-1]["cdf"], {"numerator": "1", "denominator": "1"}
        )
        self.assertEqual(
            duration["censor_fraction"], {"numerator": "0", "denominator": "1"}
        )
        self.assertEqual(len(self.result.cdfs[1]["support"]), 65)

    def test_every_contrast_mapping_and_literal_side_swap(self):
        from survey.war1_study.manifest import proposal
        from survey.war1_study.test_fixtures import ROOT

        roster = proposal(ROOT)
        pairs = {
            (row["contrast_id"], row["endpoint_id"]): row
            for row in self.result.contrasts
        }
        for group in roster["contrast_groups"]:
            for endpoint_id in roster["scalar_endpoint_ids"]:
                row = pairs[group["contrast_id"], endpoint_id]
                wanted = (
                    endpoint_id
                    if group["right_endpoint_mapping"] == "identity"
                    else roster["side_swap_endpoint_mapping"][endpoint_id]
                )
                self.assertEqual(row["right_endpoint_id"], wanted)
                if group["kind"] == "side_swap" and ".NR." not in group["left_arm"]:
                    self.assertEqual(
                        row["estimate"], {"numerator": "0", "denominator": "1"}
                    )
        equality = next(
            row
            for row in self.result.contrasts
            if row["contrast_id"] == "equality:core.E.N64.aimed_fire.dt1-16"
            and row["endpoint_id"] == "population.blue.t1"
        )
        self.assertEqual(equality["estimate"], {"numerator": "1", "denominator": "1"})

    def test_independent_reanalysis_is_byte_identical_with_bounded_reports(self):
        import tempfile
        from pathlib import Path
        from unittest.mock import patch

        with patch(
            "os.urandom", side_effect=AssertionError("entropy forbidden")
        ), patch(
            "subprocess.Popen", side_effect=AssertionError("subprocess forbidden")
        ):
            again = analysis.analyze(self.manifest, self.archive, ())
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary).resolve()
            analysis.save_analysis(
                root / "first",
                self.result,
                {"evidence_label": "synthetic_construction_only"},
            )
            analysis.save_analysis(
                root / "second",
                again,
                {"evidence_label": "synthetic_construction_only"},
            )
            for path in (root / "first").iterdir():
                self.assertLessEqual(path.stat().st_size, 16777216)
                self.assertEqual(
                    path.read_bytes(), (root / "second" / path.name).read_bytes()
                )


class SavedSyntheticPopulationTests(unittest.TestCase):
    def test_full_saved_census_streams_all_52224_without_scientific_authorization(self):
        import tempfile
        from pathlib import Path
        from unittest.mock import patch
        from survey.war1_study import archive, test_fixtures as fixtures

        manifest = fixtures.synthetic_manifest()
        with tempfile.TemporaryDirectory(prefix="synthetic-construction-") as temporary:
            root = Path(temporary).resolve()
            fixtures.save_campaign(root, manifest)
            with patch(
                "subprocess.Popen", side_effect=AssertionError("simulator forbidden")
            ), patch(
                "os.urandom", side_effect=AssertionError("scientific entropy forbidden")
            ):
                saved = archive.read_campaign(root, manifest)
                self.assertTrue(
                    saved.complete, saved.header["validation"]["errors"][:3]
                )
                self.assertEqual(
                    saved.census,
                    {
                        "planned": 52224,
                        "attempted": 52224,
                        "complete_valid": 52224,
                        "failed": 0,
                        "invalid": 0,
                        "partial": 0,
                        "pending": 0,
                        "unstarted": 0,
                    },
                )
                self.assertEqual(sum(1 for _ in saved.outcomes()), 52224)
            self.assertFalse(saved.header["validation"]["primary_available"])
            self.assertLess(
                sum(p.stat().st_size for p in root.rglob("*") if p.is_file()),
                4294967296,
            )
