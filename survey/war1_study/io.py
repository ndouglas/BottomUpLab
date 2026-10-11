"""Strict bounded JSON and create-only, synchronized regular files.

Ancestor checks and O_NOFOLLOW reject ordinary symlinks. This is not
adversarial descriptor-relative protection against ancestor replacement races.
"""

from decimal import Decimal, DecimalException
import json
import math
import os
from pathlib import Path
import stat

from .contracts import Json


METADATA_LIMIT = 16 * 1024 * 1024


def _limit(limit: int) -> None:
    if type(limit) is not int or limit < 0:
        raise ValueError("byte limit must be a nonnegative integer")


def _bounded(data: bytes, limit: int) -> None:
    _limit(limit)
    if not isinstance(data, bytes) or len(data) > limit:
        raise ValueError("bytes required within the declared byte limit")


def strict_json(data: bytes, limit: int) -> Json:
    """Retain Decimal tokens so schema validation precedes binary64 conversion."""
    _bounded(data, limit)

    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError(f"duplicate JSON field: {key[:128]!a}")
            result[key] = value
        return result

    def constant(value):
        raise ValueError(f"nonfinite JSON literal: {value}")

    def normalize(value):
        if isinstance(value, Decimal):
            number = float(value)
            if not math.isfinite(number) or (value != 0 and number == 0):
                raise ValueError("JSON number is nonfinite or underflows binary64")
            return value
        if isinstance(value, str):
            value.encode("utf-8", errors="strict")
        elif isinstance(value, dict):
            return {normalize(key): normalize(child) for key, child in value.items()}
        elif isinstance(value, list):
            return [normalize(child) for child in value]
        return value

    try:
        text = data.decode("utf-8", errors="strict")
        if text.startswith("\ufeff"):
            raise ValueError("JSON UTF-8 BOM forbidden")
        result = json.loads(
            text,
            object_pairs_hook=pairs,
            parse_int=int,
            parse_float=Decimal,
            parse_constant=constant,
        )
        if not isinstance(result, dict):
            raise ValueError("JSON root must be an object")
        return normalize(result)
    except (
        UnicodeError,
        json.JSONDecodeError,
        RecursionError,
        OverflowError,
        DecimalException,
    ) as exc:
        raise ValueError(f"invalid JSON: {exc}") from exc


def canonical_bytes(value: object) -> bytes:
    """Encode validated values, normalizing Decimal display/config numbers only."""

    def decimal_number(number):
        if not isinstance(number, Decimal):
            raise TypeError("unsupported JSON value")
        resolved = float(number)
        if not math.isfinite(resolved) or (number != 0 and resolved == 0):
            raise ValueError("Decimal is nonfinite or underflows binary64")
        return resolved

    try:
        return (
            json.dumps(
                value,
                sort_keys=True,
                separators=(",", ":"),
                ensure_ascii=False,
                allow_nan=False,
                default=decimal_number,
            )
            + "\n"
        ).encode("utf-8")
    except (UnicodeError, TypeError, OverflowError) as exc:
        raise ValueError(f"value cannot be encoded as canonical JSON: {exc}") from exc


def _checked_path(path: Path) -> Path:
    # absolute() retains .. so an earlier symlink component cannot disappear.
    path = Path(path).absolute()
    for ancestor in (*reversed(path.parents), path):
        try:
            mode = ancestor.lstat().st_mode
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(mode):
            raise ValueError(f"symlink path forbidden: {ancestor}")
        if ancestor != path and not stat.S_ISDIR(mode):
            raise ValueError(f"non-directory ancestor: {ancestor}")
    return path


def read_regular(path: Path, limit: int) -> bytes:
    _limit(limit)
    path = _checked_path(path)
    try:
        info = path.lstat()
        if not stat.S_ISREG(info.st_mode):
            raise ValueError(f"input must be a regular file: {path}")
        if info.st_size > limit:
            raise ValueError(f"input exceeds byte limit: {path}")
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
        with os.fdopen(fd, "rb") as file:
            info = os.fstat(file.fileno())
            if not stat.S_ISREG(info.st_mode) or info.st_size > limit:
                raise ValueError(f"opened input is nonregular or oversized: {path}")
            data = file.read(limit + 1)
        _bounded(data, limit)
        return data
    except OSError as exc:
        raise ValueError(f"cannot read regular input {path}: {exc}") from exc


def create_bytes(path: Path, data: bytes, limit: int) -> None:
    _bounded(data, limit)
    path = _checked_path(path)
    try:
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
        with os.fdopen(fd, "wb") as file:
            written = file.write(data)
            if written != len(data):
                raise ValueError(f"incomplete file write: {path}")
            file.flush()
            os.fsync(file.fileno())
        parent = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        try:
            os.fsync(parent)
        finally:
            os.close(parent)
    except OSError as exc:
        raise ValueError(
            f"cannot create synchronized destination {path}: {exc}"
        ) from exc
