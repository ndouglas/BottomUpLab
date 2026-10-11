"""Strict JSON and bounded create-only filesystem behavior."""

from decimal import Decimal
import os
from pathlib import Path
import tempfile
import unittest
from survey.war1_study import io


class JsonTests(unittest.TestCase):
    def test_strict_numbers_preserve_integer_and_decimal_types(self):
        self.assertEqual(
            io.strict_json(b'{"seed":18446744073709551615,"dt":0.015625}', 100),
            {"seed": 2**64 - 1, "dt": 0.015625},
        )
        self.assertIs(type(io.strict_json(b'{"n":7}', 100)["n"]), int)
        self.assertIs(type(io.strict_json(b'{"n":7.0}', 100)["n"]), Decimal)

    def test_duplicate_nested_unknown_number_and_trailing_input_are_rejected(self):
        for data in (
            b'{"a":1,"a":2}',
            b'{"a":{"b":1,"b":2}}',
            b'{"a":[{"b":1,"b":2}]}',
            b'{"a":NaN}',
            b'{"a":Infinity}',
            b'{"a":1e999}',
            b'{"a":1e-999}',
            b'{"a":-1e-999}',
            b"{} {}",
            b"[]",
            b"\xef\xbb\xbf{}",
            b'{"a":"\xff"}',
            b'{"a":"\\ud800"}',
        ):
            with self.subTest(data=data), self.assertRaises(ValueError):
                io.strict_json(data, 100)

    def test_duplicate_field_error_details_stay_within_contract_bound(self):
        key = "é" * 4096
        data = ('{"' + key + '":1,"' + key + '":2}').encode()
        with self.assertRaises(ValueError) as raised:
            io.strict_json(data, 20000)
        self.assertLessEqual(len(str(raised.exception).encode("utf-8")), 4096)

    def test_duplicate_surrogate_key_error_detail_is_bounded_utf8(self):
        for key in ("\ud800", "\ud800" * 4096):
            escaped = key.encode("ascii", errors="backslashreplace")
            data = b'{"' + escaped + b'":1,"' + escaped + b'":2}'
            with self.subTest(key_length=len(key)):
                with self.assertRaises(ValueError) as raised:
                    io.strict_json(data, 50000)
                detail = str(raised.exception).encode("utf-8")
                self.assertLessEqual(len(detail), 4096)

    def test_extreme_decimal_exponent_is_refused_as_value_error(self):
        with self.assertRaises(ValueError):
            io.strict_json(b'{"n":1e9999999999999999999999999999999999}', 100)

    def test_json_limit_is_checked_before_parsing(self):
        with self.assertRaises(ValueError):
            io.strict_json(b" " * (16 * 1024 * 1024 + 1), 16 * 1024 * 1024)
        self.assertEqual(io.strict_json(b"{}", 2), {})
        with self.assertRaises(ValueError):
            io.strict_json(b"{}", 1)

    def test_canonical_bytes_are_utf8_sorted_compact_with_final_newline(self):
        self.assertEqual(
            io.canonical_bytes({"z": 2, "a": "é"}), b'{"a":"\xc3\xa9","z":2}\n'
        )
        with self.assertRaises(ValueError):
            io.canonical_bytes({"n": float("nan")})


class DecimalWireTests(unittest.TestCase):
    def test_canonical_decimal_conversion_preserves_integer_seeds(self):
        self.assertEqual(io.canonical_bytes({"x":Decimal("0.1"),"seed":2**64-1}),
                         b'{"seed":18446744073709551615,"x":0.1}\n')
        for value in (Decimal("1e-999"),Decimal("1e999"),Decimal("NaN")):
            with self.subTest(value=value), self.assertRaises(ValueError):
                io.canonical_bytes({"x":value})


class FileTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()

    def test_create_and_read_obey_exact_boundary_without_overwrite(self):
        path = self.root / "new.json"
        io.create_bytes(path, b"{}", 2)
        self.assertEqual(io.read_regular(path, 2), b"{}")
        with self.assertRaises(ValueError):
            io.create_bytes(path, b"changed", 20)
        self.assertEqual(path.read_bytes(), b"{}")
        with self.assertRaises(ValueError):
            io.read_regular(path, 1)

    def test_oversized_write_does_not_create_destination(self):
        path = self.root / "new"
        with self.assertRaises(ValueError):
            io.create_bytes(path, b"abc", 2)
        self.assertFalse(path.exists())

    def test_symlink_and_existing_symlink_ancestors_are_refused(self):
        path = self.root / "data"
        path.write_bytes(b"{}")
        link = self.root / "link"
        link.symlink_to(path)
        ancestor = self.root / "ancestor"
        ancestor.symlink_to(self.root, target_is_directory=True)
        for bad in (link, ancestor / "data"):
            with self.subTest(path=bad), self.assertRaises(ValueError):
                io.read_regular(bad, 100)
        for bad in (link, ancestor / "fresh"):
            with self.subTest(path=bad), self.assertRaises(ValueError):
                io.create_bytes(bad, b"{}", 100)
        dangling = self.root / "dangling"
        dangling.symlink_to(self.root / "absent")
        with self.assertRaises(ValueError):
            io.create_bytes(dangling, b"{}", 100)

    def test_symlink_before_dotdot_component_is_refused(self):
        (self.root / "data").write_bytes(b"{}")
        link = self.root / "link"
        link.symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(ValueError):
            io.read_regular(link / ".." / "data", 100)
        with self.assertRaises(ValueError):
            io.create_bytes(link / ".." / "fresh", b"{}", 100)

    def test_directories_and_fifos_are_refused_without_blocking(self):
        fifo = self.root / "fifo"
        os.mkfifo(fifo)
        for path in (self.root, fifo):
            with self.subTest(path=path), self.assertRaises(ValueError):
                io.read_regular(path, 100)
            with self.subTest(write=path), self.assertRaises(ValueError):
                io.create_bytes(path, b"{}", 100)
