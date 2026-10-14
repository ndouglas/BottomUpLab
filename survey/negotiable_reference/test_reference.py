"""Independent mathematical fixtures and reference integrity behavior."""

import copy
from fractions import Fraction
import itertools
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from survey.negotiable_reference import reference


def wire(q):
    return {"num": str(q.numerator), "den": str(q.denominator)}


class ReferenceTests(unittest.TestCase):
    def test_source_cake_policy_uses_each_principals_prior(self):
        record = reference.build_cake()
        rows = [r for r in record["rows"] if r["policy_id"] == 2]
        self.assertTrue(all(r["expected"] == [wire(Fraction(27)), wire(Fraction(27))]
                            for r in rows))

    def test_reference_rejects_missing_history(self):
        record = reference.build_priority()
        record["rows"].pop()
        with self.assertRaisesRegex(ValueError, "4599"):
            reference.validate_reference(record, "priority")

    def test_source_examples_and_all_nine_expectations(self):
        rows = reference.build_cake()["rows"][:9]
        expected = [(30, 0), (29, 18), (27, 27), (21, 2), (20, 20),
                    (18, 29), (3, 3), (2, 21), (0, 30)]
        self.assertEqual([r["expected"] for r in rows],
                         [[wire(Fraction(a)), wire(Fraction(b))] for a, b in expected])

    def test_boundaries_derived_from_score_equalities_keep_all_ties(self):
        boundaries = [Fraction(1, 19), Fraction(2, 11), Fraction(9, 11), Fraction(18, 19)]
        self.assertEqual(reference.derive_boundaries(), boundaries)
        record = reference.build_cake()
        expected = [[5, 8], [2, 5], [1, 2], [0, 1]]
        for weight, maximizers in zip(boundaries, expected):
            row = next(r for r in record["rows"] if r["weight"] == wire(weight))
            self.assertEqual(row["global_maximizers"], maximizers)
        self.assertTrue(all(r["global_maximizers"] == r["local_maximizers"]
                            for r in record["rows"]))

    def test_mathematical_mixture_only_improvement(self):
        # Synthetic mathematical fixture; this is not a modified cake scenario.
        points = [(Fraction(1), Fraction(1)), (Fraction(3), Fraction(0)),
                  (Fraction(0), Fraction(3))]
        result = reference.classify(points, 0)
        self.assertFalse(result["paper_weak"]["efficient"])
        witness = result["paper_weak"]["witness"]
        self.assertGreater(len(witness["parts"]), 1)
        reference.validate_witness(witness, points, points[0], "paper_weak")

    def test_mathematical_segment_distinguishes_efficiency_definitions(self):
        points = [(Fraction(0), Fraction(0)), (Fraction(0), Fraction(1))]
        result = reference.classify(points, 0)
        self.assertTrue(result["paper_weak"]["efficient"])
        self.assertFalse(result["strong"]["efficient"])

    def test_mathematical_point_hull_retains_duplicate_ids(self):
        point = (Fraction(1), Fraction(1))
        self.assertTrue(reference.classify([point, point], 1)["strong"]["efficient"])
        self.assertEqual(len(reference.build_pareto()["rows"]), 9)

    def test_affine_reduction_preserves_payoff_and_three_part_limit(self):
        points = [(Fraction(0), Fraction(0)), (Fraction(2), Fraction(0)),
                  (Fraction(2), Fraction(2)), (Fraction(0), Fraction(2))]
        weights = {i: Fraction(1, 4) for i in range(4)}
        reduced = reference.reduce_coefficients(weights, points)
        self.assertLessEqual(len(reduced), 3)
        self.assertEqual(sum(reduced.values()), 1)
        self.assertTrue(all(w > 0 for w in reduced.values()))
        self.assertEqual(tuple(sum(w * points[i][axis] for i, w in reduced.items())
                               for axis in range(2)), (1, 1))

    def test_all_priority_prefixes_direct_products_match_sequential(self):
        for model in reference.MODELS:
            for prior in reference.PRIORS:
                for length in range(9):
                    for history in itertools.product((0, 1), repeat=length):
                        self.assertEqual(reference.direct_priority(model, prior, history),
                                         reference.sequential_priority(model, prior, history))

    def test_zero_priors_and_impossibility_remain_absorbing(self):
        model = (Fraction(1), Fraction(4, 5))
        self.assertEqual(reference.direct_priority(model, Fraction(1), [0, 1, 0]),
                         {"status": "impossible", "first_impossible_prefix": 2})
        self.assertEqual(reference.direct_priority(model, Fraction(0), [1, 0]),
                         {"status": "supported", "weights": [wire(Fraction(0)), wire(Fraction(1))]})
        self.assertEqual(reference.direct_priority(model, Fraction(1, 2), [1]),
                         {"status": "supported", "weights": [wire(Fraction(0)), wire(Fraction(1))]})

    def test_every_record_validates_and_roundtrips(self):
        for kind, builder in self.builders():
            record = builder()
            reference.validate_reference(record, kind)
            self.assertEqual(reference.decode_reference(reference.canonical_bytes(record), kind), record)

    def builders(self):
        return [("cake", reference.build_cake), ("pareto", reference.build_pareto),
                ("priority", reference.build_priority)]

    def test_duplicate_json_keys_rejected_before_schema_validation(self):
        with self.assertRaisesRegex(ValueError, "duplicate.*schema"):
            reference.decode_reference(b'{"schema":1,"schema":2}', "cake")

    def test_duplicate_rows_rejected(self):
        for kind, builder in self.builders():
            record = builder()
            record["rows"][-1] = copy.deepcopy(record["rows"][0])
            with self.assertRaisesRegex(ValueError, "duplicate"):
                reference.validate_reference(record, kind)

    def test_stale_settings_and_extra_keys_rejected(self):
        for field in ("schema", "settings", "rows"):
            record = reference.build_cake()
            record[field] = None
            with self.assertRaises(ValueError):
                reference.validate_reference(record, "cake")
        record = reference.build_cake()
        record["settings"]["history_horizon"] = 7
        with self.assertRaisesRegex(ValueError, "settings"):
            reference.validate_reference(record, "cake")
        record = reference.build_cake()
        record["rows"][0]["extra"] = 1
        with self.assertRaisesRegex(ValueError, "keys"):
            reference.validate_reference(record, "cake")

    def test_missing_rows_rejected_for_every_kind(self):
        for kind, builder in self.builders():
            record = builder()
            record["rows"].pop()
            with self.assertRaisesRegex(ValueError, "row count"):
                reference.validate_reference(record, kind)

    def test_noncanonical_fractions_rejected(self):
        for bad in ({"num": "00", "den": "1"}, {"num": "+1", "den": "1"},
                    {"num": "-0", "den": "1"}, {"num": "2", "den": "2"},
                    {"num": "1", "den": "0"}, {"num": "1", "den": "-1"},
                    {"num": 1, "den": "1"}, {"num": str(2**127), "den": "1"}):
            record = reference.build_cake()
            record["rows"][0]["expected"][0] = bad
            with self.assertRaisesRegex(ValueError, "fraction"):
                reference.validate_reference(record, "cake")

    def test_out_of_order_history_rejected(self):
        record = reference.build_priority()
        record["rows"][1], record["rows"][2] = record["rows"][2], record["rows"][1]
        with self.assertRaisesRegex(ValueError, "order"):
            reference.validate_reference(record, "priority")

    def test_invalid_witness_weights_rejected(self):
        record = reference.build_pareto()
        row = next(r for r in record["rows"] if not r["strong"]["efficient"])
        row["strong"]["witness"]["parts"][0]["weight"] = wire(Fraction(-1))
        with self.assertRaisesRegex(ValueError, "weight"):
            reference.validate_reference(record, "pareto")

    def test_tampered_values_or_boolean_ids_rejected(self):
        for kind, builder in self.builders():
            record = builder()
            if kind == "cake":
                record["rows"][0]["objective"] = wire(Fraction(111))
            elif kind == "pareto":
                record["rows"][0]["strong"]["efficient"] = False
            else:
                record["rows"][0]["result"]["weights"][0] = wire(Fraction(1))
            with self.assertRaises(ValueError):
                reference.validate_reference(record, kind)
        record = reference.build_priority()
        record["rows"][0]["model_id"] = False
        with self.assertRaises(ValueError):
            reference.validate_reference(record, "priority")

    def test_oversize_decode_rejected_before_json_parsing(self):
        with patch.object(reference.json, "loads", side_effect=AssertionError("parsed")):
            with self.assertRaisesRegex(ValueError, "8 MiB"):
                reference.decode_reference(b" " * (8 * 1024 * 1024 + 1), "cake")

    def test_exclusive_write_is_canonical_and_preserves_existing_file(self):
        record = reference.build_cake()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cake.json"
            reference.write_exclusive(record, path)
            original = path.read_bytes()
            self.assertEqual(original, (json.dumps(record, sort_keys=True, separators=(",", ":")) + "\n").encode())
            with self.assertRaises(FileExistsError):
                reference.write_exclusive(record, path)
            self.assertEqual(path.read_bytes(), original)

    def test_oversize_write_does_not_create_output(self):
        record = reference.build_cake()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cake.json"
            with patch.object(reference, "MAX_BYTES", 100):
                with self.assertRaisesRegex(ValueError, "8 MiB"):
                    reference.write_exclusive(record, path)
            self.assertFalse(path.exists())

    def test_malformed_kind_is_contextual_value_error(self):
        record = reference.build_cake()
        record["settings"]["reference_kind"] = []
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(ValueError, "kind"):
                reference.write_exclusive(record, Path(directory) / "invalid.json")


if __name__ == "__main__":
    unittest.main()
