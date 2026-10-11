# History-dependent pricing: cloned component-feedback intervention

**Completed 2026-10-11; exploratory controlled intervention.** Refreshing continuation feedback reduced the mean certified profit available from sustained deviations by **63.5%** across eight paired seeds. Three resulting controller pairs deter sustained deviations from every phase of their learned cycles. Refreshing both reward and continuation feedback reduced the mean by **91.7%**, with four cycle-level passes. Every pair still fails the stronger all-state test.

This experiment follows the [frozen-policy audit](2026-10-11-collusion-frozen-policy-audit.md) and [saved-score diagnosis](2026-10-11-collusion-saved-score-diagnostic.md), under the [research method](../research-method.md). It tests whether the feedback rule changes the vulnerability of jointly learned, history-dependent strategies. The user approved eight shared prefixes and 32 cloned continuations, bounded by 40 million total matrix periods. Actual exposure was **34,044,310 periods**. No extra arms, parameter tuning, exploration restart or repeated training campaign occurred.

## Design and fixed judgment

Replay seeds 1–8 under the original project-default memory1/asynchronous settings to exactly one million completed periods. Both firms retain one previous price pair, giving 225 states and 15 actions per firm. Alpha is .15, delta .95 and exploration decay beta 4e-6; grid and score initialization are Calvano's, with project RNG and lowest ties. These are conditional baseline-variant results, not the authors' RAN2/random-tie population.

Carry a passive immediate-reward EMA R from initialization, starting at `(1-delta)*Q`; define B as the residual `Q-R`. Clone each complete live world, RNG and ledger into four arms before selecting treatment. Every chosen action retains the exact native total-score update and reward EMA. Extra feedback applies to unchosen actions at the visited state:

| Arm | Extra feedback |
| --- | --- |
| Control | None |
| Reward | Refresh R toward current counterfactual immediate profit; update Q correspondingly |
| Future | Refresh residual B toward the learner's own estimated discounted continuation; R unchanged |
| Both | Native synchronous whole-Q target and reward EMA |

All targets use the same pre-write world. Each candidate price retains its own resulting state and its firm's best score at that state. This preserves legitimate differences in future punishment or recovery. Certified optimal values are used only in evaluation, never as learning targets. B is a specified accounting residual, not an independently identified true forecast. The treatment effect concerns these declared update rules and subsequent coupled trajectories.

All eight controls had to reproduce their archived runs before any intervention began. Each branch then stops at its native stopping event or at one million additional periods, whichever occurs first; absolute period is capped at two million. Native exploration, stability counters and stopping rules are inherited unchanged.

At each terminal controller, freeze both full state-dependent policies. For each firm, independently calculate the learned-policy discounted value Vπ and optimal unilateral value V\* against the responding rival. The primary reduction measure E is `(1-delta)*(V*-Vpi)`, averaged equally over both firms and every phase of that branch's learned cycle, then equally over the eight seeds. It is a constant-profit-equivalent scaling of discounted gains, not an observed long-run profit statistic.

Removal has a separate stricter rule: every raw discounted-gain interval's upper bound must be at most 1e-8 for both firms at every cycle phase. Any certified lower bound above that threshold leaves a vulnerability. Numerical boundary cases remain unresolved. Mean reductions alone do not certify removal.

## Primary results

| Arm | Mean E | Reduction versus control | Cycle-level passes | Native finishes | Horizon-censored |
| --- | ---: | ---: | ---: | ---: | ---: |
| Control | .042868 | — | 0/8 | 6 | 2 |
| Reward | .031371 | 26.8% | 0/8 | 8 | 0 |
| Future | .015642 | 63.5% | 3/8 | 5 | 3 |
| Both | .003560 | 91.7% | 4/8 | 5 | 3 |

Percentages are ratios of the eight-seed means, not averages of individual percentage changes. The primary paired future-minus-control mean is **−.02722544** in E units. Seven seeds improve and seed 4 worsens. Future feedback has lower E than reward feedback in six seeds, with reversals in seeds 4 and 5. Reward alone leaves seeds 1 and 4 unchanged and worsens seeds 3 and 7. Both improves seven seeds but also worsens seed 4; in seed 3, future passes while Both remains exploitable. Richer feedback therefore has a substantial conditional effect without uniformly dominating other arms.

| Seed | Control E | Reward E | Future E | Both E |
| --- | ---: | ---: | ---: | ---: |
| 1 | .018304 | .018304 | .000000* | .000000* |
| 2 | .064311 | .048986 | .038257 | .004026 |
| 3 | .028434 | .047742 | .000000* | .005721 |
| 4 | .007283 | .007283 | .020663 | .010298 |
| 5 | .046002 | .028753 | .028978 | .000000* |
| 6 | .053773 | .032194 | .031518 | .008438 |
| 7 | .052676 | .053512 | .000000* | .000000* |
| 8 | .072159 | .014195 | .005721 | .000000* |

An asterisk denotes a certified cycle-level pass within tolerance, not a claim of exact mathematical equality or global equilibrium. All three Future passes—seeds 1, 3 and 7—are native-finished. Both passes seeds 1, 5, 7 and 8, but only seed 5 is native-finished; the other three are censored snapshots. The table retains every planned pair and both censoring categories.

## Supported high prices and remaining weaknesses

The three Future passes settle on the same fixed price pair: approximately **(1.66021, 1.62146)**, above the model's competitive benchmark of approximately 1.47293. Following the learned policy at this cycle offers no profitable sustained departure above tolerance against the rival's committed full response map. This is a meaningful incentive-support success; the high prices cannot be dismissed merely because other baseline controllers were exploitable.

The success is narrower than sequential optimality throughout the game. All seven cycle-passing pairs still contain profitable improvements for some firm in the shared union of states reachable under unilateral departures. None of the 32 pairs passes over all 225 states. Thus the audit supports deterrence from the stated cycle against committed strategies while leaving stronger off-path rationality and robustness claims unestablished.

The fixed original-fork cycle provides another useful distinction. Future seed 3 and Both seeds 5, 7 and 8 pass on their resulting cycles but fail on the original fork's cycle states. Changing the realized basin can contribute to measured relief; these interventions do not repair every history in a strategy map. Shared reachability unions are evaluated for both firms and are not a general closure under arbitrary joint deviations.

A competitive-looking outcome and a supported strategy are also separate measurements. Mean prices in actual observed final 100k tails were 1.789339 for Control, 1.776615 for Reward, 1.672759 for Future and 1.613251 for Both. Corresponding normalized profit gains were .850561, .847195, .563458 and .441315. These tails include actual exploratory play, end at different times and are secondary descriptive outcomes. They neither substitute for the sustained-value test nor establish a competitive limit.

## Timing and feedback costs

Terminal comparisons measure policies yielded by the bounded learner with its native stopping rule. Treatment changes choices, rival responses, visited states and stopping times; terminal outcomes do not represent equal-duration learning. All 32 branches nevertheless reached the planned first 100k continuation checkpoint in this execution. No early-terminal reuse was necessary.

At that common checkpoint, mean E was .053007 for Control, .045552 for Reward, .030362 for Future and .012252 for Both. Future-minus-control was −.02264548. Strict cycle passes numbered 0, 0, 0 and 2 respectively. All these checkpoints were active learning snapshots; their labels describe that time rather than their eventual stopping outcomes.

| Arm | Actual continuation periods | Reward + future component refreshes |
| --- | ---: | ---: |
| Control | 6,555,099 | 26,220,396 |
| Reward | 5,420,352 | 173,451,264 |
| Future | 6,924,904 | 221,596,928 |
| Both | 7,143,955 | 428,637,300 |

The eight shared prefixes add eight million periods. Reward and Future each provide one extra component refresh per unchosen action at a visited state; Both provides two. Their eventual durations differ. Targets needed only for diagnostics are recorded separately from semantic feedback. Across the matrix, including prefixes, there were 68,088,620 firm opportunities, 545,697,908 unchosen Q writes, 419,889,216 reward refreshes and 462,016,672 future refreshes. The result establishes feedback-rule effects under the declared period/stopping budget; it does not establish compute-matched superiority or a cheapest practical remedy.

## Verification and preservation

The temporary helper was built in an isolated worktree from base `989f0689`; its collusion runtime matched the original archived source before the helper hook was added. Private helper commit **`300c9deb35f5e6e23370a8ba5960b84e4e583575`** is preserved and never merged into production. All eight controls match their original complete score tables, recovered full policy maps, current actions, states, periods, outcomes and counts. Original saved pre-update frames are checked during prefix/control replay. Thirty-two serialized fork representations match their corresponding prefix representation before treatment.

The workspace suite passed **3,531 tests, zero failures, 106 existing ignored**. That compiler read eight new helper fixtures before a ninth isolated test was added; all nine final helper tests and six example tests subsequently passed. Formatting, strict core all-target Clippy and release build passed. The retained initial Clippy failure led only to equivalent modulo predicates and test iterators, independently reviewed. Numerical and learning algorithms were unchanged. The Python evaluator passed 27 synthetic tests, including full synthetic matrix validation, repeated-output identity and retained-candidate recertification. A review corrected intermediate checkpoint labels before scientific execution.

The final launcher independently checked 3,170 pinned files, the complete tracked tree, clean tracked source, original inputs, source/schema/protocol, executable and analysis identities before starting a single campaign. Actual matrix exposure was 34,044,310 periods, within the approved 40 million. Matrix execution took 76.77 seconds on the recorded native build; broader compilation/test/evaluation costs are separate. Helper fixtures and the existing authors-code docking tests are separately recorded verification exposure, not additional intervention arms.

Post-run checking covers all hashes, original control identities, cloned representations, native/censored labels, component count geometry, timestamps, sampled pre/post arithmetic, unchanged unvisited rows and observed denominators. **781,020 sampled action transitions** were checked; this does not claim independent reconstruction of every unsaved transition.

The unchanged certified evaluator uses exact native binary64 rewards/gamma, direct Decimal80 learned-policy evaluation plus restricted value iteration, and optimal value iteration plus Howard evaluation. Exact rational Bellman residuals establish errors at most 1e-10 per value and 2e-10 per gap. The maximum observed value-error bound was 1.899e-11. All four candidate arrays are retained. Independent post-run checking linked them to native policies/payoffs and verified **72 checkpoints, 576 candidate arrays and 32,400 firm/state rows**, including witness policies, cycles, closures, population means, paired contrasts and statuses. Both saved-data analyses produced 73 identical outputs plus an identical hash index. These numerical intervals quantify solver error, not sampling uncertainty across seeds.

Local evidence is preserved at `survey/out/collusion-memory1-components-20261011/`: protocol, frozen manifest, original input identities, private source and Git bundle, executable/build identity, failed checks and reviews, all 32 branches, both saved analyses, candidate recertification, independent verifier, summaries and completed tracker. Large files are compressed with decompressed hashes verified. Original archives and other campaigns remain unchanged; only this temporary helper worktree/branch is cleaned up. Initial unavailable native-statistics NaNs serialize as null: checkpoints are audit representations, not complete lossless restart files. Live worlds used native in-memory Clone throughout.

Key SHA256 identities:

- Original input index: `7abacb4441421f056853fa7f0885125752a2d4a7fdfb522b7d084780250aa090`.
- Executable: `11a3d78b31acc02890d166b3ce3ea8af69247272d72f5556088d853487adeee9`.
- Frozen launch manifest: `2cd816f056254ed594744971e5da3867849255ca5dbcdeb7fc372a75ed1f8c2e`.
- Campaign hash index: `0aa954092dd0ae9bd1587ca91e6d5360e81f3a54384358187a921fea482ca23a`.
- Both analysis reports: `fa666609a4b1827768659a6e0936e5ff16ce15a5b8d7e143ae5d07d05ad9e422`.

## Scientific interpretation and next question

The intervention strengthens the selective-feedback account: changing continuation feedback changes learned-controller vulnerability substantially in these cloned states. It also preserves useful counterevidence. Reward learning contributes, some remedies fail or worsen outcomes, and supported high-price cycles can emerge when action-dependent futures are retained. The result qualifies the reliability of the baseline learner without treating every elevated price as spurious or every failed learner as a coding defect.

This does not identify the sole mechanism, establish population prevalence, reproduce a follow-up paper's exact regime or prove a universal cure. Endogenous paths and stopping, finite exploration, the declared R/B accounting, added feedback and remaining off-path incentives limit the inference. The [earlier source assessment](2026-10-10-algorithmic-pricing-scientific-assessment.md) retains the literature and novelty boundaries.

The next useful question is why off-path weaknesses survive: sparse revisitation, outdated continuation targets and changing rivals remain candidates. The saved visitation, ledger and value data can support a further descriptive inquiry. Any new causal campaign, additional seeds, exploration reset or parameter change needs a separately fixed design. No push, publication or author contact occurred.
