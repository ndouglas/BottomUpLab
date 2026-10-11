# Surface experiment discovery and browser visualization

**Date:** 2026-10-08
**Revised:** 2026-10-10, after the visualization backfill was delivered.
**Status:** scope approved conversationally on 2026-10-10; revised written spec awaiting review. No implementation plan, runtime change, or new scientific measurement exists.
**Builds on:** [active surface experiments](../../active-surface.md),
[shared surface experiments](../../shared-surface.md), and the
[collective-agency program](2026-10-02-minds-collective-agency-program-design.md).

## Purpose and current gap

The next milestone is an Agent selecting a useful investigation from raw actions
without receiving the successful action sequence. The user also expects these
experiments to be visible and understandable in the web interface. Browser
visualization is therefore a deliverable of this increment.

The completed shared- and active-surface studies already have checked Rust/CLI
diagnostics, product guides, worker-backed WASM episode exports, and Agent and
Researcher playback in Experiments → Episodes. The backfill now exposes fifteen
studies, including P3/P4 caching. Its delivered source is `83727ab`, with exact
CI and Pages verification. Missing visualization was the reason for deferring
this draft; that gap has been closed.

This proposal adds bounded raw probe selection to the existing episode interface.
It retains one investigator and a supplied informed responder. Gates,
keys, shared research artifacts, additional adaptive investigators, and oversight
remain subsequent designs.

## Question and competing explanations

Does choosing individual calibration actions improve downstream task utility
relative to choosing how many rounds of the supplied probe recipe to purchase?

An advantage could come from more useful evidence, a different opportunity to
influence the supplied responder, or avoiding unnecessary investigation. A
changed trace or greater model certainty alone establishes none of those gains.
Zero improvement is an acceptable result. The chosen raw policy could reproduce
the supplied recipe, choose a different recipe, or skip calibration.

Keep three primary policy comparisons under the same initial resources, maximum
clock opportunities, priors, prices, helper knowledge, and live task:

- **RawAdaptive:** choose each owned calibration action and whether to purchase
  another round, optimizing expected investigator net task utility.
- **SuppliedAdaptive:** purchase rounds of the original calibration recipe, with
  the same optimal live continuation. This is a restricted-action comparator.
- **NoProbe:** stop before calibration and use the same optimal live continuation.

Two additional controls provide interpretation: **KnownRaw** gives the raw
planner the matching in-family point prior at time zero; **InspectOnly** stops
before calibration and uses the retained inspection routine on every live trial.
KnownRaw has the same physical action opportunities as RawAdaptive, with its
additional information explicitly labeled. InspectOnly is a benchmark for
trusted target observation, not an accuracy guarantee after an episode fails.
No policy, horizon, prior, or price sweep may be added after seeing measurements.

## Fixed census and comparisons

Primary collection is exhaustive, not a sampled seed ensemble:

- Four in-family mechanisms × two investigator roles × three primary policies
  × all 256 equally weighted complete four-trial bit sequences: 6,144 episodes.
- KnownRaw and InspectOnly over the same mechanism/role/sequence grid:
  4,096 control episodes, for 10,240 in-family episodes in total.
- DataFlip × both roles × RawAdaptive, SuppliedAdaptive, NoProbe, and InspectOnly
  × all 256 sequences: 2,048 separately labeled transfer episodes.

The total is 48 settings and 12,288 episodes. KnownRaw is excluded from transfer:
there is no matching DataFlip hypothesis in the supplied catalog and no oracle
control may quietly supply it. A complete task sequence identifies an episode;
no additional seed or randomized policy is introduced.

Report RawAdaptive minus SuppliedAdaptive, RawAdaptive minus NoProbe, and
SuppliedAdaptive minus NoProbe separately for each of the eight in-family
mechanism/role strata. Report the uniform-prior expectation separately by role;
do not pool roles or substitute that expectation for actual-mechanism outcomes.
KnownRaw and InspectOnly are reference endpoints. All declared strata, zero or
adverse differences, failures, and computation limits remain in the census.

Freeze this census, tie rules, the exact output/metric definitions, independent
references, and resource limits in a reviewed protocol before judged production
collection. Exact averages over the complete finite sequence population need
no sampled-seed confidence interval or significance test.

## Bounded world and decision contract

Retain the four physical mechanisms, original A/B slot ownership, up to three
calibration rounds, four live trials, 48 initial credits per Agent, fair independent
task bits, reward 12 per correct prediction, and the original read/write/wait and
inspection prices. Evaluate both investigator roles.

The modeled mechanisms remain SharedPersistent (one retained field),
SharedResetting (one field cleared at round boundaries), PrivatePersistent
(separate retained fields), and Inert (writes accepted without changing a field).
All clear fields at phase/trial resets. Read/write/wait cost one; trusted
inspection costs four. Each purchased round retains four slots, two per Agent,
so each Agent pays two credits per round.

At each calibration round boundary the investigator chooses free Stop or commits
to the next full round. Stop remains public; skipped rounds have no actions or
charges. Within a purchased round, at each owned slot RawAdaptive chooses Read,
Wait, or Write of one of the six nonblank symbols. All these actions cost one.
There is no target inspection during calibration and no stop halfway through a
purchased round. The responder retains its explicitly supplied calibration rule
and known-mechanism live controller.

Live choices remain the completed study's Inspect/Attempt routines and optimal
continuation. This first increment discovers calibration sequences, not arbitrary
live communication, symbol meanings, or helper behavior. The alphabet, helper
responses, encodings, hypotheses, and task relevance remain supplied knowledge.
Public-facing labels must describe this boundary accurately.

The investigator receives only its own prefix, current private bit when issued,
own prior, own credits, and public clock. Actual mechanism, future bits, peer
history, write lineage, and realized reward feedback remain evaluator data.
Own action selection is an intervention, not another likelihood factor.

Joint exact inference and backward induction retain original candidate identity
and full relevant state. New calibration action choices require a new compiled
policy and policy-conditioned responder replay. The old fixed or active replay
cannot be reused with the wrong partner policy. Stop on a genuinely unsupported
local history, retaining charged costs and partial observations; unavailable
terminal outcomes remain unavailable.

Fix action ties before reference enumeration: Stop before Continue at round
boundaries; Wait, Read, Write(Probe0), Write(Ack0), Write(Probe1), Write(Ack1),
Write(Data0), Write(Data1) at owned paid calibration slots. Blank writes remain
excluded. Live ties retain Inspect before Attempt; prediction ties retain zero.
Alternative values follow the declared policy's continuation, not a silently
stronger continuation for restricted controls.

The responder's physical calibration rule and known-mechanism live action rule
are identical across the primary policies. Its target prediction replays the
declared investigator policy using candidate-local histories and issued bits;
it receives neither the investigator's private prefix nor future bits. Replay
includes raw decisions at owned slots and role B's live boundary after an
unobserved role-A action. Old fixed-policy likelihoods cannot replace that replay.

## Identifiability and unfamiliar mechanisms

Before production collection, derive which candidate mechanisms can be
distinguished by the permitted actions and horizon, for each role. Retain
indistinguishable equivalence classes. Do not require every hypothesis to become
identifiable or reward confidence without considering task utility.

Retain DataFlip as a separately labeled out-of-catalog transfer case. It already
inverts writes of Data0/Data1 in the physical World implementation. The old
calibration recipe did not write those symbols; RawAdaptive can. Consequently,
the previous recipe's calibration likelihood equivalence does not automatically
hold for this new action set. Derive the new likelihoods independently, preserve
unsupported histories, and do not expand the Agent's model after observing a
failure. Discovering a contradiction is distinct from identifying the inversion
rule or acquiring a successful repair.

Primary exact in-family comparisons and secondary transfer remain separate.
Compare policies on matched actual mechanism, investigator role, and complete
bit sequence. Report investigator, responder, and group outcomes separately;
the objective optimizes the investigator alone.

Every transfer setting reports completion/failure mass, failure owner and first
unsupported prefix, charged partial costs, available observations/predictions,
and nominal beliefs. True-model probability and identification are unavailable
for DataFlip. Terminal reward, utility and accuracy remain unavailable when the
episode stops before their definition; no zero reward is invented. Do not
average successful survivors or form terminal paired differences where either
endpoint is unavailable. Report corresponding unavailable mass and partial costs
across the full declared population. Contradiction does not add a new hypothesis
or establish a successful exploit repair.

## Browser experience

Add **Surface experiment discovery** to Experiments → Episodes, reusing the
surface renderer, worker client, timeline, retention, comparisons, links and
export/import boundary. Shared/active-surface entries remain independently
selectable and compatible. Keep existing typography and controls; show the two
Agents, field and chronological interaction without another navigation shell.

The viewer supports:

- Selecting study, mechanism, investigator role, policy, and complete bit sequence.
- Play, pause, step, reset, and timeline navigation through a deterministic episode.
- Seeing writes, reads, round resets, public stopping, trusted inspection,
  predictions, and charged credits at the exact event where they occur.
- Viewing an Agent's local observations and model/target beliefs at the selected
  checkpoint, with chosen action and available alternative values.
- Comparing RawAdaptive, SuppliedAdaptive, and NoProbe on the same episode input.
- Opening summary results separately from a single episode's outcome.
- Exporting a bounded episode and sharing its reproducible input selection.

Default to an Agent perspective. A clearly labeled **Researcher view** can reveal
the actual mechanism, private states, and write lineage. These display data never
reach the controller. A perspective switch must not change actions or results.
Do not reveal future observations in earlier timeline positions. Show a supported
wrong prediction differently from an unsupported-history termination.

Keep the completed active-surface entry available beside the new study.
SuppliedAdaptive must reproduce its original Adaptive behavior and outcomes
under the same declared inputs; it does not rewrite that measured engine.
Verify complete original payloads, clocks, histories, decisions and costs against
retained first-report/reference projections. Measured artifacts stay unchanged.

The viewer operates on a single bounded episode and compact summaries; it does
not download or parse a 250 MB or multi-GB full diagnostic to display one trace.
Rust supplies policy, inference, actions, and checked exports through WASM in a
worker. TypeScript renders those exports and handles navigation. Avoid a second
physics or inference implementation in the frontend.

Separate running from replaying. Playback advances an already checked timeline;
running produces that timeline through Rust. A worker can be terminated to cancel
an expensive request. No long synchronous planner call runs on the page thread.
Retain 64 KiB input/decompressed-link, 16 MiB episode, 4,096 checkpoint, one-worker
request, 60-second timeout/cancellation, and two-retained-record limits. A browser
run selects one complete sequence and the fixed horizon; it does not collect all
12,288 episodes or ingest full diagnostics. Lossless integer transport and fresh
exact reconstruction remain import authority. No new float tolerance is added.

Compute time, memory, state/branch counts, policy/episode serialized sizes and
cancellation require a reviewed precollection feasibility gate. The protocol
fixes native allocation/state/branch limits before judged collection, informed
by independent reference feasibility and labeled engineering measurements.
Resource exhaustion is computational failure, not unsupported model evidence or
an episode outcome. If exact planning cannot fit the browser limits, review a
source-bound policy-table delivery or explicit design amendment. Do not silently
shorten the horizon, prune actions, or substitute an approximation. A browser
example is not the complete scientific census.

Keyboard controls, readable labels, responsive layout, visible focus, and reduced
motion are required. A visual mockup is useful when refining the viewer layout;
the visual companion need not run for the textual scientific design discussion.

## Verification, measurement, and preservation

Use fresh implementers and independent reviewers for approved implementation.
An independent numerical author derives raw-action policies and values before
exposure to production source or measurements. Retain independently derived
identifiability classes, exact initial values, continuation checks and complete
declared comparison projections. Cover role clocks, private choices,
local information boundaries, paid transitions, action-conditioned inference,
policy-conditioned responder replay, ties, and exact identifiability.

Confirm SuppliedAdaptive reproduces the original Adaptive policy. Independently
establish its prescribed-action embedding in RawAdaptive under identical
responder physical behavior, clocks and information. Under that embedding the
raw optimum cannot be worse in its well-specified expected investigator objective;
check both roles with exact fractions. Policy-conditioned responder predictions
remain explicit in responder/group outcomes. If the embedding or equal-responder
assumptions fail, resolve the protocol before collection rather than infer
dominance from a larger menu. Expected-objective dominance guarantees neither
improvement for every actual mechanism/sequence nor robust transfer.

Verify native/WASM parity on complete bounded episode exports, including failures
and perspectives. Web tests check timeline order, resets, comparison input
matching, privacy display, cancellation, unsupported outcomes, and exports.
Perform browser visual and keyboard verification of actual rendered episodes.

Register the fixed census above, priors, tie ordering, source/toolchain identities,
resource limits, metrics and pre-mortem before judged collection. Independent
numerical review, native/WASM/browser feasibility and source-preservation gates
must pass before the sole scientific invocation. Engineering runs prove
correctness/feasibility and cannot provide a favorable collection subset. Freeze
the reviewed source and retain first/repeat outputs exclusively. Preserve prior
diagnostic bytes, source namespaces, and product guide prefixes. Revisions after
measurements require retained originals and an explicitly revised series.

Report exact costs, accuracy, net utility, selected action sequences, posterior
calibration, identification/equivalence classes, failures, and computation.
Distinguish investigation evidence from effects on the supplied partner. Report
all declared comparisons even when no raw-probe advantage appears. Describe
in-family target-posterior calibration over the complete sequence population;
unsupported/out-of-catalog values remain explicit, not clipped or assigned false
true-model probability. Changed traces, beliefs or partner predictions alone
establish no net benefit.

Add a new versioned study namespace and source-bound references. Legacy engines,
exports, measurements and scientific guide prefixes remain protected. Common
module additions may require current-workspace identity guard updates for other
viewers: record before/after hashes and preserved original exports, retain old
receipts, and reject incompatible old imports contextually. Never relabel a
historical measurement with a new source identity or overwrite its first report.

Use fresh subagent producers, independent task/whole-branch reviews, scoped
edited-file Rustfmt and normal project gates in the existing crowd worktree.
Preserve unrelated work and evidence; never stage .claude/, papers/, survey/out/
or ignored evidence. Commit working increments, then safely merge/push completed
work and verify exact-commit CI/Pages under standing authorization. An owned
implementation tracker must not replace another campaign's tracker.

## Scientific anchors and claim boundary

[Khetarpal et al. (2020)](https://proceedings.mlr.press/v119/khetarpal20a.html)
provides an affordance and transition-model framework.
[Wang, Wang, and Powell (2016)](https://proceedings.mlr.press/v48/wangb16.html)
motivates selecting costly observations for subsequent decision benefit.
[Pacheco and Fisher (2019)](https://proceedings.mlr.press/v89/pacheco19a.html)
studies sequential information-reduction objectives, useful for distinguishing
information gain from the task-utility objective selected here. These
primary-source landing pages and abstracts were checked again on 2026-10-10.
Our solver is exact finite inference and
backward induction, not their learned-affordance, knowledge-gradient classifier
or variational-information algorithms. No paper-specific replication or numerical
target is claimed; later such claims need a methods review and separately
registered comparisons.

This is a supplied-model, bounded action-selection experiment. It does not
establish learned language, spontaneous helper incentives, unrestricted exploit
discovery, collective research, or incident fidelity. It prepares the next
gate/key/persistent-surface design with visible, inspectable individual behavior.

## Approval and next artifact

The user approved the conversational raw-probe scope on 2026-10-10. This revised
written spec requests review of the census, comparison assumptions, transfer
failures and viewer reuse together. Calibration action selection is the new
freedom; helper behavior, model catalog, alphabet, encodings and live routines
remain supplied. Written-spec approval permits a staged implementation plan
using the standing subagent execution method. Implementation follows plan review;
registered protocol and numerical/feasibility gates precede collection. No plan,
new experimental result or runtime change exists yet.
