"""Exact rational bounds around correctly rounded stdlib Decimal functions.

Decimal.exp/ln/sqrt use nearest-half-even, regardless of the context's
rounding direction. Each evaluated endpoint receives its own half-ULP envelope.
All arithmetic after those evaluations uses Fraction, avoiding hidden rounding.
"""

from dataclasses import dataclass
from decimal import (
    Decimal,
    localcontext,
    ROUND_FLOOR,
    ROUND_CEILING,
    DecimalException,
    Subnormal,
    Underflow,
    Clamped,
)
from fractions import Fraction
import math
import re
import struct

from .contracts import Json


@dataclass(frozen=True)
class Interval:
    lo: Fraction
    hi: Fraction

    def __post_init__(self):
        if (
            not isinstance(self.lo, Fraction)
            or not isinstance(self.hi, Fraction)
            or self.lo > self.hi
        ):
            raise ValueError("ordered rational interval required")

    def wire(self, scale: int | None = None) -> Json:
        lower, upper = self.lo, self.hi
        if scale is not None:
            lower, upper = enclose(lower, scale).lo, enclose(upper, scale).hi

        def endpoint(value):
            # Avoid Python's global integer-string limit and bound before conversion.
            if abs(value.numerator) >= 10**4096 or value.denominator >= 10**4096:
                raise ValueError("rational endpoint exceeds 4096 digits")
            return {
                "numerator": str(value.numerator),
                "denominator": str(value.denominator),
            }

        return {"lo": endpoint(lower), "hi": endpoint(upper)}


def enclose(value: Fraction, scale: int = 10**12) -> Interval:
    if type(scale) is not int or scale <= 0:
        raise ValueError("positive integer scale required")
    scaled = value * scale
    lower = scaled.numerator // scaled.denominator
    upper = -((-scaled.numerator) // scaled.denominator)
    return Interval(Fraction(lower, scale), Fraction(upper, scale))


def _decimal_bound(value: Fraction, precision: int, rounding: str) -> Decimal:
    with localcontext() as context:
        context.prec = precision
        context.rounding = rounding
        return Decimal(value.numerator) / Decimal(value.denominator)


def _rounded_function(value: Decimal, name: str, precision: int) -> Interval:
    with localcontext() as context:
        context.prec = precision
        context.clear_flags()
        try:
            result = getattr(value, name)()
        except DecimalException as error:
            raise ValueError("unresolved Decimal operation") from error
        if any(context.flags[flag] for flag in (Subnormal, Underflow, Clamped)):
            raise ValueError("unresolved Decimal underflow/overflow")
        if not result.is_finite() or (result == 0 and name == "exp"):
            raise ValueError("unresolved Decimal underflow/overflow")
        error = Fraction(10) ** (result.adjusted() - precision + 1) / 2
        return Interval(Fraction(result) - error, Fraction(result) + error)


def _function_interval(value: Fraction, name: str, precision: int) -> Interval:
    if type(precision) is not int or not 100 <= precision <= 400:
        raise ValueError("precision must be between 100 and 400")
    lo = _decimal_bound(value, precision, ROUND_FLOOR)
    hi = _decimal_bound(value, precision, ROUND_CEILING)
    lower = _rounded_function(lo, name, precision).lo
    upper = _rounded_function(hi, name, precision).hi
    return Interval(lower, upper)


def exp_interval(x: Fraction, precision: int = 100) -> Interval:
    return _function_interval(x, "exp", precision)


def confidence_radius(slots: int, n: int, alpha: Fraction) -> Interval:
    if (
        type(slots) is not int
        or slots <= 0
        or type(n) is not int
        or n <= 0
        or not 0 < alpha < 1
    ):
        raise ValueError("positive slots/n and alpha in (0,1) required")
    logarithm = _function_interval(Fraction(2 * slots) / alpha, "ln", 100)
    lower = _function_interval(logarithm.lo / (2 * n), "sqrt", 100).lo
    upper = _function_interval(logarithm.hi / (2 * n), "sqrt", 100).hi
    return Interval(lower, upper)


def binary64(bits: str) -> Fraction:
    if not isinstance(bits, str) or re.fullmatch(r"[0-9a-f]{16}", bits) is None:
        raise ValueError("binary64 bits must be sixteen lowercase hex digits")
    value = struct.unpack(">d", bytes.fromhex(bits))[0]
    if not math.isfinite(value):
        raise ValueError("nonfinite binary64 input")
    return Fraction.from_float(value)


def quantized_numerator(p: Fraction) -> int:
    if not 0 <= p <= 1:
        raise ValueError("probability outside [0,1]")
    scaled = p * 2**53
    return -(-scaled.numerator // scaled.denominator)


def probability_certificate(row: Json) -> Json:
    expected = {
        "config",
        "blue",
        "red",
        "target_index",
        "hazard_bits",
        "dose_bits",
        "p_bits",
        "q_numerator",
    }
    if not isinstance(row, dict) or set(row) != expected:
        raise ValueError("malformed numerical target")
    config = row["config"]
    if not isinstance(config, dict) or set(config) != {
        "blue",
        "red",
        "blue_rate",
        "red_rate",
        "dt",
        "max_steps",
        "geometry",
    }:
        raise ValueError("malformed config")
    blue, red, index = row["blue"], row["red"], row["target_index"]
    if (
        any(type(x) is not int for x in (blue, red, index))
        or (blue, red) not in ((1, 1), (2, 1))
        or index not in (0, 1)
    ):
        raise ValueError("target outside registered tiny states")
    if any(
        type(config[x]) not in (int, float, Decimal)
        for x in ("blue_rate", "red_rate", "dt")
    ):
        raise ValueError("invalid config number")
    try:
        rates = [
            Fraction.from_float(float(config[x])) for x in ("blue_rate", "red_rate")
        ]
        dt = Fraction.from_float(float(config["dt"]))
    except (ValueError, OverflowError) as error:
        raise ValueError("nonfinite config number") from error
    if rates != [1, 1] or dt not in (Fraction(1, 16), Fraction(1, 32), Fraction(1, 64)):
        raise ValueError("unregistered tiny rates/timestep")
    if (
        any(type(config[x]) is not int for x in ("blue", "red", "max_steps"))
        or config["blue"] not in (1, 2)
        or config["red"] != 1
        or config["max_steps"] * dt != 8
        or blue > config["blue"]
        or red > config["red"]
    ):
        raise ValueError("config differs from approved tiny descriptor")
    if config["geometry"] == "duel_contact":
        hazard = rates[1 - index]
    elif config["geometry"] == "aimed_fire":
        hazard = rates[1 - index] * [red, blue][index] / [blue, red][index]
    else:
        raise ValueError("unknown geometry")
    dose = hazard * dt
    if binary64(row["hazard_bits"]) != hazard or binary64(row["dose_bits"]) != dose:
        raise ValueError("native hazard/dose differs from exact dyadic arithmetic")
    p = binary64(row["p_bits"])
    q = quantized_numerator(p)
    if type(row["q_numerator"]) is not int or row["q_numerator"] != q:
        raise ValueError("native q differs from exact p threshold")
    for precision in (100, 200, 400):
        exponential = exp_interval(-dose, precision)
        ideal = Interval(1 - exponential.hi, 1 - exponential.lo)
        rounded = float(ideal.lo)
        if rounded == float(ideal.hi):
            center = Fraction.from_float(rounded)
            ulp = max(
                Fraction.from_float(math.nextafter(rounded, math.inf)) - center,
                center - Fraction.from_float(math.nextafter(rounded, -math.inf)),
            )
            error = Interval(p - ideal.hi, p - ideal.lo)
            if max(abs(error.lo), abs(error.hi)) > 4 * ulp:
                raise ValueError("native p fails complete four-ULP enclosure")
            return {
                **row,
                "exact_hazard": Interval(hazard, hazard).wire(),
                "exact_dose": Interval(dose, dose).wire(),
                "dose_error": Interval(Fraction(0), Fraction(0)).wire(),
                "p_star": ideal.wire(),
                "p_error": error.wire(),
                "ulp": Interval(ulp, ulp).wire(),
                "precision": precision,
                "rounded_p_star_bits": struct.pack(">d", rounded).hex(),
                "quantization_error": Interval(
                    Fraction(q, 2**53) - p, Fraction(q, 2**53) - p
                ).wire(),
            }
    raise ValueError("p-star binary64 rounding unresolved at 400 digits")


def difference(a: Interval, b: Interval) -> Interval:
    """Outward interval for left minus right; no independence assumption."""
    return Interval(a.lo - b.hi, a.hi - b.lo)
