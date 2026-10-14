# Negotiable assistance — finite foundation findings

**Observed:** 2026-10-14. **Disposition:** qualified finite analytic agreement for three registered checks; partial completion of reproduction queue item 41, with no milestone number assigned. These are repository research findings, not an externally published account.

The cake example and the registered supplied-history priority algebra agree exactly with independently approved references. Both retained reports return **Holds** for `negotiable.cake`, `negotiable.pareto` and `negotiable.priority`, covering 225, nine and 4,599 rows respectively. The complete canonical mathematical archives are byte-identical. Their raw timed reports differ and both originals remain retained.

The scientific result is narrow but useful: differing beliefs allow a contingent allocation that both principals prefer **in their own expectations**, and likelihood-weighted priorities implement the registered algebra without reviving excluded models or resetting impossible observations. This does not establish realized welfare, fairness, truth calibration, the general sequential theorem, or the paper's gridworld behavior.

## Source, question and prospective scope

Desai, Critch and Russell, *Negotiable Reinforcement Learning for Pareto Optimal Sequential Decision-Making*, NeurIPS 2018, §3, Definition 1, Lemmas 2–3 and equations 3–8: [primary proceedings](https://proceedings.neurips.cc/paper_files/paper/2018/hash/5b8e4fd39d9786228649a8a8bec4e008-Abstract.html), [supplement archive](https://proceedings.neurips.cc/paper_files/paper/2018/file/5b8e4fd39d9786228649a8a8bec4e008-Supplemental.zip). The supplement contains one PDF and no implementation code. Its §1 permits selecting a random policy once before observations and retaining it for the episode, giving the convex-mixture interpretation used here.

The local main PDF SHA-256 is `8eaba6509a3fc39a08905ccadac6ad2cd69c308d9b8cb404248fb6a944a60492`; the supplement PDF is `643655853ddf456513aa430094449db81bb939447ac1ad63bf5b864826b5c140`. Original PDFs, extracted text and source-informed review are retained locally. The [approved design](2026-10-14-negotiable-assistance-design.md) fixed the equations and finite domains; the [foundation plan](../plans/2026-10-14-negotiable-assistance.md) separated engineering verification from later scientific collection. Independent source and numerical approval, including the v2 parser-fix re-review, preceded production. Fresh Task 4 and whole-branch reviews subsequently passed for that foundation. The [collection plan](../plans/2026-10-14-negotiable-collection.md) records the separate user authorization on 2026-10-14 for collector certification, exactly one first/repeat pair, comparison and interpretation.

The reproduced question is how a jointly owned assistant should combine **specified** utilities and different beliefs. This is a source-equation reproduction with a separately constructed exact reference, not original-author-code docking. Critch's [2017 precursor](https://arxiv.org/abs/1701.01302) and Fickinger et al.'s 2020 *Multi-Principal Assistance Games* remain related, distinct studies. Strategic demonstrations and learned preferences in the latter are outside this slice. No novelty claim is made, and no broader literature-priority conclusion is inferred from these checks.

## Cake: subjective agreement and exact ties

Actions are Alice-only, split and Bob-only, with realized utility pairs `(30,0)`, `(20,20)` and `(0,30)`. Alice predicts red with probability `9/10`; Bob predicts it with probability `1/10`. Each evaluates their own utility under their own observation distribution. ID `3 × red_action + green_action` retains all nine deterministic policies:

| Policy ID | Red action | Green action | Subjective expected utilities (Alice, Bob) | Efficient under both conventions |
|---|---|---|---|---|
| 0 | Alice-only | Alice-only | (30, 0) | Yes |
| 1 | Alice-only | Split | (29, 18) | Yes |
| 2 | Alice-only | Bob-only | (27, 27) | Yes |
| 3 | Split | Alice-only | (21, 2) | No |
| 4 | Split | Split | (20, 20) | No |
| 5 | Split | Bob-only | (18, 29) | Yes |
| 6 | Bob-only | Alice-only | (3, 3) | No |
| 7 | Bob-only | Split | (2, 21) | No |
| 8 | Bob-only | Bob-only | (0, 30) | Yes |

Policy 2 improves on always splitting from `(20,20)` to `(27,27)` in these **subjective expectations**. On any realized episode it gives one person 30 and the other zero. There is no single actual cake on which both receive 27. No true data-generating distribution is specified or estimated by this check; a favorable subjective bet therefore does not establish calibrated truth or actual joint welfare improvement.

For every registered initial Alice weight `w`, the check maximizes `w E_A[U_A] + (1−w) E_B[U_B]` over all policies. The domain is the union of `{k/20: k=0..20}` and the four exact boundary weights below: 25 weights and 225 objective cells. Whole-policy and observation-local maximization agree, including every maximizing tie. Alice's local priority is `9w/(1+8w)` at red and `w/(9−8w)` at green; the action thresholds are `1/3` and `2/3`.

| Initial Alice weight | Complete maximizing policy IDs |
|---|---|
| 0 ≤ w < 1/19 | 8 |
| w = 1/19 | 5, 8 |
| 1/19 < w < 2/11 | 5 |
| w = 2/11 | 2, 5 |
| 2/11 < w < 9/11 | 2 |
| w = 9/11 | 1, 2 |
| 9/11 < w < 18/19 | 1 |
| w = 18/19 | 0, 1 |
| 18/19 < w ≤ 1 | 0 |

The interval description follows directly from the exact linear objectives; the collected census contains the 25 registered weights rather than a continuum of observations. The zero/one endpoints are retained. This finite mixture/local-decision agreement does not prove Lemma 3 for arbitrary POMDPs.

## Pareto conventions and stochastic mixtures

`paper_weak` follows Definition 1: another feasible point must strictly improve **both** coordinates to dominate. `strong` separately treats nondecrease of both with at least one strict improvement as domination. These are different definitions. They happen to give identical labels on these nine cake policies: frontier vertices 0, 1, 2, 5 and 8, with 3, 4, 6 and 7 dominated. Synthetic unit fixtures distinguish the conventions, including boundary-only improvement; the cake data do not demonstrate a classification disagreement.

Classification compares each deterministic point with the **entire convex hull** of all nine policy payoffs, including every permitted stochastic mixture. It is not restricted to pairwise deterministic improvement or a sampled probability grid. The nondominated stochastic frontier includes the edges connecting those five vertices. The production segment/crossing method and independent reference hull/clipping method use different exact constructions.

All eight retained dominance witnesses per archive (four dominated policies under two conventions) were independently reconstructed using their rational mixture coefficients: coefficients are nonnegative and sum to one, the resulting payoff equals the recorded payoff, and the named inequalities hold. Positive supporting normals certify each of the five efficient vertices against the whole hull. Some valid witness decompositions differ from the sealed reference, so **full Pareto reference-row byte equality is not claimed**. The payoffs, classifications and independently checked witness meanings agree. This is qualified finite geometric agreement, not a new proof of the paper's general characterization or a claim that Pareto efficiency alone chooses fair bargaining weights.

## Supplied histories: priorities and impossible observations

The registered domain has red-likelihood pairs `(9/10,1/10)`, `(1/2,1/2)` and `(1,4/5)`, complementary green likelihoods, initial Alice weights `{0,1/2,1}`, and every binary history of length zero through eight. Its `3 × 3 × 511 = 4,599` cells are exhaustive supplied histories, not sampled trajectories or 4,599 independent empirical runs. The extra likelihood pairs, prior grid and horizon are engineering fixtures chosen in the approved design, not a claim that the paper ran these experiments.

Both archives retain **4,097 supported** and **502 impossible** cells. Sequential normalization agrees exactly with direct initial-weight times history-likelihood products on every supported cell. For equal initial weights in the cake model, Alice's priority becomes `9/10` after red and `1/10` after green. Equal-belief likelihoods preserve the prior on supported histories. Under `(1,4/5)`, green eliminates the deterministic model if its competitor has positive prior mass. A model given zero prior mass remains excluded.

Zero total observation mass produces an explicit impossible result with the first impossible prefix. Every extension remains unavailable under that prior. There is no epsilon, equal-weight reset or revival. Impossibility is part of the complete output census, not missing execution. These checks support the exogenous-observation specialization of equation 8. They do not evaluate action-dependent evidence, policy recursion or strategic reporting. A later sequential study must condition likelihoods on the intervened action history, rather than treat the agent's own choices as independent evidence about a model.

## Collection identity and observed resources

The collector at commit `07046f43ee03e247ee1f521f5578180b7c86bd59` was invoked once. It spawned the approved absolute binary command `survey/target/release/survey --only negotiable --seeds 1`, once for first and once for repeat, without rebuilding or executing another group. These exact checks use no random draws and ignore seeds; the fixed argument remains recorded. The controller returned zero in 4.87611991725862 seconds, from `2026-10-14T20:31:36.585282+00:00` to `2026-10-14T20:31:41.461658+00:00`.

The source root was `/Users/nathan/Projects/ndouglas/SugarScape` at approved commit `9e35bba81bb099810f253c1cbaae52b5d9168548`. Compiled F5 source SHA-256 was `dede454af44b33817fcf14cbd8688633abfcd1c901e744c95c4ad676a6debf70`, with 1,101 selected inputs. The 23,531,488-byte binary SHA-256 was `c4feba2fc6f554097ae3b33d60abcd84dca30073c18f6a4b96fa0e294898a9fe`. Before/after snapshots matched the reviewed certificate and the independent postcheck verified current selected inputs, source stamp, binary, collector, source PDFs and reference identities. Platform was macOS 15.6.1 arm64, Rust/Cargo 1.98.1 and Python 3.13.5; the receipt preserves the full toolchain and environment.

| Receipt (units explicit) | First | Repeat |
|---|---:|---:|
| Child exit status | 0 | 0 |
| Child elapsed seconds | 0.8145599169656634 | 0.7402115846052766 |
| Direct-child peak RSS bytes (`wait4`) | 223,150,080 | 250,724,352 |
| Sampled process-group peak RSS bytes | 211,353,600 | 236,552,192 |
| Total archive bytes, including status | 2,612,223 | 2,612,231 |
| Raw report bytes | 854,000 | 854,001 |
| Canonical mathematical archive bytes | 709,659 | 709,659 |

Fixed per-attempt caps were 120 seconds, 536,870,912 RSS bytes (512 MiB), and 33,554,432 retained bytes (32 MiB). Preflight certified a 13,918,566-byte packing bound. No failed scientific attempt, favorable retry or observed cap breach occurred. RSS acceptance uses nominal 50 ms process-group `ps` sampling plus final direct-child `wait4` peak. This is observed enforcement, not a hard allocation limit: sampling and `ps` delay can miss transient group peaks, final direct-child overshoot is detected after exit, and a deliberately session-escaping descendant is outside process-group containment. Those limitations constrain resource claims.

| Artifact SHA-256 | First | Repeat |
|---|---|---|
| Raw `report.json` | `0e502d346a35220f20f3b72208083d8dbd393280c5440a3123da21c1b15749b2` | `378afcdb64010c5e81e1af67cf14731a1d1555d16143c723b94a858817910ed7` |
| Complete `canonical.json` | `eaa42360cc1946670d767c0a7a8f2df3468f71122cf093f195faa4baebed3cd5` | `eaa42360cc1946670d767c0a7a8f2df3468f71122cf093f195faa4baebed3cd5` |

Raw-to-canonical mathematical records agree exactly; parsed outer rows also agree after removing only `seconds`. The independent postcheck recomputed every cake objective/tie and supported/impossible priority result, checked complete row censuses, validated all retained Pareto witnesses and verified artifact membership, hashes and sizes. Cake and priority rows equal the sealed fixtures in full; Pareto witness representation is the stated exception. A standalone postcheck initially omitted the canonical format's terminal newline and failed. Its original failure is retained; the corrected UTF-8/newline verification passed without a scientific rerun or archive change. Fresh Stage 1 re-review and Stage 2 retained-evidence review passed.

## Durable evidence and remaining question

The compact [result table](../../../survey/out/negotiable/results.md) and [machine-readable provenance receipt](../../../survey/out/negotiable/provenance.json) are tracked. The receipt was derived read-only from actual retained statuses, canonical records, provenance snapshots and execution checks; it records full artifact paths, hashes, byte counts, resources, source/binary/settings/collector identities and review identities. Its local derivation-script identity is also retained. It intentionally omits the 1,101-entry input list in favor of the complete hashed certificate.

Full originals remain at `/Users/nathan/.config/superpowers/worktrees/SugarScape/negotiable-alignment-design/.superpowers/negotiable-assistance/collection/{first,repeat}/`. Execution receipts and the independent verifier remain under `/Users/nathan/.config/superpowers/worktrees/SugarScape/negotiable-collection/.superpowers/negotiable-collection/execution/`. The receipt is **not a portable raw-evidence bundle**: raw archives, source PDFs and reviews are local ignored files. Accepted archives were created exclusively and remain hash-verified ordinary owner-writable files; they are not protected by an OS immutable flag. Current hashes establish retained identity, not an independent filesystem history audit. The collector removed only its owned legacy staging files after accepted archival and released the lock; both are absent.

Permanent claim entries [RR-NRL-01](../../reproducibility.md#rr-nrl-01), [RR-NRL-02](../../reproducibility.md#rr-nrl-02) and [RR-NRL-03](../../reproducibility.md#rr-nrl-03) preserve the separate cake, hull and priority scopes. The next scientific question is whether an explicitly recovered or prospectively labeled reconstruction of the action-dependent sequential setting reproduces the paper's priority-shifting policy behavior. The map, transitions/boundaries, rewards/termination, weights, horizon/discount, belief points, solver convergence and tie rules need their own source review and authorization. The present supplied histories cannot answer that question. Strategic demonstrations, bargaining, fairness, truth calibration and wider robustness likewise remain unresolved. No new model, external publication, push or merge is part of this collection.
