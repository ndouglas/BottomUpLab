# Transferable keys and private incentives

**Date:** 2026-10-11
**Status:** written design approved by the user's “LGTM!” on 2026-10-12, following review of commit `1d42e54485d9918ef937398c44ecfbccba95e8e5`. The carrier-only direction, physical rules, information boundaries, census, controls, and web delivery scope are approved. The implementation plan is being prepared for separate review; no production change, independent numerical reference, or measurement exists for this increment.

**Builds on:** [Gate Assistance](../../gate-assistance.md), [the approved gate design](2026-10-11-gate-assistance-design.md), and C2 of the [collective-agency program](2026-10-02-minds-collective-agency-program-design.md). The separate Assistance Games paper reproduction remains owned by its own worktree.

## Question and boundary

When does handing over an access tool enable useful assistance, and when does that assistance serve the giver's private payoff? Separate successful transfer, physical delivery, private incentives, and coordination. Preserve non-delivery equilibria alongside cooperative ones.

Introduce one unique transferable key in a new sibling `key_delegation` laboratory. Possession authorizes only the carrier to cross a locked edge. It never leaves the gate open for other Agents. This isolates access delegation before persistent unlocking, boards, revocation, copying, theft, negotiated payments, learning, or unfamiliar mechanisms.

Retain the finite committed-schedule controller family, individual net payoffs, and an information-matched team benchmark. Agents execute supplied raw words; an exhaustive solver classifies the game. No equilibrium-selection or learned-cooperation claim follows from enumeration. The new experiment ships with actual native/WASM episode parity, private and Researcher web playback, and retained measured results.

## Physical world and ownership

Use the same three-cell graph, with labels interpreted explicitly:

    Staging P -- Outside O --[locked edge]-- Cache C

P–O is always traversable. O–C requires the moving Agent to carry the key. There is no global open state and no Hold action in this new laboratory. The original plate-and-gate laboratory, its source identities, settings, first/repeat reports, controls, and browser defaults remain unchanged.

One resource token starts at C and is delivered at O. The Giver can approach the Receiver and pass the key; the Receiver can cross, pick up, and deliver the resource. Both can carry the key, but the Giver cannot retrieve the resource. These asymmetric capabilities are supplied. Two Agents can share a cell without blocking movement.

The Receiver starts at O. Handoff distance `d=0` places the Giver at O; `d=1` places the Giver at P. This is distance to the handoff, distinct from the old experiment's distance to its plate. Cross-study comparisons must name that difference rather than treat the distance labels as interchangeable.

In every main census stratum the key starts carried by the Giver. Its owner is always exactly one of these two Agents. It cannot be dropped, duplicated, consumed, remotely passed, or taken without an executed Pass. A successful Pass changes ownership once from Giver to Receiver. There is no return action in this first controller family. Possession travels with the carrier, including through O–C; delivery ends the episode without a postterminal key-return phase.

The transfer-enabled flag `e` changes only whether Pass is permitted. With transfer disabled, the key and all attempted actions still exist; a Pass pays its ordinary attempt cost and fails without changing ownership. Access checks remain based on actual possession in both conditions. This is the matched delegation ablation.

## Clock, actions, and costs

There are six rounds, Giver then Receiver, ordered by role rather than opaque Agent name. Both start with 32 credits. A slot consumes an opportunity even when waiting or blocked. There is no automatic gate reset, opening, or closing transaction in this laboratory. The public clock marks round/role boundaries and termination.

Giver words have six tokens over:

- **Wait:** zero cost; no physical change.
- **Approach:** one credit; attempt P→O. At O it is blocked and still costs one. The Giver cannot enter C.
- **Pass:** cost `t`, including blocked attempts. Success requires transfer enabled, possession by the Giver, and co-location with the Receiver. The target is the other known Agent. Success atomically transfers the unique key; failure changes no position, cargo, resource, or key ownership.

Receiver words have six tokens over:

- **Wait:** zero cost; no physical change.
- **Advance:** one credit, including blocked attempts. Execute exactly one own-state-derived primitive: empty at O attempts Move to C; empty at C attempts Pickup; carrying at C attempts Move to O; carrying at O attempts Deliver. Both directions of O–C require actual own key possession. Pickup requires the loose resource at C. Deliver consumes cargo, settles reward, and terminates.

Advance remains a supplied route routine. Pass is a physical ownership transaction, not an inferred intention, accepted promise, or message. The Receiver need not spend a separate acceptance action: unsolicited receipt is a proposed modeling choice. The sender alone pays `t`; the recipient gains access without a separate charge. Future consent or bilateral exchange requires another design.

Blocked actions pay their own cost, preserve physical state, and retain a contextual reason. Validate before mutation and use checked credit arithmetic. Maximum Giver spending is 24 credits and maximum Receiver spending is six, so these public budgets do not bind in the registered family. Resource limits and malformed inputs are operational errors, distinct from valid blocked attempts and completed non-delivery.

Successful delivery pays total reward 12: public share `b` to the Giver and `12-b` to the Receiver. Private net is received reward minus actual own spending; group net is the sum. This automatic reward rule supplies incentives without bargaining or promises. Reward is granted once after Deliver, remaining slots are not executed or charged, and deadline at the end of round six grants zero reward while preserving spending.

## Information, observations, and transfer receipts

Topology, role capabilities, initial positions, initial key ownership, resource location, prices, reward split, transfer permission, budgets, and horizon are common supplied knowledge. The experiment contains no latent initial mechanism or secret initial signal.

An Agent receives its own word, own position/cargo/key possession/credits, own action feedback, committed local history, and public clock/termination. The peer's word, credits, private history, and evaluator tables are unavailable to execution.

Local sensing uses static graph distance at most one, even across the locked edge. Visible occupants expose identity, resource cargo, and key possession, but not their schedule, credits, or reward. A visible loose resource is observed normally. The edge's access rule is supplied; an Agent's access depends on its own possession. The key is not an extra global sensor or a readable message.

Both observe initially; each senses immediately before and after its own action. Passive Agents do not receive refreshed maps or peer observations after a peer action. A successful Pass also delivers a narrowly scoped ownership receipt to both participants: clock, counterpart identity, and own possession before/after. This updates the Receiver's own possession immediately, including between its ordinary sensing events, without refreshing its map, peer credits, or private observations. Receipts are part of each participant's committed history, not researcher-only data. Failed Pass produces only the acting Giver's failure feedback, without notifying the Receiver.

Delivery settlement updates each Agent's own reward/net history separately from local sensing; public termination does not reveal peer finances. Observation timestamps remain attached to actual sensing. Label any inferred state separately. Read-only replay, comparison, and browser capture create no new sensing or transfer receipts.

The team benchmark uses only the same public initial information and the same separate local execution interfaces. Changing the objective to group net grants no extra sensors, actions, or private-state access. Opaque consistent renaming must preserve decisions and physics.

## Schedule game and fixed census

Encode Giver words most-significant-round first with Wait=0, Approach=1, Pass=2: 729 words. Encode Receiver words with Wait=0, Advance=1: 64 words. Keep all 46,656 raw pairs per stratum, including unexecuted suffixes. Realized trace equality does not imply equality under unilateral deviation.

Proposed fixed grid:

- Transfer permission `e` in {disabled, enabled}.
- Attempt price `t` in {1, 2, 4}.
- Giver delivery share `b` in {0, 3, 6}.
- Handoff distance `d` in {0, 1}.

These 36 strata comprise 1,679,616 raw pairs. Ordered enumeration is permission, price, share, distance, Giver word, Receiver word, with listed values in the order above. No seeds, sample-based equilibrium claim, adaptive grid, or postresult price search is needed. Disabled-transfer prices remain separate because attempted blocked transfers are priced, even though waiting avoids them.

For every pair calculate both exact private nets, unilateral best-response maxima, regrets, and pure equilibrium membership with all payoff ties retained. Equilibrium uses payoff equality alone, without spending or delivery tie-breaks. Report the entire pure set, delivering subset, counts, private/group ranges, and team gaps; an empty set is explicitly empty.

The team representative maximizes group net, then minimizes total spending, then delivery round, then Giver ID and Receiver ID. Non-delivery sorts after round six. The highest-group pure representative uses the same secondary ordering within the pure set. The lexicographic pure representative is a separate display choice. None is presented as an emergent selection mechanism or an unrestricted history-dependent optimum.

Five main-census controls per stratum:

1. **SoloAttempt:** Giver waits; Receiver advances every round. This exposes locked attempts and is distinct from the exact best solo value.
2. **SuppliedHandoff:** at `d=0`, Giver passes in round 1; Receiver advances in rounds 1–4. At `d=1`, Giver approaches in round 1 and passes in round 2; Receiver waits in round 1 and advances in rounds 2–5. Other slots wait. Keep the same scripts when transfer is disabled, including their paid failures.
3. **TeamOptimal:** the declared representative.
4. **NashLex:** lexicographically first pure pair, or unavailable.
5. **NashBestGroup:** highest-group pure representative, or unavailable.

Add one explicitly labeled **InitiallyEquipped** physical control per stratum: the Receiver begins with the key, the Giver waits, and the Receiver advances in rounds 1–4. Initial ownership differs from the main census, so this is an access/cost ceiling control, not an equilibrium member or matched causal transfer estimate. Validate its complete episode separately and expose the different initial condition in the web view.

Retain all 216 control outcomes or explicit unavailability records. Report 18 enabled-minus-disabled contrasts, 24 adjacent-price contrasts, 24 adjacent-share contrasts, and 18 far-minus-near contrasts: 84 total. Pair conditions by all unchanged grid fields. Distinguish fixed-word outcome contrasts from separately reselected team/equilibrium summaries; selector changes are not fixed-policy causal effects. Do not use the historical holding-gate census as a matched causal comparison.

## Predictions, measurements, and pre-mortem

These are prospective algebraic predictions, not new observed outcomes. They use the proposed rules and do not depend on running production code:

- Disabled transfer with initial Giver possession prevents main-census delivery, regardless of schedules. The distinct InitiallyEquipped control can still deliver.
- Enabled SuppliedHandoff delivers in round `4+d`, spending `(t+d,4)` and yielding private nets `(b-t-d,8-b)` and group net `8-t-d`. Its group benefit is positive throughout the proposed grid, while the Giver's benefit can be negative, zero, or positive.
- All-Wait is a main-census pure equilibrium: a lone deviation cannot induce delivery when its peer continues waiting. Transferring access does not itself solve coordination.
- With transfer enabled, the proposed minimal handoff should be a pure equilibrium exactly when `b >= t+d`, within this grid where the Receiver's delivery net remains positive. With transfer disabled, that supplied pair is not a pure equilibrium: the Receiver improves by waiting instead of paying for blocked attempts. A fresh independent derivation must check necessity, sufficiency, competing schedules, and ties rather than adopting this prediction as a reference constant.
- At fixed permission/price/distance and fixed raw pair, changing share preserves all physical events, spending, and group net. Reward shifts only for delivered episodes. Transfer permission can change later physical traces and is not merely a reward change.

Measure attempted, successful, and blocked actions; transfer attempt cost and Approach cost separately; key-owner transitions; delivery/round; private spending/reward/net; group net; unused credits; all pure memberships, regrets, ranges, and team gaps. Retain exact controls and chronology witnesses. Raw schedule counts include suffix multiplicity and are not behavioral probabilities.

Separate in-world credits, primitive/compiler work, elapsed time, and actual memory. Zero delivery, profitable team actions with unprofitable givers, boundary indifference, refusal, and multiple equilibria remain valid results. A favorable handoff script does not establish spontaneous delegation, generosity, trust, negotiated cooperation, or discovery.

## Independent registration, feasibility, and preservation

Before production implementation, an independently reviewed declarative protocol fixes every event, transaction, transfer receipt, settlement, wire field, canonical exclusion, and report order. Separate source-informed protocol author/reviewer from source-blind numerical author/reviewer. Independently derive complete payoff projections, pure sets, maxima, representative selections, controls, contrasts, and exact trace identities for all 36 strata. Preserve each derivation before comparison.

Before collection, demonstrate complete-census feasibility, resource caps, selected native/WASM record limits, actual Node parity, and real-browser acceptance. The doubled census is an explicit feasibility risk. Register caps and deterministic accounting before judged invocation. If it cannot fit, preserve engineering failures and review an amendment; never silently sample, shrink the grid, prune schedules, or loosen production timeouts.

Bind actual scientific source/reference bytes and toolchain/native/WASM identities. Freeze registered settings, protocol, independent references, ordered census, and caps. Fresh precollection review precedes an exclusive first report and exactly one repeat. Retain stdout/stderr/exits, including failures; independently validate every raw outcome and complete control history. A compact native summary must freshly validate the retained first report and preserve all strata, controls, contrasts, and provenance without another diagnostic run.

Preserve the prior Gate Assistance sources and retained evidence rather than importing its measurements into this new series. Report every revision made after seeing results. A changed scientific rule requires a separately registered series. Product documentation reports corrected findings and provenance; debugging chronology stays in retained engineering evidence.

## Architecture and web delivery

Keep physical execution, schedule evaluation, strict transport, diagnostics, and source binding in the native sibling laboratory; reuse established utilities only without changing frozen scientific behavior. A new bounded browser adapter consumes native results through the existing worker and Episodes shell. JavaScript projects records and renders them; it contains no alternative physics or payoff solver.

Selected runs retain established input/link, record, checkpoint, one-outstanding-request, cancellation, and 60-second worker bounds unless a separately reviewed feasibility amendment is required. Register the key study distinctly from Gate Assistance and Assistance Games. Existing selectors, records, schema identities, generated fixtures, defaults, and golden reductions remain byte-preserved except independently reviewed new-study registration metadata.

Show the graph, locked edge, carrier-only crossing rule, current visible key possession, explicit handoff events, dated observations, paid blocked attempts, and private cost/reward ledgers. Agent playback masks peer finances/words and researcher tables even at termination and in comparisons. Passive receipt updates must remain distinguishable from sensing. Researcher mode exposes the full evaluator trace.

Offer supplied, team, pure, and raw-word examples with honest labels and unavailable reasons. Recorded results expose all 36 strata and 84 contrasts independently of selected runs. Lossless export retains exact integer values and provenance. Selected runs and replay never replace the recorded census. The InitiallyEquipped control prominently shows its changed starting ownership.

Delivery requires native/reference/actual WASM parity, meaningful privacy/chronology tests, appropriate full project gates, fresh independent task/whole review, actual-browser acceptance on final served artifacts, safe integration/push, and exact-commit CI/Pages verification. Use the existing crowd worktree, fresh subagent-driven implementation with independent reviews, and standing incremental commit authorization. Never stage `.claude/`, papers, survey output, or ignored evidence. Preserve concurrent work and all retained evidence. Format only edited Rust files with child traversal disabled; never run `cargo fmt` in survey.

## Primary-source scope and subsequent steps

[Miller, Donnelley, and Karp (2007), Horton](https://www.usenix.org/legacy/event/hotsec07/tech/full_papers/miller/miller_html/index.html), describes transferable bearer authority and separates delegation from attribution. It motivates distinguishing who can act from who performed a handoff. This physical, unique, noncopyable key is an original bounded analogy; it does not reproduce Horton's protocol, cryptography, revocation, or object-reference semantics.

[Khetarpal et al. (2020), A Theory of Affordances in Reinforcement Learning](https://proceedings.mlr.press/v119/khetarpal20a.html), links state-dependent action feasibility with planning and model learning. This study tests supplied feasibility and incentives, without implementing that learning algorithm. Both primary sources were inspected on 2026-10-11; these are conceptual anchors rather than paper-specific reproductions or novelty claims.

After verified ownership and cost mechanics, the next separate design is paid persistent information in the embodied world, building on existing shared/active surface work. [Heylighen (2016), Stigmergy](https://researchportal.vub.be/en/publications/stigmergy-as-a-universal-coordination-mechanism-i-definition-and-/), supplies a conceptual account of coordination through environmental traces. Learned use or repurposing of a channel, unfamiliar mechanisms, collective research, and incident-shaped oversight require subsequent discriminating experiments.

Approval of this written design permits preparing the implementation plan. The user then reviews that plan before subagent-driven execution; independent registration and precollection gates precede scientific measurements.
