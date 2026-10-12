# Assistance games (milestone 39) — findings

Measured 2026-10-11 under the decision rules fixed in the [design](2026-10-11-assistance-games-design.md)
before any measurement. Sources: Hadfield-Menell, Dragan, Abbeel & Russell 2016, *Cooperative inverse
reinforcement learning* (NeurIPS; the paperclip game, §3.3 and appendix §6), and the same authors'
2017 *The off-switch game* (IJCAI; §§3–5, Figures 3–4), both under `papers/alignment/`.

**Summary.** All 21 registered claims were run. All 12 exact claims hold: every paperclip-game
closed form (the expert threshold, R's best reply, the posteriors 1/4 and 1/6, the 41/92 and 51/92
teaching thresholds, Theorem 3 at θ = 0.49, the optimal joint policy at 6373/92) and every off-switch
theorem (Theorem 1, Corollary 1, Theorem 2's decomposition and condition), with our enumeration
of the pure mutual best responses. Figure 3's three direction claims also hold. All three Figure 4 direction claims hold under the preregistered
two-standard-error rule, **but only under our reconstruction of Figure 4's setup** (a Boltzmann human
at β = 0.5, a noisy report, Bayesian posterior mean). With the paper's rational human the value is flat
in σ. Three of our own claims are reported without a target: that flat rational case and the two
dynamics. The pooling
pair is not a pure mutual best response, but fictitious play started from it never leaves it.

## Provenance

The survey ran from `survey/` as `survey --only assistance` (release build), single process, on an
arm64 MacBookPro18,4 (ten logical CPUs), Rust 1.98.1, at core commit `e64cac3` with the claims as
committed in `25fa869`. Wall time was 2 min 36 s, nearly all of it Figure 4's 39 one-seed worlds
(100 ticks × 1,000 episodes each). The printed table is tracked at
[`survey/out/assistance/results.md`](../../../survey/out/assistance/results.md) (SHA-256
`0cecd217…0e4b`); the per-claim JSON `survey/out/results-assistance.json` is ignored, like every
`results-*.json`. No run was repeated or discarded, and no rule, grid, β or tolerance changed after
the measurement.

## Claim table

Verdicts follow the survey's vocabulary; `Untestable` marks the three claims reported without a
target. "Ours" marks our extensions (survey source `App`); the rest cite the papers (`Book`).

| id | rule | verdict | measured |
|---|---|---|---|
| `assistance.expert-threshold` | δ^E makes (0,2) below θ = 1/2 and (2,0) above (appendix eq. 2), exactly over 92 bins | Holds | thresholds (0.5, 0.5) |
| `assistance.robot-best-reply` | R's best reply to δ^E is (0,90), (50,50), (90,0) (appendix eq. 3), exactly | Holds | map [0, 1, 2] |
| `assistance.posterior-quarter` | after (0,2) under δ^E, R's posterior mean is 1/4, within 1e-9 | Holds | 0.24999999999999997 |
| `assistance.posterior-sixth` | under cuts at 1/3 and 2/3, after (0,2) R acts as θ = 1/6 (§3.3), within 1e-9 | Holds | 0.16666666666666666 |
| `assistance.teaching-thresholds` | H's best reply to br(δ^E) makes (1,1) on [41/92, 51/92] (proof of Theorem 3), exactly in rationals | Holds | pieces 0–41/92 → (0,2), 41/92–51/92 → (1,1), 51/92–1 → (2,0) |
| `assistance.theorem-3` | br(br(δ^E)) ≠ δ^E and H makes (1,1) at θ = 0.49 (Theorem 3) | Holds | action at 0.49 is (1,1); thresholds 0.4457, 0.5543 |
| `assistance.optimal-joint` | the exhaustive optimum is the 41/92, 51/92 rule with eq. 3's map | Holds | map [0, 1, 2], value 6373/92 ≈ 69.2717 against 69 for the expert pair |
| `assistance.pooling-not-equilibrium` (ours) | the pure mutual best responses, enumerated; the teaching pair is one and the pooling pair is not | Holds | six pairs, R's maps the six permutations of [0, 1, 2] |
| `assistance.theorem-1` | rational H: Δ ≥ 0 on 42 Gaussian priors, strictly where \|μ\|/σ ≤ 4; agreement with eq. 3 within 1e-9; two-point priors | Holds | smallest Δ 0 (below double precision at large \|μ\|/σ), smallest with \|μ\|/σ ≤ 4 3.573e-6; largest gap to eq. 3 2.6e-16; two-point priors as stated |
| `assistance.corollary-1` | point-mass belief: Δ ≤ 0, zero only at β = 0 (Corollary 1), at u ∈ {±0.1, ±1}, β ∈ {0, 0.5, 2} | Holds | all 12 points as stated |
| `assistance.theorem-2-agree` | eq. 7's decomposition equals the direct Δ within 1e-9 | Holds | largest gap 4.9e-15 over 80 points |
| `assistance.theorem-2-condition` | w(a) optimal iff \|μ\|Pr(C)/σ² < E[π̇] (eq. 8), at every point with \|Δ\| > 1e-9 | Holds | 60 of 64 points away from Δ = 0, all agree |
| `assistance.figure-3-gradient` | at μ ∈ {−1/4, 0, 1/4}: the grid minimum at (σ 0.05, β 4), the maximum at (σ 1, β 0), Δ nondecreasing in σ along every β row and nonincreasing in β along every σ column, no violation | Holds | μ = ±1/4: min −0.1209, max 0.2863; μ = 0: min 0.0002, max 0.3989; 0 adjacent-cell violations at each μ |
| `assistance.figure-3-zero-mean` | at μ = 0, Δ ≥ 0 and R waits in all 420 cells | Holds | smallest Δ 1.562e-4 |
| `assistance.figure-3-sign-regions` | where Δ ≤ 0 R switches off at μ < 0 and acts at μ > 0; both regions nonempty | Holds | μ = −1/4: 336 of 420 cells, all off; μ = 1/4: 335 of 420, all act |
| `assistance.figure-4-peak` | V peaks at the true posterior SD (1/√2, grid point 0.7): no σ beats it by more than 2 SE of the difference, for \|A\| ∈ {1, 4, 8} | Holds | V(0.7) = 0.3269 ± 0.0020, 0.7623 ± 0.0023, 1.0141 ± 0.0023; no violators |
| `assistance.figure-4-delta` | Δ rises with σ: no adjacent fall beyond 2 SE, for each \|A\| | Holds | no falls; Δ from −0.0977 to 0.3020 (\|A\| = 1), −0.1044 to 0.2454 (4), −0.1064 to 0.1728 (8) |
| `assistance.figure-4-actions` | the drop in V from the peak to σ = 1.5 does not shrink beyond 2 SE as \|A\| grows | Holds | 0.0222 ± 0.0028, 0.0339 ± 0.0032, 0.0494 ± 0.0032 |
| `assistance.figure-4-rational-flat` (ours) | reported: V at σ ∈ {0.1, 0.7, 1.5} with a rational human | Untestable (reported) | 0.3992, 0.3993, 0.3993; spread below 1e-4 against SE ≈ 0.0018 |
| `assistance.equilibria-iterated` (ours) | reported: the pair reached from four starts, 20 seeds, 200 ticks | Untestable (reported) | see below |
| `assistance.equilibria-learning` (ours) | reported: the same under fictitious play | Untestable (reported) | see below |

The full Figure 4 curves are in the results file. For |A| = 1, V rises from 0.2852 at σ = 0.1 to
0.3269 at 0.7 and falls to 0.3047 at 1.5; Δ is negative below σ ≈ 0.65 and positive above. With
more actions the zero crossing moves right (between 0.7 and 0.8 for |A| = 4, between 0.9 and 1.0
for |A| = 8), so a robot choosing among more actions is overconfident enough to bypass the human
over a wider range of σ.

## Results that began as planning findings

**Pooling is not an equilibrium, but it is where fictitious play stays.** H's best reply to any
constant map is the expert rule, since R's deployment then does not depend on what H makes, so the
pooling pair (always (1,1), always (50,50)) is not a mutual best response. The enumeration finds
exactly six pure mutual best responses, R's maps being the six permutations of [0, 1, 2]. They are the
paper's teaching equilibrium and five relabelings of which of H's three actions signals which band
of θ. All six have the same expected reward. Iterated best response from pooling reaches the
paper's pair in all 20 seeds (median tick 2). Fictitious play from pooling stays there in all 20
seeds (final distance 84 throughout, reward 51 per game; confirmed by a retrospective diagnostic
reading R's map, [1, 1, 1], at the horizon). The reason lies in the tables: an action H never makes
keeps its uniform starting table, so H expects R's average deployment after it (worth 46.67) and
never tries it. R never sees those actions and never revises its (50,50). So pooling is a rest
point of our fictitious play (a self-confirming outcome, sustained by off-path beliefs), not an
equilibrium. It depends on the uniform table prior, which is our choice.

**Figure 4 needs a noisy human.** With β = 0, Theorem 1 makes R wait at every σ, so the value
cannot peak: V = 0.3992–0.3993 at σ 0.1, 0.7 and 1.5. The figure's shape appears only with a
noisy human. At our β = 0.5 all three of its directions hold. The paper does not state the human
it used for Figure 4, so this is a reconstruction (see below).

**Quadrature.** Gauss–Hermite quadrature converges slowly when β is below about half of σ (the
logistic's poles sit πβ from the real axis), so Theorem 2's 1e-9 agreement rule could not have held
over Figure 3's grid. The composite trapezoid rule with step min(σ, β)/8 over μ ± 9σ meets it with
room to spare (largest gap 4.9e-15; unit tests show density 4 and 64 agree). This is a numerical
method, not a modeling choice, and no verdict depends on it beyond the agreement tolerance.

## The dynamics

Twenty seeds, 200 ticks of 1,000 games, default settings; "settled" is the last tick at which the
distance to the paper's pair changed (median over seeds).

| Start | Iterated best response | Fictitious play |
|---|---|---|
| expert | 20/20 at the paper's pair, settled by tick 1 | 20/20, settled by tick 6 |
| teaching (the paper's rule) | 20/20, never moves | 20/20, settled by tick 2 |
| pooling | 20/20, settled by tick 2 | 0/20: stays at pooling (distance 84) |
| uniform random | 12/20; the other 8 reach relabeled equilibria (distances 95 ×6, 84, 53), settled by tick 2 | 19/20, settled by tick 3; one (seed 17) reaches a relabeled equilibrium (distance 53) |

A retrospective diagnostic read the final maps of the off-paper runs: [1, 2, 0] and [2, 0, 1]
(distance 95), [2, 1, 0] (84) and [0, 2, 1] (53). Each is one of the six enumerated equilibria, with a
realized mean reward of 68.1–69.5 per game. This is sampling noise around the common expected
6373/92 ≈ 69.27. So from an informed start both dynamics find the paper's teaching equilibrium. From
a random start they find *a* teaching equilibrium, and which signal means which band depends on the
start, as expected in a common-payoff signaling game. Distance 84 is ambiguous between the
[2, 1, 0] relabeling and the pooling pair; the diagnostic tells them apart.

## What depends on a reconstruction choice

- **Figure 4, entirely.** The paper does not specify the human's temperature, the observation model
  or the action construction. Our setup is `protocol: report`: utilities drawn from N(0, 1), a noisy
  report with SD 1, R's Bayesian posterior mean, |A| candidates, and β = 0.5 for the human. Holds
  means "the directions appear under this reconstruction", not that the paper's setup is recovered.
  The rational case is the counterfactual that shows why a noisy human is needed. The peak test
  resolves σ only to the 0.1 grid. Neighbors 0.6 and 0.8 lie within a few SE of 0.7, so the
  claim is that no σ beats the true posterior SD, not that the maximum is sharply located.
- **The Figure 3 grid.** σ 0.05–1.00 by 0.05 and β 0–4 by 0.2 are ours, read from the figure's
  axes; the directions are judged over that grid only. The formal statement of the caption's
  "lowest … highest" (corners plus adjacent-cell monotonicity) was adopted before measurement.
- **Fictitious play.** The tables' uniform prior (count 1 per cell), θ revealed to R after
  deployment, the weight 1 and the tie rule are ours. The pooling rest point depends on the prior.
  Iterated best response's order (H then R), ties to the smaller action and off-path actions resolved
  by the prior are also ours, as are the four starts.
- **Discretization.** 92 θ bins make the paper's thresholds bin edges; the exact claims use
  rational arithmetic where the paper's quantities are rational.

The paperclip closed forms and the off-switch theorems do not depend on these choices beyond the
numerical tolerances stated.

## Open questions

- Which human did the authors use for Figure 4? Their code is not public as far as we have checked.
  A β sweep of Figure 4 (planning saw a plateau at β = 0.25) would map where the peak appears;
  that would be a new, separately preregistered measurement.
- Does fictitious play with a different table prior, or with exploration, escape pooling, and how
  often does it pick each relabeling?
- Milli et al. 2017 (*Should robots be obedient?*) and *Inverse reward design* are the follow-ups
  queued with this item; neither was measured here.
- A2 of the [alignment program](../../studies/2026-10-11-alignment-program.md) (the off-switch among
  foragers) can reuse `offswitch::incentive` directly.
