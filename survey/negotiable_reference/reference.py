"""Exact finite references derived independently of the Rust implementation.

Sources: Desai, Critch and Russell (NeurIPS 2018), main §3/equations 3–8;
supplement §1 permits selecting a random policy once before observations.
These engineering fixtures check finite algebra, not the general theorem or
the paper's gridworld experiments. Synthetic hull tests are mathematical only.
"""

from dataclasses import dataclass
from fractions import Fraction as F
import hashlib
import itertools
import json
from math import prod
from pathlib import Path
import re


SCHEMA = "negotiable-foundation-v1"
MAX_BYTES = 8 * 1024 * 1024
TOTAL_MAX_BYTES = 16 * 1024 * 1024
SOURCE_HASHES = {
    "main_pdf": "8eaba6509a3fc39a08905ccadac6ad2cd69c308d9b8cb404248fb6a944a60492",
    "supplement_pdf": "643655853ddf456513aa430094449db81bb939447ac1ad63bf5b864826b5c140",
}
UTILITIES = ((F(30), F(0)), (F(20), F(20)), (F(0), F(30)))
MODELS = ((F(9, 10), F(1, 10)), (F(1, 2), F(1, 2)), (F(1), F(4, 5)))
PRIORS = (F(0), F(1, 2), F(1))
HORIZON = 8
WEIGHTS = tuple(sorted({F(k, 20) for k in range(21)} |
                       {F(1, 19), F(2, 11), F(9, 11), F(18, 19)}))
POLICIES = tuple(itertools.product(range(3), repeat=2))
ROW_COUNTS = {"cake": 225, "pareto": 9, "priority": 4599}


def wire(value):
    value = F(value)
    return {"num": str(value.numerator), "den": str(value.denominator)}


def settings(kind):
    if type(kind) is not str or kind not in ROW_COUNTS:
        raise ValueError(f"unknown reference kind: {kind!r}")
    return {
        "reference_kind": kind,
        "source_hashes": dict(SOURCE_HASHES),
        "observations": ["red", "green"],
        "actions": [
            {"id": i, "name": name, "utilities": [wire(q) for q in utility]}
            for i, (name, utility) in enumerate(zip(
                ("alice_only", "equal_split", "bob_only"), UTILITIES))
        ],
        "policies": [{"id": i, "red_action": red, "green_action": green}
                     for i, (red, green) in enumerate(POLICIES)],
        "weights": [wire(q) for q in WEIGHTS],
        "likelihood_models": [{"id": i, "red_likelihoods": [wire(q) for q in model]}
                              for i, model in enumerate(MODELS)],
        "cake_model_id": 0,
        "priors": [[wire(q), wire(1 - q)] for q in PRIORS],
        "history_horizon": HORIZON,
        "pareto_definitions": {
            "paper_weak": "dominated iff a hull point strictly improves both coordinates",
            "strong": "dominated iff a hull point weakly improves both and strictly improves at least one",
        },
    }


def cake_payoffs():
    """Each principal evaluates their own utility using their own likelihoods."""
    alice_red, bob_red = MODELS[0]
    return [
        (alice_red * UTILITIES[red][0] + (1 - alice_red) * UTILITIES[green][0],
         bob_red * UTILITIES[red][1] + (1 - bob_red) * UTILITIES[green][1])
        for red, green in POLICIES
    ]


def local_scores(weight, observation):
    alice_red, bob_red = MODELS[0]
    alice_likelihood = alice_red if observation == 0 else 1 - alice_red
    bob_likelihood = bob_red if observation == 0 else 1 - bob_red
    masses = (weight * alice_likelihood, (1 - weight) * bob_likelihood)
    return [masses[0] * a + masses[1] * b for a, b in UTILITIES]


def argmax(values):
    best = max(values)
    return [i for i, value in enumerate(values) if value == best]


def derive_boundaries():
    """Solve pairwise affine local score equalities; retain upper-envelope ties."""
    boundaries = set()
    for observation in (0, 1):
        intercepts = local_scores(F(0), observation)
        slopes = [end - start for end, start in
                  zip(local_scores(F(1), observation), intercepts)]
        for first, second in itertools.combinations(range(3), 2):
            difference = slopes[first] - slopes[second]
            if difference == 0:
                continue
            weight = (intercepts[second] - intercepts[first]) / difference
            if 0 < weight < 1:
                winners = argmax(local_scores(weight, observation))
                if first in winners and second in winners:
                    boundaries.add(weight)
    return sorted(boundaries)


def build_cake():
    payoffs = cake_payoffs()
    rows = []
    for weight in WEIGHTS:
        objectives = [weight * a + (1 - weight) * b for a, b in payoffs]
        global_ids = argmax(objectives)
        local_ids = sorted(3 * red + green for red, green in itertools.product(
            argmax(local_scores(weight, 0)), argmax(local_scores(weight, 1))))
        if global_ids != local_ids:
            raise ValueError(f"cake derivations disagree at weight {weight}")
        for policy_id, (payoff, objective) in enumerate(zip(payoffs, objectives)):
            rows.append({"weight": wire(weight), "policy_id": policy_id,
                         "expected": [wire(q) for q in payoff], "objective": wire(objective),
                         "global_maximizers": global_ids[:], "local_maximizers": local_ids[:]})
    return {"schema": SCHEMA, "settings": settings("cake"), "rows": rows}


@dataclass
class Vertex:
    payoff: tuple
    coefficients: dict


def _cross(a, b, c):
    return ((b[0] - a[0]) * (c[1] - a[1]) -
            (b[1] - a[1]) * (c[0] - a[0]))


def convex_hull(points):
    """Monotone chain; geometric duplicates merge but original IDs stay in points."""
    representatives = {}
    for policy_id, point in enumerate(points):
        representatives.setdefault(point, policy_id)
    ordered = sorted(representatives)
    if len(ordered) <= 1:
        hull = ordered
    else:
        lower, upper = [], []
        for chain, sequence in ((lower, ordered), (upper, reversed(ordered))):
            for point in sequence:
                while len(chain) >= 2 and _cross(chain[-2], chain[-1], point) <= 0:
                    chain.pop()
                chain.append(point)
        hull = lower[:-1] + upper[:-1]
    return [Vertex(point, {representatives[point]: F(1)}) for point in hull]


def _blend(first, second, proportion):
    payoff = tuple((1 - proportion) * a + proportion * b
                   for a, b in zip(first.payoff, second.payoff))
    ids = first.coefficients.keys() | second.coefficients.keys()
    coefficients = {i: (1 - proportion) * first.coefficients.get(i, F(0)) +
                    proportion * second.coefficients.get(i, F(0)) for i in ids}
    return Vertex(payoff, {i: w for i, w in coefficients.items() if w > 0})


def _intersection(first, second, axis, threshold):
    proportion = (threshold - first.payoff[axis]) / (second.payoff[axis] - first.payoff[axis])
    return _blend(first, second, proportion)


def _unique_vertices(vertices):
    result = {}
    for vertex in vertices:
        result.setdefault(vertex.payoff, vertex)
    return list(result.values())


def _clip(vertices, axis, threshold):
    """Clip a polygon, segment or point against one closed half-plane."""
    if not vertices:
        return []
    if len(vertices) == 1:
        return vertices if vertices[0].payoff[axis] >= threshold else []
    if len(vertices) == 2:
        first, second = vertices
        first_in = first.payoff[axis] >= threshold
        second_in = second.payoff[axis] >= threshold
        if first_in and second_in:
            return vertices
        if not first_in and not second_in:
            return []
        crossing = _intersection(first, second, axis, threshold)
        return _unique_vertices([first, crossing] if first_in else [crossing, second])
    result = []
    previous = vertices[-1]
    for current in vertices:
        previous_in = previous.payoff[axis] >= threshold
        current_in = current.payoff[axis] >= threshold
        if previous_in != current_in:
            result.append(_intersection(previous, current, axis, threshold))
        if current_in:
            result.append(current)
        previous = current
    return _unique_vertices(result)


def _affine_dependence(ids, points):
    """Find a rational null vector for columns (1,x,y) using elimination."""
    matrix = [[F(1) for _ in ids],
              [points[i][0] for i in ids], [points[i][1] for i in ids]]
    pivots = []
    row = 0
    for column in range(len(ids)):
        pivot = next((r for r in range(row, 3) if matrix[r][column]), None)
        if pivot is None:
            continue
        matrix[row], matrix[pivot] = matrix[pivot], matrix[row]
        divisor = matrix[row][column]
        matrix[row] = [value / divisor for value in matrix[row]]
        for other in range(3):
            if other != row:
                multiple = matrix[other][column]
                matrix[other] = [a - multiple * b for a, b in zip(matrix[other], matrix[row])]
        pivots.append(column)
        row += 1
        if row == 3:
            break
    free = next(column for column in range(len(ids)) if column not in pivots)
    vector = [F(0) for _ in ids]
    vector[free] = F(1)
    for row, column in enumerate(pivots):
        vector[column] = -matrix[row][free]
    return vector


def reduce_coefficients(coefficients, points):
    """Carathéodory reduction by affine dependence, without changing payoff."""
    weights = {i: F(w) for i, w in coefficients.items() if w > 0}
    while len(weights) > 3:
        ids = sorted(weights)[:4]
        direction = _affine_dependence(ids, points)
        step = min(weights[i] / d for i, d in zip(ids, direction) if d > 0)
        for i, d in zip(ids, direction):
            weights[i] -= step * d
        weights = {i: w for i, w in weights.items() if w > 0}
    return weights


def _witness(vertex, points):
    coefficients = reduce_coefficients(vertex.coefficients, points)
    return {"parts": [{"policy_id": i, "weight": wire(w)}
                      for i, w in sorted(coefficients.items())],
            "payoff": [wire(q) for q in vertex.payoff]}


def classify(points, candidate_id):
    """Decide both domination definitions over the entire exact payoff hull."""
    candidate = points[candidate_id]
    feasible = convex_hull(points)
    for axis in range(2):
        feasible = _clip(feasible, axis, candidate[axis])
    strong = next((v for v in feasible if any(a > b for a, b in
                                             zip(v.payoff, candidate))), None)
    weak = next((v for v in feasible if all(a > b for a, b in
                                           zip(v.payoff, candidate))), None)
    if weak is None:
        for first, second in itertools.combinations(feasible, 2):
            midpoint = _blend(first, second, F(1, 2))
            if all(a > b for a, b in zip(midpoint.payoff, candidate)):
                weak = midpoint
                break
    return {label: {"efficient": vertex is None,
                    "witness": None if vertex is None else _witness(vertex, points)}
            for label, vertex in (("paper_weak", weak), ("strong", strong))}


def build_pareto():
    points = cake_payoffs()
    rows = [{"policy_id": i, "payoff": [wire(q) for q in point], **classify(points, i)}
            for i, point in enumerate(points)]
    return {"schema": SCHEMA, "settings": settings("pareto"), "rows": rows}


def _priority_inputs(model, prior, history):
    if not isinstance(model, (tuple, list)) or len(model) != 2:
        raise ValueError("priority model must contain two likelihoods")
    for name, value in (("prior", prior), ("Alice likelihood", model[0]),
                        ("Bob likelihood", model[1])):
        if not isinstance(value, (F, int)) or isinstance(value, bool) or not 0 <= value <= 1:
            raise ValueError(f"priority {name} must be a rational probability")
    if not isinstance(history, (tuple, list)) or len(history) > HORIZON:
        raise ValueError(f"priority history must contain at most {HORIZON} observations")
    if any(type(observation) is not int or observation not in (0, 1) for observation in history):
        raise ValueError("priority history observations must be 0 red or 1 green")


def direct_priority(model, prior, history):
    """Recompute each prefix from initial weights and likelihood products."""
    _priority_inputs(model, prior, history)
    masses = (F(prior), 1 - F(prior))
    for prefix_length in range(1, len(history) + 1):
        masses = tuple(initial * prod(
            (red if observation == 0 else 1 - red for observation in history[:prefix_length]),
            start=F(1)) for initial, red in zip((F(prior), 1 - F(prior)), model))
        if sum(masses) == 0:
            return {"status": "impossible", "first_impossible_prefix": prefix_length}
    total = sum(masses)
    return {"status": "supported", "weights": [wire(mass / total) for mass in masses]}


def sequential_priority(model, prior, history):
    """Separate normalized stepwise update for exhaustive cross-checking."""
    _priority_inputs(model, prior, history)
    weights = [F(prior), 1 - F(prior)]
    for prefix_length, observation in enumerate(history, 1):
        likelihoods = model if observation == 0 else [1 - q for q in model]
        masses = [w * q for w, q in zip(weights, likelihoods)]
        total = masses[0] + masses[1]
        if total == 0:
            return {"status": "impossible", "first_impossible_prefix": prefix_length}
        weights = [mass / total for mass in masses]
    return {"status": "supported", "weights": [wire(q) for q in weights]}


def history_inputs():
    for model_id in range(len(MODELS)):
        for prior_index in range(len(PRIORS)):
            for length in range(HORIZON + 1):
                for history in itertools.product((0, 1), repeat=length):
                    yield model_id, prior_index, history


def build_priority():
    rows = []
    for model_id, prior_index, history in history_inputs():
        result = direct_priority(MODELS[model_id], PRIORS[prior_index], history)
        if result != sequential_priority(MODELS[model_id], PRIORS[prior_index], history):
            raise ValueError(f"priority derivations disagree: {model_id}, {prior_index}, {history}")
        rows.append({"model_id": model_id, "prior_index": prior_index,
                     "history": list(history), "result": result})
    return {"schema": SCHEMA, "settings": settings("priority"), "rows": rows}


def canonical_bytes(record):
    try:
        return (json.dumps(record, sort_keys=True, separators=(",", ":"),
                           ensure_ascii=False, allow_nan=False) + "\n").encode("utf-8")
    except (ValueError, TypeError, UnicodeError, RecursionError) as error:
        raise ValueError(f"cannot encode canonical reference: {error}") from error


def _keys(value, expected, context):
    if type(value) is not dict or set(value) != set(expected):
        raise ValueError(f"{context}: keys must be exactly {sorted(expected)}")


def read_fraction(value, context):
    _keys(value, ("num", "den"), f"{context} fraction")
    numerator, denominator = value["num"], value["den"]
    if (type(numerator) is not str or type(denominator) is not str or
            not re.fullmatch(r"(?:0|-[1-9][0-9]*|[1-9][0-9]*)", numerator) or
            not re.fullmatch(r"[1-9][0-9]*", denominator) or
            len(numerator) > 40 or len(denominator) > 39):
        raise ValueError(f"{context}: fraction requires canonical decimal strings")
    num, den = int(numerator), int(denominator)
    if not -(2**127) <= num < 2**127 or not den < 2**127:
        raise ValueError(f"{context}: fraction is outside i128 bounds")
    result = F(num, den)
    if wire(result) != value:
        raise ValueError(f"{context}: fraction must be reduced")
    return result


def _pair(value, context):
    if type(value) is not list or len(value) != 2:
        raise ValueError(f"{context}: expected two fractions")
    return tuple(read_fraction(q, f"{context}[{i}]") for i, q in enumerate(value))


def _integer(value, low, high, context):
    if type(value) is not int or not low <= value <= high:
        raise ValueError(f"{context}: expected integer in [{low},{high}]")
    return value


def _ids(value, context):
    if type(value) is not list or not value:
        raise ValueError(f"{context}: expected nonempty policy ID list")
    ids = [_integer(i, 0, 8, context) for i in value]
    if ids != sorted(set(ids)):
        raise ValueError(f"{context}: policy IDs must be sorted and unique")
    return ids


def validate_witness(witness, points, candidate, definition):
    context = f"{definition} witness"
    _keys(witness, ("parts", "payoff"), context)
    parts = witness["parts"]
    if type(parts) is not list or not 1 <= len(parts) <= 3:
        raise ValueError(f"{context}: must contain one to three parts")
    coefficients = {}
    previous = -1
    for part in parts:
        _keys(part, ("policy_id", "weight"), context)
        policy_id = _integer(part["policy_id"], 0, len(points) - 1, context)
        if policy_id <= previous:
            raise ValueError(f"{context}: policy IDs must be ordered and unique")
        previous = policy_id
        weight = read_fraction(part["weight"], f"{context} weight")
        if weight < 0:
            raise ValueError(f"{context}: negative weight")
        coefficients[policy_id] = weight
    if sum(coefficients.values()) != 1:
        raise ValueError(f"{context}: weights must sum to one")
    payoff = _pair(witness["payoff"], context)
    recomputed = tuple(sum(weight * points[i][axis] for i, weight in coefficients.items())
                       for axis in range(2))
    if payoff != recomputed:
        raise ValueError(f"{context}: payoff does not equal its mixture")
    improves = [a > b for a, b in zip(payoff, candidate)]
    if (any(a < b for a, b in zip(payoff, candidate)) or
            not (all(improves) if definition == "paper_weak" else any(improves))):
        raise ValueError(f"{context}: payoff does not satisfy domination")


def validate_reference(record, kind):
    """Check settings/census first, then every input, exact value and witness."""
    expected_settings = settings(kind)
    _keys(record, ("schema", "settings", "rows"), f"{kind} reference")
    if record["schema"] != SCHEMA:
        raise ValueError(f"{kind}: stale schema")
    if canonical_bytes(record["settings"]) != canonical_bytes(expected_settings):
        raise ValueError(f"{kind}: stale or malformed settings")
    rows = record["rows"]
    if type(rows) is not list or len(rows) != ROW_COUNTS[kind]:
        raise ValueError(f"{kind}: row count must be exactly {ROW_COUNTS[kind]}")
    seen = set()
    payoffs = cake_payoffs()
    expected_cake = build_cake()["rows"] if kind == "cake" else None
    expected_histories = list(history_inputs()) if kind == "priority" else None
    for index, row in enumerate(rows):
        context = f"{kind} row {index}"
        if kind == "cake":
            _keys(row, ("weight", "policy_id", "expected", "objective",
                        "global_maximizers", "local_maximizers"), context)
            weight = read_fraction(row["weight"], context)
            policy_id = _integer(row["policy_id"], 0, 8, context)
            key = (weight, policy_id)
            _pair(row["expected"], f"{context} expected")
            read_fraction(row["objective"], f"{context} objective")
            _ids(row["global_maximizers"], context)
            _ids(row["local_maximizers"], context)
            wanted_key = (WEIGHTS[index // 9], index % 9)
        elif kind == "pareto":
            _keys(row, ("policy_id", "payoff", "paper_weak", "strong"), context)
            policy_id = _integer(row["policy_id"], 0, 8, context)
            key = policy_id
            wanted_key = index
            if _pair(row["payoff"], context) != payoffs[policy_id]:
                raise ValueError(f"{context}: wrong payoff")
            classification = classify(payoffs, policy_id)
            for definition in ("paper_weak", "strong"):
                value = row[definition]
                _keys(value, ("efficient", "witness"), f"{context} {definition}")
                if type(value["efficient"]) is not bool:
                    raise ValueError(f"{context}: efficiency must be boolean")
                if value["efficient"] != classification[definition]["efficient"]:
                    raise ValueError(f"{context}: wrong {definition} classification")
                if value["efficient"]:
                    if value["witness"] is not None:
                        raise ValueError(f"{context}: efficient point cannot have witness")
                else:
                    validate_witness(value["witness"], payoffs, payoffs[policy_id], definition)
        else:
            _keys(row, ("model_id", "prior_index", "history", "result"), context)
            model_id = _integer(row["model_id"], 0, 2, context)
            prior_index = _integer(row["prior_index"], 0, 2, context)
            history = row["history"]
            if type(history) is not list:
                raise ValueError(f"{context}: history must be an array")
            _priority_inputs(MODELS[model_id], PRIORS[prior_index], history)
            key = (model_id, prior_index, tuple(history))
            wanted_key = expected_histories[index]
            result = row["result"]
            if type(result) is not dict:
                raise ValueError(f"{context}: result must be an object")
            if result.get("status") == "supported":
                _keys(result, ("status", "weights"), context)
                weights = _pair(result["weights"], f"{context} weights")
                if any(q < 0 or q > 1 for q in weights) or sum(weights) != 1:
                    raise ValueError(f"{context}: invalid normalized weights")
            elif result.get("status") == "impossible":
                _keys(result, ("status", "first_impossible_prefix"), context)
                _integer(result["first_impossible_prefix"], 1, len(history), context)
            else:
                raise ValueError(f"{context}: unknown support status")
            expected = direct_priority(MODELS[model_id], PRIORS[prior_index], history)
            if result != expected:
                raise ValueError(f"{context}: wrong priority result")
        if key in seen:
            raise ValueError(f"{context}: duplicate input row {key!r}")
        seen.add(key)
        if key != wanted_key:
            raise ValueError(f"{context}: input order or registered domain mismatch")
        if kind == "cake" and row != expected_cake[index]:
            raise ValueError(f"{context}: wrong expectations, objective or maximizers")


def decode_reference(data: bytes, kind: str) -> dict:
    if type(data) is not bytes or len(data) > MAX_BYTES:
        raise ValueError(f"{kind}: reference bytes exceed 8 MiB or are not bytes")

    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError(f"{kind}: duplicate JSON key {key!r}")
            result[key] = value
        return result

    def constant(value):
        raise ValueError(f"{kind}: invalid JSON constant {value}")

    try:
        record = json.loads(data.decode("utf-8"), object_pairs_hook=pairs, parse_constant=constant)
    except (UnicodeError, json.JSONDecodeError, RecursionError) as error:
        raise ValueError(f"{kind}: invalid reference JSON: {error}") from error
    validate_reference(record, kind)
    return record


def write_exclusive(record, path):
    if type(record) is not dict or type(record.get("settings")) is not dict:
        raise ValueError("reference settings required before writing")
    kind = record["settings"].get("reference_kind")
    data = canonical_bytes(record)
    if len(data) > MAX_BYTES:
        raise ValueError(f"{kind}: output exceeds 8 MiB")
    validate_reference(record, kind)
    with open(path, "xb") as output:
        output.write(data)


def generate_and_seal(fixture_directory, seal_path):
    """Create-only fixture set and an ignored, source-bound engineering seal."""
    directory = Path(fixture_directory)
    builders = (("cake", build_cake), ("pareto", build_pareto), ("priority", build_priority))
    records = [(kind, build()) for kind, build in builders]
    total = sum(len(canonical_bytes(record)) for _, record in records)
    if total > TOTAL_MAX_BYTES:
        raise ValueError("reference set exceeds 16 MiB")
    paths = [directory / f"{kind}-reference.json" for kind, _ in records]
    seal_path = Path(seal_path)
    if any(path.exists() for path in [*paths, seal_path]):
        raise FileExistsError("fixture or seal exists; reviewed references cannot be overwritten")
    directory.mkdir(parents=True, exist_ok=True)
    fixtures = {}
    for (kind, record), path in zip(records, paths):
        write_exclusive(record, path)
        data = path.read_bytes()
        decoded = decode_reference(data, kind)
        if decoded != record or data != canonical_bytes(decoded):
            raise ValueError(f"{kind}: fixture readback differs")
        fixtures[kind] = {"path": str(path), "sha256": hashlib.sha256(data).hexdigest(),
                          "bytes": len(data), "row_count": len(decoded["rows"]),
                          "settings_sha256": hashlib.sha256(canonical_bytes(decoded["settings"])).hexdigest()}
    generator_files = [Path(__file__).with_name(name)
                       for name in ("__init__.py", "reference.py", "test_reference.py")]
    seal = {"schema": SCHEMA, "source_hashes": dict(SOURCE_HASHES), "fixtures": fixtures,
            "total_bytes": total, "generator": {
                str(path): hashlib.sha256(path.read_bytes()).hexdigest() for path in generator_files},
            "derived_boundaries": [wire(q) for q in derive_boundaries()]}
    with seal_path.open("xb") as output:
        output.write(canonical_bytes(seal))
    return seal


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--fixtures", type=Path, required=True)
    parser.add_argument("--seal", type=Path, required=True)
    arguments = parser.parse_args()
    print(json.dumps(generate_and_seal(arguments.fixtures, arguments.seal), indent=2, sort_keys=True))
