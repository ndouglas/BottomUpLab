"""Hand-derived count laws independent of native settlement."""

from fractions import Fraction as F
import unittest
from survey.war1_study import oracle


class OracleTests(unittest.TestCase):
    def test_geometric_one_step_categories(self):
        self.assertEqual(oracle.geometric_sum(F(3, 8), 2), F(11, 8))
        self.assertEqual(
            oracle.count_transition(1, 1, (1, 1), (F(1, 4), F(1, 2))),
            {(1, 0): F(3, 8), (0, 1): F(1, 8), (0, 0): F(1, 8), (1, 1): F(3, 8)},
        )

    def test_two_vs_one_product_binomial(self):
        self.assertEqual(
            oracle.count_transition(2, 1, (2, 1), (F(1, 2), F(1, 4))),
            {
                (2, 1): F(3, 16),
                (2, 0): F(1, 16),
                (1, 1): F(3, 8),
                (1, 0): F(1, 8),
                (0, 1): F(3, 16),
                (0, 0): F(1, 16),
            },
        )

    def test_empty_side_absorbs(self):
        self.assertEqual(
            oracle.count_transition(2, 0, (0, 0), (F(1, 2), F(1, 2))), {(2, 0): F(1)}
        )

    def test_s_one_mathematical_limit(self):
        self.assertEqual(oracle.geometric_sum(F(1), 512), 512)


class PreparedOracleTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        from pathlib import Path
        from survey.war1_study import manifest
        from survey.war1_study.test_numerics import bits
        import math

        cls.root = Path(__file__).resolve().parents[2]
        cls.manifest = manifest.validate_manifest(
            manifest.make_fixture(cls.root), cls.root
        )
        cls.export = dict(
            schema="war1-study-numerics-v1",
            purpose="construction_fixture",
            manifest_sha256=cls.manifest.sha256,
            rules_id="war1-engagement-v1",
            binding_state="engineering_unbound",
            compiled_inputs_sha256=None,
            states=[],
        )
        for arm in manifest.proposal(cls.root)["arms"]:
            if arm["group"] != "oracle":
                continue
            c = arm["config"]
            for blue in range(1, c["blue"] + 1):
                targets = []
                for hazard in (
                    (1 / blue, blue) if c["geometry"] == "aimed_fire" else (1, 1)
                ):
                    dose = hazard * float(c["dt"])
                    p = -math.expm1(-dose)
                    targets.append(
                        dict(
                            hazard_bits=bits(hazard),
                            dose_bits=bits(dose),
                            p_bits=bits(p),
                            q_numerator=math.ceil(p * 2**53),
                        )
                    )
                cls.export["states"].append(
                    dict(arm_id=arm["arm_id"], blue=blue, red=1, targets=targets)
                )

    def test_all_approved_descriptors_and_censor_atom_are_retained(self):
        refs = oracle.prepare_oracles(self.manifest, self.export)
        self.assertEqual(len(refs), 12)
        for ref in refs:
            self.assertEqual(len(ref.scalars), 14)
            self.assertTrue(ref.arm_id.startswith("oracle."))
            duration = ref.cdfs["duration.U_over_H"]
            self.assertEqual(duration[-1], (F(1), oracle.Interval(F(1), F(1))))
            self.assertLess(duration[-2][1].hi, F(1))
            self.assertLessEqual(
                max(v.hi - v.lo for v in ref.scalars.values()), F(1, 10**10)
            )
            if ref.ctmc_one_vs_one:
                state = next(
                    row for row in self.export["states"] if row["arm_id"] == ref.arm_id
                )
                qb, qr = (F(t["q_numerator"], 2**53) for t in state["targets"])
                k = len(duration)
                s = (1 - qb) * (1 - qr)
                a = (1 - s**k) / (1 - s)
                expected = {
                    "ending.blue_survivor": (1 - qb) * qr * a,
                    "ending.red_survivor": qb * (1 - qr) * a,
                    "ending.double_extinction": qb * qr * a,
                    "ending.horizon": s**k,
                    "duration.restricted_mean_over_H": a / k,
                }
                for endpoint, value in expected.items():
                    self.assertLessEqual(ref.scalars[endpoint].lo, value)
                    self.assertGreaterEqual(ref.scalars[endpoint].hi, value)
                self.assertEqual(len(ref.ctmc_one_vs_one["criteria"]), 5)
                if ref.arm_id.endswith("dt1-64"):
                    self.assertTrue(
                        all(
                            x["passes"]
                            for x in ref.ctmc_one_vs_one["criteria"].values()
                        )
                    )

    def test_numeric_identity_and_state_census_must_match(self):
        from copy import deepcopy

        for field, value in [
            ("manifest_sha256", "0" * 64),
            ("rules_id", "wrong"),
            ("states", []),
            ("compiled_inputs_sha256", "0" * 64),
        ]:
            data = deepcopy(self.export)
            data[field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                oracle.prepare_oracles(self.manifest, data)

    def test_source_bound_export_accepts_well_formed_stamp_and_refuses_invalid_binding(
        self,
    ):
        from copy import deepcopy

        data = deepcopy(self.export)
        data["binding_state"] = "source_bound"
        data["compiled_inputs_sha256"] = "ab" * 32
        self.assertEqual(len(oracle.prepare_oracles(self.manifest, data)), 12)
        for stamp in (None, "", "AB" * 32, "ab" * 31, True):
            data["compiled_inputs_sha256"] = stamp
            with self.subTest(stamp=stamp), self.assertRaises(ValueError):
                oracle.prepare_oracles(self.manifest, data)

    def test_modified_p_and_q_are_refused(self):
        from copy import deepcopy

        for field, value in [("p_bits", "3fb999999999999a"), ("q_numerator", 1)]:
            data = deepcopy(self.export)
            data["states"][0]["targets"][0][field] = value
            with self.subTest(field=field), self.assertRaises(ValueError):
                oracle.prepare_oracles(self.manifest, data)


class FiniteHorizonTests(unittest.TestCase):
    def test_one_step_keeps_final_extinction_separate_from_censoring(self):
        ref = oracle.finite_horizon(
            (1, 1), 1, {(1, 1): (F(1, 4), F(1, 2))}, "aimed_fire", (1,)
        )
        self.assertEqual(
            ref["endings"],
            dict(
                blue_survivor=F(3, 8),
                red_survivor=F(1, 8),
                double_extinction=F(1, 8),
                rate_zero=F(0),
                horizon=F(3, 8),
            ),
        )
        self.assertEqual(ref["duration_cdf"], ((F(1), F(1)),))
        self.assertEqual(ref["duration_mean"], F(1))

    def test_two_steps_restricted_duration_and_survivor_counts(self):
        ref = oracle.finite_horizon(
            (1, 1), 2, {(1, 1): (F(1, 4), F(1, 2))}, "aimed_fire", (1, 2)
        )
        self.assertEqual(ref["endings"]["horizon"], F(9, 64))
        self.assertEqual(ref["endings"]["blue_survivor"], F(33, 64))
        self.assertEqual(ref["duration_mean"], F(11, 16))
        self.assertEqual(ref["duration_cdf"], ((F(1, 2), F(5, 8)), (F(1), F(1))))
        self.assertEqual(ref["means"][2], (F(21, 32), F(5, 16)))

    def test_zero_q_mathematical_limit(self):
        ref = oracle.finite_horizon(
            (1, 1), 2, {(1, 1): (F(0), F(0))}, "duel_contact", (1, 2)
        )
        self.assertEqual(ref["endings"]["horizon"], 1)
        self.assertEqual(ref["duration_mean"], 1)

    def test_two_vs_one_duel_exposes_only_one_per_side(self):
        ref = oracle.finite_horizon(
            (2, 1),
            1,
            {(2, 1): (F(1, 2), F(1, 2)), (1, 1): (F(1, 2), F(1, 2))},
            "duel_contact",
            (1,),
        )
        self.assertEqual(
            ref["endings"],
            dict(
                blue_survivor=F(1, 2),
                red_survivor=F(0),
                double_extinction=F(0),
                rate_zero=F(0),
                horizon=F(1, 2),
            ),
        )
        self.assertEqual(ref["means"][1], (F(3, 4), F(1, 2)))
        self.assertEqual(
            ref["survivor_cdfs"],
            (
                ((F(0), F(0)), (F(1, 2), F(1, 2)), (F(1), F(1))),
                ((F(0), F(1, 2)), (F(1), F(1))),
            ),
        )
