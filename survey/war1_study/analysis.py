"""Fixed-family summaries of retained evidence; no simulation or seed draws."""

from fractions import Fraction
from collections import Counter
from dataclasses import dataclass
from functools import lru_cache
from pathlib import Path

from .contracts import Json, Manifest
from .archive import ArchiveRead
from .io import canonical_bytes, create_bytes, METADATA_LIMIT
from .manifest import proposal
from .numerics import Interval, confidence_radius, difference
from .oracle import Oracle


def mapped_endpoint(endpoint_id, mapping):
    if mapping == "identity":
        return endpoint_id
    if mapping != "swap_blue_red":
        raise ValueError("unknown endpoint mapping")
    return (
        endpoint_id.replace("blue", "SIDE")
        .replace("red", "blue")
        .replace("SIDE", "red")
    )


POPULATIONS = {
    f"population.{side}.t{t}": (index, j)
    for j, t in enumerate((1, 2, 4, 8))
    for index, side in enumerate(("blue", "red"))
}


def endpoint(row: Json, endpoint_id: str) -> Fraction | None:
    if endpoint_id in POPULATIONS:
        side, j = POPULATIONS[endpoint_id]
        denominator = row["initial_counts"][side]
        return (
            Fraction(row["observations"][j]["counts"][side], denominator)
            if denominator
            else None
        )
    if endpoint_id in ("duration.restricted_mean_over_H", "duration.U_over_H"):
        if row["ending"]["reason"] == "rate_zero":
            return None
        denominator = row["final_checkpoint"]["config"]["max_steps"]
        return Fraction(row["completed_steps"], denominator) if denominator else None
    if endpoint_id in ("survivors.blue_fraction", "survivors.red_fraction"):
        side = int(endpoint_id == "survivors.red_fraction")
        denominator = row["initial_counts"][side]
        return Fraction(row["final_counts"][side], denominator) if denominator else None
    if endpoint_id in ("ending.blue_survivor", "ending.red_survivor"):
        return Fraction(
            row["ending"]["reason"] == "one_side_extinction"
            and row["ending"]["winner"] == endpoint_id.split(".")[1].split("_")[0]
        )
    if endpoint_id in (
        "ending.double_extinction",
        "ending.rate_zero",
        "ending.horizon",
    ):
        return Fraction(row["ending"]["reason"] == endpoint_id.split(".")[1])
    raise ValueError(f"unknown endpoint: {endpoint_id}")


@dataclass(frozen=True)
class Analysis:
    scalars: tuple[Json, ...]
    cdfs: tuple[Json, ...]
    contrasts: tuple[Json, ...]
    cdf_comparisons: tuple[Json, ...]
    oracle_checks: tuple[Json, ...]
    census: Json
    primary_available: bool


def rational(value):
    return dict(numerator=str(value.numerator), denominator=str(value.denominator))


@lru_cache(maxsize=4)
def _radius(slots, n):
    return confidence_radius(slots, n, Fraction(1, 40))


def _band(mean, radius):
    return Interval(max(Fraction(0), mean - radius), min(Fraction(1), mean + radius))


def _support(config, endpoint_id):
    if endpoint_id == "duration.U_over_H":
        k = config["max_steps"]
        return tuple(Fraction(j, k) for j in range(1, k + 1))
    n = config["blue" if endpoint_id == "survivors.blue_fraction" else "red"]
    return tuple(Fraction(j, n) for j in range(n + 1)) if n else ()


def _cdf_distance(left, right):
    """Union support, right-continuous; intervals may be empirical or ideal."""
    points = sorted(set(left) | set(right))
    a = b = Interval(Fraction(0), Fraction(0))
    lower = upper = Fraction(0)
    for point in points:
        a = left.get(point, a)
        b = right.get(point, b)
        delta = difference(a, b)
        lower = max(lower, max(delta.lo, -delta.hi, Fraction(0)))
        upper = max(upper, abs(delta.lo), abs(delta.hi))
    return Interval(lower, upper)


def _oracle_for(arm, oracles, manifest):
    wanted = arm["arm_id"]
    for oracle in oracles:
        if manifest.purpose == "construction_fixture":
            states = oracle.numerical_certificate.get("states", [])
            if not states or states[0].get("config") != arm["config"]:
                continue
        elif oracle.arm_id != wanted:
            continue
        if oracle.numerical_certificate.get("manifest_sha256") != manifest.sha256:
            return None
        return oracle
    return None


def analyze(
    manifest: Manifest, archive: ArchiveRead, oracles: tuple[Oracle, ...]
) -> Analysis:
    approved = proposal(Path(__file__).resolve().parents[2])
    scalar_ids = approved["scalar_endpoint_ids"]
    cdf_ids = [e["id"] for e in approved["cdf_endpoints"]]
    arms = {a["arm_id"]: a for a in manifest.arms}
    sums = {a: {e: Fraction(0) for e in scalar_ids} for a in arms}
    unavailable = {a: Counter() for a in arms}
    hist = {a: {e: Counter() for e in cdf_ids} for a in arms}
    observed_counts = Counter()
    references = {a: Counter() for a in arms}
    reference_missing = {a: Counter() for a in arms}
    seen = set()
    stream_error = None
    try:
        for row in archive.outcomes():
            key = row["key"]
            ordinal = key["ordinal"]
            if (
                type(ordinal) is not int
                or not 0 <= ordinal < len(manifest.cases)
                or ordinal in seen
            ):
                raise ValueError("duplicate/invalid outcome ordinal")
            case = manifest.cases[ordinal]
            if (key["arm_id"], key["replicate_index"], row["seed"]) != (
                case.key.arm_id,
                case.key.replicate_index,
                case.seed,
            ):
                raise ValueError("outcome key/seed mismatch")
            seen.add(ordinal)
            a = key["arm_id"]
            observed_counts[a] += 1
            for e in scalar_ids:
                value = endpoint(row, e)
                if value is None:
                    unavailable[a][e] += 1
                else:
                    sums[a][e] += value
            for e in cdf_ids:
                value = endpoint(row, e)
                if value is None:
                    unavailable[a][e] += 1
                else:
                    hist[a][e][value] += 1
            for j, t in enumerate((1, 2, 4, 8)):
                for side, index in [("blue", 0), ("red", 1)]:
                    e = f"population.{side}.t{t}"
                    ref = row["observations"][j]["reference"]
                    denominator = row["initial_counts"][index]
                    if ref is None or not denominator:
                        reference_missing[a][e] += 1
                    else:
                        references[a][e] += (
                            Fraction.from_float(float(ref["forces"][index]))
                            / denominator
                        )
    except (ValueError, OSError) as error:
        stream_error = str(error)
    complete = (
        archive.complete and len(seen) == len(manifest.cases) and stream_error is None
    )
    validation = archive.header.get("validation", {})
    primary = complete and validation.get("primary_available") is True
    base_reason = (
        None
        if primary
        else (
            "incomplete_archive"
            if not complete
            else validation.get("reason")
            or "monitored_success_or_authorization_unavailable"
        )
    )
    registered = primary and validation.get("registered") is True
    common = dict(
        registered=registered,
        evidence_label=archive.header.get(
            "evidence_label",
            (
                "construction_fixture"
                if manifest.purpose == "construction_fixture"
                else "retained_campaign"
            ),
        ),
    )
    scalars = []
    scalar_values = {}
    scalar_bands = {}
    cdfs = []
    cdf_values = {}
    cdf_radii = {}
    for a, arm in arms.items():
        n = arm["replicates"]
        h = _radius(924, n).hi if primary else None
        d = _radius(198, n).hi if primary else None
        for e in scalar_ids:
            reason = base_reason
            mean = sums[a][e] / n if complete and not unavailable[a][e] else None
            if unavailable[a][e]:
                reason = "zero_or_unavailable_denominator_or_duration"
            band = _band(mean, h) if mean is not None and primary else None
            scalar_values[a, e] = mean
            scalar_bands[a, e] = band
            reference = None
            if e.startswith("population."):
                ref = (
                    references[a][e] / n
                    if complete and not reference_missing[a][e]
                    else None
                )
                reference = dict(
                    mean=rational(ref) if ref is not None else None,
                    deviation=(
                        rational(mean - ref)
                        if mean is not None and ref is not None
                        else None
                    ),
                    interval=(
                        difference(band, Interval(ref, ref)).wire(10**12)
                        if band and ref is not None
                        else None
                    ),
                    reason="reference_unavailable" if ref is None else base_reason,
                )
            scalars.append(
                dict(
                    **common,
                    arm_id=a,
                    endpoint_id=e,
                    planned_n=n,
                    observed_n=observed_counts[a],
                    mean=rational(mean) if mean is not None else None,
                    interval=band.wire(10**12) if band else None,
                    radius=_radius(924, n).wire(10**12) if primary else None,
                    reason=reason,
                    reference=reference,
                )
            )
        for e in cdf_ids:
            support = _support(arm["config"], e)
            valid = complete and not unavailable[a][e] and bool(support)
            values = {}
            running = 0
            points = []
            for point in support:
                running += hist[a][e][point]
                value = Fraction(running, n) if valid else None
                if value is not None:
                    values[point] = Interval(value, value)
                band = _band(value, d) if value is not None and primary else None
                points.append(
                    dict(
                        point=rational(point),
                        cdf=rational(value) if value is not None else None,
                        band=band.wire(10**12) if band else None,
                    )
                )
            cdf_values[a, e] = values if valid else None
            cdf_radii[a, e] = d
            cdfs.append(
                dict(
                    **common,
                    arm_id=a,
                    endpoint_id=e,
                    planned_n=n,
                    support=points,
                    radius=_radius(198, n).wire(10**12) if primary else None,
                    censor_fraction=(
                        rational(sums[a]["ending.horizon"] / n) if complete else None
                    ),
                    reason=(
                        "zero_or_unavailable_denominator_or_duration"
                        if unavailable[a][e] or not support
                        else base_reason
                    ),
                )
            )
    contrasts = []
    for group in approved["contrast_groups"]:
        a, b = group["left_arm"], group["right_arm"]
        if a not in arms or b not in arms:
            continue
        for e in scalar_ids:
            mapped = mapped_endpoint(e, group["right_endpoint_mapping"])
            x, y = scalar_values[a, e], scalar_values[b, mapped]
            left, right = scalar_bands[a, e], scalar_bands[b, mapped]
            band = difference(left, right) if left and right else None
            if band and a == b and e == mapped:
                band = Interval(Fraction(0), Fraction(0))
            contrasts.append(
                dict(
                    **common,
                    **group,
                    endpoint_id=e,
                    right_endpoint_id=mapped,
                    estimate=(
                        rational(x - y) if x is not None and y is not None else None
                    ),
                    interval=band.wire(10**12) if band else None,
                    reason=(
                        base_reason
                        if x is not None and y is not None
                        else "endpoint_unavailable"
                    ),
                )
            )
    comparisons = []
    for group in approved["cdf_comparisons"]:
        a, b = group["left_arm"], group["right_arm"]
        if a not in arms or b not in arms:
            continue
        left = cdf_values[a, group["left_endpoint"]]
        right = cdf_values[b, group["right_endpoint"]]
        distance = (
            _cdf_distance(left, right)
            if left is not None and right is not None
            else None
        )
        envelope = (
            cdf_radii[a, group["left_endpoint"]] + cdf_radii[b, group["right_endpoint"]]
            if primary
            else None
        )
        band = (
            _band(distance.lo, envelope) if distance is not None and primary else None
        )
        comparisons.append(
            dict(
                **common,
                **group,
                distance=rational(distance.lo) if distance else None,
                interval=band.wire(10**12) if band else None,
                envelope=(
                    Interval(envelope, envelope).wire(10**12)
                    if envelope is not None
                    else None
                ),
                reason=base_reason if distance else "endpoint_unavailable",
            )
        )
    oracle_binding_error = False
    if "validated_oracle_rows_sha256" in archive.header:
        import hashlib
        from .archive_provenance import oracle_rows

        if (
            hashlib.sha256(canonical_bytes(oracle_rows(oracles))).hexdigest()
            != archive.header["validated_oracle_rows_sha256"]
        ):
            oracles = ()
            oracle_binding_error = True
    checks = []
    for a, arm in arms.items():
        if arm["group"] != "oracle" and not (
            manifest.purpose == "construction_fixture"
            and arm["profile"] in ("1v1", "2v1")
        ):
            continue
        oracle = _oracle_for(arm, oracles, manifest)
        oracle_reason = (
            (
                "oracle_binding_mismatch"
                if oracle_binding_error
                else "oracle_unavailable"
            )
            if oracle is None
            else base_reason
        )
        if manifest.purpose == "construction_fixture" and not oracle_binding_error:
            oracle_reason = "unregistered_construction_panel"
        for e in scalar_ids:
            ideal = oracle.scalars.get(e) if oracle else None
            observed = scalar_bands[a, e]
            verdict = None
            if primary and ideal is not None and observed is not None:
                verdict = (
                    "incompatible_with_ideal_word_law"
                    if observed.hi < ideal.lo or ideal.hi < observed.lo
                    else "not_rejected"
                )
            checks.append(
                dict(
                    **common,
                    arm_id=a,
                    oracle_id=oracle.arm_id if oracle else None,
                    kind="scalar",
                    endpoint_id=e,
                    ideal=ideal.wire(10**12) if ideal else None,
                    verdict=verdict,
                    reason=(
                        oracle_reason
                        if ideal is not None or oracle is None
                        else "oracle_endpoint_unavailable"
                    ),
                )
            )
        for e in cdf_ids:
            ideal = dict(oracle.cdfs[e]) if oracle and e in oracle.cdfs else None
            observed = cdf_values[a, e]
            delta = (
                _cdf_distance(observed, ideal)
                if observed is not None and ideal is not None
                else None
            )
            verdict = None
            if primary and delta is not None:
                verdict = (
                    "incompatible_with_ideal_word_law"
                    if delta.lo > cdf_radii[a, e]
                    else "not_rejected"
                )
            checks.append(
                dict(
                    **common,
                    arm_id=a,
                    oracle_id=oracle.arm_id if oracle else None,
                    kind="cdf",
                    endpoint_id=e,
                    distance=delta.wire(10**12) if delta else None,
                    verdict=verdict,
                    reason=(
                        oracle_reason
                        if ideal is not None or oracle is None
                        else "oracle_endpoint_unavailable"
                    ),
                )
            )
        if arm["config"]["blue"] == arm["config"]["red"] == 1:
            ctmc = oracle.ctmc_one_vs_one if oracle else None
            for e in (
                "ending.blue_survivor",
                "ending.red_survivor",
                "ending.double_extinction",
                "ending.horizon",
                "duration.restricted_mean_over_H",
            ):
                criterion = ctmc["criteria"].get(e) if ctmc else None
                checks.append(
                    dict(
                        **common,
                        arm_id=a,
                        oracle_id=oracle.arm_id if oracle else None,
                        kind="ctmc_one_vs_one",
                        endpoint_id=e,
                        required_at_this_dt=Fraction(arm["config"]["dt"])
                        == Fraction(1, 64),
                        criterion=criterion,
                        reason=None if criterion else "oracle_unavailable",
                    )
                )
    census = {
        **archive.census,
        "analysis_observed_keys": len(seen),
        "analysis_stream_error": stream_error,
        "registered": registered,
        "evidence_label": common["evidence_label"],
    }
    return Analysis(
        tuple(scalars),
        tuple(cdfs),
        tuple(contrasts),
        tuple(comparisons),
        tuple(checks),
        census,
        primary,
    )


def save_analysis(out: Path, analysis: Analysis, binding: Json) -> None:
    """Create-only deterministic report files, each bounded before publication."""
    out = Path(out)
    out.mkdir(parents=True, exist_ok=False)
    for name in ("scalars", "cdfs", "contrasts", "cdf_comparisons", "oracle_checks"):
        create_bytes(
            out / f"{name}.json",
            canonical_bytes(
                dict(
                    schema="war1-study-analysis-rows-v1",
                    binding=binding,
                    rows=getattr(analysis, name),
                )
            ),
            METADATA_LIMIT,
        )
    create_bytes(
        out / "summary.json",
        canonical_bytes(
            dict(
                schema="war1-study-analysis-v1",
                binding=binding,
                census=analysis.census,
                primary_available=analysis.primary_available,
            )
        ),
        METADATA_LIMIT,
    )
