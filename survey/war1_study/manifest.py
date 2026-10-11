"""Exact immutable W1-A roster and pure injected-source seed preparation."""

from collections.abc import Callable
from copy import deepcopy
from decimal import Decimal
import hashlib
from pathlib import Path
import re
import struct

from .contracts import Case, Json, Key, Manifest, _freeze
from .io import METADATA_LIMIT, canonical_bytes, read_regular, strict_json

PROTOCOL = Path("docs/superpowers/specs/2026-10-10-war-1-measurement-design.md")
PROPOSAL = Path("docs/superpowers/specs/2026-10-10-war-1-measurement-proposal.json")
PROTOCOL_SHA256 = "a649e960ee574a2d4f5112b760f57701e22c2cbcf2e89d4915a31557ecb8348f"
PROPOSAL_SHA256 = "f58f0827eb70c411d60289802c3f0d5aaa1f0745d782dc6cc0914dc3333b2abe"
SCHEMA = "war1-study-manifest-v1"
SAMPLER_VERSION = "1"
MANIFEST_FIELDS = {
    "schema",
    "purpose",
    "protocol_sha256",
    "proposal_sha256",
    "roster_sha256",
    "sampler",
    "arms",
    "cases",
}
SAMPLER_FIELDS = {"method", "bytes_drawn", "seed_bytes_sha256", "command", "version"}
CASE_FIELDS = {"ordinal", "arm_id", "replicate_index", "seed"}


def _sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _fields(value, expected, context):
    if not isinstance(value, dict) or set(value) != expected:
        raise ValueError(f"{context}: unknown or missing fields")


def _text(value, context, maximum=192):
    if not isinstance(value, str) or not value or len(value.encode("utf-8")) > maximum:
        raise ValueError(
            f"{context}: expected nonempty text within {maximum} UTF-8 bytes"
        )


def _uint(value, context, maximum=2**64 - 1):
    if type(value) is not int or not 0 <= value <= maximum:
        raise ValueError(f"{context}: expected unsigned integer within range")


def _same(value, expected):
    if type(value) is not type(expected):
        return False
    if isinstance(expected, dict):
        return set(value) == set(expected) and all(
            _same(value[k], v) for k, v in expected.items()
        )
    if isinstance(expected, list):
        return len(value) == len(expected) and all(
            _same(v, e) for v, e in zip(value, expected)
        )
    return value == expected


def proposal(root: Path) -> Json:
    """Refuse any edit, even one outside the arm list, to the approved inputs."""
    protocol = read_regular(root / PROTOCOL, METADATA_LIMIT)
    roster = read_regular(root / PROPOSAL, METADATA_LIMIT)
    if _sha(protocol) != PROTOCOL_SHA256 or _sha(roster) != PROPOSAL_SHA256:
        raise ValueError("approved protocol/proposal bytes changed")
    return strict_json(roster, METADATA_LIMIT)


def _fixture_arms(approved: Json) -> list[Json]:
    lookup = {arm["arm_id"]: arm for arm in approved["arms"]}
    arms = []
    for group, profile, scale in (
        ("oracle", "1v1", 1),
        ("oracle", "2v1", 1),
        ("core", "E", 64),
        ("scale", "C", 256),
    ):
        for geometry in ("aimed_fire", "duel_contact"):
            original = f"{group}.{profile}.N{scale}.{geometry}.dt1-64"
            arm = deepcopy(lookup[original])
            arm["arm_id"] = f"fixture.{profile}.N{scale}.{geometry}.dt1-64"
            arm["group"] = "construction_fixture"
            arm["replicates"] = 2
            arm["seed_namespace"] = "construction_fixture/" + arm["arm_id"]
            arms.append(arm)
    return arms


def _validate_arms(value, expected):
    if not isinstance(value, list) or len(value) != len(expected):
        raise ValueError("manifest arm census differs from complete roster")
    for arm, approved in zip(value, expected):
        _fields(arm, set(approved), "arm")
        _fields(arm["config"], set(approved["config"]), "config")
        for field, wanted in approved.items():
            if field == "config":
                for name, config_value in wanted.items():
                    actual = arm["config"][name]
                    if name in ("blue_rate", "red_rate", "dt"):
                        try:
                            matches = type(actual) in (
                                int,
                                float,
                                Decimal,
                            ) and struct.pack(">d", actual) == struct.pack(
                                ">d", config_value
                            )
                        except (OverflowError, struct.error):
                            matches = False
                        if not matches:
                            raise ValueError(
                                f"config {name} binary64 bits differ from approved roster"
                            )
                    elif not _same(actual, config_value):
                        raise ValueError(f"config {name} differs from approved roster")
            elif not _same(arm[field], wanted):
                raise ValueError(f"arm {field} differs from approved roster/order")
        _text(arm["arm_id"], "arm ID")
        _text(arm["seed_namespace"], "seed namespace")


def validate_manifest(data: bytes, root: Path) -> Manifest:
    raw = strict_json(data, METADATA_LIMIT)
    _fields(raw, MANIFEST_FIELDS, "manifest")
    if raw["schema"] != SCHEMA:
        raise ValueError("unknown manifest schema")
    purpose = raw["purpose"]
    if purpose not in ("construction_fixture", "prospective_seed_population"):
        raise ValueError("unknown manifest purpose")
    approved = proposal(root)
    for field, expected in (
        ("protocol_sha256", PROTOCOL_SHA256),
        ("proposal_sha256", PROPOSAL_SHA256),
    ):
        if raw[field] != expected:
            raise ValueError(f"manifest {field} binding mismatch")
    expected = (
        _fixture_arms(approved)
        if purpose == "construction_fixture"
        else approved["arms"]
    )
    _validate_arms(raw["arms"], expected)
    if raw["roster_sha256"] != _sha(canonical_bytes(raw["arms"])):
        raise ValueError("actual arm roster SHA256 mismatch")
    count = sum(arm["replicates"] for arm in expected)
    if not isinstance(raw["cases"], list) or len(raw["cases"]) != count:
        raise ValueError("case census differs from exact population")
    sampler = raw["sampler"]
    _fields(sampler, SAMPLER_FIELDS, "sampler")
    method = (
        "construction-seeds-v1"
        if purpose == "construction_fixture"
        else "os.urandom-u64le-v1"
    )
    _text(sampler["version"], "sampler version", 128)
    if sampler["method"] != method or sampler["version"] != SAMPLER_VERSION:
        raise ValueError("sampler method/version mismatch")
    _uint(sampler["bytes_drawn"], "sampler bytes drawn")
    if sampler["bytes_drawn"] != (
        0 if purpose == "construction_fixture" else count * 8
    ):
        raise ValueError("sampler draw byte census mismatch")
    if (
        not isinstance(sampler["command"], list)
        or not 1 <= len(sampler["command"]) <= 32
    ):
        raise ValueError("sampler command must be a bounded nonempty argv")
    for item in sampler["command"]:
        _text(item, "sampler command argument", 1024)
    if purpose == "construction_fixture" and sampler["command"] != [
        "construction-fixture-factory"
    ]:
        raise ValueError("construction fixture command mismatch")
    if not isinstance(sampler["seed_bytes_sha256"], str) or not re.fullmatch(
        "[0-9a-f]{64}", sampler["seed_bytes_sha256"]
    ):
        raise ValueError("invalid sampler seed-byte SHA256")
    digest = hashlib.sha256()
    cases = []
    ordinal = 0
    for arm in expected:
        for index in range(arm["replicates"]):
            case = raw["cases"][ordinal]
            _fields(case, CASE_FIELDS, "case")
            for field in ("ordinal", "replicate_index", "seed"):
                _uint(case[field], field)
            if (case["ordinal"], case["arm_id"], case["replicate_index"]) != (
                ordinal,
                arm["arm_id"],
                index,
            ):
                raise ValueError("case key/ordinal/order differs from declared roster")
            if purpose == "construction_fixture" and case["seed"] != (7, 8)[index]:
                raise ValueError("construction fixture seed must be 7/8")
            digest.update(case["seed"].to_bytes(8, "little"))
            cases.append(Case(Key(ordinal, arm["arm_id"], index), case["seed"]))
            ordinal += 1
    # Fixtures draw no entropy: their draw transcript is empty, not their fixed seeds.
    wanted_digest = (
        _sha(b"") if purpose == "construction_fixture" else digest.hexdigest()
    )
    if sampler["seed_bytes_sha256"] != wanted_digest:
        raise ValueError("sampler seed-byte transcript SHA256 mismatch")
    frozen = _freeze(raw)
    return Manifest(
        frozen, data, _sha(data), purpose, tuple(frozen["arms"]), tuple(cases)
    )


def draw_u64(entropy: Callable[[int], bytes]) -> tuple[int, bytes]:
    block = entropy(8)
    if not isinstance(block, bytes) or len(block) != 8:
        raise ValueError("entropy source must return exactly eight bytes")
    return int.from_bytes(block, "little"), block


def _build(arms, purpose, cases, sampler):
    value = {
        "schema": SCHEMA,
        "purpose": purpose,
        "protocol_sha256": PROTOCOL_SHA256,
        "proposal_sha256": PROPOSAL_SHA256,
        "roster_sha256": _sha(canonical_bytes(arms)),
        "sampler": sampler,
        "arms": arms,
        "cases": cases,
    }
    data = canonical_bytes(value)
    if len(data) > METADATA_LIMIT:
        raise ValueError("generated manifest exceeds metadata cap")
    return data


def make_fixture(root: Path) -> bytes:
    arms = _fixture_arms(proposal(root))
    cases = []
    for arm in arms:
        for index, seed in enumerate((7, 8)):
            cases.append(
                {
                    "ordinal": len(cases),
                    "arm_id": arm["arm_id"],
                    "replicate_index": index,
                    "seed": seed,
                }
            )
    return _build(
        arms,
        "construction_fixture",
        cases,
        {
            "method": "construction-seeds-v1",
            "bytes_drawn": 0,
            "seed_bytes_sha256": _sha(b""),
            "command": ["construction-fixture-factory"],
            "version": SAMPLER_VERSION,
        },
    )


def sample_manifest(
    root: Path,
    entropy: Callable[[int], bytes],
    *,
    command: tuple[str, ...] = ("python3", "-m", "survey.war1_study", "prepare"),
) -> bytes:
    """Pure factory: the caller supplies draws; this function never opens OS entropy.

    The method labels the future preparation protocol. Synthetic factory output
    cannot establish genuine OS sampling or independently authorize collection.
    """
    if not isinstance(command, tuple) or not 1 <= len(command) <= 32:
        raise ValueError("sampler command must be a bounded nonempty tuple")
    for item in command:
        _text(item, "sampler command argument", 1024)
    arms = proposal(root)["arms"]
    count = sum(arm["replicates"] for arm in arms)
    sampler = {
        "method": "os.urandom-u64le-v1",
        "bytes_drawn": count * 8,
        "seed_bytes_sha256": "f" * 64,
        "command": list(command),
        "version": SAMPLER_VERSION,
    }
    # Bound the entire encoded list before allocating cases or consuming draws.
    # Every ordinal/index/seed below has no more digits than these maxima.
    maximum_bytes = len(_build(arms, "prospective_seed_population", [], sampler))
    for arm in arms:
        largest = {
            "ordinal": count - 1,
            "arm_id": arm["arm_id"],
            "replicate_index": arm["replicates"] - 1,
            "seed": 2**64 - 1,
        }
        maximum_bytes += arm["replicates"] * (len(canonical_bytes(largest)) - 1)
    maximum_bytes += count - 1  # Commas inserted into the empty array.
    if maximum_bytes > METADATA_LIMIT:
        raise ValueError(
            "maximum encoded manifest exceeds metadata cap before seed draws"
        )
    cases = []
    digest = hashlib.sha256()
    for arm in arms:
        for index in range(arm["replicates"]):
            seed, block = draw_u64(entropy)
            digest.update(block)
            cases.append(
                {
                    "ordinal": len(cases),
                    "arm_id": arm["arm_id"],
                    "replicate_index": index,
                    "seed": seed,
                }
            )
    sampler["seed_bytes_sha256"] = digest.hexdigest()
    return _build(arms, "prospective_seed_population", cases, sampler)
