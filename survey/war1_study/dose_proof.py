"""Independent exact-rational domain proof for the complete approved roster.

For aimed fire, rate * attackers / targets * dt is increasing in attackers
and decreasing in targets; the closed integer rectangle 1..initial therefore
has extrema at opposite corners. Duel dose is constant. This covers every
interior count, including states a trajectory may never visit.

Native order is (rate * integer / integer) * dt (duel: rate * dt). All input
counts are exact binary64 and all intermediate values are positive normal
numbers in this fixed roster. Three round-to-nearest operations give relative
error <= gamma_3 = 3u/(1-3u) < 2^-50, u=2^-53. We use this conservative envelope,
without claiming any general libm accuracy. For rounded dose d, p*=1-exp(-d)
>= d/(1+d); d<9 and e<3 imply 1-p*>1/3^9. Both margins exceed 2^-53.
Native preflight additionally checks actual kernel p and q for every interior
state, without constructing an Engine. Neither check extends the tiny-law
four-ULP certificate or gives a general libm/RNG/path-error guarantee.
"""

from fractions import Fraction as F
import hashlib
import re

from .io import canonical_bytes
from .manifest import PROTOCOL_SHA256, PROPOSAL_SHA256, _same, proposal


def _r(value):
    return f"{value.numerator}/{value.denominator}"


def dose_proof(root, manifest_sha256, compiled_inputs_sha256):
    for value in (manifest_sha256, compiled_inputs_sha256):
        if not isinstance(value, str) or re.fullmatch("[0-9a-f]{64}", value) is None:
            raise ValueError("dose proof requires exact manifest/source identities")
    approved = proposal(root)["arms"]
    if len(approved) != 66:
        raise ValueError("dose proof approved census")
    arms = []
    extrema = []
    for arm in approved:
        c = arm["config"]
        b, r = c["blue"], c["red"]
        targets = []
        for rate, attackers, defenders in (
            (c["red_rate"], r, b),
            (c["blue_rate"], b, r),
        ):
            rate, dt = F.from_float(float(rate)), F.from_float(float(c["dt"]))
            # Fixed input/normal-operation bounds, not assertions about libm.
            if not (
                1 <= attackers <= 512
                and 1 <= defenders <= 512
                and F(1, 4) <= rate <= 1
                and F(1, 64) <= dt <= F(1, 16)
            ):
                raise ValueError("dose proof arithmetic domain")
            lo, hi = rate * dt, rate * dt
            if c["geometry"] == "aimed_fire":
                lo, hi = lo / defenders, hi * attackers
            elif c["geometry"] != "duel_contact":
                raise ValueError("dose proof geometry")
            extrema.extend((lo, hi))
            targets.append([_r(lo), _r(hi)])
        work = (b + r) * c["max_steps"]
        arms.append(
            dict(
                arm_id=arm["arm_id"],
                config_sha256=hashlib.sha256(canonical_bytes(c)).hexdigest(),
                interior_counts=[b, r],
                interior_target_states=2 * b * r,
                targets=targets,
                case_work=work,
                quantization_only_path_bound=_r(F(work, 2**53)),
            )
        )
    lo, hi = min(extrema), max(extrema)
    error = F(1, 2**50)
    native_lo, native_hi = lo * (1 - error), hi * (1 + error)
    probability_lo, survival_lo = native_lo / (1 + native_lo), F(1, 3**9)
    resolution = F(1, 2**53)
    if not (
        native_lo > 0
        and native_hi < 9
        and probability_lo > resolution
        and survival_lo > resolution
    ):
        raise ValueError("dose proof resolution/saturation margin")
    work = max(a["case_work"] for a in arms)
    return dict(
        schema="war1-study-dose-proof-v1",
        method="monotone-interior-rectangle-gamma3-v1",
        protocol_sha256=PROTOCOL_SHA256,
        proposal_sha256=PROPOSAL_SHA256,
        approved_roster_sha256=hashlib.sha256(canonical_bytes(approved)).hexdigest(),
        manifest_sha256=manifest_sha256,
        compiled_inputs_sha256=compiled_inputs_sha256,
        arms=arms,
        interior_target_states=sum(a["interior_target_states"] for a in arms),
        positive_dose_extrema=[_r(lo), _r(hi)],
        native_relative_error_bound=_r(error),
        native_dose_enclosure=[_r(native_lo), _r(native_hi)],
        probability_lower_bound=_r(probability_lo),
        survival_lower_bound=_r(survival_lo),
        sampler_resolution=_r(resolution),
        maximum_case_work=work,
        quantization_only_path_bound=_r(F(work, 2**53)),
    )


def validate_dose_proof(packet, root, manifest_sha256, compiled_inputs_sha256):
    expected = dose_proof(root, manifest_sha256, compiled_inputs_sha256)
    if not _same(packet, expected):
        raise ValueError(
            "dose proof differs from independent complete-roster recomputation"
        )
