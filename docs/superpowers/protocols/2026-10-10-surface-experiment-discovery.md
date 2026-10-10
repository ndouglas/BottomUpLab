# Surface experiment discovery: declarative protocol 1

**Date:** 2026-10-10. **Status:** source-informed declarative handoff;
independent numerical derivation, review, and feasibility registration are pending.
This document contains rules and output contracts, not measured findings, raw
optimal values, derived identifiability classes, or registered numerical caps.
Production implementation and judged collection require the remaining Task 1
gates in the [approved implementation plan](../plans/2026-10-10-surface-experiment-discovery.md).
The [approved design](../specs/2026-10-08-surface-experiment-discovery-design.md)
sets the scientific scope. Protocol version is `1`; diagnostic version is
`surface-discovery-diagnostic-v1`.

## Independence and supplied information

The declarative producer inspected the original physical types, World, supplied
controller, and active chronology, local records, replay, evaluation, and metric
contracts. Read receipts and input hashes are retained in the owned ignored
`protocol-provenance.json`. No measured report, numerical reference fixture,
production planner, or raw optimum informed this handoff. The numerical author
receives the approved design and reviewed declarative handoff, without runtime
source, legacy/new measurements, or copied fixture values. The author must retain
all input/read receipts, independently implement physics, inference, planning,
and replay, and seal outputs before exposure to production implementation.

The alphabet, four candidate mechanisms, helper's calibration recipe and live
controller, meaning of Data0/Data1, prices, fair-bit task distribution, and live
routines are supplied knowledge. The new freedom is choosing owned calibration
actions. It does not discover a language, helper incentives, arbitrary live
communication, a new hypothesis, or an inversion repair. One investigator acts
against a supplied informed responder; both physical roles are evaluated.

## Physical contract and private information

Roles are `A`, `B`; array order everywhere is `[A, B]`. Default opaque identifiers
are `{"surface":"status-field","agents":["Agent-A","Agent-B"]}`. All identifiers
must be nonblank, and Agent identifiers distinct; valid renaming has no behavioral
effect. Mechanism order is `SharedPersistent`, `SharedResetting`,
`PrivatePersistent`, `Inert`. Symbols are `Blank`, `Probe0`, `Ack0`, `Probe1`,
`Ack1`, `Data0`, `Data1`; Blank is a reset value and cannot be written.

| Actual mechanism | Write effect | Own Read | Between rounds |
| --- | --- | --- | --- |
| SharedPersistent | Replace one shared field | Current shared symbol | Retain |
| SharedResetting | Replace one shared field | Current shared symbol | Clear after slot 4 |
| PrivatePersistent | Replace writer's private field | Reader's private symbol | Retain both |
| Inert | Accept without changing any field | Blank | Remain blank |
| DataFlip (transfer) | Shared replacement, invert only Data0↔Data1 | Current stored shared symbol | Retain |

All fields and write origins clear on entry to calibration (only if its first
round is purchased) and at the start of each live trial. Reset never erases
transcripts, credits, earlier issued bits, or controller memory. DataFlip leaves
Probe/Ack symbols unchanged; its physical behavior is absent from the catalog.

Every paid action returns an own `Event`: position, chosen action, outcome, and
own credits remaining. Read returns a symbol; Write returns generic `Accepted`,
even for Inert or an inverted symbol; Wait returns `Waited`; trusted inspection
returns the current own target bit. No acceptance result discloses mechanism,
field contents, or write origin. Read/Write/Wait cost 1; inspection costs 4.
Prices apply even to uninformative observations and to the observation that
first contradicts the catalog. Initial credits are 48 per Agent. Illegal phase,
wrong owner, blank write, missing target, or insufficient credits must fail before
changing World, histories, credits, or clock.

For sequence index `s` in `0..255`, trial `t` in `0..3`, A's issued bit is bit
`2t` of `s` and B's bit is bit `2t+1`. These eight bits are independent and fair;
each complete sequence has mass 1/256. A predicts B's current bit; B predicts A's.
No bit is issued during calibration. At live entry each Agent receives only its
own current bit, immediately after its own Reset record; no future bit is issued.
Evaluator storage of the complete sequence is never a controller input.

Controller inputs are own prefix, own prior, issued current private bit, own
credits, public clock, and belief derived from that information. Actual mechanism,
peer actions/outcomes/credits/private choices, future bits, write lineage, and
realized rewards remain evaluator data. Public opportunity order is observable,
not the realized peer action. Own action selection is an intervention, not a
likelihood factor. Free public stopping is observable evidence. Private round,
slot, and live selections remain only in the investigator's prefix; the responder
does not receive their `OwnChoice` records.

## Complete clock and paid/free transitions

All rounds use slot ownership **A/B/B/A** at slots **1/2/3/4**. Calibration has
at most three purchased rounds. Each live trial has exactly three full rounds
(12 paid slots, six owned by each Agent), and there are exactly four live trials.

1. Start with empty prefixes, no issued bits, credits `[48,48]`, and public
   `Boundary(Probe { completed: 0 })`.
2. At `Probe { completed: c }`, the investigator freely selects Stop or Continue
   for `c<3`; at `c=3` only Stop is legal. Continue commits the entire next round,
   not individual spending or a half-round stop. It queues the next transition;
   it does not itself pay, reset, or issue a bit.
3. First Continue enters calibration with a free Reset for both own histories.
   Later Continue retains the field resulting from the preceding round's finish.
   The next public paid position is calibration round `c+1`, slot 1.
4. For raw policies, before each investigator-owned calibration slot expose
   `Boundary(Calibration(position))` only to the investigator; the responder's
   same-time checkpoint is `BeforeSlot(position)`. Free selection records an
   own Calibration choice and queues exactly that paid action. It does not
   publicly announce the action. Supplied policies execute the recipe directly,
   with no fabricated private slot-choice records. Helper-owned slots execute
   the helper recipe directly.
5. Each paid slot executes once and returns `AfterSlot(position)` in both
   prefixes; only its owner's transcript gains the paid Event. Check each local
   history for support before advancing. The paid return precedes round finish,
   reset, another action, or another issued bit.
6. A separate free advance after slot 4 calls round finish. SharedResetting clears
   then, including after live round 3; other mechanisms retain. After calibration
   round slot 4 the next checkpoint is `Probe { completed: round }`. No additional
   LocalEntry Reset is fabricated for a round finish; the clock and known rules
   determine it. After other slots advance to the next public opportunity.
7. Stop adds the investigator's own Stop entry and one `PublicProbeStop` entry
   to both prefixes atomically. Stop costs zero and queues live entry. Skipped
   calibration rounds have no Reset, Read, Write, Wait, choice, or charge.
8. Free live entry clears all fields, appends own live Reset and own PrivateBit
   immediately together, and clears the selected live routine. For investigator
   A, expose `Boundary(Trial { trial })` at round 1/slot 1; responder B sees
   `BeforeSlot` of that position. For investigator B, first execute helper A's
   round 1/slot 1 and return `AfterSlot`; only the subsequent free advance exposes
   B's Trial boundary at round 1/slot 2. A sees `BeforeSlot` of that position.
   B knows the clock advanced but does not observe A's paid action or outcome.
9. A free live choice queues Inspect or Attempt at that boundary. Both use all
   six owned live slots. Remaining public slots continue in A/B/B/A order.
10. After round 3/slot 4, return its paid AfterSlot, then free round finish and
    `Prediction { trial }`. Infer both own target posteriors before predicting.
    The next free advance starts the next live trial with Reset/PrivateBit; after
    trial 3 it reaches Finished. No reward feedback is appended to any prefix.

Public Probe boundaries are shared. Private Calibration and Trial boundaries
are investigator checkpoints with responder `BeforeSlot` at the same clock.
Repeated identical responder prefixes before/after a private selection count
each original candidate once, never as separate evidence.

Each purchased calibration round charges each Agent 2. Attempt costs 6 per live
trial; Inspect costs 9 (one inspection plus five waits). The maximum charge is
`3×2 + 4×9 = 42`, leaving at least 6 credits under this fixed horizon. Those are
schedule arithmetic, not optimized values or measured resource requirements.

## Supplied calibration and live routines

These recipes use each acting Agent's own physical observations only. A raw
investigator replaces only its own cells; the helper's cells remain fixed even
when the raw investigator changes evidence available to the helper.

| Calibration round | Slot 1: A | Slot 2: B | Slot 3: B | Slot 4: A |
| --- | --- | --- | --- | --- |
| 1 | Write(Probe0) | Read | Write(Ack0) iff own round 1/slot 2 Read was Probe0; otherwise Wait | Read |
| 2 | Read | Read | Write(Probe1) | Read |
| 3 | Write(Ack1) iff own earlier round 2/slot 4 Read was Probe1; otherwise Wait | Read | Wait | Wait |

The helper has nominal point prior on the actual catalog mechanism; for DataFlip
the nominal model remains SharedPersistent. Its physical live controller uses
that supplied model knowledge. With nominal SharedPersistent, A sends its own
issued bit as Data0/Data1 at round 1/slot 1 and reads at round 3/slot 1 iff its own
round 1/slot 1 action was a Write in this trial. B reads at round 2/slot 2 and sends
its own bit at round 2/slot 3. All remaining helper slots Wait. With any other
nominal model, the helper inspects at its send opportunity and Waits everywhere
else. The helper is never given the investigator's chosen live routine.

| Investigator routine | Role A | Role B | Other owned slots |
| --- | --- | --- | --- |
| Attempt | Write(own Data bit) at 1/1; Read at 3/1 | Read at 2/2; Write(own Data bit) at 2/3 | Wait |
| Inspect | InspectOwnTarget at 1/1 | InspectOwnTarget at 2/3 | Wait |

Investigator Attempt uses these send/read positions regardless of inferred
mechanism or whether communication succeeds. Inspect is a trusted target
observation only if that episode reaches it without earlier termination.
Helper target predictions use its own posterior from candidate-local replay of
the declared investigator policy, never actual hidden targets or peer history.
Physical helper actions are identical across the three primary policies under
matched local conditions; prediction differences are reported separately.

## Policies, objective, inference, and ties

| Policy | Investigator prior | Calibration | Live continuation |
| --- | --- | --- | --- |
| RawAdaptive | Uniform catalog | Free round purchase and eight owned-slot actions | Optimal Inspect/Attempt |
| SuppliedAdaptive | Uniform catalog | Free round purchase; supplied recipe | Optimal Inspect/Attempt |
| NoProbe | Uniform catalog | Stop at zero | Optimal Inspect/Attempt |
| KnownRaw | Matching catalog point prior | Same raw menu and horizon | Optimal Inspect/Attempt |
| InspectOnly | Uniform catalog | Stop at zero | Inspect every reached trial |

Raw and KnownRaw paid-slot tie order is Wait, Read, Write(Probe0), Write(Ack0),
Write(Probe1), Write(Ack1), Write(Data0), Write(Data1). Calibration inspection,
blank write, arbitrary live actions, and mid-round Stop are excluded. Round
ties select Stop before Continue; live ties select Inspect before Attempt.
At three rounds only Stop remains. Constrained controls expose only legal
choices, not extra alternatives evaluated with a stronger policy. Predictions
choose one iff exact target probability is strictly greater than 1/2; ties choose
zero. Numeric rational comparison is by exact value, not float epsilon or the
structural order of numerator/denominator pairs.

The objective is expected **investigator** terminal reward minus future paid
costs. Correct prediction pays 12, incorrect prediction 0, with four targets.
At a decision, already paid costs are sunk; alternative values include every
future owned paid action and future investigator reward under that policy's
declared continuation. Initial value includes all investigator costs. Calibration
information gain, model certainty, helper reward, and group reward are not
separate bonuses. Terminal prediction reward is expectation `12×max(p,1−p)`;
this is a planning calculation from local target mass, not reward feedback.

Original candidates are `(model, sequence)` with immutable ID
`model_index×256 + sequence`, in model order above. Uniform investigator mass is
1/1024 per candidate; matching point prior mass is 1/256 for its model's IDs and
zero elsewhere. Never renumber survivors or multiply original prior mass twice.
Each candidate contains its World fields/origins, both candidate-local histories,
credits, clock, issued bits, and helper policy state. Future bits remain latent
until their issue time. Investigator inference conditions on chronological own
observations and public events, replaying its chosen actions as interventions;
it can evaluate legal counterfactual own choices. Responder inference reexecutes
the **declared compiled investigator policy** on candidate-local investigator
histories. Off-policy prefixes must not silently become responder evidence.
Support is not a model-marginal vector: equal model marginals can retain different
field contents, peer histories, or conditional task bits and require different
values. Caches must retain all relevant candidate identity and state.

Each Agent's inference is separate. An actual unsupported peer prefix never
prunes the other Agent's candidate support. A helper's point prior is original
supplied information, not successful observation-based identification. For an
unknown investigator, per-role full-horizon mechanism equivalence means equality
of the law of own chronological observable prefixes under **every** permitted
investigator intervention strategy with the fixed candidate-local helper and
fair sequences. Ignore an injected mechanism label in priors when comparing
these laws. The numerical author derives this partition separately for role A
and B, with separating witnesses or exhaustive indistinguishability arguments.
It must be distinguished from certainty attained by a chosen optimal policy.

The author must independently construct the supplied-recipe embedding into raw
owned opportunities, preserving helper physical actions, clocks, and information.
Only after that check does menu inclusion imply RawAdaptive ≥ SuppliedAdaptive
in the well-specified expected investigator objective, separately for each role
under the original prior. It guarantees neither strict improvement, per-actual-
mechanism/sequence dominance, responder/group dominance, nor DataFlip transfer.

## Failure and availability

Validate local inference at the initial clock and after every paid observation,
private selection, and explicit public transition, before another transition.
Use stable owner check order A then B at a shared checkpoint; record the first
unsupported owner and complete first prefix. If unsupported, halt immediately:
no later Wait, round finish, Reset, PrivateBit, prediction, or charge. Keep the
contradicting paid observation, charged partial costs, both partial histories,
previous predictions and prediction beliefs, first failure, last supported own
belief, and partial nominal catalog discoveries. A supported wrong prediction
is an ordinary outcome, not an unsupported failure.

For a stopped episode, terminal `correct`, `reward`, `net`, final model projection,
and accuracy remain null for both Agents, even if earlier trial predictions
exist. Never invent terminal zero or average only complete survivors. Available
partial predictions remain inspectable. DataFlip true-model probability and
unique true-model identification are null even in a completed episode; nominal
catalog beliefs and certainty remain labeled nominal. Discovering unsupported
evidence neither adds DataFlip to the catalog nor identifies its inversion rule.

Malformed protocol/history/report, arithmetic overflow, physical legality
failure, state/branch/allocation exhaustion, worker timeout, and cancellation
are computation/validation failures, returned contextually outside Episode
outcomes. They cannot count as unsupported mass, missing settings, a favorable
subset, or successful empty reports. Retain failure receipts and resolve the
gate or review an explicit design amendment.

## Frozen ordered census

Setting order is environment first (four catalog mechanisms then DataFlip),
investigator A then B, policy order RawAdaptive, SuppliedAdaptive, NoProbe,
KnownRaw, InspectOnly for each in-family role. DataFlip uses RawAdaptive,
SuppliedAdaptive, NoProbe, InspectOnly, excluding KnownRaw. Sequence order in
every setting is ascending 0..255 exactly once. Setting indexes are zero-based.

| Index | Environment | Investigator | Policy |
| --- | --- | --- | --- |
| 0 | SharedPersistent | A | RawAdaptive |
| 1 | SharedPersistent | A | SuppliedAdaptive |
| 2 | SharedPersistent | A | NoProbe |
| 3 | SharedPersistent | A | KnownRaw |
| 4 | SharedPersistent | A | InspectOnly |
| 5 | SharedPersistent | B | RawAdaptive |
| 6 | SharedPersistent | B | SuppliedAdaptive |
| 7 | SharedPersistent | B | NoProbe |
| 8 | SharedPersistent | B | KnownRaw |
| 9 | SharedPersistent | B | InspectOnly |
| 10 | SharedResetting | A | RawAdaptive |
| 11 | SharedResetting | A | SuppliedAdaptive |
| 12 | SharedResetting | A | NoProbe |
| 13 | SharedResetting | A | KnownRaw |
| 14 | SharedResetting | A | InspectOnly |
| 15 | SharedResetting | B | RawAdaptive |
| 16 | SharedResetting | B | SuppliedAdaptive |
| 17 | SharedResetting | B | NoProbe |
| 18 | SharedResetting | B | KnownRaw |
| 19 | SharedResetting | B | InspectOnly |
| 20 | PrivatePersistent | A | RawAdaptive |
| 21 | PrivatePersistent | A | SuppliedAdaptive |
| 22 | PrivatePersistent | A | NoProbe |
| 23 | PrivatePersistent | A | KnownRaw |
| 24 | PrivatePersistent | A | InspectOnly |
| 25 | PrivatePersistent | B | RawAdaptive |
| 26 | PrivatePersistent | B | SuppliedAdaptive |
| 27 | PrivatePersistent | B | NoProbe |
| 28 | PrivatePersistent | B | KnownRaw |
| 29 | PrivatePersistent | B | InspectOnly |
| 30 | Inert | A | RawAdaptive |
| 31 | Inert | A | SuppliedAdaptive |
| 32 | Inert | A | NoProbe |
| 33 | Inert | A | KnownRaw |
| 34 | Inert | A | InspectOnly |
| 35 | Inert | B | RawAdaptive |
| 36 | Inert | B | SuppliedAdaptive |
| 37 | Inert | B | NoProbe |
| 38 | Inert | B | KnownRaw |
| 39 | Inert | B | InspectOnly |
| 40 | DataFlip | A | RawAdaptive |
| 41 | DataFlip | A | SuppliedAdaptive |
| 42 | DataFlip | A | NoProbe |
| 43 | DataFlip | A | InspectOnly |
| 44 | DataFlip | B | RawAdaptive |
| 45 | DataFlip | B | SuppliedAdaptive |
| 46 | DataFlip | B | NoProbe |
| 47 | DataFlip | B | InspectOnly |

The 24 primary in-family settings contribute 6,144 episodes; 16 controls
contribute 4,096; eight transfer settings contribute 2,048. Total: 40 in-family
settings/10,240 episodes plus eight transfer settings/2,048 episodes =
48 settings/12,288 episodes. No random policy, seed ensemble, tuning flag,
postmeasurement policy/price/prior/horizon sweep, or KnownRaw transfer is allowed.

## Exact native reference wire contract

This is the numerical author's complete handoff schema. It describes native JSON,
before browser lossless-integer conversion; it does not require importing Rust.
Objects reject unknown/missing fields. All declared nullable fields must be
present with JSON null when unavailable; arrays preserve order. Integer values
must be exact JSON integers, never rounded IEEE-754 values or fractional floats.
Bounded values use the ranges stated below. Native `u64` and `i64` bounds apply;
arithmetic beyond them is an error, never truncation. Browser conversion later
uses canonical decimal strings through the existing lossless boundary; reference
fixtures used with native serde retain integer numerators and denominators.

`Probability` is `{"numerator":u64,"denominator":u64}` with positive denominator,
0≤numerator≤denominator, reduced by gcd; `ScoreFraction` uses signed i64 numerator
and positive u64 denominator, likewise reduced. Canonical zero is 0/1. All exact
means, differences, values, and probabilities use these pairs; no float tolerance.
`Option<T>` is `T|null`; `[T;2]` is exactly two A/B values. `Vec<T>` is an ordered
JSON array. Unit enum variants serialize as strings; tuple/record variants use
one-key externally tagged objects, not flattened discriminators.

| Type | Exact wire form |
| --- | --- |
| Role | `"A"` or `"B"` |
| Mechanism | One of four catalog name strings in declared order |
| Environment | `{"InFamily":"SharedPersistent"}` (or another catalog name), or `"DataFlip"` |
| Symbol | One of the seven declared name strings |
| Action | `"Read"`, `"Wait"`, `"InspectOwnTarget"`, or `{"Write":"Data0"}` (any nonblank symbol) |
| Outcome | `{"Read":"Blank"}` (any symbol), `"Accepted"`, `"Waited"`, `{"Inspected":false}` |
| Phase | `"Calibration"` or `{"Live":{"trial":0}}`, trial 0..3 |
| Position | `{phase:Phase,round:u8,slot:u8}`, round 1..3, slot 1..4 |
| OwnPrior | `"Uniform"` or `{"PointMass":"SharedPersistent"}` (any catalog mechanism) |
| Ids | `{surface:String,agents:[String;2]}`, validated opaque IDs |
| PolicyKind | `"RawAdaptive"`, `"SuppliedAdaptive"`, `"NoProbe"`, `"KnownRaw"`, `"InspectOnly"` |
| Choice | `"Stop"`, `"Continue"`, `{"Calibration":Action}`, `"Inspect"`, `"Attempt"` |
| Boundary | `{"Probe":{"completed":0}}` (0..3), `{"Calibration":Position}`, `{"Trial":{"trial":0}}` (0..3) |
| Checkpoint | `{"Boundary":Boundary}`, `{"BeforeSlot":Position}`, `{"AfterSlot":Position}`, `{"Prediction":{"trial":0}}`, `"Finished"` |
| Event | `{position:Position,action:Action,outcome:Outcome,credits_after:u8}`, credits 0..48 |
| LocalEntry | `{"Reset":{"phase":Phase}}`, `{"PrivateBit":{"trial":0,"bit":false}}`, `{"Action":Event}` |
| Entry | `{"Physical":LocalEntry}`, `{"OwnChoice":{"boundary":Boundary,"choice":Choice}}`, `{"PublicProbeStop":{"completed":0}}` |
| Protocol | `{investigator:Role,policy:PolicyKind,ids:Ids}` |
| Prefix | `{role:Role,ids:Ids,own_prior:OwnPrior,checkpoint:Checkpoint,entries:Vec<Entry>}` |
| Belief | `{models:[Probability;4],target:Option<Probability>}`, model masses sum to 1 in catalog order; target means P(current peer bit=1) |
| View | `{prefix:Prefix,credits:u8,private_bit:Option<bool>,belief:Belief}`; credits/bit derived from own entries |
| ChoiceValue | `{choice:Choice,value:ScoreFraction}` |
| Decision | `{choice:Choice,alternatives:Vec<ChoiceValue>,continuation_policy:PolicyKind}` |
| DecisionTrace | `{prefix:Prefix,decision:Decision}`; prefix is before the recorded selection |
| Lineage | `{writer:Role,write_position:Position,symbol:Symbol,task_trial:Option<u8>}` |
| PrivilegedStep | `{role:Role,event:Event,read_lineage:Option<Lineage>}` |
| AgentMetrics | `{spent:u8,reward:Option<i64>,net:Option<i64>,correct:Option<u8>,reads:u8,writes:u8,waits:u8,inspections:u8,attempted_sends:u8,lineage_reads:u8}` |
| Failure | `{role:Role,prefix:Prefix,last_supported:Option<Belief>,spent:[u8;2]}` |
| CatalogCertaintySource | `"Supplied"` or `"Observed"` |
| CatalogDiscovery | `{model:Mechanism,source:CatalogCertaintySource,checkpoint:Checkpoint,spent:u8}` |
| ModelProjection | `{identified_catalog_model:Option<Mechanism>,identification_source:Option<CatalogCertaintySource>,final_belief:Belief,true_model_mass:Option<Probability>,uniquely_identified:Option<bool>,first_identification:Option<Checkpoint>,spent_to_identify:Option<u8>}` |
| Episode | `{sequence:u16,histories:[Prefix;2],decisions:Vec<DecisionTrace>,metrics:[AgentMetrics;2],predictions:[Vec<bool>;2],prediction_beliefs:[Vec<Belief>;2],privileged:Vec<PrivilegedStep>,failure:Option<Failure>,model_projections:[Option<ModelProjection>;2],catalog_discoveries:[Option<CatalogDiscovery>;2]}` |
| Aggregate | `{valid_mass:Probability,failed_mass:Probability,spent:[ScoreFraction;2],reward:[Option<ScoreFraction>;2],net:[Option<ScoreFraction>;2],group_net:Option<ScoreFraction>}` |
| Panel | `{protocol:Protocol,environment:Environment,episodes:Vec<Episode>,aggregate:Aggregate}` |
| PairedComparison | `{investigator:Role,environment:Environment,ids:Ids,left_policy:PolicyKind,right_policy:PolicyKind,difference:Aggregate}` |
| ActualMechanismWeight | `{model:Mechanism,weight:Probability}` |
| UniformMechanismSummary | `{protocol:Protocol,actual_mechanism_distribution:[ActualMechanismWeight;4],aggregate:Aggregate}` |
| Limits | `{states:u64,branches:u64,allocation_bytes:u64}`, all positive once registered |
| SearchStats | `{candidate_worlds:u64,decision_states:u64,branches:u64,allocation_bytes:u64}` |
| Step | `{histories:[Prefix;2],events:Vec<[Role,Event]>,next:Option<Boundary>,spent_delta:[u8;2]}`; events are ordered role/Event pairs |
| Error | `{"InvalidProtocol":String}`, `{"InvalidHistory":String}`, `{"UnsupportedHistory":{"prefix":Prefix}}`, `{"ResourceLimit":{"resource":String,"limit":u64}}`, `{"InvalidReport":String}`, `"ArithmeticOverflow"`, `{"Physical":PhysicalError}` |
| PhysicalError | `{"InvalidProtocol":String}`, `{"InvalidAction":String}`, `{"InvalidReport":String}`, `{"UnsupportedHistory":{"prefix":PhysicalPrefix}}`, or `"ArithmeticOverflow"` |

PhysicalPrefix is the legacy shared local record
`{role:Role,ids:Ids,checkpoint:PhysicalCheckpoint,own_prior:OwnPrior,entries:Vec<LocalEntry>}`;
PhysicalCheckpoint is `"EpisodeStart"`, `{"Action":Position}`,
`{"AfterSlot":Position}`, or `{"Prediction":{"trial":0}}`. It is used only
inside retained shared physical errors, not as discovery inference input.
The discovery error conversion preserves ArithmeticOverflow at its own level.

The code-like object forms in the table are schema notation, not JSON literals.
Complete JSON syntax examples follow; they are local chronology examples, not
computed optimum/decision/reference values.

```json
{"investigator":"B","policy":"RawAdaptive","ids":{"surface":"status-field","agents":["Agent-A","Agent-B"]}}
```

An initial investigator prefix has no reset or private bit:

```json
{"role":"B","ids":{"surface":"status-field","agents":["Agent-A","Agent-B"]},"own_prior":"Uniform","checkpoint":{"Boundary":{"Probe":{"completed":0}}},"entries":[]}
```

A legal hypothetical raw investigator B prefix immediately after its first
calibration paid Wait (peer A's earlier paid event is absent):

```json
{"role":"B","ids":{"surface":"status-field","agents":["Agent-A","Agent-B"]},"own_prior":"Uniform","checkpoint":{"AfterSlot":{"phase":"Calibration","round":1,"slot":2}},"entries":[{"OwnChoice":{"boundary":{"Probe":{"completed":0}},"choice":"Continue"}},{"Physical":{"Reset":{"phase":"Calibration"}}},{"OwnChoice":{"boundary":{"Calibration":{"phase":"Calibration","round":1,"slot":2}},"choice":{"Calibration":"Wait"}}},{"Physical":{"Action":{"position":{"phase":"Calibration","round":1,"slot":2},"action":"Wait","outcome":"Waited","credits_after":47}}}]}
```

A hypothetical B Trial boundary after Stop at zero and helper A's first action
retains credits 48 and no peer action record. Helper A's own same-clock prefix
uses BeforeSlot(live trial 0, round 1, slot 2), and contains its paid Event:

```json
{"role":"B","ids":{"surface":"status-field","agents":["Agent-A","Agent-B"]},"own_prior":"Uniform","checkpoint":{"Boundary":{"Trial":{"trial":0}}},"entries":[{"OwnChoice":{"boundary":{"Probe":{"completed":0}},"choice":"Stop"}},{"PublicProbeStop":{"completed":0}},{"Physical":{"Reset":{"phase":{"Live":{"trial":0}}}}},{"Physical":{"PrivateBit":{"trial":0,"bit":false}}}]}
```

Prefix validation enforces exact owned slot order, current phase, one own bit
per live Reset, reconstructible credits, proper outcome/action pairing, legal
choice time/owner/menu, no repeated Stop, and no missing Continue or raw slot
intervention. A responder never contains an OwnChoice or private Boundary.
Public clock changes can occur without a new own paid record. No synthetic peer
paid observation is used to explain these gaps. Calibration target belief is
null; a live decision/prediction requires an issued own bit and target posterior.
Episode sequence is 0..255; completed correct count is 0..4. Each prediction
has one matching prediction belief; on failure the two arrays remain partial.

Lineage stores the **attempted** written symbol, not DataFlip's stored inverse.
`task_trial` is Some(trial) only for Data0/Data1 written in live phase; calibration
data writes have null task_trial. A read captures lineage at that paid event;
non-Reads have null read_lineage. Lineage never appears in Prefix, View, or
inference support. `lineage_reads` counts live Reads whose origin is the other
Agent and current trial; `attempted_sends` counts live Data0/Data1 Writes only.

## Reference artifact schemas and coverage

All three tracked reference files share `metadata` below. Each `Receipt` is
`{path:String,bytes:u64,sha256:String}` with lowercase 64-hex SHA256, immutable
path/byte count/hash and no copied production value. Metadata is
`{protocol_version:u8,protocol_sha256:String,author:String,inputs:Vec<Receipt>,program:Receipt,tests:Receipt,derivation_receipts:Vec<Receipt>,full_projection:Receipt}`.
Protocol hash binds this reviewed declarative file's exact bytes, recorded
externally to avoid a self-referential digest. Author identity and independent
read log are retained. If registration changes protocol bytes, reseal dependent
references or explicitly retain the derivation input hash with a reviewed
non-scientific-registration delta; never silently claim a different input hash.
Complete population projections remain ignored and sealed; compact tracked
fixtures reference them by immutable receipts.

`planning-reference.json` root is
`{version:"surface-discovery-planning-reference-v1",metadata:Metadata,initial_values:Vec<InitialValue>,prefix_witnesses:Vec<PrefixWitness>,identifiability:Vec<Identifiability>,embedding:Vec<Embedding>}`.

| Record | Exact fields and meaning |
| --- | --- |
| InitialValue | `{protocol:Protocol,prior:OwnPrior,value:ScoreFraction}` |
| CandidateMass | `{id:u16,mass:Probability}`; ascending original IDs 0..1023, each once, strictly positive normalized posterior mass |
| PrefixWitness | `{label:String,protocol:Protocol,prior:OwnPrior,inference:String,prefix:Prefix,belief:Option<Belief>,candidate_support:Vec<CandidateMass>,decision:Option<Decision>,continuation_value:Option<ScoreFraction>}` |
| Identifiability | `{investigator:Role,classes:Vec<Vec<Mechanism>>,derivation:Receipt,separating_witnesses:Vec<PrefixWitness>}` |
| Embedding | `{investigator:Role,prior:OwnPrior,supplied_value:ScoreFraction,embedded_value:ScoreFraction,raw_value:ScoreFraction,equal_helper_receipt:Receipt}` |

`inference` is exactly `InvestigatorInference` or `ResponderReplay`. Supported
witness belief equals the support's exact model/target projection; unsupported
witnesses have null belief, empty support, and null decision/value. Investigator
decision witnesses include every legal alternative in tie order, the declared
continuation policy, chosen optimum, and its exact continuation value. Responder
witnesses have null decision/value and must be on declared-policy replay; own
counterfactual intervention witnesses are explicitly InvestigatorInference.

Initial values cover both roles × all five policies with Uniform prior except
KnownRaw, which has four point-prior rows per role: 16 distinct compilations.
Values need not be duplicated by actual environment when protocol/prior agree.
The full independent projection covers every supported decision prefix/action
continuation; compact witnesses cover round purchase/stopping, all raw action
forms, B consecutive slots and live boundary, differing joint state with equal
marginals, ties/interventions, responder private-selection deduplication,
identifiability, and DataFlip support/contradiction. If optimal paths lack a
particular anticipated event, retain legal counterfactual witnesses or document
absence without adding a policy to the census. Identifiability has exactly two
rows, complete catalog partitions and independent derivations; no class is
invented in this declarative document. Embedding has exactly two Uniform rows.

`evaluation-reference.json` root is
`{version:"surface-discovery-evaluation-reference-v1",metadata:Metadata,episodes:Vec<EpisodeRow>,panels:Vec<PanelRow>,comparisons:Vec<PairedComparison>,role_expectations:Vec<UniformMechanismSummary>,role_contrasts:Vec<RoleContrast>,coverage:Coverage}`.

| Record | Exact fields and meaning |
| --- | --- |
| EpisodeRow | `{protocol:Protocol,prior:OwnPrior,environment:Environment,sequence:u16,episode:Episode}`; outer and episode sequence agree |
| PanelRow | `{setting:u8,protocol:Protocol,prior:OwnPrior,environment:Environment,aggregate:Aggregate,projection:PanelProjection}` |
| PanelProjection | `{unavailable_mass:Probability,accuracy:[Option<ScoreFraction>;2],action_counts:[MeanCounts;2],calibration_rounds:Vec<CountMass>,failures:Vec<FailureMass>,identification:[IdentificationProjection;2],posterior_calibration:[Vec<CalibrationBin>;2],episode_projection_receipt:Receipt}` |
| MeanCounts | `{reads:ScoreFraction,writes:ScoreFraction,waits:ScoreFraction,inspections:ScoreFraction,attempted_sends:ScoreFraction,lineage_reads:ScoreFraction}` |
| CountMass | `{count:u8,mass:Probability}`; calibration purchased-round distribution, count 0..3 |
| FailureMass | `{role:Role,checkpoint:Checkpoint,mass:Probability,prefixes:Receipt}`; full original failure prefixes sealed in referenced projection |
| IdentificationProjection | `{true_model_mass:Option<ScoreFraction>,uniquely_identified_mass:Option<Probability>,discoveries:Vec<DiscoveryMass>}` |
| DiscoveryMass | `{model:Mechanism,source:CatalogCertaintySource,checkpoint:Checkpoint,spent:u8,mass:Probability}`; first nominal certainty, denominator full population |
| CalibrationBin | `{trial:u8,probability:Probability,issued_mass:Probability,ones_mass:Probability}`; exact issued prediction records, denominator full population |
| RoleContrast | `{investigator:Role,left_policy:PolicyKind,right_policy:PolicyKind,difference:Aggregate}`; uniform actual-catalog paired contrast |
| Coverage | `{settings:u8,sequences_per_setting:u16,in_family_episodes:u64,transfer_episodes:u64,total_episodes:u64,full_episodes:Receipt,legacy_mapping_receipt:Receipt}` |

Panels include all 48 settings in frozen order with all scalar aggregate and
projection values. Comparisons include all 24 primary in-family stratum rows in
environment/role order, each ordered Raw−Supplied, Raw−NoProbe, Supplied−NoProbe.
Role expectations include both roles × five policies (10 rows); KnownRaw is the
mixture of matching informed point controls, not one reused point prior.
Role contrasts include both roles × three primary contrasts (six rows). Tracked
episodes are bounded complete witnesses; the sealed full_episodes contains all
12,288 EpisodeRows in setting/sequence order, never only survivors. All 40
in-family and eight transfer aggregate/projection rows are required even if
all values are zero, adverse, unavailable, or show no distinct raw trace.

PanelProjection is a reference/summary projection; it does not silently add
fields to the native Episode or Aggregate mirror. Full episode projection
receipts retain chosen chronological calibration actions, available predictions,
first unsupported prefix and partial cost, all beliefs/nominal discoveries,
and the scalar derivations. They allow independent recomputation of every
projection field without a production fixture or cached passed flag.

`limits-reference.json` root, once the pending independent gate completes, is
`{version:"surface-discovery-limits-reference-v1",metadata:Metadata,counter_definition:String,allocation_definition:String,runs:Vec<FeasibilityRun>,maximum_requirements:Limits,registered_limits:Limits,rounding:String,host_capacity:HostCapacity,review_receipt:Receipt}`.
`FeasibilityRun` is
`{label:String,protocol:Protocol,prior:OwnPrior,completed:bool,stats:SearchStats,wall_time_ns:u64,peak_rss_bytes:u64,policy_bytes:u64,replay_bytes:u64,full_projection_bytes:u64,receipt:Receipt}`.
`HostCapacity` is `{available_memory_bytes:u64,check_receipt:Receipt}`.
Counts, elapsed time, actual peak memory, and serialized sizes are fresh author
outputs; no run is claimed in this document. Every distinct compilation and
complete enumeration requirement must be covered; completed references, not
an early partial run, establish maximum_requirements. A failed run is retained
with its contextual receipt and cannot register successful limits.

## Canonical legacy mapping and preservation

SuppliedAdaptive must equal the retained active Adaptive comparator, including
complete histories, clock positions, decisions/alternatives, beliefs, paid
actions/outcomes, predictions, costs, discoveries, failure, and lineage under
matched environment/role/sequence/IDs. No extra private slot choice is invented
for supplied controls. Map only names as follows; all other fields and values
are unchanged:

| Discovery | Original active equivalent |
| --- | --- |
| Protocol.investigator / comparison.investigator | experimenter |
| SuppliedAdaptive | Adaptive |
| NoProbe | NoProbe |
| InspectOnly | InspectOnly |
| Choice.Stop | StopProbing |
| Choice.Continue | ContinueProbe |
| Choice.Attempt | AttemptCommunication |
| Choice.Inspect | Inspect |
| Boundary.Probe { completed } | ProbeChoice { completed } |
| Boundary.Trial { trial } | TrialChoice { trial } |

For Decision, map continuation_policy and each ChoiceValue.choice along with
chosen choice; do not alter exact values or alternative order. Map nested
boundaries/choices throughout Prefix entries, DecisionTrace, Failure,
CatalogDiscovery, ModelProjection, and checkpoints. Physical variants and
positions retain spelling/shape. RawAdaptive, KnownRaw, Calibration boundaries,
and Calibration choices have **no** legacy equivalent. KnownRaw must not be
mapped to old Known: its point prior is paired with new raw opportunities.
FixedThree is not a discovery policy. Legacy measured artifacts, engines,
settings, guide prefix bytes, and original source receipts remain unchanged.

## Metrics and exact contrasts

Episode spent is 48 minus remaining credits, including partial failures. Reads,
writes, waits, inspections, attempted sends, and lineage reads count executed
paid events only. Complete correct is the number of correct peer-bit predictions;
reward is `12×correct`; net is reward−spent; accuracy is correct/4. Group net is
A net+B net. Investigator and responder labels are derived from the protocol;
never swap the physical A/B arrays when changing investigator role.

All panel means use the full 256 sequences; exact probability/mass denominators
are 256, reduced canonically. valid_mass counts episodes without Failure;
failed_mass counts unsupported episodes; they sum to 1. unavailable_mass counts
episodes whose terminal metrics are undefined (equal to failed_mass for the
declared census, not computation failures). Spent and action-count means remain
defined on partial traces. Terminal means and accuracy are null for a panel if
any terminal component is unavailable; no survivors-only mean is substituted.
Full terminal model-mass and true-identification means likewise become null if
terminal inference is missing. For DataFlip both true-model measures are always
null. Partial first nominal discoveries remain available with source labels;
their masses use the full population and do not purport to identify DataFlip.

Posterior calibration bins describe emitted prediction records for each Agent
and trial: probability is the recorded P(target=1), issued_mass is the fraction
of all declared episodes emitting a prediction with that probability, and
ones_mass is the fraction of all episodes both in that bin and having true
target 1. For an in-family Uniform actual-mechanism expectation, aggregate bins
over all four mechanisms at weight 1/4; compare ones_mass to
probability×issued_mass. A fixed-actual-mechanism stratum alone need not exhibit
prior-mixture calibration. KnownRaw calibration is computed within each informed
model, then mixed explicitly. Transfer bins remain labeled nominal, with absent
prediction mass retained; no supported-survivor calibration claim is made.

Each primary contrast pairs the same actual mechanism, investigator, IDs, and
sequence. Compute left−right episode metrics before averaging; spent differences
remain available even when either endpoint stops. Terminal paired differences
are null if either endpoint is unavailable for any sequence; paired failed_mass
and unavailable mass are the union, not the sum of endpoint failures. Preserve
both endpoint completion/failure masses and partial costs in panel projections.
The 24 stratum contrasts stay separate. Uniform role expectations and six role
contrasts use an outer actual-mechanism weight 1/4, independent of an Agent's
OwnPrior; never pool roles or replace stratum results with this expectation.
KnownRaw/InspectOnly are reference endpoints, not added primary treatments.

Confidence/identification must distinguish supplied point certainty, first
observed nominal certainty, full permitted-horizon equivalence, and the chosen
policy's identification. Changed traces, confidence, or responder predictions
alone are not downstream utility improvement. Report zero/adverse outcomes and
all declared failure/unavailable mass. Exact finite population means require no
sampled-seed confidence interval or significance test.

## Pending independent numerical and feasibility gate

Before a production author uses reference values, the fresh numerical author
must test independent physical mechanics first, derive exact initial and
continuation values/support/identifiability/DataFlip likelihoods, enumerate all
declared projections, and establish the equal-helper embedding. Use standard
library exact fractions and genuine independent RED/GREEN receipts. No production
import, subprocess invocation of an engine, fixture copying, raw optimum hints,
or measurement-guided choices are allowed. A distinct numerical reviewer
recomputes checks and verifies the author's input/read receipts and seal.

**State, branch, and allocation caps are not yet registered.** The numerical
author defines how each reference state/branch is counted and how retained and
transient allocation bytes are estimated, records actual peak memory separately,
and completes full independent enumeration before choosing caps. For each
resource, register twice the largest completed reference requirement, rounding
up to a documented accounting unit without reducing that ceiling. Record exact
maxima, multiplication/rounding arithmetic, formulas, serialized sizes, all run
receipts, and independent review in limits-reference.json. Reviewer checks host
capacity. Registered Limits must be positive u64 values; pending status cannot
be represented by zero, invented constants, or a successful Limits loader.

If reference enumeration cannot complete, retain failure evidence and seek a
design amendment; do not invent outputs or shorten actions/horizon. Task 3 can
revise engineering caps before judged collection only through a retained,
independently reviewed protocol revision with a demonstrated accounting reason;
scientific actions/horizon and old receipts remain preserved. Task 8 freezes
final caps, source inventory, compiler/toolchain, executable/WASM identities,
reference hashes, and registration before judged collection.

Browser limits remain 64 KiB input/decompressed link, 16 MiB episode, 4,096
checkpoints, one worker request, 60-second timeout/cancellation, and two retained
records. A worker runs one selected sequence; the page thread replays the checked
timeline. Actual WASM time/memory, policy/replay/record sizes, fresh exact native
parity, pending-request cancellation, and rendered privacy are later feasibility
gates. If exact execution cannot fit, review a source-bound policy-table delivery
or explicit design amendment. No approximation, action pruning, tolerance, or
silent horizon reduction is authorized.

## Pre-mortem and collection gate

| Failure mode | Required evidence or prevention before judged collection |
| --- | --- |
| B selects before A's first action or learns that action | Both-role chronological prefix witnesses; B boundary after A paid AfterSlot/free advance; peer event absent |
| B consecutive slots become A/B/A/B | Exact A/B/B/A paid order and owned transcript checks |
| Raw helper behavior accidentally changes with comparator | Supplied embedding replays equal physical rules/clocks/information; independent both-role expected-objective check |
| Cached model marginals lose task-relevant hidden state | Same-marginal/different-joint-support continuation witnesses and full candidate IDs |
| Own choices counted as evidence or peer private choice revealed | Intervention tests, candidate-local replay, original-world deduplication and display allowlists |
| DataFlip data write contradicts earlier than old recipe | Fresh raw likelihood derivation; first-owner prefix, charged partial event and immediate halt |
| Unsupported episode becomes zero or survivor success | Full census, null terminals, union unavailable paired mass, complete partial-cost projections |
| Supplied comparator subtly changes | Canonically mapped complete legacy payload equality, not aggregate-only agreement |
| Resource exhaustion appears as scientific contradiction | Contextual resource errors and pending-cap/WASM gates; no episode conversion |
| Fixture/summary/import tampering passes a cached flag | Fresh exact reconstruction and all scalar/prefix/reference/census checks |
| Browser leaks later bits or lineage into Agent view | Default local projection, checkpoint-specific allowlists, native/WASM parity and real browser receipts |
| Favorable subset or rerun replaces first output | Frozen 48×256 census/source; exclusive first/repeat receipts and byte equality; retain failed output |
| Historical sources/reports are relabeled | Before/after source inventories, unchanged legacy exports/guide prefixes and original source hashes |
| New discovery claim outruns supplied rules | Report bounded supplied-model calibration selection, zero/adverse findings, nominal transfer and no repair claim |

No judged collection begins until independent declarative/numerical reviews,
native and browser feasibility, preservation receipts, exact reference coverage,
and source/cap registration pass. Engineering runs are labeled separately and
cannot select the collection subset. First/repeat outputs are exclusive and
immutable. Any scientific revision after measurements preserves original outputs
and uses an explicitly revised series; delivery-only source deltas are recorded
separately from the first-measurement source identity.
