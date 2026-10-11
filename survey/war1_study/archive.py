"""Bounded saved-evidence validation. No engine, RNG restoration or subprocess."""

from collections.abc import Callable, Iterator
from dataclasses import dataclass
from decimal import Decimal
from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path
import re
import struct

from .contracts import Json, Manifest, _freeze
from .io import METADATA_LIMIT, canonical_bytes, read_regular, strict_json
from .manifest import _fields, _text, _uint

ROW_LIMIT = 32768
BLOCK_LIMIT = 128 * (ROW_LIMIT + 1024)
CAMPAIGN_LIMIT = 4 * 1024**3


@dataclass(frozen=True)
class ArchiveRead:
    header: Json
    outcomes: Callable[[], Iterator[Json]]
    census: Json
    audits: tuple[Json, ...]
    complete: bool


def _require(condition, message):
    if not condition:
        raise ValueError(message)


def _hash(value):
    _require(
        isinstance(value, str) and re.fullmatch("[0-9a-f]{64}", value) is not None,
        "invalid SHA256",
    )


def _number(value):
    _require(
        type(value) in (int, float, Decimal) and math.isfinite(value),
        "nonfinite/invalid number",
    )
    return Fraction(value)


def _pair(value, *, integer=True):
    _require(isinstance(value, list) and len(value) == 2, "expected two side values")
    for v in value:
        if integer:
            _uint(v, "count", 768)
        else:
            _require(_number(v) >= 0, "negative number")


def _config(actual, expected):
    _fields(actual, set(expected), "config")
    for k, v in expected.items():
        if k in ("blue_rate", "red_rate", "dt"):
            _number(actual[k])
            _require(
                struct.pack(">d", actual[k]) == struct.pack(">d", v),
                "config binary64 mismatch",
            )
        else:
            _require(type(actual[k]) is type(v) and actual[k] == v, "config mismatch")


def case_bytes(config, seed):
    """Native struct order and exact dyadic numbers in the closed approved roster.

    This is intentionally not canonical_bytes (sorted keys/newline). Approved
    rate/dt values use ordinary decimal spelling in both serde and Python.
    """
    c = {
        k: config[k]
        for k in ("blue", "red", "blue_rate", "red_rate", "dt", "max_steps", "geometry")
    }
    for k in ("blue_rate", "red_rate", "dt"):
        c[k] = float(c[k])
    return json.dumps(
        dict(config=c, seed=seed), separators=(",", ":"), allow_nan=False
    ).encode()


def _ending(config, counts, step):
    if counts == [0, 0]:
        reason, winner, censored = "double_extinction", None, False
    elif counts[0] == 0:
        reason, winner, censored = "one_side_extinction", "red", False
    elif counts[1] == 0:
        reason, winner, censored = "one_side_extinction", "blue", False
    elif config["blue_rate"] == config["red_rate"] == 0:
        reason, winner, censored = "rate_zero", None, False
    elif step == config["max_steps"]:
        reason, winner, censored = "horizon", None, True
    else:
        return None
    return dict(reason=reason, winner=winner, censored=censored)


def _check_ending(actual, expected):
    if actual is None:
        _require(expected is None, "missing ending")
        return
    _fields(actual, {"reason", "winner", "censored"}, "ending")
    _require(type(actual["censored"]) is bool and actual == expected, "ending mismatch")


def _checkpoint(p, config, seed):
    _fields(
        p,
        {
            "schema",
            "config",
            "seed",
            "step",
            "active_steps",
            "blue_alive",
            "red_alive",
            "contact_rng",
            "casualty_rng",
            "ending",
        },
        "checkpoint",
    )
    _require(p["schema"] == "war1-checkpoint-v1", "checkpoint schema")
    _config(p["config"], config)
    _uint(p["seed"], "checkpoint seed")
    _require(p["seed"] == seed, "checkpoint seed mismatch")
    for k in ("step", "active_steps"):
        _uint(p[k], k, config["max_steps"])
    _require(p["active_steps"] == p["step"], "checkpoint clocks")
    for side, lo, hi in [
        ("blue", 0, config["blue"]),
        ("red", config["blue"], config["blue"] + config["red"]),
    ]:
        ids = p[f"{side}_alive"]
        _require(isinstance(ids, list) and len(ids) <= 768, "checkpoint actor bound")
        for i in ids:
            _uint(i, "actor ID", hi - 1)
        _require(
            all(i >= lo for i in ids) and all(a < b for a, b in zip(ids, ids[1:])),
            "checkpoint ordered actor IDs",
        )
    for k in ("contact_rng", "casualty_rng"):
        _text(p[k], "PCG text", 128)
        rng = strict_json(p[k].encode(), 128)
        _fields(rng, {"state"}, "PCG state")
        _uint(rng["state"], "PCG state", 2**128 - 1)
        _require(rng["state"] % 2 == 1, "PCG state must be odd")
    counts = [len(p["blue_alive"]), len(p["red_alive"])]
    _check_ending(p["ending"], _ending(config, counts, p["step"]))
    return counts


def _key(key, manifest):
    _fields(key, {"ordinal", "arm_id", "replicate_index"}, "key")
    _uint(key["ordinal"], "ordinal", len(manifest.cases) - 1)
    _uint(key["replicate_index"], "replicate index")
    case = manifest.cases[key["ordinal"]]
    _require(
        key
        == dict(
            ordinal=case.key.ordinal,
            arm_id=case.key.arm_id,
            replicate_index=case.key.replicate_index,
        ),
        "key/order mismatch",
    )
    return case


def _reference(obs, time):
    ref, error = obs["reference"], obs["reference_error"]
    _require((ref is None) != (error is None), "reference availability partition")
    if error is not None:
        _fields(error, {"field", "detail"}, "reference error")
        _text(error["field"], "reference field")
        _text(error["detail"], "reference detail", 4096)
        return
    _fields(
        ref,
        {
            "requested_time",
            "evaluated_time",
            "forces",
            "regime",
            "extinction_time",
            "at_boundary",
            "survivor",
        },
        "reference",
    )
    _require(
        _number(ref["requested_time"]) == time
        and 0 <= _number(ref["evaluated_time"]) <= time,
        "reference clocks",
    )
    _pair(ref["forces"], integer=False)
    _require(
        ref["regime"]
        in ("initial_extinction", "finite_extinction", "asymptotic", "rate_zero")
        and type(ref["at_boundary"]) is bool
        and ref["survivor"] in (None, "blue", "red"),
        "reference fields",
    )
    if ref["extinction_time"] is not None:
        _require(_number(ref["extinction_time"]) >= 0, "reference extinction time")


def validate_outcome(row: Json, manifest: Manifest) -> None:
    _fields(
        row,
        {
            "schema",
            "key",
            "seed",
            "case_input_sha256",
            "manifest_sha256",
            "completed_steps",
            "observed_steps",
            "active_steps",
            "calendar_time",
            "active_time",
            "initial_counts",
            "final_counts",
            "ending",
            "observations",
            "casualty_totals",
            "contributed_exposure_totals",
            "final_checkpoint",
            "transcript_sha256",
            "audit_descriptor",
        },
        "outcome",
    )
    case = _key(row["key"], manifest)
    config = next(a["config"] for a in manifest.arms if a["arm_id"] == case.key.arm_id)
    _uint(row["seed"], "seed")
    _require(
        row["schema"] == "war1-study-outcome-v1"
        and row["seed"] == case.seed
        and row["manifest_sha256"] == manifest.sha256,
        "outcome identity",
    )
    _require(
        row["case_input_sha256"]
        == hashlib.sha256(case_bytes(config, case.seed)).hexdigest(),
        "case input hash",
    )
    _hash(row["transcript_sha256"])
    counts = _checkpoint(row["final_checkpoint"], config, case.seed)
    for k in ("initial_counts", "final_counts", "casualty_totals"):
        _pair(row[k])
    _pair(row["contributed_exposure_totals"], integer=False)
    _require(
        row["initial_counts"] == [config["blue"], config["red"]]
        and row["final_counts"] == counts,
        "outcome counts",
    )
    for k in ("completed_steps", "observed_steps", "active_steps"):
        _uint(row[k], k, config["max_steps"])
        _require(row[k] == row["final_checkpoint"]["step"], "outcome clock mismatch")
    for k in ("calendar_time", "active_time"):
        _require(
            _number(row[k]) == row["completed_steps"] * Fraction(config["dt"]),
            "outcome time mismatch",
        )
    _require(row["ending"] is not None, "successful outcome must end")
    _check_ending(row["ending"], row["final_checkpoint"]["ending"])
    _require(
        all(
            f + d == i
            for f, d, i in zip(counts, row["casualty_totals"], row["initial_counts"])
        ),
        "casualty accounting",
    )
    observations = row["observations"]
    _require(
        isinstance(observations, list) and len(observations) == 4, "observation census"
    )
    previous = row["initial_counts"]
    for obs, time in zip(observations, (1, 2, 4, 8)):
        _fields(
            obs,
            {"step", "counts", "origin", "reason", "reference", "reference_error"},
            "observation",
        )
        _uint(obs["step"], "observation step", config["max_steps"])
        _pair(obs["counts"])
        _require(
            obs["step"] * Fraction(config["dt"]) == time, "observation time mismatch"
        )
        _require(
            all(f <= n <= p for f, n, p in zip(counts, obs["counts"], previous)),
            "observation count ordering",
        )
        if obs["step"] > row["completed_steps"]:
            _require(
                obs["origin"] == "derived_absorbing" and obs["counts"] == counts,
                "future capture origin/count",
            )
            _text(obs["reason"], "future frame reason", 4096)
        else:
            _require(
                obs["origin"] == "native" and obs["reason"] is None,
                "native capture origin/reason",
            )
            if obs["step"] == row["completed_steps"]:
                _require(obs["counts"] == counts, "terminal observation")
        _reference(obs, Fraction(time))
        previous = obs["counts"]
    audit = row["audit_descriptor"]
    if case.key.replicate_index == 0:
        _fields(
            audit, {"path", "sha256", "bytes", "acknowledged_steps"}, "audit descriptor"
        )
        _require(audit["path"] == f"audits/{case.key.ordinal}.jsonl", "audit path")
        _hash(audit["sha256"])
        _uint(audit["bytes"], "audit bytes", (config["max_steps"] + 3) * ROW_LIMIT)
        _uint(audit["acknowledged_steps"], "audit steps", 512)
        _require(
            audit["acknowledged_steps"] == row["completed_steps"],
            "audit steps mismatch",
        )
    else:
        _require(audit is None, "undeclared audit subset")


def _lines(path, limit, maximum):
    """Bound each allocation and the count before decoding JSON objects."""
    from .io import _checked_path
    import os
    import stat

    path = _checked_path(path)
    _require(stat.S_ISREG(path.lstat().st_mode), "archive input must be regular")
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, "rb") as stream:
        _require(
            stat.S_ISREG(os.fstat(stream.fileno()).st_mode),
            "archive input must be regular",
        )
        for index in range(maximum + 1):
            line = stream.readline(limit + 1)
            if not line:
                return
            _require(
                index < maximum and len(line) <= limit, "oversized/excess archive line"
            )
            _require(line.endswith(b"\n"), "torn archive line")
            yield line, strict_json(line, limit)


def _audit(root, row):
    d = row["audit_descriptor"]
    c = row["final_checkpoint"]["config"]
    digest = hashlib.sha256()
    semantic = hashlib.sha256(b"war1-study-transcript-v1\0")
    total = 0
    alive = [set(range(c["blue"])), set(range(c["blue"], c["blue"] + c["red"]))]
    counts = row["initial_counts"]
    exposures = [Fraction(0), Fraction(0)]
    deaths = [0, 0]
    step = 0
    stage = 0
    captures = {
        o["step"]: o["counts"] for o in row["observations"] if o["origin"] == "native"
    }
    for raw, r in _lines(root / d["path"], ROW_LIMIT, c["max_steps"] + 3):
        digest.update(raw)
        total += len(raw)
        kind = r.get("kind")
        if kind == "header":
            _fields(r, {"kind", "schema", "key", "case_json"}, "audit header")
            _require(
                stage == 0
                and r["schema"] == "war1-study-audit-v1"
                and r["key"] == row["key"],
                "audit header identity",
            )
            payload = r["case_json"]
            _text(payload, "audit case", ROW_LIMIT)
            _require(
                payload.encode() == case_bytes(c, row["seed"]),
                "audit native case bytes",
            )
            stage = 1
        elif kind == "initial":
            _fields(r, {"kind", "checkpoint_json"}, "audit initial")
            _require(stage == 1, "audit initial order")
            payload = r["checkpoint_json"]
            _text(payload, "checkpoint", ROW_LIMIT)
            p = strict_json(payload.encode(), ROW_LIMIT)
            initial = _checkpoint(p, c, row["seed"])
            _require(
                initial == row["initial_counts"] and p["step"] == 0,
                "audit initial state",
            )
            stage = 2
        elif kind == "frame":
            _fields(r, {"kind", "step", "cause", "frame_json"}, "audit frame")
            _require(
                stage == 2 and r["cause"] == "benchmark_exposure",
                "audit frame cause/order",
            )
            _uint(r["step"], "audit frame step", 512)
            _require(r["step"] == step + 1, "audit frame sequence")
            payload = r["frame_json"]
            _text(payload, "frame", ROW_LIMIT)
            f = strict_json(payload.encode(), ROW_LIMIT)
            _fields(
                f,
                {
                    "step",
                    "start",
                    "survivors",
                    "active_steps",
                    "calendar_time",
                    "active_time",
                    "exposure",
                    "casualties",
                    "ending",
                },
                "frame",
            )
            for k in ("step", "active_steps"):
                _uint(f[k], k, 512)
                _require(f[k] == step + 1, "frame clocks")
            for k in ("calendar_time", "active_time"):
                _require(_number(f[k]) == (step + 1) * Fraction(c["dt"]), "frame time")
            _pair(f["start"])
            _pair(f["survivors"])
            _require(f["start"] == counts, "frame start accounting")
            _require(_ending(c, counts, step) is None, "frame after absorption")
            _check_ending(f["ending"], _ending(c, f["survivors"], step + 1))
            e = f["exposure"]
            _fields(
                e,
                {
                    "contact_pairs",
                    "exposed",
                    "contributed_rate",
                    "integrated",
                    "target_probability",
                },
                "exposure",
            )
            sources = [min(counts)] * 2 if c["geometry"] == "duel_contact" else counts
            pairs = (
                min(counts)
                if c["geometry"] == "duel_contact"
                else counts[0] * counts[1]
            )
            _uint(e["contact_pairs"], "contact pairs", 768**2)
            _require(e["contact_pairs"] == pairs, "contact accounting")
            _pair(e["exposed"])
            _pair(e["contributed_rate"], integer=False)
            _pair(e["integrated"], integer=False)
            _require(e["exposed"] == sources, "exposed counts")
            _require(
                isinstance(e["target_probability"], list)
                and len(e["target_probability"]) == 2,
                "probability pair",
            )
            for p in e["target_probability"]:
                if p is not None:
                    _fields(
                        p,
                        {"hazard", "dose", "ideal", "realized", "saturated"},
                        "probability",
                    )
                    for name in ("hazard", "dose", "ideal", "realized"):
                        _require(_number(p[name]) >= 0, "negative probability field")
                    _require(
                        p["ideal"] <= 1
                        and p["realized"] <= 1
                        and type(p["saturated"]) is bool,
                        "probability bounds",
                    )
            casualties = f["casualties"]
            _require(
                isinstance(casualties, list) and len(casualties) <= 768,
                "casualty bound",
            )
            losses = [0, 0]
            for dead in casualties:
                _fields(dead, {"id", "side"}, "casualty")
                _require(dead["side"] in ("blue", "red"), "casualty side")
                side = int(dead["side"] == "red")
                _uint(dead["id"], "casualty actor", 767)
                _require(dead["id"] in alive[side], "duplicate/invalid casualty actor")
                alive[side].remove(dead["id"])
                losses[side] += 1
            for side, name in enumerate(("blue_rate", "red_rate")):
                rate = Fraction(c[name]) * sources[side]
                integrated = rate * Fraction(c["dt"])
                _require(
                    f["survivors"][side] + losses[side] == counts[side]
                    and losses[side] <= sources[side],
                    "frame deaths/counts",
                )
                _require(
                    _number(e["contributed_rate"][side]) == rate
                    and _number(e["integrated"][side]) == integrated,
                    "frame contributed exposure",
                )
                exposures[side] += integrated
                deaths[side] += losses[side]
            step += 1
            counts = f["survivors"]
            if step in captures:
                _require(counts == captures[step], "audit observation mismatch")
        elif kind == "terminal":
            _fields(
                r,
                {
                    "kind",
                    "checkpoint_json",
                    "completed_steps",
                    "observed_steps",
                    "active_steps",
                },
                "audit terminal",
            )
            _require(
                stage == 2 and step == row["completed_steps"], "audit terminal order"
            )
            for k in ("completed_steps", "observed_steps", "active_steps"):
                _uint(r[k], k, 512)
                _require(r[k] == row[k], "audit terminal clocks")
            payload = r["checkpoint_json"]
            _text(payload, "checkpoint", ROW_LIMIT)
            p = strict_json(payload.encode(), ROW_LIMIT)
            _checkpoint(p, c, row["seed"])
            _require(p == row["final_checkpoint"], "audit terminal checkpoint")
            _require(
                [set(p["blue_alive"]), set(p["red_alive"])] == alive,
                "audit terminal survivor identities",
            )
            stage = 3
        else:
            raise ValueError("unknown audit row")
        encoded = payload.encode()
        semantic.update(len(encoded).to_bytes(8, "big"))
        semantic.update(encoded)
    _require(
        stage == 3 and digest.hexdigest() == d["sha256"] and total == d["bytes"],
        "audit file digest/length/completion",
    )
    _require(
        semantic.hexdigest() == row["transcript_sha256"],
        "audit semantic transcript hash",
    )
    _require(
        counts == row["final_counts"]
        and deaths == row["casualty_totals"]
        and exposures == [Fraction(v) for v in row["contributed_exposure_totals"]],
        "audit terminal accounting",
    )


def _descriptor(root, index, planned):
    path = f"blocks/block-{index:06}.descriptor.json"
    raw = read_regular(root / path, METADATA_LIMIT)
    d = strict_json(raw, METADATA_LIMIT)
    _fields(
        d,
        {"path", "sha256", "bytes", "first_ordinal", "acknowledged_cases"},
        "block descriptor",
    )
    _require(d["path"] == f"blocks/block-{index:06}.jsonl", "block path/order")
    for k, cap in [
        ("bytes", BLOCK_LIMIT),
        ("first_ordinal", planned - 1),
        ("acknowledged_cases", 128),
    ]:
        _uint(d[k], k, cap)
    _require(
        d["first_ordinal"] == 128 * index
        and d["acknowledged_cases"] == min(128, planned - index * 128),
        "closed block census",
    )
    _hash(d["sha256"])
    return d


def _block(root, d):
    raw = read_regular(root / d["path"], BLOCK_LIMIT)
    if "file_sha256" in d:
        _require(
            hashlib.sha256(raw).hexdigest() == d["file_sha256"],
            "open block bytes changed",
        )
        raw = raw[: d["bytes"]]
    _require(
        len(raw) == d["bytes"] and hashlib.sha256(raw).hexdigest() == d["sha256"],
        "block digest/length changed",
    )
    yield from _block_rows(raw)


def _block_rows(raw):
    # One bounded block, at most 256 bounded lines; never an archive object graph.
    offset = 0
    for index in range(257):
        if offset == len(raw):
            return
        end = raw.find(b"\n", offset, offset + ROW_LIMIT + 1)
        _require(
            index < 256 and end >= 0 and end - offset + 1 <= ROW_LIMIT,
            "torn/oversized block row",
        )
        line = raw[offset : end + 1]
        offset = end + 1
        row = strict_json(line, ROW_LIMIT)
        if row.get("kind") == "start":
            _require(len(line) <= 1024, "start byte quota")
        yield row


def _block_namespace(root, planned):
    directory = root / "blocks"
    actual = (
        frozenset(p.name for p in directory.iterdir())
        if directory.exists()
        else frozenset()
    )
    allowed = {
        f"block-{i:06}.{suffix}"
        for i in range((planned + 127) // 128)
        for suffix in ("jsonl", "descriptor.json")
    }
    _require(actual <= allowed, "unexpected block namespace paths")
    return actual


def _failure_prefixes(root, planned):
    """Validate the acknowledged byte boundary before interpreting open rows."""
    from .archive_provenance import validate_receipt, validate_census

    failure = strict_json(
        read_regular(root / "failure.json", METADATA_LIMIT), METADATA_LIMIT
    )
    _fields(failure, {"detail", "census", "receipt", "case_failure"}, "failure receipt")
    validate_census(failure["census"], planned)
    receipt = failure["receipt"]
    validate_receipt(receipt, planned)
    if failure["case_failure"] is not None:
        validate_receipt(
            failure["case_failure"]["acknowledged_archive_prefix"], planned
        )
    remaining = receipt["bytes"]
    digest = hashlib.sha256()
    acknowledged = 0
    prefixes = {}
    for index in range((planned + 127) // 128):
        path = f"blocks/block-{index:06}.jsonl"
        if not (root / path).exists():
            continue
        raw = read_regular(root / path, BLOCK_LIMIT)
        size = min(remaining, len(raw))
        part = raw[:size]
        if (root / f"blocks/block-{index:06}.descriptor.json").exists():
            _require(size == len(raw), "closed block exceeds acknowledged prefix")
        # Parsing enforces the exact complete-record boundary and row/count caps.
        acknowledged += sum(row.get("kind") == "outcome" for row in _block_rows(part))
        digest.update(part)
        remaining -= size
        prefixes[index] = dict(
            path=path,
            bytes=size,
            sha256=hashlib.sha256(part).hexdigest(),
            file_sha256=hashlib.sha256(raw).hexdigest(),
        )
    _require(
        remaining == 0
        and digest.hexdigest() == receipt["sha256"]
        and acknowledged == receipt["acknowledged_cases"],
        "failure acknowledged prefix mismatch",
    )
    return prefixes


def _failure(root, manifest, census, receipt, states, attempted):
    from .archive_provenance import validate_receipt, validate_census

    failure = strict_json(
        read_regular(root / "failure.json", METADATA_LIMIT), METADATA_LIMIT
    )
    _fields(failure, {"detail", "census", "receipt", "case_failure"}, "failure receipt")
    _text(failure["detail"], "failure detail", 4096)
    validate_census(failure["census"], len(manifest.cases))
    validate_receipt(failure["receipt"], len(manifest.cases))
    _require(failure["receipt"] == receipt, "failure acknowledged prefix mismatch")
    f = failure["case_failure"]
    if f is not None:
        _fields(
            f,
            {
                "kind",
                "detail",
                "key",
                "attempted_step",
                "semantic_steps",
                "observed_steps",
                "durable_audit_steps",
                "final_checkpoint",
                "acknowledged_archive_prefix",
            },
            "case failure",
        )
        case = _key(f["key"], manifest)
        config = next(
            a["config"] for a in manifest.arms if a["arm_id"] == case.key.arm_id
        )
        _text(f["kind"], "failure kind")
        _text(f["detail"], "failure detail", 4096)
        for k in ("semantic_steps", "observed_steps", "durable_audit_steps"):
            _uint(f[k], k, config["max_steps"])
        _require(
            f["durable_audit_steps"] <= f["observed_steps"] <= f["semantic_steps"],
            "failure clocks",
        )
        if f["attempted_step"] is not None:
            _uint(f["attempted_step"], "attempted step", config["max_steps"])
            _require(
                f["attempted_step"] in (f["semantic_steps"], f["semantic_steps"] + 1),
                "failure attempted step",
            )
        if f["final_checkpoint"] is not None:
            _checkpoint(f["final_checkpoint"], config, case.seed)
            _require(
                f["final_checkpoint"]["step"] == f["semantic_steps"],
                "failure checkpoint clock",
            )
        else:
            _require(f["semantic_steps"] == 0, "missing actual failure checkpoint")
        validate_receipt(f["acknowledged_archive_prefix"], len(manifest.cases))
        _require(
            f["acknowledged_archive_prefix"] == receipt, "failure case prefix mismatch"
        )
        ordinal = case.key.ordinal
        if (
            f["kind"] == "archive_start"
            and states[ordinal] == "unstarted"
            and f["final_checkpoint"] is None
            and f["semantic_steps"] == 0
        ):
            _require(
                all(state == "complete_valid" for state in states[:ordinal]),
                "failure start order",
            )
            states[ordinal] = "partial"
            attempted.add(ordinal)
            census["attempted"] = len(attempted)
            census["partial"] += 1
            census["unstarted"] -= 1
    _require(failure["census"] == census, "failure census mismatch")


def read_campaign(root: Path, manifest: Manifest) -> ArchiveRead:
    from .archive_provenance import (
        validate_source,
        archive_metadata_namespace,
        inference_provenance,
        validate_receipt,
        validate_census,
    )

    root = Path(root)
    header = strict_json(
        read_regular(root / "header.json", METADATA_LIMIT), METADATA_LIMIT
    )
    _fields(header, {"schema", "mode", "binding", "planned"}, "header")
    _require(
        header["schema"] == "war1-study-archive-v1"
        and header["mode"] == manifest.purpose,
        "archive schema/mode",
    )
    _uint(header["planned"], "planned", 52224)
    _require(header["planned"] == len(manifest.cases), "planned census")
    _require(
        read_regular(root / "manifest.json", METADATA_LIMIT) == manifest.raw_bytes,
        "saved manifest bytes differ",
    )
    binding = header["binding"]
    _fields(
        binding,
        {
            "manifest_sha256",
            "compiled_inputs_sha256",
            "executable_sha256",
            "declaration_sha256",
            "acceptance_sha256",
            "execution_approval_sha256",
        },
        "binding",
    )
    for key, value in binding.items():
        if (
            key
            in ("declaration_sha256", "acceptance_sha256", "execution_approval_sha256")
            and value is None
        ):
            continue
        _hash(value)
    _require(binding["manifest_sha256"] == manifest.sha256, "manifest binding")
    if manifest.purpose == "construction_fixture":
        _require(
            all(
                binding[k] is None
                for k in (
                    "declaration_sha256",
                    "acceptance_sha256",
                    "execution_approval_sha256",
                )
            ),
            "fixture authorization binding",
        )
    validate_source(root, header, manifest)
    planned = len(manifest.cases)
    states = ["unstarted"] * planned
    attempted = set()
    descriptors = []
    outcome_descriptors = []
    audits = []
    errors = []
    namespace = None
    try:
        namespace = _block_namespace(root, planned)
    except ValueError as error:
        errors.append(str(error))
    failure_prefixes = None
    if (root / "failure.json").exists():
        try:
            failure_prefixes = _failure_prefixes(root, planned)
        except (ValueError, OSError, KeyError, TypeError) as error:
            errors.append(f"failure: {error}")
    prefix = hashlib.sha256()
    prefix_bytes = 0
    for index in range((planned + 127) // 128):
        blockpath = root / f"blocks/block-{index:06}.jsonl"
        descriptorpath = root / f"blocks/block-{index:06}.descriptor.json"
        if not blockpath.exists() and not descriptorpath.exists():
            continue
        first = index * 128
        last = min(first + 128, planned)
        closed = descriptorpath.exists()
        ordinal = first
        try:
            if closed:
                d = _descriptor(root, index, planned)
                raw = read_regular(blockpath, BLOCK_LIMIT)
                _require(
                    hashlib.sha256(raw).hexdigest() == d["sha256"]
                    and len(raw) == d["bytes"],
                    "closed block digest/bytes",
                )
                prefix.update(raw)
                prefix_bytes += len(raw)
                del raw
                rows = _block(root, d)
            elif failure_prefixes is not None:
                d = failure_prefixes[index]
                raw = read_regular(blockpath, BLOCK_LIMIT)[: d["bytes"]]
                prefix.update(raw)
                prefix_bytes += len(raw)
                rows = _block(root, d)
                errors.append(f"block {index}: unclosed_block")
            else:
                # Without a closed descriptor or verified failure receipt no
                # open outcome has evidence of a successful synchronization.
                def open_rows():
                    for raw, tagged in _lines(blockpath, ROW_LIMIT, 256):
                        if tagged.get("kind") == "start":
                            _require(len(raw) <= 1024, "start byte quota")
                        yield tagged

                rows = open_rows()
                errors.append(f"block {index}: unclosed_block")
            ordinal = first
            started = False
            for tagged in rows:
                _require(ordinal < last, "extra/duplicate block key")
                case = manifest.cases[ordinal]
                if tagged.get("kind") == "start":
                    _fields(
                        tagged, {"kind", "key", "seed", "case_input_sha256"}, "start"
                    )
                    actual = _key(tagged["key"], manifest)
                    _require(
                        actual == case and not started, "wrong/duplicate start key"
                    )
                    _uint(tagged["seed"], "start seed")
                    _require(tagged["seed"] == case.seed, "start seed mismatch")
                    config = next(
                        a["config"]
                        for a in manifest.arms
                        if a["arm_id"] == case.key.arm_id
                    )
                    _require(
                        tagged["case_input_sha256"]
                        == hashlib.sha256(case_bytes(config, case.seed)).hexdigest(),
                        "start case hash",
                    )
                    started = True
                    attempted.add(ordinal)
                    states[ordinal] = "partial"
                elif tagged.get("kind") == "outcome":
                    _fields(tagged, {"kind", "outcome"}, "tagged outcome")
                    row = tagged["outcome"]
                    _require(
                        isinstance(row, dict) and isinstance(row.get("key"), dict),
                        "outcome/key object required",
                    )
                    _require(
                        started and row["key"].get("ordinal") == ordinal,
                        "outcome without ordered start",
                    )
                    try:
                        validate_outcome(row, manifest)
                        if row["audit_descriptor"] is not None:
                            _audit(root, row)
                            audits.append(row["audit_descriptor"])
                        states[ordinal] = (
                            "complete_valid"
                            if closed or failure_prefixes is not None
                            else "partial"
                        )
                    except (ValueError, OSError) as error:
                        states[ordinal] = "invalid"
                        errors.append(f"key {ordinal}: {error}")
                    ordinal += 1
                    started = False
                else:
                    raise ValueError("unknown block row kind")
            if closed:
                _require(ordinal == last and not started, "incomplete closed block")
                descriptors.append(d)
            if closed or failure_prefixes is not None:
                outcome_descriptors.append(d)
        except (ValueError, OSError) as error:
            errors.append(f"block {index}: {error}")
            if not closed and ordinal < last and states[ordinal] == "unstarted":
                states[ordinal] = (
                    "partial" if str(error) == "torn archive line" else "invalid"
                )
                attempted.add(ordinal)
            for ordinal in range(first, last):
                if closed and states[ordinal] == "unstarted":
                    states[ordinal] = "invalid"
    census = dict(
        planned=planned,
        attempted=len(attempted),
        **{
            k: states.count(k)
            for k in (
                "complete_valid",
                "failed",
                "invalid",
                "partial",
                "pending",
                "unstarted",
            )
        },
    )
    receipt = dict(
        sha256=prefix.hexdigest(),
        bytes=prefix_bytes,
        acknowledged_cases=census["complete_valid"],
    )
    if (root / "failure.json").exists():
        errors.append("failure_receipt_present")
        try:
            _failure(root, manifest, census, receipt, states, attempted)
        except ValueError as error:
            errors.append(f"failure: {error}")
    candidate = None
    if (root / "candidate-index.json").exists():
        try:
            candidate = strict_json(
                read_regular(root / "candidate-index.json", METADATA_LIMIT),
                METADATA_LIMIT,
            )
            _fields(
                candidate,
                {"schema", "mode", "binding", "census", "receipt", "blocks", "monitor"},
                "candidate",
            )
            _require(
                candidate["schema"] == "war1-study-candidate-index-v1"
                and candidate["mode"] == header["mode"]
                and candidate["binding"] == binding,
                "candidate binding/schema",
            )
            validate_census(candidate["census"], planned)
            validate_receipt(candidate["receipt"], planned)
            _require(
                canonical_bytes(candidate["census"]) == canonical_bytes(census)
                and canonical_bytes(candidate["receipt"]) == canonical_bytes(receipt)
                and canonical_bytes(candidate["blocks"])
                == canonical_bytes(descriptors),
                "candidate census/receipt/blocks",
            )
            _text(candidate["monitor"], "candidate monitor", 192)
            if manifest.purpose == "construction_fixture":
                _require(
                    candidate["monitor"]
                    == "construction_deterministic_no_scientific_resource_decision",
                    "fixture monitor declaration",
                )
        except ValueError as error:
            errors.append(f"candidate: {error}")
    else:
        errors.append("candidate_index_unavailable")
    complete = census["complete_valid"] == planned and not errors
    available, reason = (
        inference_provenance(root, header, manifest, candidate)
        if complete
        else (False, "incomplete_archive")
    )
    validated_header = {
        **header,
        "validation": dict(
            primary_available=available,
            reason=reason,
            registered=available,
            errors=errors,
            partial_keys=[
                dict(
                    ordinal=c.key.ordinal,
                    arm_id=c.key.arm_id,
                    replicate_index=c.key.replicate_index,
                )
                for c, state in zip(manifest.cases, states)
                if state in ("partial", "pending")
            ],
            acknowledged_archive_prefix=dict(
                sha256=prefix.hexdigest(),
                bytes=prefix_bytes,
                acknowledged_cases=census["complete_valid"],
            ),
            non_audit_transcripts="replay_checkable_hash_only_full_frames_not_retained",
        ),
    }
    valid = frozenset(i for i, s in enumerate(states) if s == "complete_valid")
    metadata_paths = archive_metadata_namespace(root)
    metadata_hashes = {
        p: hashlib.sha256(read_regular(p, METADATA_LIMIT)).hexdigest()
        for p in metadata_paths
    }

    def recheck_metadata():
        _require(
            archive_metadata_namespace(root) == metadata_paths,
            "archive metadata namespace changed after validation",
        )
        _require(
            _block_namespace(root, planned) == namespace,
            "block namespace changed after validation",
        )
        for path, expected in metadata_hashes.items():
            _require(
                hashlib.sha256(read_regular(path, METADATA_LIMIT)).hexdigest()
                == expected,
                "archive metadata changed after validation",
            )
        validate_source(root, header, manifest)

    def outcomes():
        # Recheck each durable block/prefix and all audit bytes. Any change aborts
        # the consumer's aggregate, even if a late file changed after early rows.
        recheck_metadata()
        for d in outcome_descriptors:
            for tagged in _block(root, d):
                if (
                    tagged["kind"] == "outcome"
                    and tagged["outcome"]["key"]["ordinal"] in valid
                ):
                    row = tagged["outcome"]
                    validate_outcome(row, manifest)
                    if row["audit_descriptor"] is not None:
                        _audit(root, row)
                    yield row
        recheck_metadata()

    return ArchiveRead(
        _freeze(validated_header),
        outcomes,
        _freeze(census),
        tuple(_freeze(d) for d in audits),
        complete,
    )
