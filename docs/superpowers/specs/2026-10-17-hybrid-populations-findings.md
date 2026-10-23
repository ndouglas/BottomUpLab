# Hybrid human–machine populations (milestone 42) — findings

Measured 2026-10-21 to 2026-10-23 under the decision rules fixed in the
[design](2026-10-17-hybrid-populations-design.md) on 2026-10-17, before any measurement. Its
amendments 1–14 (planning, 2026-10-20) and 15 (the gap-filling grids and guards, 2026-10-21)
predate every scored run. These are working notes. They report measurements and verdicts, and
interpretations are labeled as such. They are not publication-ready claims.

## Sources

All local copies are under `papers/alignment/` (gitignored).

- Booker, Miranda, Moreno López, et al., "Discriminatory or Samaritan — which AI is needed for
  humanity?", arXiv 2306.17747v2 (the latest version; no published version found). It covers the
  donation game, the finite well-mixed chain (Eqs. 1–15), the replicator equations (Eqs. 16–20),
  and the lattice and Barabási–Albert runs (Figs. 4–6). No code exists.
- Guo et al., "Facilitating cooperation in human-agent hybrid populations through autonomous
  agents," iScience 2023 (arXiv 2307.01123v1). The supplement's Theorems 1–3 and Algorithms I–II
  come from the arXiv source (`SI/2supp.tex`, under `code/guo-et-al-2023-iscience-supplement-arxiv-source/`).
  No code exists.
- Si, He, Shen and Tanimoto, "Cooperative bots exhibit nuanced effects on cooperation across
  strategic frameworks," J. R. Soc. Interface 22:20240427 (2025), which is the reference text. The
  preprint is arXiv 2406.14913v1. The published C++ is Zenodo 10.5281/zenodo.13957161 (26 files:
  21 C++ files, one per panel, and 5 notebooks), under `code/si-et-al-2024-jrsi-cooperative-bots-zenodo-13957161/`.
- Shirado and Christakis, "Locally noisy autonomous agents improve global human coordination in
  network experiments," Nature 545:370 (2017), with its Supplementary Information. Only its model
  is used: the bot rule, the network generator, the solution count and Extended Data Fig. 2.
- Cited, not reproduced: Fernández Domingos et al., Sci. Rep. 12:8492 (2022); Ishowo-Oloko et al.,
  Nat. Mach. Intell. 1:517 (2019), with its OSF deposit 10.17605/OSF.IO/AK3TF (under
  `code/ishowo-oloko-et-al-2019-natmachintell-osf-ak3tf/`); and Crandall et al., Nat. Commun. 9:233
  (2018).

## Summary

All 28 rows ran or were accounted for: 15 Holds, 7 Fails, and 6 reported without a target (the
pilot, three evaluation rows that ran, and two evaluation rows not run because they were over
budget). No run
count was reduced.

**Booker.** The exact chain reproduces Fig. 1. The all-cooperate weight switches on at 3 Samaritan
bots at β = 0.1 and at 58 at β = 1, and never reaches 0.5 at β = 2.5. Samaritan bots beat
discriminatory ones at low β, and the reverse holds at high β. Both printed errata are confirmed:
the printed G_D is off by a factor of M + 1, and the printed α_c is 1/(1 + e^β) where the general
formula gives 1 − e^{−β}. Eq. 18 fails as registered: for Samaritan bots the payoff difference does
not depend on b, and x = 0 is not a fixed point at α = 0.5. Eq. 20 fails on its b* statement. On
the lattice, Samaritan bots raise cooperation at β = 0.1 and barely help at β = 5, as claimed, under
both step units. Topology does change the final state, against the claim: Barabási–Albert and the
lattice differ by more than 0.1 at 6 of 12 points.

**Guo.** The replicator phase diagrams, the master-equation roots, Algorithm I's agreement with them,
and all six network slopes of Fig. 6 hold. That includes the snowdrift reversal, where cooperation
falls with φ on the lattice and rises on Erdős–Rényi and Barabási–Albert graphs. Fig. 7's "ten hub
bots move cooperation" holds in the stag hunt and fails in the Prisoner's Dilemma. There, the network
without bots already cooperates at 0.961, so the φ = 1 gap cannot clear 2 SE. The hub
explanation's evaluation row keeps a positive slope with random bots and mean payoffs, about a
quarter of its size with summed payoffs.

**Si et al.** Our port docks against the authors' own compiled code at all 36 points. The well-mixed
claims of Fig. 1 hold both as the code runs them (with an integer division that drops S and P) and
on the stated game. The lattice claims of Fig. 3 hold: bots suppress cooperation in the discrete and
continuous frameworks and promote it in the mixed one. Fig. 6's interior optimum holds on the lattice
(the caption's setting) only as "θ = 1 does worse than θ = 0.9". It fails in the deposited
well-mixed code, where human cooperation equals θ. Many mixed cells read exactly 0.900 for a
reason the code fixes: humans only imitate, bots never change, and nothing mutates a strategy, so
humans settle at the bots' θ.

**Coloring.** Extended Data Fig. 2's correlation is reproduced (−0.992 against the paper's −0.990).
In our reconstruction of the human players, 10% bot noise does not beat noiseless bots, so the
mechanism-sufficiency claim fails twice. At the errant rate fitted to the paper's control (0.120),
three central bots help more the less noisy they are, each step by more than 2 SE. At the paper's
measured 0.041, 10% noise does no better than noiseless, and every difference is within 2 SE. This
says nothing about the paper's human result.

## Provenance

- **Survey.** `cargo run --release --manifest-path survey/Cargo.toml -- --only hybrid`, with stdout
  written to `survey/out/hybrid/results.md`. It ran at core commit `04246f3` on an Apple M1 Max
  MacBook Pro (MacBookPro18,4, ten logical CPUs) with Rust 1.98.1 on 10 threads. It started
  2026-10-21 at 17:13 EDT and ended 2026-10-23 at about 08:00 EDT, a wall time of 142,978 s
  (39.7 h). The machine was under heavy contention from other processes for most of the run (load
  average 200–320), so the wall time far exceeds the pilot's uncontended projections (the rows
  below sum to about 17.6 h). Commits: `05e5dff` (results), `bd7e7ff` (the pilot fix), `58e5fff` (the
  pilot row spliced in, cargo's build lines dropped). `results.md` SHA-256:
  `935e6178f5755a469e638995e103c8f591591634eef2aec5bcbc6582cd376a4e`.
- **Pilot and run counts (amendment 8).** The pilot times one run of each protocol and projects
  each scored row on 10 threads. The committed pilot row was rerun on 2026-10-23 at `bd7e7ff`. The
  first version grouped the Si lattice point sets into one line and miscounted S3 (126 point sets
  against the 94 it runs). The independent review caught this as finding I1. Per row, the
  projections are: B5 0.00 h, B5 `sweep_steps` 0.38 h, B6 0.00 h, G3 0.00 h, G4 2.08 h, G5 3.46 h,
  S1 1.28 h, S2 code 1.79 h, S2 stated 1.77 h, S3 3.72 h, S4 lattice 0.43 h, S4 well-mixed 0.27 h,
  C1 and C2 0.00 h. For the evaluation rows: the hub evaluation 0.69 h, S3 `per_round` 1.66 h, S2
  `preprint_n` 0.08 h. No scored row exceeds 8 h, so **no run count was changed at any date**.
  Every row ran at its protocol's count: 30 (Booker lattice), 10 networks × 20 (Booker BA), 20
  (Algorithm I), 60 (Guo networks), 100 (Si), 200 sessions (C2), and 230 graphs × 100 repeats (C1).
  Two evaluation rows were projected over budget and not run: S2 `journal_n` at 17.3 h and S3
  `journal_n` at 53.1 h.
- **Docking build (S1, amendment 2).** `survey/hybrid_si/dock.py` copies the deposit into a scratch
  directory, patches it, compiles it with Apple clang 17.0.0 (clang-1700.0.13.5) using
  `-std=c++17 -O2`, and runs every chunk under `/usr/bin/sandbox-exec`. The profile is
  `(version 1)(allow default)(deny network*)(deny file-write* (require-not (subpath (param "BOX"))))(deny process-fork)(deny mach-lookup)(deny process-exec)(allow process-exec (literal (string-append (param "BOX") "/run")))`.
  The reference run used the profile recorded in `receipts.json` (network denied; signals to
  other processes not yet denied); later runs also deny signals to other processes.
  It applies five changes and nothing else. Four are patches to the source: stray characters before
  `#include` (Figs. 3B and 3C), `loop` made a constant (`SI_LOOP`), `sgenrand(time(0))` changed to
  `sgenrand(SI_SEED)`, and the r and ρ loops pinned to one point. The pinned loops set exact
  literals where the original accumulated 0.01 steps, which is a few ULP of difference. The fifth
  is in the harness: each point's 100 runs are split into four processes of 25, with seeds
  4p + 1 … 4p + 4, so seeds 1–144 are all present. The receipts are in
  `survey/out/hybrid/si-reference/receipts.json`:

  | panel | file | source SHA-256 | patched SHA-256 |
  |---|---|---|---|
  | 1A | `For data_Figure1A_Discrete.cpp` | `3f6220af6c19a28d…` | `25f1052bde93b0c7…` |
  | 1B | `For data_Figure1B_Continuous.cpp` | `fb841277ba86ead1…` | `2fba01439a3f4275…` |
  | 1C | `For data_Figure1C_Mixed.cpp` | `579d99b2568258a2…` | `e14c43543e0b65e5…` |
  | 3A | `For data_Figure3A_Discrete strategy.cpp` | `ce55af63c21295b2…` | `a08b68b5e22b47f5…` |
  | 3B | `For data_Figure3B_Continuous strategy.cpp` | `150e47b90036fbe6…` | `e4eea299062b59fe…` |
  | 3C | `For data_Figure3C_Mixed strategy.cpp` | `0be7c88b36db8d68…` | `fc4af5725a17f293…` |

  The full hashes are in the receipt. The independent review reapplied the patch function and
  matched both hashes for all six files. The 144 chunks took 31,354 s of process time, at up to ten
  at once. The first invocation stopped when the disk filled and was resumed from the chunks already
  on disk. The receipt's `wall_seconds`, 2,791 s, is the resumed invocation's. The reference was
  committed at `04246f3`, before the survey.
- **The port against the C++** (independent review, line by line). The payoffs, the Fermi rule (β =
  10 on both paths), the update order, who may act, and the measure all match. The C++'s `cal_data`
  takes the mean strategy over ordinary players, averaged over the last 2,000 rounds, and so does
  ours. Two things differ. The order of random draws differs in places. The initial discrete bot
  draw compares `sv >= randf()`, which differs from ours only at equality.

## Claim table

Verdicts are copied from `results.md`. `Untestable` marks a row reported without a target. Rows
marked "evaluation" are not scored as the paper's claims. B3, B5, S2, S4 and C2 each have two rows.

| id | rule | verdict | measured |
|---|---|---|---|
| `hybrid.b1-switch` | exact chain, N = 100, b = 2: the first M with all-C weight ≥ 0.5 in [1, 8] at β 0.1, in [50, 65] at β 1, none ≤ 100 at β 2.5 | Holds | β 0.1 → 3; β 1 → 58; β 2.5 → none |
| `hybrid.b2-samaritan-vs-discriminatory` | mean all-C weight over M 0..100 × b 1.1..3: Samaritan higher at β 0.1, discriminatory higher at β 2.5 | Holds | β 0.1: Samaritan 0.9708, discriminatory 0.1801; β 2.5: Samaritan 0.0000, discriminatory 0.1616 |
| `hybrid.b3-eq18` | Eq. 18 as printed: b*(0.1) < b*(0.5) at α 0.1, β_AI 1; \|ẋ(0)\| < 1e-12 at α 0.5 for every b (registered expectation: fails) | Fails | part 1 Holds: b* 1.1 at β_H 0.1, none at 0.5; part 2 Fails: largest \|ẋ(0)\| 1.345e-1; spread of ẋ(x) across b 0.000 |
| `hybrid.b3-eq20` | the same two statements on Eq. 20 | Fails | part 1 Fails: b* none at either β_H; part 2 Holds: \|ẋ(0)\| 0.000; spread across b 2.134e-3 |
| `hybrid.b4-errata` | printed G_D and α_c off by > 1e-9 relative; corrected within 1e-9 | Holds | G_D printed off by 1, 10, 50, 100 (relative) at M 1, 10, 50, 100, corrected ≤ 1.2e-13 (largest 1.137e-13); α_c root scan 0.095163, 0.632121, 0.917915 at β 0.1, 1, 2.5 against printed 0.475021, 0.268941, 0.075858; corrected exact |
| `hybrid.b5-lattice` | 30 runs, 10⁵ updates: β 0.1 ≥ 0.5 at α 0.25 and 0.5, ≤ 0.2 at α 0.01; β 5 every α ≤ 0.5 within 0.2 of α 0 | Holds | β 0.1: 0.724, 0.993, 0.034; β 5: α 0 0.000, largest rise 0.056 |
| `hybrid.b5-lattice-sweep-steps` (variant) | B5's rule, one tick = N updates | Holds | β 0.1: 0.941, 1.000, 0.025; β 5: largest rise 0.056 |
| `hybrid.b6-topology` | \|lattice − BA\| ≤ 0.1 at α 0.1–0.4 × β {0.1, 1, 5} | Fails | fails at all four α at β 0.1 (0.289 vs 0.137, 0.590 vs 0.342, 0.829 vs 0.561, 0.957 vs 0.742) and at α 0.4 at β 1 (0.038 vs 0.269) and β 5 (0.025 vs 0.201); holds at the other six |
| `hybrid.g1-phase-diagrams` | numerical classification on 101 × 101 matches the closed form off boundaries; y_min = 0.4 within 1e-9 | Holds | no mismatch in snowdrift (10,198 cells), stag hunt (10,194) or dilemma (10,201); y_min 0.399999999999 |
| `hybrid.g2-master-equation` | Fig. 5's master-equation statements (amendment 15's grids) | Holds | PD K 100: largest stable root 0.0000, no empty point; snowdrift K 100: 0.7143 → 1.0000 at φ 0.1 and 0.7143 → 0.0876 at φ 0.9; K 1: φ 0.9 above φ 0.1 at every y in all three games |
| `hybrid.g3-algorithm-i` | Algorithm I within 0.05 of the stable root, 20 runs | Holds | all 75 cells, e.g., snowdrift y 5 φ 0.9 K 100 0.085 vs 0.088, dilemma y 0.5 φ 0.9 K 1 0.759 vs 0.756 |
| `hybrid.g4-networks` | slope against φ, 60 runs, bootstrap CI excludes 0, direction per network | Holds | snowdrift: lattice −0.142 (−0.144 to −0.141), ER +0.158, BA +0.495; dilemma: lattice +0.107, ER +0.358, BA +0.953 |
| `hybrid.g5-hubs` | PD and stag hunt: ten-hub mean at φ 1 above none by > 2 SE and slope CI > 0; all three games: ten random within 2 SE of none at every φ | Fails | dilemma hubs Fail: 0.999 vs none 0.961 (gap 0.038, 2 SE 0.047), slope +1.113 (CI lower +1.092); stag hunt hubs Hold: 1.000 vs 0.733 (gap 0.267, 2 SE 0.115); every random arm Holds |
| `hybrid.g-hub-explanation` (evaluation) | snowdrift on BA, 4,871 random bots, summed against mean payoffs | Untestable | slope +0.4949 (+0.4824 to +0.5072) summed; +0.1316 (+0.1293 to +0.1341) mean |
| `hybrid.s1-docking` | 27 lattice + 9 well-mixed points, 100 runs a side: within 3 combined SE or < 0.02 | Holds | 36 of 36; largest difference 0.038 (lattice mixed r 0.05 ρ 0, 1.1 combined SE, a bimodal point with SE 0.024 a side); largest in SE units 2.5 (lattice continuous r 0.05 ρ 0.3, −0.0028) |
| `hybrid.s2-well-mixed-code` | every ρ > 0 above ρ 0 at every r; continuous and mixed ≥ 0.8 at ρ 0.5, r ≤ 0.1; integer division | Holds | e.g., discrete ρ 0.1: 0.900, 0.429, 0.290, 0.226, 0.187, 0.161 against ρ 0 0.052, 0, …; continuous ρ 0.5 r ≤ 0.08: 0.900, 0.900, 0.892; mixed 0.900, 0.900, 0.900 |
| `hybrid.s2-well-mixed-stated` | the same, float division (the stated game) | Holds | discrete ρ 0.1: 0.900, 0.196, 0.080, 0.043, 0.025, 0.016; continuous rows equal the code's; mixed ρ 0.5 r ≤ 0.08: 0.900, 0.900, 0.897 |
| `hybrid.s3-lattice` | ρ 0.5 below ρ 0 by > 2 SE at every r ≤ 0.12 (discrete, continuous), above at every r ≤ 0.2 (mixed) | Holds | discrete r 0.10: 0.159 vs 0.513; continuous 0.166 vs 0.481; mixed 0.629 vs 0.096; smallest gaps 0.117 (discrete, r 0), 0.088 (continuous, r 0), 0.071 (mixed, r 0) |
| `hybrid.s4-optimum-lattice` | ρ 0.3, r 0.01: argmax over θ inside (0, 1), above θ 1 by > 2 SE (the caption's lattice) | Holds | argmax θ 0.9 (0.8997), θ 1 0.6323, margin 0.2674 vs 2 SE 0.0009; by θ 0.001, 0.100, …, 0.900, 0.632 |
| `hybrid.s4-optimum-well-mixed` | the same in the deposited well-mixed code | Fails | argmax θ 1.0 (1.0000); by θ 0.000, 0.100, …, 1.000 |
| `hybrid.s3-per-round` (evaluation) | S3 under `mixed_draw: per_round` | Untestable | by the scored rule it would Fail: mixed r 0: 0.900 vs 0.993 without bots; bots ahead from r 0.01 on |
| `hybrid.s2-preprint-n` (evaluation) | S2 code at N = 500 | Untestable | by the scored rule it would Hold |
| `hybrid.s2-journal-n` (evaluation) | S2 at N = 10⁵ | Untestable | not run: projected 17.3 h |
| `hybrid.s3-journal-n` (evaluation) | S3 at 316 × 316 | Untestable | not run: projected 53.1 h |
| `hybrid.c1-convergence` | 230 seeded graphs: Pearson r of ln solutions and ln mean steps ≤ −0.9 | Holds | −0.9918 (the paper's −0.990); 384 repeats censored at the 10⁷ cap, counted at the cap |
| `hybrid.c2-mechanism` | at the fitted e, 10% central bots solve more often than 0% and than 30%, each by > 2 SE, 200 sessions per arm | Fails | fitted e 0.120 (control 0.675; the paper's 20/30); noise 0, 0.1, 0.3: 0.885, 0.760, 0.630 |
| `hybrid.c2-observed-e` | the same at e 0.041 | Fails | 0.775, 0.755, 0.710 |
| `hybrid.pilot` | the projections above | Untestable | reported without a target |

## What the sources say and what the code does

**Si et al.'s integer division and the well-mixed game.** Every well-mixed discrete and mixed file
computes the payoff as `(double) c / nn * M[a][0] + (nn - c) / nn * M[a][1]` with `int c, nn`. The
second share is integer division, so it is 0 unless no other player cooperates. A cooperator then
earns c/(N − 1) and a defector (1 + r)·c/(N − 1). S = −r and P = 0 never enter. Defection's
advantage becomes r·x instead of the stated game's r, so selection against cooperators is weaker,
and the bots' lift is larger under the code. At ρ = 0.1 and r = 0.04, discrete cooperation is 0.429
under the code against 0.196 on the stated game. At ρ = 0.5 and r = 0.2 it is 0.314 against 0.124.
The continuous files never divide, so their rows are identical under both. Fig. 1's qualitative
claims hold either way (both S2 rows Hold). Its magnitudes are the code's.

**The deposit does not compile as published.** Figs. 3B, 3C and Fig. 5's continuous file begin with
a stray `c` or `cc` before `#include`. Every well-mixed file initializes a variable-length array
(`double record_fc[loop]={0}` with `int loop`), which clang rejects. Both are fixed by the
patches above and by nothing else.

**Why the mixed cells read 0.900.** This mechanism was confirmed against the C++ by the independent
review: it is the code's own dynamics and its own measure, not a port artifact. Humans only copy
strategy values already present. Bots hold θ forever and are role models, and no path mutates a
strategy. Once every human holds θ the state is absorbing, and the measure, the humans' mean
strategy, reads θ exactly. In the mixed framework, therefore, human cooperation settles at the bots'
θ wherever selection is weak enough for the bots' strategy to spread. That covers S2 and S3 at r near
0, S2 `preprint_n`, and S4 at every θ ≤ 0.9. Interpretation: "bots promote cooperation in the mixed
framework" is, in this model, the statement that fixed role models fix their own θ in a population of
imitators. Below θ, at larger r on the lattice, imitators drift away from it.

**Fig. 6: code against caption, and which reading each S4 verdict judges.** The deposited Fig. 6
Mixed file is well-mixed: it has no neighbor table and sets `Kappa = pow(10, m)`. The journal's
Fig. 6 caption says "regular lattice". `s4-optimum-lattice` judges the claim as published, on the
caption's lattice, and it Holds. Its means equal θ up to 0.9 and then drop to 0.632 at θ = 1
(dated note, 2026-10-23, review M2). The argmax is therefore the last interior grid point, and the
evidence is "θ = 1 does worse than θ = 0.9 by 0.27", not a peak located inside (0, 1). The verdict
stands under the registered rule. `s4-optimum-well-mixed` judges the deposited code, and it Fails:
cooperation equals θ, so θ = 1 is the argmax. Under integer division at r = 0.01 the selection gap
is r·x ≈ 0.009, which with β = 10 is nearly neutral. Interpretation: this fits the caption's
lattice being the figure's true setting and the deposited file not being what drew it. Neither S4
point set is docked (dated note, 2026-10-23, review M3). S1 docks θ = 0.9 at r ≥ 0.05, while S4
reads r = 0.01, every θ, and θ = 1. The port is line-for-line, so there is no specific reason to
doubt these values. Docking them would need a sixth patch (pinning θ), a dated amendment, and an
evaluation row of its own.

**κ.** On the lattice the C++ computes `1/(1+exp((x−n)/Kappa))` with Kappa 0.1, a temperature.
Well-mixed it computes `1/(1+exp((c−n)*Kappa))` with Kappa 10, an inverse temperature. Both are
β = 10.

**N.** The code uses 10⁴ for both structures. The journal says 10⁵ (§2.4), and the preprint says 500
well-mixed. The preprint's N, as an evaluation row, would Hold. The journal's N was projected at
17.3 h (S2) and 53.1 h (S3, on the nearest square, 316 × 316), over amendment 8's 8 h budget, and
was not run.

**The design's engine line**, "bots never update and are never focal", has two exceptions in the
sources, both kept: Guo's Algorithm II picks a bot as focal to redraw its action (amendment 4), and
Si's well-mixed mixed path redraws the picked node's action, bots included (amendment 3). On no path
does a bot imitate.

**Booker's Eq. 18 is b-independent.** For Samaritan bots the payoff difference is
(R − T)[(1 − α)x + α] + (S − P)(1 − α)(1 − x) = −c identically, because R − T = S − P = −c. So
ẋ is the same for every b (measured spread 0.000), and h(0) = α/(1 + e^{β_AI c}) > 0. At α = 0.5 and
β_AI = 1 that is 0.5/(1 + e) = 0.1345, the measured |ẋ(0)|. x = 0 is therefore not a fixed point,
against the text's "0 and 1 are the unique fixed points" at α = 0.5. This failure was the registered
expectation. Part 1 of the row passes, but it passes without b entering it (dated note, 2026-10-23,
review M5). Since x̄ does not depend on b, b* is either 1.1 or ∞, and b*(0.1) = 1.1 < b*(0.5) = ∞
only because β_H = 0.1 cooperates at every b and β_H = 0.5 at none. For Eq. 20, Δf = −c(1 − α) + αb
is negative on the whole grid at α = 0.1, so x̄ = 0 and b* = ∞ at both β_H, and "∞ < ∞" fails.
Its fixed-point part holds trivially from the x factor.

**Booker's printed forms.** The printed G_D (Eq. 14) is the general product times M + 1, so its
relative error is exactly M. The printed simplified α_c is 1/(1 + e^β), while the general formula
at equal β gives 1 − e^{−β(T−R)} = 1 − e^{−β}. That matches the root scan to 1e-16 (0.0952, 0.632
and 0.918 at β = 0.1, 1 and 2.5, against the printed 0.475, 0.269 and 0.076).

**Booker's networks.** The paper's protocol, 10⁵ single updates, is about 40 updates per node on
the 2,500-node lattice, and it is not stationary at β = 0.1. At α = 0.25 the lattice reads 0.724
under the paper's step and 0.941 when one tick is N updates. B6's rule is unclear about "final
state" (dated note, 2026-10-23, review M4). Both arms follow the paper's protocol, but N differs by
the paper (2,500 against 1,000), so Barabási–Albert humans get 2.5× as many updates per node, and
the β = 0.1 failures mix topology with elapsed time. The verdict stands without them. At α = 0.4
and β = 5, where the lattice reads the same under both step units (0.242 at α = 0.75 in both rows),
the lattice gives 0.025 and Barabási–Albert 0.201. Interpretation: summed payoffs on
Barabási–Albert hubs help cooperators there. A `sweep_steps` row for both arms would separate the
two effects.

**Guo's hub inference and what the evaluation row shows.** Guo et al. write that the snowdrift
reversal "may be attributed to AAs occupying network hub nodes". Fig. 6's 4,871 bots are placed at
random, and so are G4's. The evaluation row keeps that placement and averages each player's payoff
over its neighbors instead of summing it. The slope falls from +0.495 to +0.132 and stays positive
(CI +0.129 to +0.134). Interpretation: bot hubs are not needed for the reversal. Degree
heterogeneity in summed payoffs carries about three quarters of the slope, and the rest survives
without it.

**G5's dilemma ceiling** (dated note, 2026-10-23; the verdict is unchanged). The rule is a
conjunction: the φ = 1 gap must exceed 2 SE *and* the slope CI must exclude 0. On Barabási–Albert
without bots, the Prisoner's Dilemma (T = 1.3, S = −0.2, K = 10) already reaches 0.961, so the
φ = 1 clause can only measure movement up against a near-ceiling baseline. The slope (+1.113, CI
lower bound +1.092) and the hub means (0.014 at φ = 0 up to 0.999 at φ = 1) show ten hub bots moving
cooperation in the paper's sense, mostly downward when they defect. A different statistic would
need an amendment and its own verdict. The arms share graphs (review M7): for a given seed, the none
and hub arms build the same Barabási–Albert graph and the same initial human states apart from the
ten hubs. The combined SE the rule names is therefore conservative for this gap, and a paired
reading would be a post-hoc rule. The same seed reuse across φ or arms holds for G4, G5 and C2, so
their intervals are conservative. C2's differences point the wrong way, so pairing cannot flip it.

**The coloring game's reconstruction.** Shirado and Christakis's humans are people. Ours are a
labeled reconstruction: greedy responders with exponential decision times of mean 1.59 s (Extended
Data Fig. 1's median) and an errant rate e. The bot rule, the graph, the solution count and Extended
Data Fig. 2 are the paper's computational parts. C2 tests whether the paper's bot rule, among these
reconstructed humans, is sufficient for the 10% central-noise advantage. It is not a test of the
human result. The reconstruction fails that test at both e values. At the fitted e = 0.120, noiseless bots do
best: 0.885 > 0.760 > 0.630, with gaps of 0.125 (2 SE 0.075) and 0.130 (2 SE 0.091). At e = 0.041 the
arms read 0.775, 0.755 and 0.710 with 2 SE of 0.085 and 0.088, so 10% noise does no better than
noiseless and no pairwise difference is resolved. Interpretation, a hypothesis not tested here: our
humans' errant moves may already supply the noise that lets a session leave a local conflict, so
the bots' noise adds nothing, and at e = 0.120 it costs them. Whatever made the paper's noisy
bots help would then lie in human behavior this greedy-plus-errant model lacks.

## Human data, cited and not reproduced

- **Shirado and Christakis 2017.** With bots of 10% noise in central locations, 17 of 20 sessions
  (85%) were solved within 5 minutes, against 20 of the 30 control sessions with humans alone
  (67%), log-rank P = 0.015. That is one of nine bot treatments (three noise levels × three
  locations), against an omnibus log-rank P = 0.024 that at least two survival curves differ.
- **Ishowo-Oloko et al. 2019.** In a repeated Prisoner's Dilemma, bots induced more cooperation than
  humans did while their partners believed them human. Disclosing their nature negated that
  advantage, the "machine penalty" (a transparency–efficiency tradeoff).
- **Fernández Domingos et al. 2022.** In the collective-risk dilemma, 66.7% of all-human groups
  reached the target (n = 12), against 86.7% when participants delegated to artificial agents
  (n = 15).

None of these is a model result, and none is reproduced here.

## Limits and next steps

- The two `journal_n` evaluation rows need runs of 17.3 h and 53.1 h on this machine
  (uncontended). Running them needs a budget beyond amendment 8's 8 h or a faster machine, and no
  change to sizes, lengths or windows.
- A reference run of the Fig. 6 Mixed file at θ ∈ {0.9, 1} and of Fig. 3C at θ = 1 would dock both
  S4 verdicts. That needs a sixth patch, a dated amendment, and a separate evaluation row.
- B6 under `sweep_steps` for both arms would separate topology from elapsed time at β = 0.1.
- S3's measured string prints means only. The 2 SE its rule uses are not shown (review M6). No
  verdict is close: the smallest gap is 0.071, with 100 runs a side.
- C2's failure is the reconstruction's. Other human models (stickier humans, slower or
  conflict-averse responders) are new variants with their own rules, not repairs.
- Ishowo-Oloko et al.'s OSF deposit holds a JavaScript S++ with its experts, per-round logs and the
  R code for Figs. 1–3. That makes S++ reproducible from code in a later milestone, alongside
  Crandall et al. 2018's Supplementary Notes and the 2014 JAIR paper.
- Experiment A4 of the [alignment program](../../studies/2026-10-11-alignment-program.md), a second
  species in the cooperation models, can reuse `hybrid::engine` (the population, Fermi imitation
  and placement).

## Register entries

RR-HY-01 through RR-HY-19 in `docs/reproducibility.md` record the provenance, the seventeen scored
claims and the evaluation rows.
