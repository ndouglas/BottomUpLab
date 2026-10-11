from fractions import Fraction as F
import unittest
from survey.war1_study import analysis, test_fixtures as fixtures
from survey.war1_study.numerics import Interval, difference


class EndpointTests(unittest.TestCase):
    def test_actual_horizon_extinction_is_not_censored(self):
        extinction = fixtures.outcome(
            ending="one_side_extinction", step=512, winner="blue", censored=False
        )
        censor = fixtures.outcome(
            ending="horizon", step=512, winner=None, censored=True
        )
        self.assertEqual(
            (
                analysis.endpoint(extinction, "ending.horizon"),
                analysis.endpoint(censor, "ending.horizon"),
            ),
            (0, 1),
        )
        self.assertEqual(
            analysis.endpoint(extinction, "duration.restricted_mean_over_H"), 1
        )
        self.assertEqual(
            analysis.endpoint(censor, "duration.restricted_mean_over_H"), 1
        )

    def test_source_horizon_not_actual_end_is_duration_denominator(self):
        row = fixtures.outcome(
            ending="one_side_extinction", step=1, winner="blue", censored=False
        )
        self.assertEqual(
            analysis.endpoint(row, "duration.restricted_mean_over_H"), F(1, 512)
        )
        self.assertEqual(analysis.endpoint(row, "population.blue.t8"), F(1, 64))

    def test_zero_denominator_and_rate_zero_are_unavailable(self):
        row = fixtures.outcome(ending="rate_zero", step=0, winner=None, censored=False)
        self.assertIsNone(analysis.endpoint(row, "duration.restricted_mean_over_H"))
        row["initial_counts"][0] = 0
        self.assertIsNone(analysis.endpoint(row, "population.blue.t1"))

    def test_unknown_endpoint_is_rejected(self):
        with self.assertRaises(ValueError):
            analysis.endpoint({}, "unknown")

    def test_difference_encloses_left_minus_right(self):
        self.assertEqual(
            difference(Interval(F(1, 4), F(1, 2)), Interval(F(1, 8), F(3, 4))),
            Interval(F(-1, 2), F(3, 8)),
        )


class AggregateTests(unittest.TestCase):
    def setUp(self):
        from survey.war1_study import manifest, archive

        self.manifest = manifest.validate_manifest(
            manifest.make_fixture(fixtures.ROOT), fixtures.ROOT
        )
        arms = {a["arm_id"]: a for a in self.manifest.arms}
        self.rows = [
            fixtures.synthetic_row(arms[c.key.arm_id], c, self.manifest.sha256)
            for c in self.manifest.cases
        ]
        self.archive = archive.ArchiveRead(
            dict(
                validation=dict(
                    primary_available=False,
                    registered=False,
                    reason="unregistered_construction_panel",
                )
            ),
            lambda: iter(self.rows),
            dict(planned=16, complete_valid=16),
            (),
            True,
        )

    def test_fixture_reports_descriptive_values_without_sampling_claims(self):
        result = analysis.analyze(self.manifest, self.archive, ())
        self.assertEqual((len(result.scalars), len(result.cdfs)), (112, 24))
        first = result.scalars[0]
        self.assertEqual(first["mean"], {"numerator": "1", "denominator": "1"})
        self.assertIsNone(first["interval"])
        self.assertEqual(first["reason"], "unregistered_construction_panel")
        self.assertFalse(result.primary_available)

    def test_missing_outcome_invalidates_every_primary_slot(self):
        self.rows.pop()
        result = analysis.analyze(self.manifest, self.archive, ())
        self.assertTrue(all(r["mean"] is None for r in result.scalars))
        self.assertTrue(
            all(r["reason"] == "incomplete_archive" for r in result.scalars)
        )

    def test_duplicate_key_rejects_replacement_even_with_equal_seed(self):
        self.rows[-1] = self.rows[0]
        result = analysis.analyze(self.manifest, self.archive, ())
        self.assertFalse(result.primary_available)
        self.assertTrue(all(r["mean"] is None for r in result.scalars))


class OracleComparisonTests(unittest.TestCase):
    setUp = AggregateTests.setUp

    def test_fixture_oracle_matching_uses_config_not_scientific_identifier(self):
        from survey.war1_study.oracle import Oracle

        arm = self.manifest.arms[0]
        oracle = Oracle(
            "oracle.1v1.N1.aimed_fire.dt1-64",
            {"ending.blue_survivor": Interval(F(0), F(0))},
            {},
            dict(
                manifest_sha256=self.manifest.sha256,
                states=[dict(config=arm["config"])],
            ),
            None,
        )
        result = analysis.analyze(self.manifest, self.archive, (oracle,))
        row = next(
            r
            for r in result.oracle_checks
            if r["arm_id"] == arm["arm_id"]
            and r["endpoint_id"] == "ending.blue_survivor"
        )
        self.assertEqual(row["oracle_id"], oracle.arm_id)
        self.assertIsNone(row["verdict"])
        self.assertEqual(row["reason"], "unregistered_construction_panel")

    def test_scalar_interval_separation_is_not_rejected_on_overlap(self):
        from survey.war1_study.oracle import Oracle
        from dataclasses import replace

        arm = self.manifest.arms[0]
        saved = replace(
            self.archive,
            header=dict(
                validation=dict(primary_available=True, registered=False, reason=None)
            ),
        )

        def check(bound):
            oracle = Oracle(
                "oracle.1v1.N1.aimed_fire.dt1-64",
                {"ending.blue_survivor": bound},
                {},
                dict(
                    manifest_sha256=self.manifest.sha256,
                    states=[dict(config=arm["config"])],
                ),
                None,
            )
            result = analysis.analyze(self.manifest, saved, (oracle,))
            return next(
                r
                for r in result.oracle_checks
                if r["arm_id"] == arm["arm_id"]
                and r["endpoint_id"] == "ending.blue_survivor"
            )["verdict"]

        self.assertEqual(check(Interval(F(1), F(1))), "not_rejected")
        # n=2 gives a clipped [0,1] confidence interval, so zero overlaps too.
        self.assertEqual(check(Interval(F(0), F(0))), "not_rejected")

    def test_provided_oracle_must_match_validated_saved_packet(self):
        from dataclasses import replace
        from survey.war1_study.oracle import Oracle

        arm = self.manifest.arms[0]
        oracle = Oracle(
            "oracle.1v1.N1.aimed_fire.dt1-64",
            {"ending.blue_survivor": Interval(F(1), F(1))},
            {},
            dict(
                manifest_sha256=self.manifest.sha256,
                states=[dict(config=arm["config"])],
            ),
            None,
        )
        saved = replace(
            self.archive,
            header=dict(
                validated_oracle_rows_sha256="0" * 64,
                validation=dict(primary_available=True, registered=False, reason=None),
            ),
        )
        result = analysis.analyze(self.manifest, saved, (oracle,))
        self.assertTrue(
            all(
                r["verdict"] is None
                for r in result.oracle_checks
                if r["kind"] != "ctmc_one_vs_one"
            )
        )


class ConstructionAvailabilityTests(unittest.TestCase):
    setUp = AggregateTests.setUp

    def test_missing_ideal_does_not_remove_construction_sampling_disclaimer(self):
        result = analysis.analyze(self.manifest, self.archive, ())
        self.assertTrue(
            all(
                r["reason"] == "unregistered_construction_panel"
                for r in result.oracle_checks
                if r["kind"] in ("scalar", "cdf")
            )
        )
