# Memory1 pricing: repairing the network of continuation estimates

**Completed 2026-10-11; bounded deterministic operator experiment.** Repeatedly updating one failing situation left its bad decisions intact. Spreading the same score-update budget across all 225 situations repaired the two selected controllers against their original fixed rivals. Allowing both bots to adapt under full coverage produced constant, near-competitive prices and certified all-state best responses against their evolved rival maps. The result establishes sufficiency of these prescribed table updates in two cases; it does not identify the historical cause of the original learning failures.

This follows the [component-feedback experiment](2026-10-11-memory1-component-refresh.md) and its [saved-data examination](2026-10-11-memory1-saved-followup.md), under the [research method](../research-method.md). The user approved eight conditions, a fixed 512-block horizon and at most 27,648,000 score updates. The matrix completed once at exactly that cap. No original training campaign, native trajectory or exploration was replayed, and no extra cases, horizons or tuning were added.

## Question and controlled reconstruction

The saved Both controllers for seeds 3 and 4 favored certified inferior actions at state 16 despite hundreds of thousands of fresh row updates. Their own one-step update targets were already close to their stored scores. The question was whether changing the allocation of updates across the network of successor situations, with and without feedback from an evolving rival, could repair those disagreements.

Each condition starts from an independent exact copy of terminal Both Q bit patterns, finite reward-EMA R, native payoffs and original greedy maps. These are score-table reconstructions, not lossless native-world restarts. No RNG, exploration, native statistics, stopping flags or history is restored. Alpha is native .15 and gamma native binary64 .95; exact lowest-index ties and state encoding `15*a0+a1` are preserved. The source terminal Both update mode is synchronous provenance.

Cross two interventions for each seed:

| Factor | Conditions |
| --- | --- |
| Rival feedback | Original terminal rival map stays fixed; or both evolving greedy maps supply targets |
| Update allocation | Repeat state 16; or visit states 0–224 in ascending order |

Every microstep prepares both firms' 15 action targets from the same prewrite tables, using each action's own successor and its firm's estimated maximum score there. Then Q updates as `Q + alpha*(reward + gamma*prewrite_maxQ - Q)`. The evaluator's optimal values never enter learning. R receives a passive immediate-reward EMA update and B remains the accounting residual Q−R. Ordered binary64 operations match the declared native formula.

Each arm has 512 blocks of 225 microsteps, without early stopping. Thus each gets 115,200 state updates and 3,456,000 Q writes; all eight total **921,600 microsteps and 27,648,000 Q writes**, plus the same number of passive R writes. Local arms revisit state 16 115,200 times. Full arms revisit each state only 512 times. Equal write budgets deliberately allocate experience differently. These microsteps are not native market periods or observed price trajectories.

## Fixed readouts and separate questions

Preserve complete tables at blocks 0/1/8/32/128/512. The primary terminal readout is state 16 selected first-action regret: the best immediate action with optimal subsequent continuation, minus the selected action with that same continuation. Learned-continuation one-shot gain and complete-policy sustained gap Vstar−Vpi remain separate. A bad policy can have downstream mistakes even when its first action is optimal.

Each checkpoint is evaluated against its current target rival and against the original terminal rival. Current targets remain original maps for frozen arms; coupled targets use the evolving counterpart's checkpoint map. Audits freeze those maps only for evaluation. Report all 225 states and both firms, with stationary-equivalent means `(1-native gamma)*(Vstar-Vpi)`, maximum raw gaps and full certified intervals. Every required raw upper bound must be <=1e-8 for a strict pass; a lower bound above that leaves a vulnerability. Numerical failures or crossing intervals remain unresolved. Cycle/global policy judgments use sustained gaps, not just first-action regret.

Frozen mode represents two independent unilateral adaptations against original rivals. Their adapted own maps generally do not supply a jointly responding pair, so a joint-cycle interpretation is withheld where the references are incompatible.

## Terminal results

Actions are zero-based grid indices. Pair entries give firm 0, firm 1. All judgments below use certified intervals, not the rounded point estimates.

| Seed | Rival mode / allocation | Rival used for this test | State 16 selected actions | State 16 first-action regret | All-state sustained-gap test |
| --- | --- | --- | --- | --- | --- |
| 3 | Frozen / local | Original | (6,6) | (.01144244,.01144244) | Vulnerable |
| 3 | Frozen / full | Original | (3,3), independently adapted | (0,0) | Pass |
| 3 | Coupled / local | Unchanged original | (6,6) | (.01144244,.01144244) | Vulnerable |
| 3 | Coupled / full | Evolved | (1,1) | (0,0) | Pass |
| 4 | Frozen / local | Original | (8,7) | (.02779067,.01339945) | Vulnerable |
| 4 | Frozen / full | Original | (4,4), independently adapted | (0,0) | Pass |
| 4 | Coupled / local | Unchanged original | (8,7) | (.02779067,.01339945) | Vulnerable |
| 4 | Coupled / full | Evolved | (1,1) | (0,0) | Pass |

The stationary positive controls succeeded within the fixed budget: full frozen updates repair both selected failures and all-state policy vulnerabilities against the original rival maps. Local updates do neither. Local terminal all-state mean stationary-equivalent gaps remain .00588737 for seed 3 and .01021091 for seed 4. Thus freshness at one row is insufficient under this operator, while adapting the broader table is sufficient in these two inputs. The contrast includes coverage, successor-score adaptation and the prescribed allocation/order; it does not uniquely isolate an historical learning mechanism.

All four local arms have **zero cumulative greedy-action changes** for both firms. Their coupling treatment never activates. Equality of frozen and coupled local results therefore cannot show that an actively changing rival is irrelevant. Full coupled arms do change maps: cumulative changes are (8,88) for seed 3 and (51,11) for seed 4, while terminal changed-row counts are (7,86) and (51,10). Full frozen cumulative changes are (32,87) and (62,17).

Both full coupled solutions choose action 1 in **every state for both firms**. Their checkpoint price pair is approximately **(1.46647,1.46647)**, near the native continuous competitive benchmark (1.47293,1.47293); their joint frozen-policy cycle is state 16. Both pass the sustained best-response test for all states against these evolved counterpart maps. This is a certified property of the resulting finite-model policies, not an observed native market path or a general equilibrium claim.

Those coupled solutions are **not** best responses to the original rival maps. Against the original reference, seed 3's state 16 first-action regrets are (.00856683,.00856683), and seed 4's are (.01245421,.01721759). Their all-state mean stationary-equivalent gaps are .00414555 and .00717993. A policy adapted to a different rival answers a different optimization problem. Current-reference success must not be presented as uniform repair against the original opponent environment.

## Why repeated local feedback can preserve a wrong choice

At the input seed 3 state 16, firm 0's learned score favors action 6 over certified alternative 3 by about .070672. Immediate profit favors 3 by .011442. The saved scores are below the certified action values (immediate profit plus optimal continuation) by about .188221 for the alternative and .106106 for the selected action. Their difference reverses the ranking, while the local score-to-own-target discrepancy is only about 9e-6. The certified optimal continuation values at the two successors coincide in this example.

Repeating only this row fits the same misleading successor estimates more closely: the row's choice stays 6 after another 115,200 visits. Updating successor situations as well can change the decisions. This local accounting and the controlled update-allocation contrast support a concrete explanation of the counterfactual repair. They do not establish when or why the native learner acquired the original estimates, nor identify reward or continuation bookkeeping as the sole historical cause.

Certified policy adequacy also does not require exact score-value convergence. At terminal, full frozen score Bellman sup residuals remain about .000116–.000271, and full coupled current-reference residuals about .000891–.001190, despite all-state policy passes. Scores, action rankings and complete-policy gains are different measurements.

## Earlier checkpoints and finite scope

Full coupled arms first pass both state 16 and all-state tests against evolved rivals at the block 32 readout, and continue passing through block 512. They remain vulnerable against original rivals. Full frozen seed 3 first passes state 16 at block 32 and all states at block 128; seed 4 first passes state 16 at block 128 and all states at block 512. These are first observed passing checkpoints, not exact transition times. At block 128, seed 4 frozen-full has repaired the selected situation but still has a worst all-state sustained gap about .00050958. Every local readout remains vulnerable.

This experiment tests counterfactual sufficiency for two selected saved failures. Exogenous visits, no exploration, all-action writes and ascending sweeps differ from native on-path learning. The result does not establish prevalence, a universal remedy, novelty, or the historical role of changing rivals. The original experiment's meaningful high-price cycle deterrence remains valid for its archived maps; this study adds separate, more comprehensively updated policies.

## Verification and preservation

The temporary helper passed 22 meaningful synthetic tests, Python compilation, and independent task spec/quality and broad source/method reviews before launch. The archived 27-test analyzer baseline passed both in the initially observed NumPy 2.4.6 environment and the explicitly selected NumPy 2.3.2 runtime used for helper tests and execution; this environment distinction is retained. Python was 3.13.5. No maintained Rust/product module changed, and no Rust suite claim is made for this Python-only task.

Independent saved-data checking verified **48 snapshots, 12,312 sampled microsteps and 369,360 sampled action updates**, plus 40 sample/checkpoint intersections, 4,020 self-successor prewrite maxima and 184,680 local untouched-successor maxima. It checked initialization bits, native reward orientation, lowest ties, cache consistency, schedules, count geometry, local unchanged rows, policy-change monotonicity and Q/R arithmetic. It did not replay the operator. Nonself full-coverage maxima at unsaved contexts remain saved operands; sparse records do not independently reconstruct every unsaved transition.

All **96 checkpoint/reference files, 192 firm audits and 768 candidate arrays** reconcile to 36 distinct complete numerical contexts and 144 independently recomputed exact-rational certificates. All audits are certified, with no numerical failures or unresolved audits; the maximum stored candidate value-error bound is about 1.88806e-11, below the inherited 1e-10 limit. Original Decimal80 direct/Howard and VI methods and thresholds remain unchanged. Two saved-only analyses produced **97 byte-identical scientific outputs**; separate runtime receipts are excluded from that comparison. No second score-update run occurred.

Evidence is preserved separately at `survey/out/collusion-memory1-operator-probe-20261011/`: approved protocol, input/runtime identities, complete matrix, both analyses, tests and retained failures, independent verifier/reviews, source, private Git bundle and completed tracker/ledger. Stored and decompressed hashes are indexed. The helper worktree and branch are retired after preservation and bundle restoration; no helper code is merged into maintained product code. Original archives, other worktrees and concurrent campaigns are preserved. No push, publication or external contact occurred.

Key identities: private helper commit `72b920ce913d9a2bfa95d61b0890027cc35851f7`; frozen launch manifest SHA256 `043fba342a2ac18e1b92a385581cf3e8de960219b00e0d90b6b77b9f2fb3aabe`; both analysis reports SHA256 `7ac9ccd460c904ef17db518feca7cd9087e90a59a4bdc3e4613a70f236c6ed8d`. The bounded question is answered; any additional learning campaign needs its own fixed design and authorization.
