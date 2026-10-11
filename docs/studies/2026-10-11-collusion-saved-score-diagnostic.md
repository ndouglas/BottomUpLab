# Saved pricing scores: continuation errors and uneven feedback

**Retrospective deterministic diagnostic, 2026-10-11.** The profitable deviations in the [frozen-policy audit](2026-10-11-collusion-frozen-policy-audit.md) coincide with misranked action scores and strongly uneven feedback. All 54,000 saved action entries had been updated at least ten times: this is not a case of entirely untried actions surviving at initialization. At the sixteen converged-cycle phases with a present action-ranking error, the learned action had received far more updates, much more recently, than the better alternative.

The latest-update records locate the disagreement in continuation estimates and adjustment toward past targets. They do not establish a unique causal explanation. The findings apply to eight archived project-default memory1/asynchronous controller pairs, with six native-converged pairs and two horizon-censored snapshots retained separately. No new training or value optimization occurred.

## Question and comparisons

Why do the saved controllers prefer actions that are inferior against their final rivals? This follow-up uses the same native payoffs, recovered policies and certified discounted values as the preceding audit. Freeze each rival's complete state-dependent policy, and compare each saved action score with two different returns:

- **Qπ:** immediate profit plus discounted return from resuming the learned policy at the resulting state.
- **Q\*:** immediate profit plus discounted return from following an optimal unilateral strategy at that state.

Qπ can justify rejecting a departure that becomes attractive under Q\*. A firm might accurately estimate what happens if it resumes its existing behavior yet underestimate an alternative plan that requires further departures. Thus error relative to Q\* is not interchangeable with error relative to Qπ.

A present chosen-action loss is V\*(s)−Q\*(s, chosen). It measures the cost of the current choice assuming optimal subsequent behavior. The preceding audit's V\*(s)−Vπ(s) includes all subsequent mistakes. A profitable sustained deviation need not require changing the very first action.

## Coverage and action ranking

The diagnostic retains every saved state/action entry: eight pairs × two firms × 225 states × 15 actions = 54,000 entries, with 3,600 state summaries. Each learned action is compared with fourteen alternatives, giving 50,400 comparisons; 21,195 comparisons contain a certified ranking disagreement under Qπ, Q\*, or both. These counts are unweighted finite-state diagnostics, not frequencies of realized play or population prevalence.

The six converged pairs have 22 distinct firm/state observations across their learned cycles:

| Cycle-phase condition | Firm/state observations |
| --- | --- |
| Current learned action is inferior with optimal continuation | 16 |
| Current action passes, but later learned actions permit a sustained gain | 5 |
| No sustained gain above tolerance | 1 |

This preserves the individual success in seed 4, firm 0. For example, both firms in seed 5 pass the present-action test at their saved final state, but their policies contain later mistakes. Censored controllers are analyzed and retained without being counted as converged outcomes.

For all sixteen present ranking errors in converged cycles, the preferred learned action has more explicit historical updates than the optimal alternative:

| Historical feedback measure | Learned chosen action | Better alternative |
| --- | --- | --- |
| Update-count range | 50,957–958,384 | 72–3,493 |
| Median update count | 115,223 | 416.5 |
| Completed periods since last update | 0–2 | 3,378–97,546 |

Counts include explicit updates that leave the numeric score unchanged. Ages use final completed period minus last update period, giving age zero for an update in the final period. The medians concern these sixteen comparisons, not all actions. Every action entry across all eight pairs has positive coverage, but coverage during a changing rival policy is not equivalent to learning an accurate value against its final policy. Uneven counts and ages are descriptive associations; endogenous action choice and visitation prevent a causal inference from this table alone.

## One concrete decision

At seed 1, firm 0's learned fixed point, action 11 prices at approximately 1.854 and action 10 at 1.815. Its rival responds through its full saved policy.

| Quantity at that state | Learned action 11 | Alternative action 10 |
| --- | --- | --- |
| Saved score | 6.265613 | 6.191584 |
| Return if learned behavior resumes, Qπ | 6.267057 | 6.221778 |
| Return with optimized continuation, Q\* | 6.666761 | 6.687798 |
| Historical explicit updates | 755,854 | 3,493 |
| Completed periods since last update | 0 | 11,055 |

The saved scores favor action 11 by 0.074029. Qπ also favors it, while Q\* favors action 10 by 0.021037. Immediate profit favors action 10 by 0.015689; optimal continuation adds another 0.005348 of advantage. Repeatedly following the optimized plan raises the starting discounted profit by 0.420740, or 6.7135%, as the preceding audit established. The single current-action loss and the whole-plan gain are different quantities.

The learned action's score is close to its return under the existing policy. The alternative's historical score fails to reflect the better subsequent plan. In plain terms: the firm mostly rehearses its familiar route, and its valuation of a less-used route misses the gains available from making better choices afterward.

## What the latest updates reveal

The original observer saved every action's latest reward, already-discounted bootstrap, old score, target, actual joint action and update period. Each final updated score reproduces the native update `old + alpha * (target - old)` exactly in binary64 arithmetic.

For each updated action, the difference between its saved score and current Q\* is decomposed into five terms:

1. Saved score minus its last target: adjustment still remaining after that update.
2. The floating-point remainder in forming that target from reward and bootstrap.
3. Last observed immediate reward minus current frozen-rival reward.
4. Last discounted bootstrap minus today's discounted optimal value at the historical next state.
5. The difference in today's optimal continuation between historical and current next states.

This identity is exact bookkeeping on retained finite candidates. The optimal values themselves carry the preceding audit's certified error bounds. Never-updated records would have null historical accounting; none occur in these snapshots.

For the 32 actions involved in the sixteen converged-cycle ranking-error comparisons, the latest observed rival response matches the current frozen response: immediate reward and next-state drift terms are zero for each action. Consequently, a different rival price at the **last** update does not account for these specific score discrepancies. Earlier opponent changes remain unobserved by this latest-step decomposition.

For seed 1, firm 0, the alternative-minus-chosen score distortion is −0.095066. Approximately −0.088223 comes from the difference in stored continuation errors and −0.006842 from adjustment toward the last targets; immediate reward and changed destination contribute zero. Target-rounding effects are negligible here. Across the sixteen comparisons, the historical continuation term has the largest absolute contribution in twelve, and the adjustment term in four. This last comparison is an exploratory descriptive summary of the accounting, not a predeclared causal test or percentage of causally explained behavior.

The frozen Bellman residual additionally compares saved scores with current reward plus discounted best saved score at the resulting state. It diagnoses inconsistency in today's frozen setting. Both rivals were changing during training, so this residual is not the learner's historical update error or evidence that the update rule was implemented incorrectly.

## Method, checks and local evidence

The protocol, fixed count bins (0, 1–9, 10–99, 100–999, at least 1,000) and 1e-8 absolute discounted-profit classification tolerance were specified before measurement. Exact rational arithmetic preserves native binary64 rewards, scores and gamma and the retained decimal value candidates. Chosen-action loss uses inherited error (1+gamma) times the optimal-value error; two-action return comparisons use twice gamma times the respective value error. Boundary-crossing intervals remain unresolved. Descriptive means and medians are point summaries, not interval estimates or statistical inference.

Eleven synthetic tests passed. Independent review checked transitions, bounds, input identities, historical field semantics and scope. Two authoritative executions after the final manifest was frozen produced identical results. Independent checking recomputed all action/state arithmetic, latest-update replay and decomposition, descriptive groups and chosen-relative comparisons against the original inputs. Original pricing archives and the preceding audit were hash-verified and preserved. No production code changed.

An initial launcher hash check used the reviewer's nine-test snapshot after two tests had been added; the check failed, but a later shell line still launched the first execution. The solver and previously written protocol were unchanged and already reviewed. The attempt and explanation are retained; the final eleven tests were verified and actual hashes frozen before the two authoritative repeats. All three executions ultimately produced identical files. This was an orchestration defect, not a parameter revision after observing results.

Evidence is preserved locally at `survey/out/collusion-score-audit-20261011/`, including the protocol, frozen manifest, temporary source/tests, input verification, compressed exact result files, repeat receipts, independent verifier and reviews, summary and completed stage tracker. Original result-file hashes accompany their compressed copies. The archive remains ignored and unpublished.

## Scientific conclusion and next test

These controllers repeatedly update familiar actions while leaving alternative plans assessed through older continuation estimates. This account is consistent with selective-feedback learning failing to discover profitable sustained deviations, and the saved records now locate concrete ranking disagreements. It strengthens that explanation without proving that feedback sparsity is the sole cause or that a practical remedy follows automatically.

The next discriminating test would clone memory1 checkpoints and vary immediate-payoff versus continuation feedback while preserving each action's own next state, with a matched control and explicit computational-cost comparison. Such an intervention could assess causal effects on learning; the present snapshot analysis cannot. Authors-code conventions and broader prevalence remain separate questions. This study does not authorize a new training campaign or publication.
