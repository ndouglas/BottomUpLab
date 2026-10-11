"""Independent full-roster arithmetic, without simulation or seed draws."""

from copy import deepcopy
from fractions import Fraction as F
from pathlib import Path
import unittest
from survey.war1_study import preparation
from survey.war1_study.manifest import proposal

ROOT = Path(__file__).resolve().parents[2]


class DoseProofTests(unittest.TestCase):
    def proof(self):
        self.assertTrue(
            callable(getattr(preparation, "dose_proof", None)),
            "preparation must independently prove all approved arms",
        )
        return preparation.dose_proof(ROOT, "a" * 64, "b" * 64)

    def test_all_arms_both_targets_exact_positive_extrema_and_quantization(self):
        proof = self.proof()
        approved = proposal(ROOT)["arms"]
        self.assertEqual(
            [a["arm_id"] for a in proof["arms"]], [a["arm_id"] for a in approved]
        )
        self.assertEqual(proof["interior_target_states"], 1367076)
        self.assertEqual(proof["positive_dose_extrema"], ["1/32768", "8/1"])
        self.assertEqual(proof["maximum_case_work"], 393216)
        self.assertEqual(proof["quantization_only_path_bound"], "3/68719476736")
        for arm, source in zip(proof["arms"], approved):
            c = source["config"]
            self.assertEqual(arm["interior_counts"], [c["blue"], c["red"]])
            for side, target, attacker in (
                (0, c["blue"], c["red"]),
                (1, c["red"], c["blue"]),
            ):
                rate = F(c["red_rate"] if side == 0 else c["blue_rate"])
                base = rate * F(c["dt"])
                extrema = (
                    [base / target, base * attacker]
                    if c["geometry"] == "aimed_fire"
                    else [base, base]
                )
                self.assertEqual(list(map(F, arm["targets"][side])), extrema)
        self.assertGreater(F(proof["probability_lower_bound"]), F(1, 2**53))
        self.assertGreater(F(proof["survival_lower_bound"]), F(1, 2**53))

    def test_exact_integer_enumeration_covers_every_interior_ratio(self):
        proof = self.proof()
        checked = 0
        for arm, approved in zip(proof["arms"], proposal(ROOT)["arms"]):
            c = approved["config"]
            for side, rate in enumerate((c["red_rate"], c["blue_rate"])):
                base = F(rate) * F(c["dt"])
                lo, hi = map(F, arm["targets"][side])
                for blue in range(1, c["blue"] + 1):
                    for red in range(1, c["red"] + 1):
                        source, target = (red, blue) if side == 0 else (blue, red)
                        n, d = base.numerator, base.denominator
                        if c["geometry"] == "aimed_fire":
                            n, d = n * source, d * target
                        # Integer cross-products avoid all floating point and
                        # enumerate the complete domain independently of corners.
                        if (
                            n * lo.denominator < lo.numerator * d
                            or n * hi.denominator > hi.numerator * d
                        ):
                            self.fail(
                                f"dose outside retained extrema: {arm['arm_id']} {side} {blue} {red}"
                            )
                        checked += 1
        self.assertEqual(checked, 1367076)

    def test_recompute_rejects_omitted_changed_malformed_and_mismatched_proof(self):
        good = self.proof()
        for mutation in (
            "arm",
            "config",
            "coverage",
            "extrema",
            "source",
            "manifest",
            "extra",
            "type",
            "rounding",
            "quantization",
        ):
            bad = deepcopy(good)
            if mutation == "arm":
                bad["arms"].pop()
            elif mutation == "config":
                bad["arms"][0]["config_sha256"] = "f" * 64
            elif mutation == "coverage":
                bad["arms"][0]["interior_counts"][0] -= 1
            elif mutation == "extrema":
                bad["arms"][0]["targets"][0][0] = "0/1"
            elif mutation == "source":
                bad["compiled_inputs_sha256"] = "f" * 64
            elif mutation == "manifest":
                bad["manifest_sha256"] = "f" * 64
            elif mutation == "extra":
                bad["valid"] = True
            elif mutation == "type":
                bad["maximum_case_work"] = float(bad["maximum_case_work"])
            elif mutation == "rounding":
                bad["native_relative_error_bound"] = "0/1"
            elif mutation == "quantization":
                bad["quantization_only_path_bound"] = "0/1"
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                preparation.validate_dose_proof(bad, ROOT, "a" * 64, "b" * 64)
