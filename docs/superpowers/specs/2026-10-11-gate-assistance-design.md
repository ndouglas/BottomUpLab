# Costly gate assistance with private payoffs

**Status:** written design approved by the user's “LGTM!” on 2026-10-11. The approved scope is known gate mechanics, individual payoffs, an information-matched team benchmark, explicit assistance costs, and a web visualization shipped with the experiment. The implementation plan is being prepared for its separate review; no production code, numerical reference or measured result exists for this increment.

## Question and scope

When can two Agents obtain a resource through a gate that requires another Agent's costly assistance? Distinguish physical feasibility, profitable individual assistance, and coordination. Jointly profitable retrieval need not uniquely select cooperation.

The first campaign is an exact, deterministic, finite game of committed six-round schedules. It supplies the map, mechanics, role capabilities, rewards, horizon, and schedule vocabulary. Report all pure equilibria of this registered schedule game, including refusal, and compare them with its team optimum. An equilibrium is a normative unilateral-deviation result, not evidence that independently learning Agents converge to it. The scripts are the first bounded controller family, not all possible history-dependent strategies.

This begins C2's gate experiment in the collective-agency program. Keys, boards, negotiation, promised transfers, hidden mechanisms, learned models, repeated interaction, and incident-shaped oversight remain later increments. Existing studies and their first reports remain frozen.

## Physical world

Use a new native gate_assistance namespace and a three-cell graph:

    Plate P -- Outside O --[gate]-- Cache C

P–O is always traversable. O–C is traversable only while the gate is open. The gate is an edge, not an occupiable cell; closing it cannot crush an Agent or erase cargo. One resource token starts at C; delivery occurs at O. The Holder can operate the plate; the Retriever can pick up and deliver the token. These asymmetric capabilities are supplied.

The Holder starts at P or O, according to the registered distance condition. The Retriever starts at O. Two Agents may occupy the same cell; occupancy creates no movement blocking or extra cost. Both know the initial configuration and deterministic rules. Each starts with 32 credits. Paid actions decrement only their owner's credits; the registered maximum possible spending is below this budget. Credits price actions, not elapsed time or computation.

There are six rounds. Each round starts by closing the gate, then schedules the Holder once and the Retriever once. Role order is Holder–Retriever, independent of opaque Agent names. Both slots occur unless delivery terminates the episode. Automatic closure costs nothing. Successful Hold opens the gate for the remainder of that round. Nothing carries gate openness into the next round.

Holder schedule tokens:

- Wait: remain in place; zero credit cost; consumes its scheduled opportunity.
- Travel: attempt the one-edge move O→P; cost one even if blocked. Travel from P is blocked, without moving.
- Hold: attempt to operate the plate; cost k even if blocked. It succeeds only at P. A blocked Hold leaves the gate closed.

Retriever schedule tokens:

- Wait: remain in place; zero credit cost; consumes its scheduled opportunity.
- Advance: one paid primitive action, cost one even if blocked. Its own state determines the primitive: empty at O attempts Move to C; empty at C attempts Pickup; carrying at C attempts Move to O; carrying at O attempts Deliver. Crossing O–C requires an open gate. Pickup requires the unique token at C. Deliver consumes cargo and terminates successfully.

Advance is a supplied shortest-route controller, not an invented action or an invisible multi-action routine. Each event records both the schedule token and the single actual primitive intent/outcome. Holders cannot retrieve and Retrievers cannot operate the plate. No alternative movement, stealing, or transferable ownership rule is introduced.

Validate transactions before mutating state. A blocked attempt pays its declared cost and changes no position, cargo, token, or reward. Credit arithmetic is checked. Configuration errors and implementation resource exhaustion are operational errors, not failed retrievals. Timeout at the end of round six is a valid completed episode with no delivery reward and actual spent credits retained.

Successful delivery pays one fixed bundle of 12 credits: b to the Holder and 12−b to the Retriever. Shares are explicit, public, and paid automatically by the task rule; there is no promise, bargaining, discretionary gift, or enforcement model. Net payoff is the Agent's delivery share, if delivered, minus its actual spending. Group payoff is the sum. Terminal reward is granted once, after the delivery action; remaining schedule tokens are not executed or charged. Public termination closes the gate freely.

## Knowledge and local execution

Initial topology, positions, token location, budgets, role rules, reward shares, and horizon are common supplied knowledge. There is no latent mechanism or private initial signal in this first study. Current observations are local: own position/cargo/credits; current cells within one graph edge, including visible occupants/cargo and visible loose token; gate state only when at O or C. Visibility distance uses the static graph, even when the gate blocks traversal. Seeing another Agent does not reveal its credits, reward, or schedule.

Each Agent receives only its own six-token schedule, local observation/history, own action feedback, and public clock/termination. The schedule index is the public round. The other Agent's schedule and evaluator state are never execution inputs. Historical observations retain timestamps; an unseen current cell is not silently refreshed. Known mechanics can support inference, but inferred state must be labeled separately from observation.

Observe both Agents initially, then observe each Agent immediately before and after its own scheduled action. Peer actions do not trigger an extra observation for the passive Agent. Its view retains the last committed local observation until its next own slot. Public reset/clock/termination and an Agent's own terminal reward settlement are separate events. Read-only browser capture cannot create observations, update memories, or change later decisions. Names are unique opaque labels; consistent renaming changes neither physics nor decisions.

The offline solver evaluates counterfactual schedule pairs against the supplied deterministic initial configuration. It does not inject a realized world or peer history into live execution. Team-selected schedules are computed from exactly the same public initial information, then executed separately with those same local interfaces. The team objective changes payoff aggregation, not sensors, actions, timing, or budgets.

## Exact schedule game and benchmarks

All Holder words of length six over Wait/Travel/Hold are retained: 729. All Retriever words over Wait/Advance are retained: 64. Each stratum has 46,656 raw pairs. Encode words most-significant-round first, with Holder digits Wait=0, Travel=1, Hold=2 and Retriever digits Wait=0, Advance=1. IDs are exact integers. Reject overflow, wrong lengths, unknown tokens, duplicate keys, and malformed inputs.

Run every pair through the authoritative kernel. For each Retriever word, retain the maximum Holder net over all 729 Holder deviations; for each Holder word, retain the maximum Retriever net over all 64 Retriever deviations. A pair is a pure Nash equilibrium exactly when both current private nets equal those maxima. Preserve every payoff tie. Do not introduce secondary preferences into the equilibrium test.

Keep all raw strategies during deviation analysis. Two words with the same realized trace can differ after a unilateral deviation activates previously unexecuted suffixes. Never discard those suffixes or merge strategies solely because a delivered episode ended early. Realized-outcome classes may be reported afterward, with their raw member IDs and no claim of strategic equivalence.

Report the entire pure equilibrium set, counts, delivery availability, and Holder/Retriever/group payoff ranges. Report minimum and maximum group payoff over equilibria and the gap to the team optimum. This reveals coordination multiplicity without positing an equilibrium-selection mechanism. Empty pure sets, if found, remain explicitly empty; no mixed equilibrium or invented representative is substituted. Idle/idle is a preregistered reference check to verify, not a license to skip enumeration.

The team benchmark maximizes group net over the identical 729×64 family. To select one inspectable representative, tie-break by lower total spending, then earlier delivery (no delivery sorts after round six), then lower Holder ID, then lower Retriever ID. The exact optimum claim is limited to this schedule/controller family.

Five registered episode controls per stratum:

1. Solo forced attempt: Holder waits all six rounds; Retriever advances all six. This exposes blocked attempts and costs; it is not the rational solo benchmark.
2. Supplied minimal assistance: near Holder holds in rounds 1 and 3; Retriever advances in rounds 1–4. Far Holder travels in round 1, holds in rounds 2 and 4; Retriever waits in round 1 and advances in rounds 2–5. All other tokens are Wait. This is a declared script, not a selfish decision claim.
3. Team-optimal representative under the declared tie rule.
4. Lexicographically first pure equilibrium, if available.
5. Highest-group-payoff pure equilibrium, with the team representative tie rule within that subset, if available. It is a researcher-selected example, not a claimed spontaneous choice.

Also retain the exact solo best-response value against the all-Wait Holder and both unilateral regrets for every pair/control. Missing equilibrium examples are unavailable, not zero-valued simulated episodes. The browser can select any valid raw pair, including non-equilibria, with its classification and regrets.

## Registered comparison scope

The proposed fixed grid is holding cost k∈{1,2,4}, Holder reward share b∈{0,3,6}, and Holder initial distance d∈{0,1}: 18 strata. Every stratum preserves the same total reward 12, six rounds, 32-credit initial budgets, action costs other than k, graph, and role capabilities. There are 839,808 schedule-pair outcomes and up to 90 registered control trajectories. No seeds, sampling, population inference, price search, or postmeasurement parameter sweep is needed.

The public diagnostic and browser inputs accept only these strata, registered representative selectors or valid raw schedule IDs, and consistent opaque renaming. Horizon, reward total, budgets, graph, timing and capabilities are fixed protocol fields, not adjustable settings. Unknown fields and out-of-census values reject contextually. The default visual example is Supplied minimal assistance at k=1, b=3, d=0, explicitly labeled as a supplied script; it is not presented as the selected individual equilibrium.

Compare reward-share conditions at fixed k and d to isolate incentive allocation while preserving physical opportunity and group reward. Compare holding costs at fixed b and d; compare travel conditions at fixed b and k. Preserve zero, adverse, boundary-indifferent, non-delivery, and multiple-equilibrium cases. Never average over an arbitrarily selected equilibrium or pool the Holder and Retriever roles. Identical opaque-name permutations are metamorphic checks, not additional independent experiments.

Measure delivery and completion round; attempted/successful/blocked primitive actions by role; travel and holding credits separately; total spending, reward, net and group net; unused budgets; full equilibrium multiplicity/payoff ranges; unilateral regret; team gap; and exact control traces. Keep computational work, wall time, memory, and in-world credits distinct. Payoffs and counts use checked integers, with exact fractions only for explicitly defined descriptive proportions.

## Independent registration and verification

Before production measurement, independently review the declarative clock/physics/payoff protocol. A fresh source-blind numerical author receives only that protocol and produces all 18 payoff tables, best responses, complete pure sets, team selections, control histories, and feasibility estimates. A separate numerical reviewer seals its own derivation before seeing the author's output. Expected values are engineering references until collection; no numerical success target is invented here.

Every raw pair's outcome projection and canonical complete-episode hash must match the independent reference. The canonical trace protocol must specify sorted object keys, exact integer atoms, UTF-8 and serialization exclusions before numerical handoff. Compare full typed histories for all available control trajectories and selected deviation/chronology witnesses. Hash equality is not an excuse to omit the independent full-pair comparison.

Feasibility measures the complete native tables, retained report, browser selected episodes, actual Node WASM, and compilation/allocation work. Register explicit caps before judged invocation. If the full design does not fit, retain the engineering evidence and review an amendment before measurement. No sampling, shortened horizon, hidden strategy pruning, or smaller census is silently substituted.

The diagnostic retains all 839,808 raw outcome/hash entries, best-response maxima, equilibrium membership, all strata, control records, counters, source identities and reference receipts. Interning compact outcomes is allowed only if reconstruction is exact and the complete raw-pair mapping remains. The compact recorded browser summary omits full payoff tables and histories while retaining every stratum's metrics, equilibrium ranges, representative identities and provenance. Validate it against the full report.

Freeze protocol, source/toolchain/native/WASM identities, references, census and caps before collection. Fresh independent precollection review precedes a committed binding. Collect an exclusive first report and exactly one repeat, record exits/stderr even on failure, compare bytes, and independently validate the entire census. Preserve failed outputs. Behavior/settings changes after results require an explicitly registered separate series; documentation/summary registration receives a separate delivery manifest. Zero or adverse findings never justify rerunning for a preferred result.

## Native and browser architecture

Implement a sibling native laboratory; do not modify frozen burrow, foraging, caching, deduction, or surface physics. Existing burrow transactions/local observations, GOAP action contracts, and native-backed episode adapters supply patterns, not a claim that their f64 planner or world schema solves this exact game. Prefer the small integer kernel and exhaustive payoff/response tables this question requires.

Extend the existing CLI, WASM episode worker, catalog, recorded-results loader, and Episodes viewer. There is no JavaScript physics or new navigation shell. Run one selected stratum/profile in the browser, not the census. Preserve the existing 64 KiB input/link, 16 MiB record, 4,096 checkpoints, one-request/60-second timeout and two-retained-record limits. Import reconstructs exact current-source native results; cached success flags, edited traces or stale identities do not authorize display as verified.

Show the three-cell map, plate, gate edge, resource, actual movement/holding, role-owned credits and action outcomes. The Agent view shows own dated local history and permissible inference; the Researcher view exposes ground truth, both schedules and terminal evaluation. At every checkpoint, including terminal and comparison, gate-study evaluator payload is Researcher-only. Actual local map data is required for this new laboratory; missing legacy map fields must not be guessed into existence.

Controls expose stratum, representative type or raw IDs, and exact selected words. Fixed-script, equilibrium, and team examples remain distinguishable. Match comparisons on every setting except the declared control/profile contrast. The full-census recorded summary stays unchanged when an episode runs or replays. Capture edits before Run/Share; invalid edits preserve successful retained records. Native authority supplies all integer atoms and event ordering.

Actual browser acceptance binds served source/WASM, worker requests/replies, complete native export/import parity, rendered values at all checkpoints, private/public/Researcher boundaries, matched comparisons, input/share/export, cancellation, timeout, narrow layout, keyboard focus, and reduced motion. Disclose capture limitations accurately. New embedded fixtures must enter affected compiled-source selectors. Authenticate any necessary common identity-guard updates and preserve every existing study's source receipts, exports and original guide prefixes.

## Scientific foundations and limits

[Nash (1950), Equilibrium Points in N-Person Games](https://pmc.ncbi.nlm.nih.gov/articles/PMC1063129/) supplies the finite-game/best-response equilibrium concept. Its original two-page text was inspected on 2026-10-11. This design enumerates pure equilibria of a specified schedule game; the paper's general mixed-strategy existence result does not guarantee pure equilibria or justify selecting a cooperative outcome.

[Khetarpal et al. (2020), What can I do here? A Theory of Affordances in Reinforcement Learning](https://proceedings.mlr.press/v119/khetarpal20a.html) connects feasible actions with planning and learned transition models. Its primary abstract/landing page was checked on 2026-10-11 as a conceptual source lead. This known-mechanics design does not implement its learning algorithm; paper-specific reproduction would require a separate full-methods and critique review.

This is a verified synthetic access-and-incentive baseline. It cannot establish learned cooperation, altruism, reciprocal trust, causal beliefs about unfamiliar mechanisms, human biomechanics, discovered language, exploit sharing, or real incident fidelity. The fixed reward split is an experimental incentive, not an institution or contract model. Future keys/boards and unknown mechanisms should build on the measured physical baseline without upgrading these claims.

## Delivery and approval

Publish corrected results in a new product guide, record stable campaign claim IDs in the reproducibility register, and link the roadmap/paper catalog with appropriate source scope. Report every stratum and equilibrium range, costs, failures and unavailable examples, plus source-bound compact recorded results. Keep implementation debugging history in evidence, not product findings.

Execution uses the existing crowd worktree and branch, fresh subagent-driven implementation and independent reviews under standing commit/merge/push authorization. Inspect Git before edits; preserve concurrent work and the complete prior evidence directory. Never stage .claude/, papers/, survey/out/ or ignored evidence. Format only edited Rust files with child traversal disabled; never run cargo fmt in survey. Run appropriate full project, actual WASM/web/studio and affected collector checks; retain notices/skips precisely. Finish with safe main integration and exact-commit CI/Pages verification.

The user approved this written spec's committed-schedule assumption, timing and zero-cost waiting, public reward allocation, complete equilibrium reporting, census, team information parity and web delivery together. Written-spec approval permits writing the implementation plan. Plan review precedes implementation; independent numerical, feasibility and precollection gates precede scientific execution.
