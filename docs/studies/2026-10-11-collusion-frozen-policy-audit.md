# Frozen history-dependent pricing policies: discounted deviation audit

**Deterministic saved-data study, completed 2026-10-11.** All six saved controller pairs that met the native convergence criterion admit profitable sustained unilateral deviations on their learned cycles. Three of those six pairs nevertheless deter every one-period departure followed by resumption of the learned policy at every cycle phase. High prices and short-lived deterrence therefore coexist with exploitable strategies in these sessions.

This is conditional evidence from eight project-default memory1/asynchronous sessions, not an authors-code replication or a population estimate. Two sessions are horizon-censored snapshots and remain separately labeled. The audit executes the proposal in the [scientific assessment](2026-10-10-algorithmic-pricing-scientific-assessment.md) under the [research method](../research-method.md), following the user's approval. No new stochastic training occurred.

## Question and fixed evaluation

Do the saved history-dependent policies deter optimized sustained deviations, and does that answer differ from a one-period departure test?

Each firm has a deterministic policy over 225 states, encoding the previous pair of price actions. There are 15 prices per firm. Freeze the rival's entire state-dependent policy and disable learning. For each firm, evaluate its own saved policy's discounted value Vπ and solve the optimal unilateral discounted Markov decision problem V*. The next state is the actual current action pair, with firm orientation preserved. The rival can respond to deviations through its saved map; its price is not held constant.

The one-period statistic maximizes immediate profit plus discounted Vπ at the resulting state, minus Vπ at the starting state. The sustained statistic is V*−Vπ. Resuming a policy can enter a different cycle. A cycle-only one-period test can therefore miss beneficial subsequent departures. An exact nonpositive one-period test at every state would instead establish optimality by policy improvement; the present finding does not invalidate that principle.

Seeds 1–8 come from the previously archived [memoryless diagnostic's memory1/asynchronous arm](2026-10-10-memoryless-pricing-diagnostic.md): alpha .15, beta 4e-6, delta .95, Calvano score initialization, project RNG and lowest ties. Six sessions met the native stopping criterion; seeds 3 and 8 reached the two-million-period limit. Frozen evaluation removes residual exploration and future learning, so its conclusion concerns the saved deterministic controllers.

## Results

Both independent numerical methods certified all 16 firm audits. None of the 3,600 state rows has an unresolved one-period or sustained classification. The table reports sustained gains from the archived final state, as percentages of that firm's learned discounted profit. Pair-level one-period safety requires both firms to pass at every state of the learned closed cycle.

| Seed | Native stopping label | Both firms one-period safe on cycle? | Firm 0 sustained gain | Firm 1 sustained gain |
| --- | --- | --- | --- | --- |
| 1 | Converged | Yes | 6.71% | 4.46% |
| 2 | Converged | No | 15.13% | 24.51% |
| 4 | Converged | Yes | 0.00% | 4.52% |
| 5 | Converged | No | 16.58% | 12.26% |
| 6 | Converged | No | 14.64% | 18.00% |
| 7 | Converged | Yes | 19.83% | 13.94% |
| 3 | Horizon-censored | Yes | 0.00% | 17.64% |
| 8 | Horizon-censored | No | 28.57% | 14.49% |

Eleven of the twelve firms in converged pairs have a profitable sustained deviation at the saved final state and every phase of their learned cycle. Seed 4's firm 0 has no improvement above the fixed tolerance there; its rival does. Thus none of the six pairs passes the joint sustained test. Both censored pairs are also exploitable, without establishing what further training would have produced.

Seed 1 gives a compact example. Its learned fixed point is state 175, with action pair (11,10), prices approximately (1.854,1.815). Firm 0's optimized departure chooses action 10 and enters state 160. Its ensuing cycle is 160→96→101→160, with own actions 6,6,10 and the rival's saved responses 6,11,10. This sustained plan improves discounted profit by 0.42074044, or 6.7135%, even though no one-period departure followed by resumption is profitable above tolerance at the original fixed point. Firm 1 can also improve there by 4.4582%. The witness is a feasible alternative strategy against a responding rival, not merely a response plot.

## Off-path support and the existing detector

The retained outputs separately identify the learned cycle, each firm's closure under arbitrary unilateral actions from that cycle, the union of those two closures evaluated for both firms, the closure from the saved final state, and all 225 states. The shared cycle-deviation union contains all 225 states in six pairs, 223 in seed 4 and 224 in seed 5. Every pair has profitable improvements within that union and across the full state space. No pair establishes optimal prescribed behavior throughout its potential punishment states. This union is not a general closure under arbitrary joint changes by both firms.

The archived native `equilibrium.on_path` flag is false in all six converged sessions; censored sessions have no native outcome. The independent sustained test agrees with that stronger native detector. The result is not evidence of a detector implementation bug, nor a fresh verification of the older register's .2% versus 49.7% sample frequencies. The disagreement demonstrated here is between cycle-only one-period deterrence and optimized sustained strategy incentives.

## Numerical certification and provenance

Before empirical evaluation, input validation bound the archived index, manifest and all 24 selected metadata/summary/trace files by hashes and byte sizes. It scanned all 15,122 selected trace frames, recovered all 450 native firm/action/rival payoff entries, and checked repeated entries for exact binary64 identity. The first seed supplies complete payoff coverage by frame 67. All 3,600 saved score rows have unambiguous greedy maxima; the minimum first/second score margin is 3.0407839101798118e-5. Recovered current-state actions match the stored actions.

Rewards and delta use their native binary64 values, including delta hexadecimal `0x1.e666666666666p-1`, rather than recomputing payoffs from current main or silently substituting intended decimal parameters. The fixed absolute classification threshold is 1e-8 discounted-profit units. Each value solution must have error at most 1e-10; the gap bound sums errors and must be at most 2e-10. Interval lower bounds above 1e-8 certify improvement; upper bounds at most 1e-8 certify no improvement above tolerance; crossing intervals remain unresolved.

Vπ uses direct finite-prefix/geometric-cycle evaluation at Decimal precision 80 and an independent restricted-action value iteration. V* uses value iteration and Howard policy iteration with direct cycle evaluation. Exact rational Bellman residuals on native binary64 coefficients and finite candidate representations yield contraction error bounds residual/(1−delta), with no residual-rounding uncertainty. One-period bounds use (1+delta) times the learned-value error. Candidate agreement and caps of 20,000 value-iteration steps and 1,000 Howard iterations are mandatory. The largest certified value error across methods is 1.896e-11. Raw, stationary-equivalent (1−delta) gap, and relative intervals are retained for every state; these are not the paper's Q-loss denominator.

An independent reviewer cleared the method, loader and code before evaluation; 14 synthetic tests passed. Two fresh executions produced byte-identical results. Post-run independent checking re-evaluated the retained direct/Howard values and feasible witnesses against the frozen inputs. The independent value-iteration candidate arrays are not separately retained: their certificates and agreement checks are execution evidence from reviewed code, not additional post-run array recertification.

Local preserved evidence: `survey/out/collusion-policy-audit-20261011/`, including temporary source/tests, frozen manifest, validated inputs, both complete result files, execution logs, summary, independent reviews and post-run verification. The archive is ignored and not published. Key SHA256 identities:

- Original input index: `7abacb4441421f056853fa7f0885125752a2d4a7fdfb522b7d084780250aa090`.
- Validated input bundle: `674d60e4f684120dd6d19013bf4678bddfd0bc6ec82a5d483a6b97dc41c02f47`.
- Temporary solver: `6b407570f47466c4c78be7913f36704c58a243bb982e8f8548fbf809e58f6205`.
- Both result files: `b6ddf2a733a8b4dd48b10f72efe97080dea1480d3eb713dd30d80c2e011b37e3`.

## Scientific interpretation and next question

For these saved sessions, price-history dependence and apparent short-term punishment do not establish incentive-compatible sustained cooperation. Stability under the native convergence rule establishes a stopping condition, not best-response optimality. The finding directly qualifies enforcement claims about these controllers while preserving the original computation and the possibility of genuinely supported strategies in other regimes.

The audit does not identify why each learned score table selects an exploitable policy. It does not establish prevalence under the authors' RAN2/random-tie conventions, invalidate every result in the paper, or prove a unique continuation-learning mechanism. Existing literature and the distinct [memoryless component evidence](2026-10-10-memoryless-component-refresh.md) remain essential context.

The next discriminating question is whether off-path score errors and selective feedback explain these profitable deviations. A later memory1 intervention should preserve each action's own next-state continuation and separate immediate-payoff feedback from continuation feedback, with source conventions and computational costs specified in advance. This study authorizes no further training, publication or author contact.

The subsequently approved [saved-score diagnostic](2026-10-11-collusion-saved-score-diagnostic.md) completed the snapshot follow-up: all action entries had historical updates, while cycle ranking errors coincide with much older and less frequent alternative-action feedback. Latest-update accounting locates continuation and adjustment discrepancies without establishing causality. No new training occurred.
