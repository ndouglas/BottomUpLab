"""Independent ideal-word tiny-count laws; never calls a simulation or RNG."""

from dataclasses import dataclass
from fractions import Fraction
from math import comb
from pathlib import Path

from .contracts import Manifest, Json
from .manifest import proposal
from .numerics import Interval, enclose, exp_interval, probability_certificate


@dataclass(frozen=True)
class Oracle:
    arm_id: str
    scalars: dict[str, Interval]
    cdfs: dict[str, tuple[tuple[Fraction, Interval], ...]]
    numerical_certificate: Json
    ctmc_one_vs_one: Json | None


def geometric_sum(s: Fraction, k: int) -> Fraction:
    if not 0 <= s <= 1 or type(k) is not int or k < 0:
        raise ValueError("geometric law requires s in [0,1] and nonnegative integer k")
    return Fraction(k) if s == 1 else (1 - s**k) / (1 - s)


def count_transition(
    blue: int, red: int, exposed: tuple[int, int], q: tuple[Fraction, Fraction]
) -> dict[tuple[int, int], Fraction]:
    if any(type(n) is not int or n < 0 for n in (blue, red, *exposed)):
        raise ValueError("counts must be nonnegative integers")
    if (
        len(exposed) != 2
        or len(q) != 2
        or exposed[0] > blue
        or exposed[1] > red
        or any(not 0 <= p <= 1 for p in q)
    ):
        raise ValueError("invalid exposure/probability")
    if blue == 0 or red == 0:
        return {(blue, red): Fraction(1)}
    losses = []
    for n, p in zip(exposed, q):
        losses.append([comb(n, d) * p**d * (1 - p) ** (n - d) for d in range(n + 1)])
    result = {
        (blue - db, red - dr): pb * pr
        for db, pb in enumerate(losses[0])
        for dr, pr in enumerate(losses[1])
        if pb * pr
    }
    if sum(result.values()) != 1:
        raise ValueError("transition mass does not equal one")
    return result


def finite_horizon(
    initial: tuple[int, int],
    k: int,
    probabilities: dict[tuple[int, int], tuple[Fraction, Fraction]],
    geometry: str,
    observation_steps: tuple[int, ...],
) -> dict:
    """Exact shared propagation API, also usable for literal K=1/2 controls.

    Zero q is a mathematical limit here, not the engine's rate-zero ending.
    """
    if (
        initial not in ((1, 1), (2, 1))
        or type(k) is not int
        or not 1 <= k <= 512
        or geometry not in ("aimed_fire", "duel_contact")
    ):
        raise ValueError("invalid tiny propagation descriptor")
    if any(type(j) is not int or not 1 <= j <= k for j in observation_steps):
        raise ValueError("observation outside horizon")
    transitions = {}
    for blue in range(initial[0] + 1):
        for red in range(initial[1] + 1):
            state = (blue, red)
            exposed = state if geometry == "aimed_fire" else (min(state), min(state))
            if blue and red and state not in probabilities:
                raise ValueError("missing interior-state probabilities")
            transitions[state] = count_transition(
                blue, red, exposed, probabilities.get(state, (Fraction(0), Fraction(0)))
            )
    current = {initial: Fraction(1)}
    duration_sum = Fraction(0)
    duration_cdf = []
    means = {}
    for step in range(1, k + 1):
        duration_sum += sum(
            mass for (blue, red), mass in current.items() if blue and red
        )
        following = {}
        for state, mass in current.items():
            for target, chance in transitions[state].items():
                following[target] = following.get(target, Fraction(0)) + mass * chance
        current = following
        if sum(current.values()) != 1:
            raise ValueError("propagated probability mass differs from one")
        survival = sum(mass for (blue, red), mass in current.items() if blue and red)
        duration_cdf.append(
            (Fraction(step, k), Fraction(1) if step == k else 1 - survival)
        )
        if step in observation_steps:
            means[step] = tuple(
                sum(
                    mass * state[side] / initial[side]
                    for state, mass in current.items()
                )
                for side in (0, 1)
            )
    endings = {
        key: Fraction(0)
        for key in (
            "blue_survivor",
            "red_survivor",
            "double_extinction",
            "rate_zero",
            "horizon",
        )
    }
    for (blue, red), mass in current.items():
        if blue and red:
            key = "horizon"
        elif blue:
            key = "blue_survivor"
        elif red:
            key = "red_survivor"
        else:
            key = "double_extinction"
        endings[key] += mass
    survivor_cdfs = []
    for side, total in enumerate(initial):
        survivor_cdfs.append(
            tuple(
                (
                    Fraction(j, total),
                    sum(mass for state, mass in current.items() if state[side] <= j),
                )
                for j in range(total + 1)
            )
        )
    return dict(
        endings=endings,
        duration_mean=duration_sum / k,
        duration_cdf=tuple(duration_cdf),
        survivor_cdfs=tuple(survivor_cdfs),
        means=means,
    )


def _ctmc(scalars: dict[str, Interval], dt: Fraction) -> Json:
    survival = exp_interval(Fraction(-16))
    departed = Interval(1 - survival.hi, 1 - survival.lo)
    categories = {
        "ending.blue_survivor": Interval(departed.lo / 2, departed.hi / 2),
        "ending.red_survivor": Interval(departed.lo / 2, departed.hi / 2),
        "ending.double_extinction": Interval(Fraction(0), Fraction(0)),
        "ending.horizon": survival,
        "duration.restricted_mean_over_H": Interval(departed.lo / 16, departed.hi / 16),
    }
    criteria = {}
    tolerance = Fraction(1, 50)
    for endpoint, reference in categories.items():
        actual = scalars[endpoint]
        difference = Interval(actual.lo - reference.hi, actual.hi - reference.lo)
        lower = (
            Fraction(0)
            if difference.lo <= 0 <= difference.hi
            else min(abs(difference.lo), abs(difference.hi))
        )
        error = Interval(lower, max(abs(difference.lo), abs(difference.hi)))
        margin = Interval(tolerance - error.hi, tolerance - error.lo)
        criteria[endpoint] = {
            "continuous": reference.wire(),
            "absolute_difference": error.wire(),
            "margin": margin.wire(),
            "passes": margin.lo >= 0,
        }
    return {
        "scope": "one_vs_one_only",
        "required_at_this_dt": dt == Fraction(1, 64),
        "tolerance": Interval(tolerance, tolerance).wire(),
        "criteria": criteria,
    }


def prepare_oracles(manifest: Manifest, numeric_export: Json) -> tuple[Oracle, ...]:
    expected = {
        "schema",
        "purpose",
        "manifest_sha256",
        "rules_id",
        "binding_state",
        "compiled_inputs_sha256",
        "states",
    }
    if not isinstance(numeric_export, dict) or set(numeric_export) != expected:
        raise ValueError("malformed numeric export")
    for field, value in [
        ("schema", "war1-study-numerics-v1"),
        ("purpose", manifest.purpose),
        ("manifest_sha256", manifest.sha256),
        ("rules_id", "war1-engagement-v1"),
        ("binding_state", "engineering_unbound"),
        ("compiled_inputs_sha256", None),
    ]:
        if numeric_export[field] != value:
            raise ValueError(f"numeric export {field} binding mismatch")
    approved = proposal(Path(__file__).resolve().parents[2])
    arms = [a for a in approved["arms"] if a["group"] == "oracle"]
    expected_states = [
        (a["arm_id"], blue, red)
        for a in arms
        for blue in range(1, a["config"]["blue"] + 1)
        for red in range(1, a["config"]["red"] + 1)
    ]
    states = numeric_export["states"]
    if not isinstance(states, list) or len(states) != len(expected_states):
        raise ValueError("numeric state census mismatch")
    by_arm = {a["arm_id"]: a for a in arms}
    probabilities = {a["arm_id"]: {} for a in arms}
    certificates = {a["arm_id"]: [] for a in arms}
    for row, key in zip(states, expected_states):
        if not isinstance(row, dict) or set(row) != {
            "arm_id",
            "blue",
            "red",
            "targets",
        }:
            raise ValueError("malformed numeric state")
        if (
            type(row["blue"]) is not int
            or type(row["red"]) is not int
            or (row["arm_id"], row["blue"], row["red"]) != key
        ):
            raise ValueError("numeric state identity/order mismatch")
        targets = row["targets"]
        if not isinstance(targets, list) or len(targets) != 2:
            raise ValueError("two numeric targets required")
        qs = []
        for index, target in enumerate(targets):
            if not isinstance(target, dict) or set(target) != {
                "hazard_bits",
                "dose_bits",
                "p_bits",
                "q_numerator",
            }:
                raise ValueError("malformed native target")
            cert = probability_certificate(
                {
                    **target,
                    "config": by_arm[key[0]]["config"],
                    "blue": row["blue"],
                    "red": row["red"],
                    "target_index": index,
                }
            )
            certificates[key[0]].append(cert)
            qs.append(Fraction(cert["q_numerator"], 2**53))
        probabilities[key[0]][key[1:]] = tuple(qs)
    results = []
    for arm in arms:
        config = arm["config"]
        arm_id = arm["arm_id"]
        dt = Fraction(config["dt"])
        k = config["max_steps"]
        observation_steps = tuple(
            int(Fraction(t) / dt) for t in approved["observation_times"]
        )
        probs = probabilities[arm_id]
        exact = finite_horizon(
            (config["blue"], config["red"]),
            k,
            probs,
            config["geometry"],
            observation_steps,
        )
        scalars = {
            f"population.{side}.t{time}": enclose(exact["means"][step][index])
            for time, step in zip(approved["observation_times"], observation_steps)
            for index, side in enumerate(("blue", "red"))
        }
        scalars.update(
            {
                f"ending.{name}": enclose(value)
                for name, value in exact["endings"].items()
            }
        )
        scalars["duration.restricted_mean_over_H"] = enclose(exact["duration_mean"])
        cdfs = {
            "duration.U_over_H": tuple(
                (point, enclose(value)) for point, value in exact["duration_cdf"]
            )
        }
        for index, side in enumerate(("blue", "red")):
            cdfs[f"survivors.{side}_fraction"] = tuple(
                (point, enclose(value))
                for point, value in exact["survivor_cdfs"][index]
            )
        if any(bound.hi - bound.lo > Fraction(1, 10**10) for bound in scalars.values()):
            raise ValueError("oracle numerical width exceeds registered bound")
        certificate = {
            "arithmetic": "exact_rational_product_binomial",
            "mass": Interval(Fraction(1), Fraction(1)).wire(),
            "endpoint_scale": 10**12,
            "states": certificates[arm_id],
            "manifest_sha256": manifest.sha256,
            "rules_id": numeric_export["rules_id"],
        }
        results.append(
            Oracle(
                arm_id,
                scalars,
                cdfs,
                certificate,
                _ctmc(scalars, dt) if config["blue"] == config["red"] == 1 else None,
            )
        )
    return tuple(results)
