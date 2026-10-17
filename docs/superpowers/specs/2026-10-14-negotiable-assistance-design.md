# Negotiable assistance — exact foundation design

**Date:** 2026-10-14. **Status:** user-approved design; finite engineering foundation implemented and required checks passed. Fresh Task 4 and whole-branch reviews passed for foundation commit `2adba597e9298e630b23ced58658d4b9a5c44c21`; their retained reports are identified in the [collection provenance receipt](../../../survey/out/negotiable/provenance.json). At that foundation handoff, scientific first/repeat collection and publication remained uninvoked. The separately authorized later collection is recorded below.
**Program:** [assistance and alignment](../../studies/2026-10-11-alignment-program.md), reproduction queue item 41. No milestone number is assigned here.
**Intent:** reproduce how differing beliefs change a jointly owned assistant's priorities, with an exact, inspectable foundation before the paper's sequential gridworld and strategic-reporting follow-up.
**Approvals:** the user approved this written design and its implementation plan; independent source and numerical v2 reviews and core Tasks 2–3 reviews approved the foundation. The implemented slice contains exact cake, stochastic Pareto and supplied-history checks. Gridworld investigation retains its separate scope and does not begin here.

## Sources and scope

Desai, Critch and Russell, *Negotiable Reinforcement Learning for Pareto Optimal Sequential Decision-Making*, NeurIPS 2018: §3 cake example, Definition 1, Lemmas 2–3, Theorem 4 and equations 3–8. [Proceedings and supplement](https://proceedings.neurips.cc/paper_files/paper/2018/hash/5b8e4fd39d9786228649a8a8bec4e008-Abstract.html).

The existing local main PDF is `papers/alignment/desai-critch-russell-2018-neurips-negotiable-reinforcement-learning.pdf`, SHA-256 `8eaba6509a3fc39a08905ccadac6ad2cd69c308d9b8cb404248fb6a944a60492`. The proceedings supplement is a ZIP containing one PDF, not implementation code; that PDF's SHA-256 is `643655853ddf456513aa430094449db81bb939447ac1ad63bf5b864826b5c140`. Preserve both source versions in ignored engineering evidence before implementation. Supplement §1 supplies the policy-mixture argument; §2 supplies trajectory illustrations.

Critch's [2017 precursor](https://arxiv.org/abs/1701.01302) and Fickinger et al.'s 2020 *Multi-Principal Assistance Games* are related sources, not interchangeable implementations. The latter concerns learning preferences from potentially strategic demonstrations; this slice assumes specified beliefs and utilities. No novelty claim or inference about truthful reporting follows from its reproduction.

Follow [research-method.md](../../research-method.md). Distinguish checking finite instances of an equation from proving its general theorem, and reproducing subjective expected utility from demonstrating fairness or realized welfare.

## Choice and boundaries

Choose a small deterministic analytical module in the existing core and new claims in the existing survey. Pure functions and explicit records are enough for this slice; no new world kind, generic POMDP solver, dependency, UI, WASM bundle or shared rational-library refactor is needed. Keep existing assistance games, defaults, outputs and claim identities unchanged.

Two alternatives are deferred: directly implementing the FrozenLake experiment introduces unrecovered solver/environment choices; starting with the 2020 strategic-demonstration game changes the scientific question before establishing the 2018 foundation.

The future gridworld design must recover or explicitly reconstruct the map, start, transition probabilities at boundaries, goal reward/termination, initial weights, horizon/discount, belief points, solver convergence and tie-breaking. The main paper states a 0.2 unintended-move probability and 331 belief points but does not specify all of those choices. Do not import contemporary Gym defaults, promise its frame-9 turn, or label a new exact solver as the authors' point-based implementation. Figures 1–2 and S1–S2 remain outside this slice's acceptance criteria.

## Cake computation

There are two observations, red and green, and three actions in fixed presentation order: give all to Alice, split equally, give all to Bob. Alice/Bob utility pairs are `(30,0)`, `(20,20)`, `(0,30)` respectively. Alice predicts red with probability `9/10`; Bob predicts red with probability `1/10`. Green probabilities are their complements. Observation likelihoods do not depend on the allocation.

Enumerate all nine deterministic policies, one action per observation, retaining stable IDs and both subjective expectations. Alice's expectation uses Alice's observation distribution; Bob's uses Bob's. Mixing the two belief distributions before evaluating each utility would compute a different objective.

The paper's red→Alice, green→Bob policy has subjective expected utilities `(27,27)`; always splitting has `(20,20)`. Always giving to Alice gives `(30,0)`, and always giving to Bob gives `(0,30)`. These are source-stated or directly derived exact targets, not observations from a new campaign.

For initial weight `w` on Alice, maximize `w E_A[U_A] + (1-w) E_B[U_B]` over all nine policies. The registered weights are the union of `{k/20: k=0..20}` and `{1/19,2/11,9/11,18/19}`: 25 distinct weights, 225 policy/objective cells. Return every maximizing policy, not just the first. Presentation order is not a scientific tie-break.

At red, Alice's normalized priority is `9w/(1+8w)`; at green it is `w/(9-8w)`. Under normalized priority `q`, Alice-only, split and Bob-only actions score `30q`, `20`, and `30(1-q)`. The action boundaries are `q=1/3` and `q=2/3`; ties at those boundaries remain sets. The four initial-weight boundaries above follow from these expressions and must be derived independently in the reference, not simply copied into a production decision table.

Check every registered weight by both whole-policy maximization and observation-local maximization. Their complete maximizing-policy sets must agree. This tests the mixture reduction for this finite game; it does not prove Lemma 3 for arbitrary POMDPs.

## Pareto terminology and stochastic policies

The paper's Definition 1 forbids another policy that makes **both** expectations strictly greater. Label this `paper_weak`: a competing point dominates only if both coordinates are strictly greater. Separately report `strong`: dominance means both coordinates are at least as large and at least one is strictly greater. Preserve both definitions, especially at zero-weight endpoints; never silently substitute the stronger convention in a paper claim.

Include randomized policies through the exact convex hull of the nine payoff pairs. The supplement permits a policy to be selected randomly before observations and retained for the whole episode. For this one-step finite game, this hull also covers behavioral randomization. Do not approximate it with a probability grid or infer stochastic non-domination solely from comparisons with the nine deterministic policies.

For each of the nine deterministic payoff points, independently decide weak and strong domination against the whole hull, with a rational witness when dominated. Keep duplicate payoff points' distinct policy IDs. Weighted maximizers with zero coefficients may be weakly efficient while failing the strong definition; this is a terminology check, not a defect to fix by pruning ties.

## Priority updates and support

For prior weights `(w_A,w_B)` and observation likelihoods `(l_A,l_B)`, form masses `(w_A l_A,w_B l_B)` and normalize by their sum. Preserve zero weights and exact zero likelihoods. If the sum is zero, return an explicit `ImpossibleObservation` result with the input and history; never reset to equal weights, add an epsilon or emit NaN.

The finite engineering reference domain contains three likelihood pairs for red: `(9/10,1/10)`, `(1/2,1/2)` and `(1,4/5)`, green complements; initial Alice weights `{0,1/2,1}`; all binary histories of lengths 0 through 8. That is `3×3×511=4599` unique input/history cells. Histories are exhaustive supplied sequences, not sampled scientific runs. Once a prefix is impossible, its extensions remain explicitly unavailable under that prior, rather than restarting normalization.

Compare sequential updating with direct initial-weight times history-likelihood products for every supported prefix. Equal beliefs preserve prior weights on supported histories. Equal initial weights and the first likelihood pair yield priorities `9/10` after red and `1/10` after green. In the deterministic-versus-stochastic pair, green eliminates the deterministic model when its competing model has positive prior mass. A model excluded by the initial prior cannot be revived by observations.

These exogenous-observation witnesses test the algebra underlying equation 8. They do not model action-dependent evidence or prove the sequential policy recursion. A later action-dependent environment must use each model's likelihood conditional on intervened action history, rather than count the agent's own policy choices as independent evidence.

## Architecture, arithmetic and errors

Add a focused `negotiable` analytical module to `sugarscape-core`, with separate cake, priority and exact-arithmetic responsibilities as warranted by implementation size. A new `negotiable` survey claim group consumes it. Native tests follow the existing pure-function assistance tests; claim output follows `survey/src/claims/assistance.rs` and the survey's claim/runner interfaces. The implementation plan will specify exact files, interfaces and registration edits after inspecting all affected consumers.

All registered scientific targets use reduced rational arithmetic and exact equality. Store signed numerators, positive denominators and checked operations; cross-cancel before multiplication where useful. The implementation must demonstrate an arithmetic bound for this fixed domain. Overflow, a zero denominator, negative probability, likelihood above one and invalid weight normalization are descriptive errors. No panicking public constructor, floating comparison tolerance or unchecked reuse of the assistance module's private `Ratio` is acceptable. Arbitrary-length histories and arbitrary-size fractions are outside the supported domain and must be rejected before expensive work if exposed through an input interface.

Output exact numerator/denominator pairs, policy IDs, tied maximizing sets, both Pareto labels and witnesses, support status and declared scope. Decimal approximations are display-only. Bind claim output to the source version and fixed settings through the existing source/provenance conventions; a new group must not disable existing collector guards or alter unrelated source stamps without exact scoped verification.

## Independent checks and acceptance

Before production implementation, independently derive reference data using Python standard-library `fractions.Fraction`, policy enumeration and a separately implemented rational convex-hull/dominance calculation. Seal settings, all 225 objective cells, nine hull classifications/witnesses and all 4599 support/update cells. A fresh source-informed reviewer checks the paper transcription; a separate numerical reviewer checks the reference. Record their scope and approvals before inspecting production results. Implementation follows the user's subagent-driven preference with independent task review.

Acceptance requires:

- Exact source-example utilities, all nine policy expectations and all 25 complete maximizing-policy sets.
- Exact equivalence of whole-policy and local decisions, including all four rational boundary ties and weights zero/one.
- Weak/strong labels checked against stochastic mixtures, with verified dominance witnesses and zero-weight counterexamples that distinguish the definitions.
- All 4599 finite history cells agree with the independent reference, including impossible prefixes, support elimination and common-belief cases.
- Behavioral rejection tests for malformed fractions/probabilities and arithmetic limits; mutants that swap priors, drop ties or reset impossible observations fail named checks.
- Relevant core/survey tests, formatting and lint pass; unchanged assistance outputs and affected source inventories are preserved through existing checks.

The later plan must predeclare bounded report sizes and execution limits, independent-reference identity, affected checks and exact first/repeat evidence paths. This design authorizes no survey campaign or publication. Deterministic unit/reference construction is distinct from collecting a source-bound scientific report. No forecast from the war work is reused here.

## Handoff

After written review, write a three-to-five-stage implementation plan covering independent registration, the exact core, survey integration and independent delivery verification. Preserve the unrelated root tracker for key delegation. Keep this design branch separate from the crowd and Gridworlds worktrees. Gridworld reproduction and strategic misreporting each require their own later scope and source review.

## Dated collection update — 2026-10-14

The user subsequently authorized collector certification, exactly one bounded first/repeat pair, comparison and interpretation under the separate [collection plan](../plans/2026-10-14-negotiable-collection.md). Fresh collector re-review and retained-evidence review passed. Both accepted reports return Holds for the three finite claims; all 4,833 mathematical rows agree across the pair. The [dated findings](2026-10-14-negotiable-assistance-findings.md), [compact result table](../../../survey/out/negotiable/results.md) and [provenance receipt](../../../survey/out/negotiable/provenance.json) preserve observations and limitations. This update extends the historical authorization boundary; it does not rewrite the original design as collection authorization. Queue item 41 remains partially reproduced, with sequential/action-dependent and strategic work unresolved. No external publication, push or merge is authorized by this update.
