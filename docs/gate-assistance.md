# Gate assistance: complete measured schedule census

This original bounded construction asks when physically useful assistance also serves each Agent's private payoff. It is not a reproduction of a published learning result. The [registered protocol](superpowers/protocols/2026-10-11-gate-assistance.md) fixes the model and exhaustive calculation; the [research register](reproducibility.md#gate-1) separates physical correctness, private incentives and coordination claims. These are working scientific findings within that registration, without a claim of literature novelty or external validity.

## Model and measurement

Two Agents share P–O–C. The Holder starts at P when distance d=0 and O when d=1; the Retriever starts at O. A token begins at C and must be delivered at O. In each of six rounds the gate closes freely, then the Holder acts, then the Retriever. Hold opens O–C for the rest of that round only when the Holder is at P. Holder Travel costs one, Hold costs k, and Retriever Advance costs one even when blocked. Wait costs zero. Each Agent begins with 32 credits. Delivery pays b to the Holder and 12−b to the Retriever exactly once, ends the episode and leaves unexecuted suffixes uncharged. A deadline without delivery pays zero and retains every actual cost.

The complete Cartesian census contains 729 Holder words × 64 Retriever words = 46,656 raw pairs per stratum, with k∈{1,2,4}, b∈{0,3,6}, d∈{0,1}: 18 strata and 839,808 rows. Raw schedules and payoff ties remain distinct even where realized traces coincide. Execution uses only own schedule, committed local observations/history, own feedback, public clock and termination. The evaluator's complete tables and the peer's private history are unavailable to the controller.

One frozen diagnostic and exactly one repeat each produced 259,691,767 bytes, exit zero and empty stderr; the complete byte strings match. Independent streaming validation checked all 839,808 outcomes/classifications, all 18 analyses and pure-equilibrium memberships, 90 full control histories and 45 contrasts against the sealed independent references. Native `summary_json` freshly validated the retained first report; it did not invoke another diagnostic. The [compact native summary](../crates/sugarscape-core/src/browser_experiments/fixtures/gate-recorded.json) retains every registered stratum, control classification, contrast, cap and receipt. Full raw tables remain in the retained evidence, outside the browser bundle.

- Frozen measurement commit: `6d346653b0d43dc49d75426feaabd5add30d0bcb`.
- Measurement executable SHA-256: `21593bb2aafc4e5fc10326a80ee9f05d42e83891afa0221a639bc2335e047dd4`.
- Raw report-file SHA-256, including final newline: `47aff8b822f4a1191e867f49afe1ab7318c898914bbf2d4823b834bebbdb9e06`.
- Source-bound report-content identity: `9bac206e4f8ac91cc101653ba4b52295526ad3596bdce0e9f85b83a3e2afed02` (a separate domain/path/length-bound codec, not the raw file hash).
- Scientific source identity: `e774ebf0e4590138c435ebee7fe148f2e22c5c4de214e9feea9b8765549dd2b1`, with 22 source and three reference receipts in the summary.

Independently reviewed validator v2 validated the report with unchanged exact scientific integer semantics, sealed references, and frozen scientific sources and settings. Its postmeasurement operational tooling revision is retained in separate engineering evidence. Delivery runtime builds are separately archived and identified; the first measurement remains bound to its preserved measurement executable.

## All registered strata

Payoffs are net credits, and intervals include every pure equilibrium. “Delivering” counts raw pure schedules, without weighting or a selection probability. Delivery-round ranges below describe only delivering pure equilibria; “none” is an empty delivering subset, not missing data. Team selection maximizes group net, then minimizes spending, then delivery time, then schedule IDs. Private equilibrium membership uses only unilateral payoff maxima. Regret is the maximum unilateral payoff gain with the peer's raw word held fixed.

| k | b | d | Pure | Delivering | Holder net range | Retriever net range | Group net range | Pure delivery rounds | Team group net | Team private nets H,R | Team regrets H,R |
|---|---|---|---:|---:|---|---|---|---|---:|---|---|
| 1 | 0 | 0 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 6 | -2,8 | 2,0 |
| 1 | 0 | 1 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 5 | -3,8 | 3,0 |
| 1 | 3 | 0 | 71 | 70 | 0…1 | 0…5 | 0…6 | 4…6 | 6 | 1,5 | 0,0 |
| 1 | 3 | 1 | 12 | 11 | 0…0 | 0…5 | 0…5 | 5…6 | 5 | 0,5 | 0,0 |
| 1 | 6 | 0 | 71 | 70 | 0…4 | 0…2 | 0…6 | 4…6 | 6 | 4,2 | 0,0 |
| 1 | 6 | 1 | 12 | 11 | 0…3 | 0…2 | 0…5 | 5…6 | 5 | 3,2 | 0,0 |
| 2 | 0 | 0 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 4 | -4,8 | 4,0 |
| 2 | 0 | 1 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 3 | -5,8 | 5,0 |
| 2 | 3 | 0 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 4 | -1,5 | 1,0 |
| 2 | 3 | 1 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 3 | -2,5 | 2,0 |
| 2 | 6 | 0 | 71 | 70 | 0…2 | 0…2 | 0…4 | 4…6 | 4 | 2,2 | 0,0 |
| 2 | 6 | 1 | 12 | 11 | 0…1 | 0…2 | 0…3 | 5…6 | 3 | 1,2 | 0,0 |
| 4 | 0 | 0 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 0 | 0,0 | 0,0 |
| 4 | 0 | 1 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 0 | 0,0 | 0,0 |
| 4 | 3 | 0 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 0 | 0,0 | 0,0 |
| 4 | 3 | 1 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 0 | 0,0 | 0,0 |
| 4 | 6 | 0 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 0 | 0,0 | 0,0 |
| 4 | 6 | 1 | 1 | 0 | 0…0 | 0…0 | 0…0 | none | 0 | 0,0 | 0,0 |

Six strata admit delivering pure equilibria: k=1 with b=3 or 6 at either distance, and k=2 with b=6 at either distance. At d=0 each has 71 pure raw pairs, 70 delivering; at d=1 each has 12, 11 delivering. These counts include unexecuted suffix multiplicity. Every stratum also has the all-Wait equilibrium with private and group net zero; it is the lexicographic representative everywhere. Thus existence of a delivering equilibrium does not imply that play will select it. At k=1,b=3,d=1 the delivering Holder earns exactly zero: private indifference still counts as equilibrium.

At k=1, team maximum is 6 at d=0 and 5 at d=1; at k=2 it is 4 and 3. Best-group pure equilibrium reaches those maxima only in the six delivering strata; the other low-cost strata retain gaps 6/5 or 4/3 despite physically feasible assistance. The all-Wait equilibrium has those same gaps even where an efficient pure equilibrium coexists. At k=4 the team representative is all-Wait with group net zero: minimum-cost delivery also yields zero at d=0 but loses one at d=1, and the registered spending tie-break chooses waiting. No delivering pure equilibrium exists at k=4 for the tested reward shares.

## Controls, blocked costs and contrasts

All 90 registered controls are available. `SoloAttempt` uses Holder 0 and Retriever 63: six paid blocked outbound Moves, spending (0,6), private net (0,−6), group net −6, and no delivery at every stratum. Its Retriever regret is 6. Its Holder regret is max(0,b−2k−d); its team gap is team maximum plus 6. The separately maximized solo outcome is zero everywhere because waiting avoids those costs. “Solo attempt” is therefore distinct from “best solo outcome.”

`SuppliedHelp` uses (540,60) at d=0 and (423,30) at d=1. It delivers in rounds 4 and 5, spends (2k+d,4), and pays private net (b−2k−d,8−b); group net is 8−2k−d. Holder regret is max(0,2k+d−b), Retriever regret is zero. These supplied words are physically useful even when the Holder loses credits. They have team gap zero at k≤2 and at k=4,d=0; at k=4,d=1 the team gap is one. `TeamOptimal`, `NashLex` and `NashBestGroup` are explicitly reselected under each stratum; they share the same physical action and information interface. The summary includes each control's profile, both regrets, pure membership, team gap, spending/reward/net, delivery round, all attempt/blocked arrays and availability reason.

Physical delivery is possible in 2,646 raw pairs at d=0 and 468 at d=1, independent of k and b. Thus 44,010 and 46,188 raw pairs respectively finish without delivery. At least one paid blocked attempt occurs in 45,104 raw pairs at d=0 and 46,083 at d=1. These are exact counts over the declared raw schedules, not behavioral probabilities. Blocked Hold/Travel/Advance costs remain charged; deadline non-delivery is a valid completed outcome.

All 45 upper-minus-lower contrasts remain in the summary: 18 holding-cost, 18 reward-share and nine distance comparisons. At fixed k/d, changing b leaves every raw profile's physical delivery, spending and group net unchanged and transfers +Δb/−Δb only when delivery occurs. This exhaustive invariant supports the stated incentive comparison within this finite model. Selector contrasts may change the selected words; they are not fixed-profile causal estimates. Cost/distance change the feasibility and incentives under supplied capabilities; no learning, equilibrium-selection mechanism, unrestricted affordance discovery or real-world cooperation conclusion was measured.

## Browser use and boundaries

Choose Gate assistance in Recorded study results for all 18 equilibrium-range rows, all 90 control outcomes and all 45 contrasts. Export Gate census summary preserves the complete lossless measured projection with integers represented as decimal strings. Selected episode runs and replay leave this retained census unchanged. The separate live default remains **Supplied minimal assistance**, k=1,b=3,d=0. Researcher mode exposes evaluator data; Agent mode keeps private masks, including terminal and comparison checkpoints.

Selected episodes retain the existing bounds: 64 KiB input/link, 16 MiB record, 4,096 checkpoints, one outstanding worker request, 60-second timeout and two retained records. Registered compiler caps are 93,312 profiles, 1,772,280 transitions and 29,098,514 logical live-payload allocation bytes (not actual RSS or allocator capacity). Native/WASM parity establishes the specified computation and presentation boundary, not a broader scientific mechanism. Further learning or selection experiments require a separate prospective design.
