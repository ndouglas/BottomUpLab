"""Independent arithmetic fixtures catch inward rounding and native bit tampering."""

from fractions import Fraction as F
import math
import struct
import unittest
from survey.war1_study import numerics


def bits(x):
    return struct.pack(">d", x).hex()


def target(dose=1 / 64):
    p = -math.expm1(-dose)
    return dict(
        config=dict(
            blue=1,
            red=1,
            blue_rate=1,
            red_rate=1,
            dt=dose,
            max_steps=512,
            geometry="aimed_fire",
        ),
        blue=1,
        red=1,
        target_index=0,
        hazard_bits=bits(1.0),
        dose_bits=bits(dose),
        p_bits=bits(p),
        q_numerator=math.ceil(p * 2**53),
    )


class NumericsTests(unittest.TestCase):
    def test_interval_rounding_is_outward(self):
        bounds = numerics.enclose(F(1, 3))
        self.assertEqual(
            (bounds.lo, bounds.hi), (F(333333333333, 10**12), F(333333333334, 10**12))
        )

    def test_exp_encloses_known_positive_value(self):
        bounds = numerics.exp_interval(F(0))
        self.assertLessEqual(bounds.lo, 1)
        self.assertGreaterEqual(bounds.hi, 1)

    def test_probability_certifies_exact_dose_and_quantization(self):
        cert = numerics.probability_certificate(target())
        self.assertEqual(cert["q_numerator"], 139643681050716)
        self.assertEqual(cert["dose_error"], numerics.Interval(F(0), F(0)).wire())

    def test_rejects_corrupted_native_bits(self):
        for key, value in [
            ("p_bits", bits(0.1)),
            ("dose_bits", bits(0.02)),
            ("hazard_bits", bits(float("inf"))),
            ("q_numerator", 1),
        ]:
            row = target()
            row[key] = value
            with self.subTest(key=key), self.assertRaises(ValueError):
                numerics.probability_certificate(row)

    def test_q_threshold_uses_ceiling_of_exact_p(self):
        p = F(1, 8)
        self.assertEqual(numerics.quantized_numerator(p), 2**50)
        self.assertEqual(
            numerics.quantized_numerator(F.from_float(math.nextafter(0.125, 1.0))),
            2**50 + 1,
        )

    def test_confidence_radius_encloses_high_precision_literal(self):
        radius = numerics.confidence_radius(924, 512, F(1, 40))
        reference = F(
            "0.104632628839603827947237775509783168683750434611563443898083044168095109277100712307738739747439763465063734241150219305785941399454850321533289853426"
        )
        self.assertLessEqual(radius.lo, reference - F(1, 10**150))
        self.assertGreaterEqual(radius.hi, reference + F(1, 10**150))
        self.assertLess(radius.hi - radius.lo, F(1, 10**80))

    def test_wire_grid_widens_without_discarding_certificate_precision(self):
        exact = F(1, 3)
        interval = numerics.Interval(exact, exact)
        self.assertEqual(interval.wire()["lo"], {"numerator": "1", "denominator": "3"})
        self.assertEqual(
            interval.wire(scale=10)["lo"], {"numerator": "3", "denominator": "10"}
        )
        self.assertEqual(
            interval.wire(scale=10)["hi"], {"numerator": "2", "denominator": "5"}
        )

    def test_unresolved_rounding_refuses_after_fixed_refinement(self):
        from unittest.mock import patch

        with patch.object(
            numerics, "exp_interval", return_value=numerics.Interval(F(0), F(1))
        ) as exp:
            with self.assertRaisesRegex(ValueError, "unresolved"):
                numerics.probability_certificate(target())
            self.assertEqual(
                [call.args[1] for call in exp.call_args_list], [100, 200, 400]
            )

    def test_exp_underflow_is_refused(self):
        with self.assertRaises(ValueError):
            numerics.exp_interval(F(-(10**7)))

    def test_four_ulp_complete_interval_boundary(self):
        row = target()
        p = struct.unpack(">d", bytes.fromhex(row["p_bits"]))[0]
        for _ in range(6):
            p = math.nextafter(p, math.inf)
        row["p_bits"] = bits(p)
        row["q_numerator"] = math.ceil(p * 2**53)
        with self.assertRaisesRegex(ValueError, "four-ULP"):
            numerics.probability_certificate(row)

    def test_exp_extreme_overflow_and_subnormal_are_refused(self):
        for exponent in (10**7, -2302600):
            with self.subTest(exponent=exponent), self.assertRaises(ValueError):
                numerics.exp_interval(F(exponent))

    def test_malformed_approved_config_is_refused(self):
        for field, value in [("blue", True), ("blue", 0), ("max_steps", 1), ("red", 2)]:
            row = target()
            row["config"][field] = value
            with self.subTest(field=field, value=value), self.assertRaises(ValueError):
                numerics.probability_certificate(row)

    def test_exp_contains_independent_150_digit_reference(self):
        reference = F(
            "0.939413062813475786119710824622305084524680890549441822009492662053540722242363319290925980170489343655974867381457012333951954788337951593064093139814"
        )
        interval = numerics.exp_interval(F(-1, 16))
        self.assertLessEqual(interval.lo, reference - F(1, 10**150))
        self.assertGreaterEqual(interval.hi, reference + F(1, 10**150))
