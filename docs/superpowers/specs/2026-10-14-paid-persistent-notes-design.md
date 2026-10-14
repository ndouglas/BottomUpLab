# Paid persistent notes after the writer leaves

**Date:** 2026-10-14

**Status:** written design approved by the user's “LGTM!” on 2026-10-14 after review of commit `e3d48044d7087c243a9044668195be42714b3f2b`. No implementation, independent numerical reference or scientific collection has begun. The implementation plan has its own review handoff.

**Base inspected:** `790b3e647d27c33a207c69f78fc0e575b2e4c539`.

**Builds on:** [Key delegation](../../key-delegation.md), [Shared surface](../../shared-surface.md), and C2/C3/C7 of the [collective-agency program](2026-10-02-minds-collective-agency-program-design.md).

## Question and intended contribution

Can a paid environmental note benefit a later reader after its writer has left, and when does that benefit justify the costs of acquiring and transmitting the information?

This is the next bounded integration slice toward collective research. It combines local access, unique-key handoff, private observation, paid persistent information and separate participant payoffs. It supplies the roles, note meanings, writer participation and reader routines. It does not establish invented communication, voluntary cooperation, distributed discovery, incident fidelity or a novelty claim.

Three approaches were considered. A static information bit with no physical access would largely repeat Shared surface. Hidden resource location would mix information value with search and information gained from failed pickups. The selected task keeps both goods available and hides which one a delivery request requires. This isolates information acquisition from navigation and makes the blank-note fallback explicit.

The retained-note versus erased-note comparison isolates persistence at matched writer cost. A no-note comparison removes writing, and a separate no-acquisition baseline charges the full information-production pipeline. A useful note need not serve its writer's private interest.

## Primary sources and claim boundaries

[Heylighen (2016), *Stigmergy as a universal coordination mechanism I*](https://researchportal.vub.be/en/publications/stigmergy-as-a-universal-coordination-mechanism-i-definition-and-/), supplies the conceptual link between environmental traces and later actions without simultaneous presence. The university's author record and abstract were inspected on 2026-10-14; the linked author PDF timed out. This is a conceptual anchor, not a reproduction of a quantitative result or evidence that our supplied notes self-organize.

[Khetarpal et al. (2020), *A Theory of Affordances in Reinforcement Learning*](https://proceedings.mlr.press/v119/khetarpal20a.html), motivates keeping state-dependent action feasibility separate from information about the task. The official proceedings abstract was inspected on 2026-10-14. This design supplies access rules and implements none of that paper's learned-affordance algorithm.

[Derex and Boyd (2016), *Partial connectivity increases cultural accumulation within groups*](https://www.pnas.org/doi/10.1073/pnas.1518798113), is a subsequent research anchor for communication topology and cumulative innovation, not a result reproduced here. The publisher article was accessible on 2026-10-14; its experiment is outside this two-participant design. Network topology, cumulative research and population replacement remain later increments.

The literature pass supports this limited mechanism question. No new literature novelty or external validity is claimed. Source-specific empirical targets require another study.

## World, task and participants

Use a new sibling laboratory, provisionally `persistent_notes`; preserve the existing Key delegation and Shared surface engines, fixtures, identities and reports.

```
Staging P -- Outside O [note field] -- [carrier-only locked edge] -- Cache C
                                                                  manifest H
                                                                  goods L, R
```

One Writer starts at C carrying the unique key. One Reader starts at O. P remains a named staging location but is unused by the registered routines. Only the key carrier may cross O–C in either direction; the gate never becomes globally open. There is no copying, theft, return or key consumption.

The manifest's request H is L or R with equal exact mass. Both labeled goods are available at C. Their visible presence and ordinary pickup feedback do not reveal H. The Reader can carry one good and deliver it at O. Any delivery terminates the task; correct delivery pays total reward 12, wrong delivery pays zero. There is no second task or postterminal opportunity to use scoring feedback.

H is fixed for the episode. Paid manifest inspection at C reveals it exactly. This prevents a cheap failed pickup from becoming an unintended target-information channel. The single delivery and supplied binary request limit generalization to repeated foraging and search.

Each participant starts with 32 credits. Credits bound spending and are distinct from rewards and net utility. Rewards settle to the original participants' ledgers, including the departed Writer. Departure removes action opportunities; it is not modeled death. Writer share b is 0, 3 or 6; Reader share g = 12−b. The b=0 rows explicitly test a writer with no later reward entitlement.

## Clock and transactions

There are four fixed Writer opportunities, then a single public departure boundary, then at most six Reader opportunities. Roles do not overlap. Wait is free in this laboratory, matching the physical Key delegation convention; the paid waits of Shared surface are not imported.

Writer slots:

1. InspectManifest at C costs s and privately returns H. The no-acquisition control executes Wait here.
2. Move C→O costs one and uses its own key.
3. PassKey costs one and atomically gives the Reader the unique key, with own-possession receipts for both participants.
4. WriteNote(H) at O costs w in the writing arms; otherwise Wait. The supplied encoder writes NoteL or NoteR.

All arms use the same fixed slot order. At the departure boundary the Writer leaves the graph and can take no further action. Its ledger and immutable trace remain. No departure timing depends on H. In the erased-note arm, the field is cleared to Blank as part of this boundary; retain and erase have identical acquisition, movement, handoff and write costs. Departure/erasure carries no in-world credit cost; that convention is reported and is not an estimate of moderation cost. If a verification witness has a failed handoff, departure retains the inactive Writer's key ownership in the ledger; it does not manufacture Reader access or delete the unique key.

Reader primitives:

- ReadNote at O costs r and returns Blank, NoteL or NoteR. It does not move, refresh other observations or expose write lineage.
- Move O→C or C→O costs one per crossing, requiring actual own key possession.
- InspectManifest at C costs i and reveals H exactly to the Reader.
- PickupL or PickupR at C costs one and gives that labeled good if cargo is empty. This does not disclose correctness.
- Deliver at O costs one, consumes cargo, settles rewards once and terminates.
- Wait is free. Unexecuted suffixes have no cost.

For malformed input or overflow, validate before mutation and fail as an operational error. Valid blocked attempts pay their ordinary attempt cost and preserve unrelated physical state. Registered routines have no expected blocks; explicit blocked witnesses are verification cases, not discarded observations. Reader horizon expiry is a valid deadline result with retained spending and zero task reward, distinct from an operationally failed or unavailable record.

## Information and prevention of accidental channels

Topology, roles, prices, shares, budgets, the request prior, the truthful encoder and the four reader routines are supplied. Controllers do not receive H, the arm label, the peer's private history, researcher state, fixture arrays, oracle output or report aggregates, except for the explicitly privileged Oracle target input.

The researcher selects the supplied Writer program before the episode. Its controller receives only that own program, public settings and own observations/feedback; WriteNote chooses its payload from the Writer's paid manifest observation. The Reader receives neither the Writer program nor the arm label. Registered Writer participation is fixed independently of H.

The Writer learns H only through its own paid manifest inspection. The Reader learns H only through its own paid manifest inspection, a nonblank read under the supplied truthful-note convention, or the explicitly privileged Oracle control. Blank is uninformative about H: write participation and erasure are fixed independently of H.

Seeing the board or another Agent does not reveal the field symbol or private request. Notes require ReadNote; the note symbol is not included in ordinary map sensing. Pickup outcomes reveal only which good was picked up. Gross reward feedback occurs after termination and cannot affect this episode's actions. Key receipts, positions, public slots and departure are independent of H before any permitted read/inspection.

Keep participant observations dated to actual sensing. A passive key receipt changes own possession without refreshing its local map. The Writer's action words, note-writing gesture and report lineage are not extra Reader observations. Observer animations and Flump tools are rendering interpretations, not controller inputs.

Do not infer an equilibrium from these controls. Allowing the Writer to condition whether it writes on H could turn silence into a signal and change Reader inference. Selective writing, negotiated payments, learning, false notes and uncertain truthfulness need separate designs.

## Four arms and four registered reader routines

| Arm | Writer acquisition | Writer slot 4 | Field after departure |
| --- | --- | --- | --- |
| Retained | InspectManifest | truthful paid WriteNote | NoteH |
| Erased | InspectManifest | identical paid WriteNote | Blank |
| NoNote | identical InspectManifest | free Wait | Blank |
| NoAcquireNoNote | free Wait | free Wait | Blank |

NoAcquireNoNote is a secondary whole-pipeline control. It changes acquisition as well as transmission and is not the matched persistence contrast. All arms retain the same movement, handoff, departure, Reader budget and horizon.

| Reader routine | Behavior |
| --- | --- |
| BlindGuess | Move C, pick L, return O, deliver; L is the declared tie choice under the uniform request prior. |
| InspectFirst | Move C, inspect H, pick H, return O, deliver. This is a forced inspection control even where inspection is privately expensive. |
| ReadAdaptive | Read at O, then move C. A nonblank truthful note selects H. After Blank, inspect iff i < g/2; otherwise pick L. Return and deliver. Equality chooses the cheaper guess. |
| Oracle | H is supplied privately to the Reader before its first opportunity; move C, pick H, return and deliver. No acquisition charge; an information ceiling, not an information-matched causal treatment. |

ReadAdaptive's blank decision compares expected own reward minus inspection cost under the same private share; all subsequent physical costs are identical. Its initial read is supplied rather than optimized. Thus it is a finite conditional routine, not a globally optimal policy or a learned participation decision.

For ReadAdaptive, a retained note requires five Reader actions; a blank followed by inspection requires six; a blank followed by guessing requires five. InspectFirst requires five, BlindGuess and Oracle four. Four physical Reader credits are spent in every completed route. The six-opportunity horizon therefore permits every registered branch, independent of H.

Optionally summarize the best expected private payoff among the three unprivileged registered routines, maximizing over both equiprobable H values before selecting a routine. Call it the **routine envelope**, disclose that its arm-specific selection is an offline comparison, and retain all ties. It is not a controller receiving privileged arm information or a proof of global optimality. Oracle is excluded from this envelope.

## Proposed finite census and execution limits

Register before collection:

- s ∈ {1,4}; w ∈ {1,2,4,6}; r ∈ {1,2}; i ∈ {2,4,8}; b ∈ {0,3,6}: 144 price/share strata.
- Four arms × four Reader routines × two requests per stratum: 4,608 complete requested episode keys, equally weighted within the two requests.
- No seed selection, stochastic training, policy search, parameter tuning or convergence extensions.
- Exactly one frozen diagnostic and exactly one byte-comparison repeat: at most 9,216 production episode evaluations. Independent precollection references and postrun saved validation are accounted separately, without additional production collections.
- At most 16 captured boundaries per episode; at most 73,728 per diagnostic. Preserve the actual typed event order rather than synthesizing extra waits.
- Proposed operational limits per diagnostic: 120 seconds, 512 MiB peak memory, 512 MiB complete report bytes. Total first/repeat reports ≤1 GiB, excluding separately bounded references/source snapshots. These are proposed caps for review, not previously approved execution or measured performance.

Before collection, a declarative protocol must supply exact ordering, supported wire shapes, deterministic accounting for computational work and logical storage, report limits, reference layout and full resource feasibility. Failure to fit is retained and requires a reviewed amendment; never silently prune prices, requests, routines, traces or failed keys.

All episode outcomes remain separate. Aggregate across H with exact rational weights and report every stratum; equal weighting over the grid is an apparatus summary, not a prevalence estimate. No filters based on success, signs or available statistics.

## Measurements and prospective algebra

Measure separately: acquisition/write/read/inspection costs; attempted and successful primitives; note survival and exact reads; key ownership and receipts; Reader request beliefs and selected good; correctness, completion clock and deadline; original Writer/Reader gross reward, spent credits, net and group net. Report the departed participant in the original cohort rather than only the live Reader.

Costs alone do not establish information transfer. Bind nonblank reads to actual stored symbols and writer observation under researcher-only lineage; controller action changes and request-sensitive selected goods must be supported by their own observable histories.

Primary contrasts pair every unchanged request, stratum and routine:

1. Retained−Erased isolates surviving information with the same producer costs.
2. Retained−NoNote includes the writing cost at matched acquisition.
3. Erased−NoNote tests paid writing without delivery of information.
4. Retained−NoAcquireNoNote reports complete production costs as a secondary contrast.

For ReadAdaptive, let q=1 when i < g/2 and 0 otherwise. After Blank its correct-delivery probability is p₀=1 when q=1 and 1/2 otherwise. These are prospective derivations, not measurements:

| Retained−NoNote quantity | Prediction |
| --- | --- |
| Writer net | b(1−p₀)−w |
| Reader net | g(1−p₀)+q·i |
| Group net | 12(1−p₀)+q·i−w |

Retained−Erased adds w back to the Writer/group expressions because both writing arms already pay it. Erased−NoNote is −w to the Writer/group and zero to the Reader. Correctness improvement is 1−p₀; when blank fallback inspects, notes save costs but add no correct answers.

Against a secondary no-acquisition, no-note baseline that chooses between BlindGuess and InspectFirst by expected Reader payoff, the retained ReadAdaptive pipeline has group gain 12(1−p₀)+q·i−s−w−r. This explicitly charges observation and reading, rather than treating existing writer knowledge as free. Selected-routine comparisons are distinct from fixed-routine causal contrasts.

The grid includes anticipated beneficial, zero and adverse group effects, and writers with negative, indifferent or positive expected private effects. Retain every case. Even positive writer differences between forced arms do not establish an adaptive incentive-compatible writing strategy or equilibrium.

## Verification and pre-mortem

Before production implementation, independently derive request-wise outcomes, costs, beliefs, clocks, all four paired contrasts and the routine envelope. Seal references before seeing production outcomes. Preserve source-informed mechanics review separately from source-blind numerical derivation/review. Use exact integers/rationals; never verify only stored pass flags or final digests.

Required witnesses cover both H values in every arm/routine; zero initial Reader target information in unprivileged routines; truthful note and overwrite/reset semantics; erasure after writing and before the Reader's read; no Writer actions or writes after departure while retained notes survive; fixed public timing; passive key receipt with stale map; wrong pickup identity without truth leakage; wrong delivery settlement; deadline; failed key crossings and wrong-location read/inspection/write; six-action blank-inspection route; i=g/2 tie; inactive Writer rejection; original-cohort accounting; overflow/invalid wire/size-limit rejection before mutation; clone/state/replay equivalence; opaque-ID invariance; strict saved-report reconstruction and tamper rejection. The equality tie is an off-grid engineering witness, not an added census stratum.

Pre-mortem: the encoder, private inspection and truthful channel are supplied; the task has only one bit; fixed writer participation suppresses strategic silence; free departure and erasure omit oversight costs; all routine routes fit the horizon by construction; budgets do not bind; fixed correct baseline policies can make benefits purely cost savings; absent-channel reads can be wasteful; acquiring information when never writing is deliberately wasted in one matched control; a routine envelope changes selection; settling a departed Writer's reward does not model mortality; the chosen reward split affects private inspection incentives; a useful note may be group-adverse after full production costs. These narrow claims rather than being reasons to discard rows.

## Architecture, preservation and visualization

Create a sibling native laboratory with separate types/validation, physical transactions, local observations, supplied routines, exact evaluation, diagnostics/references and strict report validation. Do not modify old scientific engines to implement the new rules or relabel their measurements. Reuse immutable utilities only with preserved behavior and source binding.

Transport inherits bounded native/WASM episodes, one outstanding worker request, fresh import validation, exact integers/rationals and separate live examples versus recorded census. Controls receive only own observation/state/action feedback. Agent perspective must not reveal H, arm, stored field, writer history or full report before authorized sensing. Researcher perspective explicitly labels hidden request, field and lineage.

The existing Flump vocabulary can show Writer inspection, paid writing, departure, Reader field reading or target inspection, key possession and carried good. Two labeled goods and a request card must remain visually distinct from the note. A retained note outlives the Writer; an erased note becomes Blank. Comic panels derive from native boundaries and captions/costs, with no invented private knowledge, intention or dialogue. A decorative medium/pose is not an additional modeled sensor.

Collection, independent postrun verification, native/WASM/browser acceptance, frozen identities, source/executable preservation and local delivery each have separate readiness gates. Push, public publication and contact remain outside this turn's authorization. The first implementation plan will use subagent-driven development in an isolated worktree and must leave other campaigns' trackers, worktrees, archives and original foraging evidence untouched.

## Readiness and next increments

The written design is ready for user review when internal arithmetic, hidden-information boundaries, controls, caps and scientific scope are coherent. Approval then permits the detailed implementation plan. That plan must name staged tests, independent registration/reference authors and reviewers, feasibility gates, source freeze and retained artifacts before judged runs.

Later increments may replace supplied writer participation with an inference-consistent game, introduce uncertain or stale notes, distribute complementary experiments, preserve knowledge across population turnover, add task pressure, and test oversight. None is silently included here. This first study establishes a transparent, falsifiable cost-and-information baseline for those questions.
