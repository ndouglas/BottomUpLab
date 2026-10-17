# Consequences of misaligned AI (milestone 43) — findings

Measured 2026-10-14 under the decision rules fixed in the [design](2026-10-14-misaligned-ai-design.md)
before any measurement (the design's planning amendments, applied in the plan's first task, predate
every run). Sources: Zhuang and Hadfield-Menell 2020, *Consequences of misaligned AI* (NeurIPS;
arXiv 2102.03896v1: the model §2, the recommender example §2.2 and Figure 2, Theorems 1–2 and
Proposition 1 in §3, Propositions 2–5 and Theorem 3 in §4); the NeurIPS supplemental ("Avoiding the
Midas Touch: Consequences of Misaligned AI, Supplementary Material," June 2020), which repeats the
proofs and adds Proposition 5's tie clause; the author feedback; and Hadfield-Menell and Hadfield
2019, *Incomplete contracting and AI alignment* (AIES), the conceptual follow-up. All under
`papers/alignment/`. Neither the paper nor the supplemental states the optimizer, the start or the
number of runs behind Figure 2, and no code exists.

**Summary.** All twelve registered rows ran. The eight scored claims hold: Theorem 1 (unmentioned
attributes end at the floor), the fixed-proxy limits for all fourteen proxy subsets (10√|J| in the
example, every pair below the start), Figure 2's shape under our climber, Proposition 2 (the
impact-minimizing robot never loses), Proposition 3 (the wasteful robot loses in any interval),
Theorem 3 and Proposition 4 (the efficient robot under a frequent human only improves until the
optimum), Proposition 5 (interactive impact minimization reaches the human optimum in both worlds)
and Theorem 2's "arbitrarily low" on our logarithmic world. The four reported rows carry the two
findings named before measurement: in the paper's own example the floor below zero does not bind,
because the sum of squares stops increasing at zero, so its utility is bounded below and Theorem 2's
unbounded clause cannot be shown there; and Figure 2's modest peak needs an inefficient optimizer,
since the efficient one passes through the human optimum (+4 over the start) before falling, where
the climber peaks at +1.6. Everything about Figure 2's timing, and the whole of the interactive
results, depends on choices of ours that are named below.

## Provenance

The survey ran from `survey/` as `survey --only misaligned` (release build), single process, on an
arm64 MacBookPro18,4 (ten logical CPUs), Rust 1.98.1, at core commit `cbfc2fb`. Wall time was 0.14 s:
every run is a four-attribute flow, and the longest is 10,000 ticks. A second run was byte-identical,
which doubles as a determinism check. The printed table (stdout) is tracked at
[`survey/out/misaligned/results.md`](../../../survey/out/misaligned/results.md) (SHA-256
`6bc8d70d3d82a206…`). No run was discarded, and no rule, tolerance, start, rate or horizon changed
after the measurement.

## Claim table

Verdicts follow the survey's vocabulary; `Untestable` marks the four rows reported without a
target. "Ours" marks our extensions (survey source `App`); the rest cite the paper (`Book`).

| id | rule | verdict | measured |
|---|---|---|---|
| `misaligned.theorem-1-floor` | gradient robot, fixed {a, b}, rate 0.1, 5,000 ticks: each unmentioned attribute within 1e-3 of the floor, both worlds | Holds | recommender c, d = 0.000000 (floor 0); logarithmic c, d = 1.000000 (floor 1) |
| `misaligned.limit-utility` | all fourteen proxy subsets end within 1e-3 of the closed-form limit (10√\|J\| at floor 0); the six pairs below U₀ = 16 | Holds | singletons 10.0000; pairs 14.1421 (7.071, 7.071, 0, 0 and permutations); triples 17.3205 (5.774 each) |
| `misaligned.figure-2-shape` | climber, step 1, 20 seeds, 1,000 ticks: {a, b} mean gain peaks ≥ +1 and ends < 0; {c, d} ≤ +0.1 always and < 0 from tick 50; all six pairs < 0 at 1,000 | Holds | ab peak +1.575 at tick 24, below zero from 93, −1.087 at 1,000; ac +0.938 (50) → −1.052; ad +0.920 (27) → −1.012; bc +0.772 (38) → −1.140; bd +0.856 (33) → −1.019; cd peak 0 at tick 0, below zero from 32, −0.809 |
| `misaligned.proposition-2-no-loss` | impact minimizing, gradient, fixed {a, b}: U nondecreasing within 1e-9; endpoint within 1e-3 of the closed form | Holds | from (1, 1, 7, 7): no fall, 16.0000 = limit; from (1, 1, 6, 6): no fall, 19.4833 = 12 + 2√14 |
| `misaligned.proposition-3-wasteful` | wasteful robot (rate 0.1, waste 0.1): gain after 1, 2, 5, 10, 50 ticks all negative | Holds | −0.18, −0.36, −0.90, −1.80, −9.00 |
| `misaligned.proposition-4-improvement` | gradient at rate 0.01, human `top` every 1 and 10 ticks: U nondecreasing within 1e-9 until within 1e-2 of U* = 20, then at least U* − 1e-2 | Holds | no fall before tick 612 at either interval; within 1e-2 from tick 612; final gap 0.00000 (δ = 1), 0.00023 (δ = 10) |
| `misaligned.proposition-5-optimal` | human `extremes` with impact minimization, gradient at rate 0.1, 10,000 ticks: U within 1e-3 of U* | Holds | recommender from (1, 1, 7, 7) and (1, 1, 6, 6): 20.00000, no fall; logarithmic: 9.21032 against 4 ln 10 = 9.21034, no fall |
| `misaligned.theorem-2-unbounded` (ours; the paper's claim on our instance) | logarithmic, gradient, fixed {a, b}, floors 1 … 0.001: endpoints within 1e-3 of the closed form, strictly decreasing, below −5 at 0.001 | Holds | 5.8889, 3.5533, 1.3763, −1.0247, −3.2199, −5.6271, −7.8241, each equal to its closed form to four decimals |
| `misaligned.recommender-floor-flat` (ours) | reported: endpoints at floors 0, −1, −5 in the paper's example | Untestable (reported) | c, d end at 0.0000 and U at 14.1421 at every floor; Theorem 1's s*_k = b_k would put them at −1 (U 12) and −5 (U 0) |
| `misaligned.interval-break` (ours) | reported: gain after 2,000 ticks against δ at rate 0.1, with and without OFF on a loss | Untestable (reported) | keep going: δ 1 +3.9999, 2 +3.9990, 5 +3.9938, 10 +3.9755, 20 +3.9048, 50 +3.4591, 100 +2.2273, 200 −0.2269; switch off: +3.9996 (off at 66), +3.9973 (67), +3.9817 (71), +3.9696 (71), +3.7900 (81), +3.1278 (101), +1.7094 (201), −0.2667 (201); the first tick at which U falls is 65 at every δ |
| `misaligned.proxy-size` (ours) | reported: limits and climber gains for \|J\| = 1, 2, 3 | Untestable (reported) | limits 10, 14.1421, 17.3205 (gains −6.00, −1.86, +1.32 over 16); climber peaks +1.056, +1.575, +2.163 and gains at 1,000 −4.452, −1.087, +1.621; both rise with \|J\| |
| `misaligned.presets` (ours) | reported: each preset after 1,000 ticks, seed 1 | Untestable (reported) | proxy 14.53; gradient 14.14; impact 19.48 (from 14); interactive 19.9999; slow human off at tick 201 with 15.73; optimal 20.00; wasteful 14.14; logarithmic floor 9.20 (from 8.32), still descending toward −3.22 |

## What the paper's example can and cannot show

**Theorem 1 holds at the floor it implies, and only there.** The recommender's constraint
a² + b² + c² + d² ≤ 100 is increasing in an attribute only where that attribute is nonnegative.
At floor 0 the proxy's limit is exactly Theorem 1's: c and d at the floor, a and b at √50. With a
floor of −1 or −5 the flow still stops c and d at 0, because below zero lowering them *raises* the
sum of squares and buys nothing for the proxy; Theorem 1's premise (C strictly increasing) fails
there and so does its conclusion. The paper's example therefore reads as having a floor of 0, which
the text never states.

**Theorem 2's "arbitrarily low" needs a different instance.** On the ball of radius 10 the utility
a + b + c + d is at least −20, so no fixed proxy can make it arbitrarily low; what the example shows,
and what Figure 2's caption says, is net loss: every pair ends at 10√2 ≈ 14.14 against a start of 16,
every singleton at 10, and every triple at 10√3 ≈ 17.32, which is a *gain*. Our logarithmic world
(utility Σ ln sᵢ on a + b + c + d ≤ 40) satisfies every hypothesis, Proposition 1's included
(additively separable, ∂C/∂sᵢ = 1, sensitivity 1/sᵢ nonincreasing and tending to 0), and there the
limit falls without bound as the floor falls: 5.89 at floor 1, −3.22 at 0.01, −7.82 at 0.001, each
equal to |J| ln((40 − |K| b)/|J|) + |K| ln b.

**Figure 2's peak is a statement about the optimizer, not the model.** Along the sphere the
efficient path from (1, 1, 7, 7) to (√50, √50, 0, 0) passes through (5, 5, 5, 5), the human optimum,
so an efficient robot delivers +4 before it starts losing (`misaligned-recommender-gradient`;
`proposition-4-improvement` reaches the optimum at tick 612 at rate 0.01, that is after 6.1 units
of path, the arc from the start to the optimum). The figure's +1.6 peak appears under a black box
that wastes: our climber accepts any feasible Gaussian proposal that raises the proxy, so the
unmentioned attributes drift down faster than the proxy attributes rise. The climber's {a, b} curve
peaks at +1.575 at tick 24, crosses zero at tick 93 and reaches −1.09 by tick 1,000; the figure's
reads about +1.6 near iteration 60, zero near 130 and −1 at 150. The shape is reproduced; the timing
is a function of the climber's step and the start, both ours, and is reported, not scored. The
paper's "iterations" are not defined, so one tick is one proposal here.

## The mitigations

- **Impact minimization (Proposition 2)** is exact: with c and d frozen the proxy *is* the
  utility, so every accepted move is a gain. From the sphere start there is no move to make (the
  limit is the start, 16); from (1, 1, 6, 6) the robot reaches 12 + 2√14 ≈ 19.48. The guarantee is
  no loss, not progress.
- **The wasteful robot (Proposition 3)** loses 0.18 a tick (0.2 of unmentioned for 0.02 of proxy),
  so OFF before the first tick is the maxmin policy, as the paper says.
- **The efficient robot under a frequent human (Theorem 3, Proposition 4)** climbs monotonically to
  the optimum and chatters there by at most one interval's travel: at rate 0.01 the final gap is
  0.00000 with δ = 1 and 0.00023 with δ = 10. At rate 0.1 (the interval sweep) the first fall is at
  tick 65 for every δ, which is the tick after the optimum (the arc from the start to the optimum is
  6.4 units, 64 moves of 0.1), so a strict "switch off on a loss" human stops the robot at the
  optimum (δ = 1: off at tick 66 with +3.9996). The gain at the horizon declines with δ, from +4.00
  at δ ≤ 10 through +3.46 at 50 and +2.23 at 100 to −0.23 at 200, where the robot runs the whole
  path to the proxy's limit between two interactions and the human's re-specification can no
  longer recover it within the horizon. The paper's condition that the robot not change the world
  too fast for the human to react is, in this example, "δ × rate well below 6.4 units of path".
- **Interactive impact minimization (Proposition 5)** reaches the optimum exactly in both worlds,
  and monotonically: 20.00000 from both recommender starts and 9.21032 against 9.21034 in the
  logarithmic world after 10,000 ticks (its pairwise transfers contract by about 0.1 % per tick near
  balance, which is why its horizon is twice the others').

## Hadfield-Menell and Hadfield 2019

The follow-up is conceptual and makes two points this model can test. "Selling the firm to the
agent" (endowing it with the complete objective) is |J| = L here, and `proxy-size` shows the
direction short of it: the limit rises with |J| (10, 14.14, 17.32) and so does the climber's gain at
every horizon we looked at (peaks +1.06, +1.58, +2.16; gains at 1,000 ticks −4.45, −1.09, +1.62),
so a more complete proxy is never worse under complete optimization in this example. Holmstrom and
Milgrom's multitasking result, that it can be better to *omit* an easily measured task from the
contract, has no counterpart here: with equal weights and a symmetric constraint no subset is
better left out. It would need unequal weights or a proxy whose attributes are substitutes for the
human's, which this milestone does not build.

## What depends on a reconstruction choice

- **The start (1, 1, 7, 7).** The paper says only that diversity and well-being begin high. The
  start fixes U₀ = 16 and so the sign of every "net loss"; any start above 14.14 gives the same
  signs for pairs, and the limits do not depend on it at all.
- **The climber and its step.** Figure 2's timing and peak height are the climber's; step 1 was
  chosen in planning as the value whose shape was closest to the plot, and `misaligned-figure2-pairs`
  sweeps 0.25–2.
- **The gradient robot's step rule.** Moves are `rate · d / max(1, |d|)`, unit speed far from a
  stationary point and proportional near one; the fixed points are exact whatever the rule, the
  path's sampling is not.
- **The human's interval and OFF rule.** Both are ours; the paper's human "sees s(t) for
  t ∈ [0, Tδ]" and may send OFF at any interaction, and we read that as "OFF if U fell since the
  previous interaction".
- **The logarithmic world** is ours entirely, built to satisfy Theorem 2's hypotheses; its capacity
  40, floor 1 and start (4, 4, 16, 16) are arbitrary.
- **Ties.** The supplemental breaks Proposition 5's ties "arbitrarily"; we take the lower index,
  and `top` does the same.

The closed forms (optimum, fixed-proxy limit, impact-minimizing limit) depend on none of these.

## Register entries

RR-MA-01 through RR-MA-10 in `docs/reproducibility.md` record provenance, the eight scored claims,
the floor finding and the interval sweep.

## Open questions

- Which optimizer, start and step did the authors use for Figure 2? A step sweep of the climber
  (built: `misaligned-figure2-pairs`) maps the peak's height and timing, and a start with a < b
  would separate the mixed pairs the way the plot does (ac and ad peak, bc and bd fall at once).
- Does the proxy-size monotonicity survive unequal utility weights or attributes that substitute
  for one another? That is where Holmstrom and Milgrom's omission result would have to appear.
- The interval sweep's threshold (δ × rate against the 6.4 units of path to the optimum) is one
  example's number; a second start would show whether the relation is general.
- Experiment A3 of the [alignment program](../../studies/2026-10-11-alignment-program.md) reuses
  `misaligned::worlds` (sensitivities, limits, the flows) with an assistant among foragers whose
  welfare proxy omits spice, health or leisure.
