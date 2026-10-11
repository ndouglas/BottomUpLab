"""Algebraic complete-wire byte certificate and conservative preparation forecast.

Bounds count object punctuation, field names, primitive decimal maxima and every
level of JSON string escaping. Encoded examples test this certificate; they do
not supply its proof. No Engine or scientific entropy is used here.
"""
from dataclasses import dataclass
from decimal import Decimal
import math
from pathlib import Path
from .manifest import proposal, _fixture_arms
from .archive_provenance import POPULATION


@dataclass(frozen=True)
class Bound:
    size: int
    escapes: int = 0


def literal(value):
    from .io import canonical_bytes
    raw = canonical_bytes(value)[:-1]
    return Bound(len(raw), raw.count(b'"') + raw.count(b'\\') + sum(b < 32 for b in raw))


def text(n):
    # Every UTF8 input byte may become six ASCII bytes; quote/backslash each
    # contributes two escapable bytes. Independent maxima safely overbound both.
    return Bound(6 * n + 2, 2 * n + 2)


def string(encoded):
    return Bound(encoded.size + encoded.escapes + 2, 2 * encoded.escapes + 2)


def array(item, count):
    return Bound(2 + count * item.size + max(0, count - 1), count * item.escapes)


def obj(**fields):
    members = [(literal(k), v) for k, v in fields.items()]
    return Bound(2 + sum(k.size + 1 + v.size for k, v in members) + max(0, len(members) - 1), sum(k.escapes + v.escapes for k, v in members))


def schema_proof():
    root = Path(__file__).resolve().parents[2]
    roster = proposal(root)
    ids = [a["arm_id"] for a in roster["arms"] + _fixture_arms(roster)]
    if any(not all(c.isascii() and (c.isalnum() or c in "._-") for c in name) for name in ids):
        raise ValueError("closed roster ID alphabet differs from byte proof")
    arm_id = Bound(max(len(s) for s in ids) + 2, 2)
    integer = Bound(3)  # population/ID/step <=768/767/512
    u64 = Bound(20)
    number = Bound(32)  # all finite shortest binary64 JSON spellings <=32 ASCII bytes
    boolean = Bound(5)
    pair = array(integer, 2)
    key = obj(ordinal=Bound(5), arm_id=arm_id, replicate_index=Bound(4))
    ending = obj(reason=literal("one_side_extinction"), winner=literal("blue"), censored=boolean)
    config = obj(blue=integer, red=integer, blue_rate=number, red_rate=number, dt=number, max_steps=integer, geometry=literal("duel_contact"))
    checkpoint = obj(schema=literal("war1-checkpoint-v1"), config=config, seed=u64, step=integer, active_steps=integer,
        blue_alive=array(integer, 768), red_alive=array(integer, 0), contact_rng=text(128), casualty_rng=text(128), ending=ending)
    # Two arrays collectively contain <=768 IDs. Splitting arrays changes at most one comma.
    checkpoint = Bound(checkpoint.size + 1, checkpoint.escapes)
    probability = obj(hazard=number, dose=number, ideal=number, realized=number, saturated=boolean)
    exposure = obj(contact_pairs=u64, exposed=pair, contributed_rate=array(number, 2), integrated=array(number, 2), target_probability=array(probability, 2))
    frame = obj(step=integer, start=pair, survivors=pair, active_steps=integer, calendar_time=number, active_time=number,
        exposure=exposure, casualties=array(obj(id=integer, side=literal("blue")), 0), ending=ending)
    empty_frame_row = obj(kind=literal("frame"), step=integer, cause=literal("benchmark_exposure"), frame_json=string(frame))
    casualty = obj(id=integer, side=literal("blue"))
    casualty_escaped_with_comma = casualty.size + casualty.escapes + 1
    frame_bytes = empty_frame_row.size + 1 + 768 * casualty_escaped_with_comma
    # Reference errors are deterministic pure functions of66configs×4times.
    # metadata checks all264 calls for printableASCII field/detail<=512 before
    # readiness. PrintableASCII worst-case quotes/backslashes require2bytes each.
    printable512 = Bound(1026, 1026)
    ref_error = obj(field=printable512, detail=printable512)
    reference = obj(requested_time=number, evaluated_time=number, forces=array(number, 2), regime=literal("initial_extinction"), extinction_time=number, at_boundary=boolean, survivor=literal("blue"))
    observation = obj(step=integer, counts=pair, origin=literal("derived_absorbing"), reason=text(128), reference=reference, reference_error=ref_error)
    # Count both nullable branches simultaneously: conservative even though exclusive.
    outcome = obj(schema=literal("war1-study-outcome-v1"), key=key, seed=u64, case_input_sha256=literal("f"*64), manifest_sha256=literal("f"*64),
        completed_steps=integer, observed_steps=integer, active_steps=integer, calendar_time=number, active_time=number, initial_counts=pair, final_counts=pair,
        ending=ending, observations=array(observation,4), casualty_totals=pair, contributed_exposure_totals=array(number,2), final_checkpoint=checkpoint,
        transcript_sha256=literal("f"*64), audit_descriptor=obj(path=literal("audits/52223.jsonl"), sha256=literal("f"*64), bytes=u64, acknowledged_steps=integer))
    outcome_bytes = obj(kind=literal("outcome"), outcome=outcome).size + 1
    start_bytes = obj(kind=literal("start"), key=key, seed=u64, case_input_sha256=literal("f"*64)).size + 1
    case = obj(config=config, seed=u64)
    envelopes = [obj(kind=literal("header"), schema=literal("war1-study-audit-v1"), key=key, case_json=string(case)),
        obj(kind=literal("initial"), checkpoint_json=string(checkpoint)),
        obj(kind=literal("terminal"), checkpoint_json=string(checkpoint), completed_steps=integer, observed_steps=integer, active_steps=integer)]
    audit_bytes = sum((a["config"]["max_steps"] + 3) * 32768 for a in roster["arms"])
    starts_outcomes = 52224 * (1024 + 32768)
    # All retained receipt files: d/a/e,b/n/r,inventory,numericexport,oracle,forecast,
    # <=32probes. Also manifest, nativeinventory, header,index,probe metrics, outer
    # failure/monitor/success, bounded logs, block descriptors, and64MiB rawsource.
    # Nine root metadata files each receive the full16MiB cap, including the
    # candidate index with408 block descriptors; logs/failure reserve are extra.
    metadata = 42 * 16777216 + 9 * 16777216 + 64*1024**2 + 408 * 1024 + 2*65536 + 1048576
    result = dict(schema="war1-study-schema-proof-v1", start_bytes=start_bytes, outcome_bytes=outcome_bytes,
        audit_frame_bytes=frame_bytes, audit_fixed_bytes=empty_frame_row.size+1, escaped_casualty_increment=casualty_escaped_with_comma,
        audit_envelope_bytes=max(e.size+1 for e in envelopes), starts_outcomes_bytes=starts_outcomes,
        audit_bytes=audit_bytes, metadata_reserved_bytes=metadata, failure_reserve_bytes=1048576,
        reserved_bytes=starts_outcomes+audit_bytes+1073741824, metadata_files_maximum=42+9+816+66+4096+2,
        source_file_count_maximum=4096, source_bytes_maximum=67108864, metadata_file_bytes_maximum=16777216,
        bound_method="complete schema algebra including nested JSON escaping; fixed roster and pure reference vocabulary")
    if start_bytes > 1024 or outcome_bytes > 32768 or frame_bytes > 28672 or result["audit_fixed_bytes"] > 4096 or casualty_escaped_with_comma > 32 or result["audit_envelope_bytes"] > 32768 or metadata > 1073741824 or result["reserved_bytes"] >= 4294967296:
        raise ValueError(f"encoded resource certificate failed: {result}")
    return result


def forecast(rows, synthetic):
    """Use separately measured actual kernel work; never divide early-exit
    elapsed by an unexecuted horizon. Fourfold margin is a forecast, not a bound.
    """
    if not isinstance(rows, list) or len(rows) != 16:
        raise ValueError("forecast requires all16 fixed construction cases")
    required = ("kernel_seconds_per_actor_step", "projection_seconds_per_step", "start_outcome_seconds", "case_overhead_seconds")
    for row in rows:
        for field in required:
            value = row.get(field)
            if type(value) not in (int, float, Decimal) or not math.isfinite(value) or value <= 0:
                raise ValueError("missing measured construction cost")
        if type(row.get("actor_steps")) is not int or row["actor_steps"] <= 0:
            raise ValueError("actual executed actor-work unavailable")
    for field in ("source_seconds", "archive_seconds_per_byte", "peak_rss_bytes"):
        value = synthetic.get(field)
        if type(value) not in (int,float,Decimal) or not math.isfinite(value) or value <= 0:
            raise ValueError("synthetic/source/peak probe unavailable")
    proof = schema_proof()
    kernel = max(float(r[required[0]]) for r in rows) * POPULATION["initial_actor_steps"]
    projection = max(float(r[required[1]]) for r in rows) * POPULATION["settlements"]
    sync = max(float(r[required[2]]) for r in rows) * POPULATION["cases"]
    case_overhead = max(float(r[required[3]]) for r in rows) * POPULATION["cases"]
    archive = float(synthetic["archive_seconds_per_byte"]) * proof["reserved_bytes"]
    audit = max(float(r["audit_seconds_per_frame"]) for r in rows) * (proof["audit_bytes"] / 32768)
    if not math.isfinite(audit) or audit < 0:
        raise ValueError("audit sync costs unavailable")
    wall = 4 * (kernel + projection + sync + case_overhead + audit + archive + float(synthetic["source_seconds"]))
    # Source64MiB, manifest16MiB (plus parsing), bounded block4.2MiB and numerical
    # packet structures. The independently measured synthetic graph is padded4x.
    peak = math.ceil(4 * synthetic["peak_rss_bytes"] + 256*1024**2)
    if wall > 43200 or peak > 2147483648:
        raise ValueError(f"preparation forecast exceeds caps: wall={wall}, RSS={peak}")
    return dict(wall_seconds=wall, peak_rss_bytes=peak, margin=4, kernel_seconds=kernel,
        projection_seconds=projection, sync_seconds=sync, case_overhead_seconds=case_overhead, audit_seconds=audit, archive_seconds=archive,
        source_seconds=float(synthetic["source_seconds"]), limitations="construction7/8 and synthetic costs; hardware/filesystem variability; monitored hardcaps remain mandatory")
