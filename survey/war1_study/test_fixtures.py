"""Deterministic synthetic evidence: never Engine output or sampled science."""

from copy import deepcopy
import hashlib
import json
from pathlib import Path

from . import manifest
from .io import canonical_bytes

ROOT = Path(__file__).resolve().parents[2]


def native(value):
    return json.dumps(value, separators=(",", ":"), allow_nan=False)


def case_text(config, seed):
    c = {
        k: config[k]
        for k in ("blue", "red", "blue_rate", "red_rate", "dt", "max_steps", "geometry")
    }
    for k in ("blue_rate", "red_rate", "dt"):
        c[k] = float(c[k])
    return native(dict(config=c, seed=seed))


def checkpoint(config, seed, step, counts, ending):
    return dict(
        schema="war1-checkpoint-v1",
        config=json.loads(canonical_bytes(config)),
        seed=seed,
        step=step,
        active_steps=step,
        blue_alive=list(range(counts[0])),
        red_alive=list(range(config["blue"], config["blue"] + counts[1])),
        contact_rng='{"state":340282366920938463463374607431768211455}',
        casualty_rng='{"state":1}',
        ending=ending,
    )


def row_for(
    arm,
    case,
    manifest_sha,
    *,
    step=1,
    ending="one_side_extinction",
    winner="blue",
    censored=False,
):
    c = arm["config"]
    counts = (
        [1, 0]
        if winner == "blue"
        else (
            [0, 1]
            if winner == "red"
            else [0, 0] if ending == "double_extinction" else [1, 1]
        )
    )
    end = dict(reason=ending, winner=winner, censored=censored)
    p = checkpoint(c, case.seed, step, counts, end)
    key = dict(
        ordinal=case.key.ordinal,
        arm_id=case.key.arm_id,
        replicate_index=case.key.replicate_index,
    )
    observations = []
    for t in (1, 2, 4, 8):
        requested = int(t / float(c["dt"]))
        capture_counts = counts if requested >= step else [c["blue"], c["red"]]
        observations.append(
            dict(
                step=requested,
                counts=capture_counts,
                origin="derived_absorbing" if requested > step else "native",
                reason=(
                    "synthetic future frame unavailable" if requested > step else None
                ),
                reference=None,
                reference_error=dict(
                    field="synthetic", detail="construction reference unavailable"
                ),
            )
        )
    return dict(
        schema="war1-study-outcome-v1",
        key=key,
        seed=case.seed,
        case_input_sha256=hashlib.sha256(case_text(c, case.seed).encode()).hexdigest(),
        manifest_sha256=manifest_sha,
        completed_steps=step,
        observed_steps=step,
        active_steps=step,
        calendar_time=step * float(c["dt"]),
        active_time=step * float(c["dt"]),
        initial_counts=[c["blue"], c["red"]],
        final_counts=counts,
        ending=end,
        observations=observations,
        casualty_totals=[c["blue"] - counts[0], c["red"] - counts[1]],
        contributed_exposure_totals=[
            step * float(c["dt"]) * float(c["blue_rate"]) * c["blue"],
            step * float(c["dt"]) * float(c["red_rate"]) * c["red"],
        ],
        final_checkpoint=p,
        transcript_sha256="0" * 64,
        audit_descriptor=None,
    )


def outcome(*, ending, step, winner, censored):
    m = manifest.validate_manifest(manifest.make_fixture(ROOT), ROOT)
    row = row_for(
        m.arms[4],
        m.cases[8],
        m.sha256,
        step=step,
        ending=ending,
        winner=winner,
        censored=censored,
    )
    attach_audit(row)
    return row


def attach_audit(row):
    """Build full synthetic native-shaped transcript, not a simulator replay."""
    c = row["final_checkpoint"]["config"]
    initial = checkpoint(c, row["seed"], 0, row["initial_counts"], None)
    parts = [case_text(c, row["seed"]), native(initial)]
    rows = [
        dict(
            kind="header",
            schema="war1-study-audit-v1",
            key=row["key"],
            case_json=parts[0],
        ),
        dict(kind="initial", checkpoint_json=parts[1]),
    ]
    # All deaths occur at the final synthetic settlement. This fixture is for
    # structural/accounting validation, never an ideal-law or RNG realization.
    exposure = [0.0, 0.0]
    for step in range(1, row["completed_steps"] + 1):
        final = step == row["completed_steps"]
        start = row["initial_counts"]
        survivors = row["final_counts"] if final else start
        sources = [min(start)] * 2 if c["geometry"] == "duel_contact" else start
        rates = [
            sources[i] * float(c[k]) for i, k in enumerate(("blue_rate", "red_rate"))
        ]
        integrated = [rate * float(c["dt"]) for rate in rates]
        exposure = [a + b for a, b in zip(exposure, integrated)]
        casualties = []
        if final:
            for side, offset, total, remain in [
                ("blue", 0, start[0], survivors[0]),
                ("red", c["blue"], start[1], survivors[1]),
            ]:
                casualties.extend(
                    dict(id=i + offset, side=side) for i in range(remain, total)
                )
        frame = dict(
            step=step,
            start=start,
            survivors=survivors,
            active_steps=step,
            calendar_time=step * float(c["dt"]),
            active_time=step * float(c["dt"]),
            exposure=dict(
                contact_pairs=(
                    min(start)
                    if c["geometry"] == "duel_contact"
                    else start[0] * start[1]
                ),
                exposed=sources,
                contributed_rate=rates,
                integrated=integrated,
                target_probability=[None, None],
            ),
            casualties=casualties,
            ending=row["ending"] if final else None,
        )
        text = native(frame)
        rows.append(
            dict(kind="frame", step=step, cause="benchmark_exposure", frame_json=text)
        )
        parts.append(text)
    parts.append(native(row["final_checkpoint"]))
    rows.append(
        dict(
            kind="terminal",
            checkpoint_json=parts[-1],
            completed_steps=row["completed_steps"],
            observed_steps=row["observed_steps"],
            active_steps=row["active_steps"],
        )
    )
    digest = hashlib.sha256(b"war1-study-transcript-v1\0")
    for text in parts:
        raw = text.encode()
        digest.update(len(raw).to_bytes(8, "big"))
        digest.update(raw)
    data = b"".join(canonical_bytes(r) for r in rows)
    row["transcript_sha256"] = digest.hexdigest()
    row["contributed_exposure_totals"] = exposure
    row["audit_descriptor"] = dict(
        path=f"audits/{row['key']['ordinal']}.jsonl",
        sha256=hashlib.sha256(data).hexdigest(),
        bytes=len(data),
        acknowledged_steps=row["completed_steps"],
    )
    return data


def synthetic_manifest():
    # Supplied constant construction bytes; no os.urandom or scientific Engines.
    return manifest.validate_manifest(
        manifest.sample_manifest(
            ROOT,
            lambda n: (7).to_bytes(n, "little"),
            command=("synthetic-construction-test",),
        ),
        ROOT,
    )


def synthetic_row(arm, case, manifest_sha):
    row = row_for(arm, case, manifest_sha)
    c = arm["config"]
    counts = [c["blue"], 0] if c["blue"] >= c["red"] else [0, c["red"]]
    row["final_counts"] = counts
    row["ending"]["winner"] = "blue" if counts[0] else "red"
    row["casualty_totals"] = [c["blue"] - counts[0], c["red"] - counts[1]]
    row["final_checkpoint"] = checkpoint(c, case.seed, 1, counts, row["ending"])
    for obs in row["observations"]:
        obs["counts"] = counts
    return row


def save_campaign(root, m, *, stop=None):
    """Write synthetic saved evidence; no operational success/authorization."""
    for sub in ("blocks", "audits", "source"):
        (root / sub).mkdir(parents=True)

    def save(path, value):
        (root / path).write_bytes(canonical_bytes(value))

    from .archive_provenance import SOURCE_REQUIRED

    sources = sorted(
        [(path, (ROOT / path).read_bytes()) for path in SOURCE_REQUIRED]
        + [
            (
                (
                    "survey/war1_study/fixtures/construction-manifest.json"
                    if m.purpose == "construction_fixture"
                    else "survey/war1_study/manifests/synthetic-construction.json"
                ),
                m.raw_bytes,
            )
        ]
    )
    sha = hashlib.sha256(b"war1-study-compiled-inputs-v1\0")
    entries = []
    for i, (path, data) in enumerate(sources):
        for part in (path.encode(), data):
            sha.update(len(part).to_bytes(8, "big"))
            sha.update(part)
        (root / f"source/{i:06}.bin").write_bytes(data)
        entries.append(
            dict(path=path, sha256=hashlib.sha256(data).hexdigest(), bytes=len(data))
        )
    binding = dict(
        manifest_sha256=m.sha256,
        compiled_inputs_sha256=sha.hexdigest(),
        executable_sha256="a" * 64,
        declaration_sha256=None,
        acceptance_sha256=None,
        execution_approval_sha256=None,
    )
    save(
        "header.json",
        dict(
            schema="war1-study-archive-v1",
            mode=m.purpose,
            binding=binding,
            planned=len(m.cases),
        ),
    )
    (root / "manifest.json").write_bytes(m.raw_bytes)
    save(
        "source-inventory.json",
        dict(
            schema="war1-study-source-inventory-v1",
            compiled_inputs_sha256=sha.hexdigest(),
            entries=entries,
        ),
    )
    arms = {a["arm_id"]: a for a in m.arms}
    blocks = []
    prefix = hashlib.sha256()
    total = 0
    count = len(m.cases) if stop is None else stop
    for block_id, first in enumerate(range(0, count, 128)):
        data = bytearray()
        for case in m.cases[first : min(first + 128, count)]:
            row = synthetic_row(arms[case.key.arm_id], case, m.sha256)
            if case.key.replicate_index == 0:
                audit = attach_audit(row)
                (root / row["audit_descriptor"]["path"]).write_bytes(audit)
            data.extend(
                canonical_bytes(
                    dict(
                        kind="start",
                        key=row["key"],
                        seed=row["seed"],
                        case_input_sha256=row["case_input_sha256"],
                    )
                )
            )
            data.extend(canonical_bytes(dict(kind="outcome", outcome=row)))
        path = f"blocks/block-{block_id:06}.jsonl"
        (root / path).write_bytes(data)
        descriptor = dict(
            path=path,
            sha256=hashlib.sha256(data).hexdigest(),
            bytes=len(data),
            first_ordinal=first,
            acknowledged_cases=min(128, count - first),
        )
        save(f"blocks/block-{block_id:06}.descriptor.json", descriptor)
        blocks.append(descriptor)
        prefix.update(data)
        total += len(data)
    census = dict(
        planned=len(m.cases),
        attempted=count,
        complete_valid=count,
        failed=0,
        invalid=0,
        partial=0,
        pending=0,
        unstarted=len(m.cases) - count,
    )
    if stop is None:
        save(
            "candidate-index.json",
            dict(
                schema="war1-study-candidate-index-v1",
                mode=m.purpose,
                binding=binding,
                census=census,
                receipt=dict(
                    sha256=prefix.hexdigest(), bytes=total, acknowledged_cases=count
                ),
                blocks=blocks,
                monitor="construction_deterministic_no_scientific_resource_decision",
            ),
        )
    return census
