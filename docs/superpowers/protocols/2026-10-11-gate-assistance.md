# Gate assistance protocol 1

Status: declarative engineering registration, awaiting independent protocol review. This document fixes the source-blind reference contract before numerical derivation; it reports no measured results. The approved design is [2026-10-11-gate-assistance-design.md](../specs/2026-10-11-gate-assistance-design.md). The new sibling namespace must preserve every prior scientific engine and receipt.

## Authority and representation

The physical and information rules below implement the approved design. Existing code supplies serialization and identity conventions, not gate results: `surface_discovery/types.rs` uses externally tagged native enum variants and strict structs; `surface_discovery/diagnostics.rs` defines `ArtifactReceipt` and `compiled_bytes_identity`; `browser_experiments/wire.rs` supplies the separate lossless browser projection. Source paths here are relative to `crates/sugarscape-core/src/`. No existing scientific source changes are required by this registration.

The native Episode is protocol version integer 1. Native JSON uses exact integer number atoms, JSON Boolean atoms, UTF-8 strings, arrays and explicit null. Every declared field is required, including nullable fields; unknown or duplicate object keys reject at every nesting level before conversion to a generic map can discard duplicates. Decimal strings are not native integers. Floats (including 1.0 and exponent forms), booleans, out-of-range integers, malformed enum payloads and wrong array lengths reject. JSON escape variants decode to the same string; no Unicode normalization is applied. IDs are nonempty distinct valid Unicode strings: Holder defaults to `Agent-A`, Retriever to `Agent-B`. Whitespace-only labels remain nonempty opaque labels; roles never depend on lexical name order. Lone surrogate code points reject. Counts crossing a JavaScript number boundary must be at most 9007199254740991; larger u64 counters/receipt sizes use the existing lossless decimal-string payload projection, never lossy conversion. Native u64 fields remain checked u64 integers.

All enum spellings are exactly the Rust variant spellings below, with ordinary externally tagged serde representation and no rename_all. Unit variants are strings: `"Holder"`, `"P"`, `"Wait"`, `"Success"`, `"Reset"`. Newtype variants are one-key objects: `{"Move":"C"}`, `{"Blocked":"GateClosed"}`, `{"Holder":"Hold"}`, `{"Terminal":"Delivered"}`, `{"Observed":{...}}`. Struct variants are one-key objects with a strict field object: `{"Action":{"role":"Holder","token":{"Holder":"Hold"},"intent":"OperatePlate","result":"Success","paid":1}}`. `LocalEntry::Public`, `OwnAction` and `Settlement` use that same struct-variant rule. `OwnWord::Holder` and `OwnWord::Retriever` wrap six-element arrays of their unit-token strings; the word newtypes themselves serialize as their six-token arrays. `occupants` contains two-element arrays `[role,cargo_boolean]`. Options always serialize as the value or null, never omitted. Native enum tagging is separate from the existing browser study discriminator, which uses snake_case.

Role-indexed arrays and histories are always Holder then Retriever. Cells are always ordered P, O, C; occupants are ordered Holder then Retriever. Never sort by IDs. Array order is semantically significant and preserved by canonicalization.

## Complete consumed schema

The following shared API is normative; the prose after it resolves chronology, validation, accounting and wire ordering. Error is an operational Rust result type rather than a scientific Episode field.


Task 1 freezes the complete serde field/enum/clock schema in the declarative protocol before numerical handoff. The following names, boundaries and behavior are fixed for later tasks. Struct fields shown are the consumed API, not an invitation to add a different model. Additional scientific event/observation fields come only from the reviewed wire protocol.

    pub enum Role { Holder, Retriever }
    pub enum Cell { P, O, C }
    pub enum HolderToken { Wait, Travel, Hold }
    pub enum RetrieverToken { Wait, Advance }
    pub struct Ids { pub holder: String, pub retriever: String }
    pub struct Stratum { holding_cost: u8, holder_share: u8, distance: u8 }
    impl Stratum {
        pub fn new(k:u8,b:u8,d:u8)->Result<Self,Error>;
        pub fn validate(&self)->Result<(),Error>;
        pub fn holding_cost(&self)->u8; pub fn holder_share(&self)->u8;
        pub fn distance(&self)->u8;
    }
    pub struct Config { pub stratum: Stratum, pub ids: Ids }
    impl Config { pub fn validate(&self)->Result<(),Error>; }
    pub struct Profile { pub holder: u16, pub retriever: u8 }
    impl Profile { pub fn new(h:u16,r:u8)->Result<Self,Error>;
                   pub fn validate(&self)->Result<(),Error>; }
    pub struct HolderWord([HolderToken;6]);
    pub struct RetrieverWord([RetrieverToken;6]);
    impl HolderWord { pub fn from_id(id:u16)->Result<Self,Error>; pub fn id(&self)->u16; }
    impl RetrieverWord { pub fn from_id(id:u8)->Result<Self,Error>; pub fn id(&self)->u8; }
    pub enum Primitive { Wait, Move(Cell), OperatePlate, Pickup, Deliver }
    pub enum Token { Holder(HolderToken), Retriever(RetrieverToken) }
    pub enum Termination { Delivered, Deadline }
    pub enum Phase { Initial, Reset, BeforeAction, AfterAction, Finished }
    pub struct Clock { pub round:u8, pub phase:Phase, pub role:Option<Role> }
    pub enum ResourceLocation { Cache, Carried, Delivered }
    pub struct PhysicalState {
        pub positions:[Cell;2], pub retriever_cargo:bool,
        pub resource:ResourceLocation, pub gate_open:bool,
        pub spent:[u8;2], pub reward:[u8;2],
    }
    pub enum BlockReason { AlreadyAtPlate, AwayFromPlate, GateClosed, NoToken,
                           WrongPosition, EmptyHands, FullHands }
    pub enum ActionOutcome { Success, Blocked(BlockReason) }
    pub enum EventKind {
        Reset,
        Action { role:Role, token:Token, intent:Primitive,
                 result:ActionOutcome, paid:u8 },
        Terminal(Termination),
    }
    pub struct Event { pub clock:Clock, pub kind:EventKind,
                       pub before:PhysicalState, pub after:PhysicalState }
    pub struct VisibleCell { pub cell:Cell, pub occupants:Vec<(Role,bool)>,
                             pub loose_token:bool }
    pub struct Observation { pub clock:Clock, pub cells:Vec<VisibleCell>,
                             pub gate:Option<bool> }
    pub enum LocalEntry {
        Public { clock:Clock, termination:Option<Termination> },
        Observed(Observation),
        OwnAction { clock:Clock, token:Token, intent:Primitive,
                    result:ActionOutcome, paid:u8 },
        Settlement { clock:Clock, reward:u8 },
    }
    pub enum OwnWord { Holder(HolderWord), Retriever(RetrieverWord) }
    pub struct View {
        pub role:Role, pub word:OwnWord, pub own_cell:Cell, pub own_cargo:bool,
        pub credits:u8, pub own_reward:u8, pub clock:Clock,
        pub history:Vec<LocalEntry>, pub last_observation:Observation,
    }
    pub struct Outcome {
        pub spent:[u8;2], pub reward:[u8;2], pub net:[i16;2],
        pub group_net:i16, pub delivery_round:Option<u8>,
        pub holder_attempts:[u8;3], pub holder_blocked:[u8;3],
        pub retriever_attempts:[u8;4], pub retriever_blocked:[u8;4],
        pub retriever_waits:u8,
    }
    pub struct Episode {
        pub version:u16, pub config:Config, pub profile:Profile,
        pub events:Vec<Event>, pub histories:[Vec<LocalEntry>;2],
        pub outcome:Outcome,
    }
    pub struct CapturePoint { pub clock:Clock, pub views:[View;2],
                              pub physical:PhysicalState }
    pub fn choose(view:&View)->Result<Primitive,Error>;
    pub fn run_episode(config:&Config,profile:Profile)->Result<Episode,Error>;
    pub fn run_with_capture<F>(config:&Config,profile:Profile,capture:&mut F)
        ->Result<Episode,Error>
        where F:FnMut(&CapturePoint)->Result<(),Error>;
    pub fn trace_hash(episode:&Episode)->Result<String,Error>;
    pub enum Error {
        InvalidInput(String), InvalidHistory(String), InvalidReport(String),
        ResourceLimit{resource:String,limit:u64}, ArithmeticOverflow,
        Io(String),
    }

Attempt arrays use Holder Wait/Travel/Hold and Retriever Move/Pickup/Move/Deliver order, with the two Move slots distinguished as outbound/return; Retriever Wait has its separate field. Wait is not folded into paid Move. Clock round is zero only for Initial; role is Some only at BeforeAction/AfterAction, and the other phases use None. Action events also record the exact role token in the reviewed schema, alongside intent, result and paid charge. Ids::default() supplies Holder Agent-A and Retriever Agent-B; reject equal or empty labels. All serde structs deny unknown fields and validators reject invalid strata, IDs, roles, clocks, lengths and types. JSON floats and Boolean-to-integer coercion never implement exact scores.

The private native World stays opaque. View is defined in Task 2 with only own role/word, own position/cargo/credits/settlement, committed local history, last observation and public clock. choose(view:&View)->Result<Primitive,Error> is pure. run_episode(config:&Config,profile:Profile)->Result<Episode,Error> uses that function, not hidden world state, for each intent. run_with_capture uses the identical kernel and read-only native capture points; it cannot create observations. Kernel emits initial, reset, before-own-action, after-own-action and terminal capture stages. Scientific Episode stores append-only histories/events, not quadratic copies of whole prefixes at every browser checkpoint.

Compile only a Stratum, using default IDs for its canonical census traces. Names cannot enter a payoff-table key. Selected renamed episodes retain their own names and hash; metamorphic tests prove equal physics/payoffs, without claiming name-dependent complete hashes equal.

    pub struct Limits { pub profiles:u64, pub transitions:u64, pub allocation_bytes:u64 }
    impl Limits { pub fn registered()->Result<Self,Error>; }
    pub struct CompileStats { pub profiles:u64, pub transitions:u64, pub peak_bytes:u64 }
    pub struct Row { pub outcome:Outcome, pub trace_hash:String }
    pub struct Classification { pub holder_regret:i16, pub retriever_regret:i16,
                                pub is_pure:bool, pub team_gap:i16,
                                pub outcome:Outcome }
    pub(crate) struct ResponseSets { pub holder_best:Vec<i16>,
        pub retriever_best:Vec<i16>, pub pure:Vec<Profile> }
    pub(crate) fn response_sets(holder_count:usize,retriever_count:usize,
        payoffs:&[[i16;2]])->Result<ResponseSets,Error>;
    pub struct Analysis {
        pub holder_best:Vec<i16>, pub retriever_best:Vec<i16>,
        pub pure:Vec<Profile>, pub team:Profile,
        pub lex_pure:Option<Profile>, pub best_group_pure:Option<Profile>,
        pub solo_best:i16,
    }
    pub struct GameTable { pub stratum:Stratum, pub rows:Vec<Row>,
                           pub analysis:Analysis, pub stats:CompileStats }
    pub enum Selector { SoloAttempt, SuppliedHelp, TeamOptimal, NashLex, NashBestGroup }
    pub fn compile(s:&Stratum,limits:&Limits)->Result<GameTable,Error>;
    pub fn resolve(table:&GameTable,selector:Selector)->Option<Profile>;
    pub fn classify(table:&GameTable,p:Profile)->Result<Classification,Error>;

Classification exposes Holder/Retriever unilateral regrets, pure membership, group team gap and actual outcome to evaluation/reporting only. It is not a View field. Raw row index is 64*holder+retriever. response_sets accepts checked nonzero dimensions within 729/64 and exactly their product payoffs, using index retriever_count*holder+retriever; production passes the complete dimensions, while algebraic unit tests can use a small explicit matrix. All private-payoff argmax ties survive. Team and representative ascending tie ranking uses (Reverse(group_net),total_spent,delivery_round.unwrap_or(7),holder,retriever), with std::cmp::Reverse integer ordering and checked spending sums. Counting exact proportions, if needed, uses the existing checked exact fraction type through an unchanged import, not new floating-point semantics.

Reports have version gate-assistance-diagnostic-v1, summaries gate-assistance-summary-v1, protocol 1. Task4 implements these exact consumed fields as strict serde types; nested definitions are frozen with Task1's wire protocol. Source/report/reference hashes, caps and control availability are explicit.

    pub struct Receipt { pub path:String, pub bytes:u64, pub sha256:String }
    pub struct Check { pub name:String, pub passed:bool, pub detail:String }
    pub struct ControlRecord { pub selector:Selector, pub profile:Option<Profile>,
        pub episode:Option<Episode>, pub classification:Option<Classification> }
    pub struct Panel { pub stratum:Stratum, pub rows:Vec<Row>, pub analysis:Analysis,
        pub controls:Vec<ControlRecord>, pub stats:CompileStats }
    pub struct Representative { pub profile:Profile, pub classification:Classification }
    pub struct PayoffRanges { pub holder:Option<[i16;2]>,
        pub retriever:Option<[i16;2]>, pub group:Option<[i16;2]> }
    pub struct EquilibriumSummary { pub pure_count:u64, pub delivering_pure_count:u64,
        pub ranges:PayoffRanges, pub range_unavailable:Option<String>,
        pub lex:Option<Representative>, pub best_group:Option<Representative> }
    pub struct SummaryControl { pub selector:Selector,
        pub profile:Option<Profile>, pub classification:Option<Classification>,
        pub unavailable:Option<String> }
    pub struct SummaryStratum { pub stratum:Stratum, pub stats:CompileStats,
        pub equilibrium:EquilibriumSummary, pub team:Representative,
        pub solo_best:i16, pub controls:Vec<SummaryControl> }
    pub struct OutcomeDelta { pub spent:[i16;2], pub reward:[i16;2], pub net:[i16;2],
        pub group_net:i16, pub delivery_round:Option<i16>,
        pub delivery_unavailable:Option<String>, pub holder_attempts:[i16;3],
        pub holder_blocked:[i16;3], pub retriever_attempts:[i16;4],
        pub retriever_blocked:[i16;4], pub retriever_waits:i16 }
    pub struct ControlContrast { pub selector:Selector,
        pub lower:Option<Representative>, pub upper:Option<Representative>,
        pub delta:Option<OutcomeDelta>, pub unavailable:Option<String> }
    pub struct PayoffRangeDelta { pub holder:[i16;2], pub retriever:[i16;2],
                                 pub group:[i16;2] }
    pub struct EquilibriumContrast { pub lower:EquilibriumSummary,
        pub upper:EquilibriumSummary, pub range_delta:Option<PayoffRangeDelta>,
        pub range_unavailable:Option<String> }
    pub enum Axis { HoldingCost, HolderShare, HolderDistance }
    pub struct Contrast { pub axis:Axis, pub lower:Stratum, pub upper:Stratum,
        pub controls:Vec<ControlContrast>, pub equilibria:EquilibriumContrast }
    pub struct Report { pub version:String, pub source_hash:String,
        pub source_receipts:Vec<Receipt>, pub reference_receipts:Vec<Receipt>,
        pub limits:Limits, pub panels:Vec<Panel>, pub contrasts:Vec<Contrast>,
        pub checks:Vec<Check>, pub passed:bool }
    pub struct Summary { pub version:String, pub report_hash:String, pub source_hash:String,
        pub settings:Vec<SummaryStratum>, pub contrasts:Vec<Contrast>,
        pub limits:Limits, pub source_receipts:Vec<Receipt>,
        pub reference_receipts:Vec<Receipt> }

SummaryStratum retains stratum, stats, pure/delivering-pure counts, optional Holder/Retriever/group net ranges, team profile/value, lex/best-group-pure profiles/values, solo_best and all five control outcomes/classifications without Episode histories. ControlContrast retains selector, both availability flags/profiles and optional differences in spending/reward/net/group/delivery and paid-action counts. EquilibriumContrast retains both complete counts/delivering counts/ranges and optional endpoint differences; it never pairs arbitrarily selected equilibria. These field groups are explicit protocol schemas, not opaque Value objects. A range or difference absent because of an empty set/unavailable control stays null with its reason.

Contrast orientation is upper minus lower on exactly one numeric axis. Include all pairs at fixed other coordinates: 18 holding-cost, 18 reward-share and nine distance contrasts, 45 total. Selector contrasts are explicitly reselected under each setting, distinct from fixed-raw-profile comparisons. At fixed k/d, check every raw pair's delivery/spending/group outcome unchanged across b; private net changes by +delta_b/-delta_b only when delivered. This is a controlled invariant, not a sampled-seed causal generalization.

    pub fn registered_strata()->Vec<Stratum>;
    pub fn diagnose(limits:&Limits)->Result<Report,Error>;
    pub fn report_integrity(report:&Report,limits:&Limits)->Result<bool,Error>;
    pub fn reconstruct_episode(report:&Report,stratum_index:usize,p:Profile)
        ->Result<Episode,Error>;
    pub fn summary_json(report:&Report)->Result<String,Error>;


## Deterministic words, input and initial state

Validate k in [1,2,4], b in [0,3,6], d in [0,1]; the registered stratum order nests k, then b, then d, each ascending. Native Config has exactly `stratum` and `ids`; Stratum has exactly `holding_cost`, `holder_share`, `distance`; Profile has exactly `holder`, `retriever`. Holder ID is 0..728 and Retriever ID 0..63. Convert IDs to exactly six most-significant-round-first radix-3 or radix-2 digits, retaining leading Wait digits. Holder digits 0/1/2 are Wait/Travel/Hold; Retriever digits 0/1 are Wait/Advance. Round q chooses element q−1. Preserve all six tokens even when termination makes a suffix unexecuted.

Initial PhysicalState is positions [P,O] for d=0 or [O,O] for d=1; retriever_cargo false; resource Cache; gate_open false; spent [0,0]; reward [0,0]. Holder cargo is always false. Credits are `32 - own spent`, never increased by reward. Own reward is accounted separately. No credit transfer or collision occurs.

The graph has exactly undirected edges P–O and O–C. The second edge requires gate_open to cross. Travel always intends Move(P), Hold always OperatePlate, and both roles' Wait intends Wait. Retriever Advance chooses using only its own state: (O,false)→Move(C), (C,false)→Pickup, (C,true)→Move(O), (O,true)→Deliver. Retriever at P or other inconsistent role/word/history/state combinations are InvalidHistory, not a synthetic fifth Advance action. The pure choose function is callable only on the active role's valid BeforeAction View; it validates its own word and clock and derives no input from a peer schedule, evaluator or hidden world. Static configuration knowledge is supplied by the protocol; it is not a license to observe an unseen current cell.

## Exact transition and terminal order

Every mutation is a validated, checked transaction. Errors abort with an operational Error; no partial successful Episode is returned. The world remains opaque to controller code. There is one authoritative kernel shared by run_episode and run_with_capture. A capture failure propagates as an operational error; capture is read-only and never adds observations. The following sequence is exact, including zero-cost and blocked actions:

1. Set clock `{round:0,phase:"Initial",role:null}`. Append Public(clock, termination:null) to both histories, then append one Observed entry to each using the initial state and this clock. Emit one Initial capture after both observations. There is no Initial Event in Episode.events.
2. For q=1..6, set Reset clock with round q and role null. Snapshot before, close the gate for free, append Reset Event with before and after (even when already closed). Append Public(clock,null) to each history and emit Reset capture. No observations occur at reset.
3. For Holder and then Retriever, set BeforeAction clock for q and that role. Append Public(clock,null) to both histories. Observe only the active role and append Observed with this clock. Emit BeforeAction capture after that observation. Call choose on the active View.
4. Determine the active schedule token, intent, declared charge and transaction result against pre-action state. Snapshot before; apply charge even for a blocked attempt; then apply only the successful primitive's physical changes. Set AfterAction clock for q and that role. Append Action Event, using this AfterAction clock, token, intent, result, paid and exact before/after snapshots. Append Public(AfterAction,null) to both histories; append OwnAction to the active history; then append its fresh Observed entry using the same AfterAction clock and post-action state. Emit AfterAction capture after all these commits. The passive role receives only Public entries and no peer feedback or fresh observation.
5. A successful Deliver consumes cargo and sets resource Delivered in step 4; its Action after snapshot and after-own observation precede reward settlement and free terminal gate closure. Immediately after its AfterAction capture, finish Delivered at round q. Otherwise finish Deadline immediately after Retriever's round-six AfterAction capture. No unexecuted slot/suffix gets an Event, charge, observation or action count.
6. Finishing uses `{round:q,phase:"Finished",role:null}`. Snapshot before; close the gate freely; on Delivered only, set reward to [b,12−b] exactly once; on Deadline reward stays [0,0]. Append one Terminal Event with that termination and before/after snapshots. Append Public(Finished,termination) to each history, then Settlement(Finished,own reward) to each history, including a zero settlement on Deadline or a zero share. Emit one Finished capture. Do not observe either role at terminal. Compute Outcome from executed events and the terminal state. Return Episode with complete append-only histories/events.

At every capture both Views.clock equal the public capture clock. View.history is precisely the committed local prefix at that boundary; last_observation is its most recent Observed value with its original timestamp. Own cell, cargo, credits and reward reflect the role's current own state, while passive last_observation stays stale. Finished captures therefore expose one's own settlement but do not refresh the local gate view. Current physical state is Researcher-only. CapturePoint and View copies do not occur inside Episode and are excluded from trace identity; the histories they expose are included once in Episode.

Initial is the only round-zero clock. Every other round is 1..6. BeforeAction and AfterAction have a non-null role; Initial, Reset and Finished have null role. Action Events carry AfterAction; Reset Events carry Reset; Terminal Events carry Finished. Matching role, token family, intent and result must be validated. A public phase announces scheduling only: it never reveals the peer token, action result, payment or private reward. Episode events contain evaluator physics; local histories do not acquire those fields implicitly.

## Primitive transaction rules and block precedence

Wait always succeeds, costs 0, and changes no physics. Holder Travel costs 1: at O move to P; at P block AlreadyAtPlate. Holder Hold costs k: at P set gate_open true; at O block AwayFromPlate. An impossible Holder location/role-action mismatch is InvalidHistory/InvalidInput rather than a new physical behavior. The Holder never goes from P to O or to C.

Retriever Advance always costs 1. Its chosen Move(C) from empty O or Move(O) from carrying C succeeds iff gate_open, otherwise blocks GateClosed. Successful Move changes only own position. Pickup at C succeeds iff empty and resource Cache, then sets cargo true and resource Carried. Deliver at O succeeds iff carrying, then sets cargo false and resource Delivered. Primitive precondition checks, for validation of submitted intents or histories, are ordered: role/capability and legal destination first (invalid input/history); required position next (WrongPosition); required cargo next (Pickup FullHands, Deliver EmptyHands); resource next (Pickup NoToken); crossing gate last (GateClosed). NoToken, WrongPosition, EmptyHands and FullHands cannot arise from a valid committed episode's shortest-route controller but have fixed diagnostic spellings. Existing ResourceLocation and cargo invariants must hold before and after every transaction; a corrupt world is an operational invalid-history error, not a blocked charge.

Blocked physical attempts change only that role's spent count. They never move, modify cargo/resource/gate, settle reward or terminate. Free reset and terminal closure are the only mutations outside actions/settlement. Checked spending must stay within 32; registered six-round schedules never require budget exhaustion behavior. No automatic wait substitution is permitted.

Outcome.spent/reward equal terminal state; net[i]=reward[i]−spent[i]; group_net=net[0]+net[1]. delivery_round is q iff Delivered and null iff Deadline. Holder attempts/blocked have slots [Wait,Travel,Hold]. Retriever attempts/blocked have slots [outbound Move(C),Pickup,return Move(O),Deliver], and Wait attempts occur only in retriever_waits. Successful counts are attempts−blocked, checked nonnegative. Counts reflect executed actions only; Wait is successful, never blocked. Holder travel spending equals Travel attempts, holding spending equals k×Hold attempts; Retriever spending equals sum of its four paid attempt slots. Unused budgets equal 32−spent per role.

## Observation function

Visibility uses the static graph, independently of gate openness: at P visible cells [P,O]; at O [P,O,C]; at C [O,C]. Include every visible cell, even empty. VisibleCell.occupants includes all roles currently at that cell, each with its actual cargo Boolean, in role order. VisibleCell.loose_token is true exactly when cell C and resource Cache. Carried or Delivered resources are never loose. Observation.gate is null at P; otherwise the actual gate_open Boolean. Only Observed entries invoke this function. No inference, credits, rewards, peer word or evaluator classification is placed in visible cells. Both share the supplied initial configuration; an initial observation still has exactly this local shape, not an omniscient map.

## Canonical complete Episode identity

The canonical body is the entire strict native Episode, with all and only declared fields, recursively sorted object keys, original array order, compact separators, exact base-ten integer number atoms (no floats/exponents), lowercase JSON true/false/null, UTF-8 without BOM and no trailing newline. Encode Unicode directly, with standard JSON escapes only for quote, backslash and controls (short escapes for backspace, tab, newline, formfeed, carriage return; lowercase `\u00xx` for other controls). Never stringify native integers before hashing. All config including IDs, full raw profile, events, both histories and Outcome are included. Excluded because they are not Episode fields: classifications, compiler statistics, selected-control labels, captures/prefix copies, source/reference/report receipts, limits and the trace hash itself.

Trace identity is the existing one-item `compiled_bytes_identity` receipt codec, not the naked canonical-body digest. Compute body bytes as above; inner sha256 is lowercase SHA-256 of those exact bytes. Form the compact receipt array with fields in this exact insertion order, **not sorted**: `[{"path":"gate-assistance-trace-v1","bytes":N,"sha256":"INNER"}]`. N is body byte length, not Unicode character count. The final trace hash is lowercase SHA-256 of this receipt-array UTF-8 without newline. Rust invokes the unchanged `surface_discovery::compiled_bytes_identity(&[("gate-assistance-trace-v1", body.as_slice())])`; it does not copy SHA code or add a dependency. Native sorted JSON can be produced through recursively sorted serde_json::Value (current workspace serde_json lacks preserve_order). Source receipts and self hashes remain in envelopes.

The source-blind author must implement exactly this independent standard-library oracle:

```python
import hashlib, json

def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'),
                      ensure_ascii=False, allow_nan=False).encode('utf-8')

def trace_hash(episode):
    body = canonical(episode)
    receipt = [{'path': 'gate-assistance-trace-v1', 'bytes': len(body),
                'sha256': hashlib.sha256(body).hexdigest()}]
    wire = json.dumps(receipt, separators=(',', ':'), ensure_ascii=False,
                      allow_nan=False).encode('utf-8')
    return hashlib.sha256(wire).hexdigest()
```

Python must reject bool as int and validate every schema before hashing; canonical() is serialization, not validation. Rust and Python must agree on escaped controls, non-ASCII labels, nulls, arrays, variants and both digest levels before trusting census hashes.

## Game, controls, reports and validation

Every stratum retains rows in holder-major order: h=0..728 outer, r=0..63 inner; index=64h+r. Holder best-response maxima are indexed by r (64 entries), Retriever maxima by h (729 entries). Pure profiles are all and only private-payoff ties to both maxima, sorted (holder,retriever). No suffix pruning or tie pruning. Classification regrets are nonnegative maxima minus actual private net; team_gap is maximum group net minus actual group_net. These fields never enter execution Views.

Team ranking is ascending (-group_net,spent[0]+spent[1],delivery_round or 7,holder,retriever). lex_pure is first in sorted pure list or null. best_group_pure minimizes team ranking within pure or is null. solo_best is maximum Retriever net at Holder ID 0. Response-set helper accepts positive checked dimensions at most 729 and 64, exactly their product payoff pairs, and uses the supplied retriever_count as row stride; smaller matrices exist only for algebraic testing.

Control order is always SoloAttempt, SuppliedHelp, TeamOptimal, NashLex, NashBestGroup. SoloAttempt is Holder [Wait×6], Retriever [Advance×6]. SuppliedHelp d=0 is Holder [Hold,Wait,Hold,Wait,Wait,Wait], Retriever [Advance,Advance,Advance,Advance,Wait,Wait]; d=1 is Holder [Travel,Hold,Wait,Hold,Wait,Wait], Retriever [Wait,Advance,Advance,Advance,Advance,Wait]. Derive numeric IDs from the declared radix codec; no reference result is assumed. The other three controls resolve through Analysis. A ControlRecord is either all profile/episode/classification present and consistent, or all three null for an unavailable equilibrium representative. Only NashLex/NashBestGroup may be unavailable, exactly when pure is empty. SummaryControl mirrors this availability and uses reason `no_pure_equilibrium`; available controls have unavailable:null. Summary uses classifications' outcomes without histories.

Pure payoff ranges are [minimum,maximum] over the entire pure set for each role and group. If pure is empty all three ranges and representatives are null, counts are zero and range_unavailable=`no_pure_equilibrium`. Otherwise ranges and both representatives are present and range_unavailable:null. delivering_pure_count counts all pure profiles with non-null delivery_round, retaining duplicate realized outcomes as separate raw profiles. All rows, events and histories reconstruct from the current registered source and exact profile before acceptance; cached passed/check Booleans are never authority.

Contrast axes occur in order HoldingCost, HolderShare, HolderDistance. For each axis, scan lower strata in registered order, then upper strata in registered order, retaining exactly pairs with lower axis value < upper and equal other coordinates: 18+18+9=45. Both control endpoints are reselected; each of the five selectors retains the common order. OutcomeDelta is upper minus lower componentwise, checked signed integers. delivery_round delta exists only if both delivered; otherwise null with delivery_unavailable=`delivery_round_unavailable`. If either control is unavailable, delta is null and unavailable=`lower_control_unavailable`, `upper_control_unavailable`, or `both_controls_unavailable`; retain each available endpoint. If both are available unavailable is null. Equilibrium range deltas are upper endpoint minus corresponding lower endpoint, never a pairing of arbitrarily chosen equilibria. They are absent with reason `lower_pure_set_empty`, `upper_pure_set_empty`, or `both_pure_sets_empty` when appropriate; otherwise range_unavailable is null.

Report.panels and Summary.settings follow registered stratum order. Panel.controls and SummaryStratum.controls follow control order. Receipt collections have unique paths and a registered deterministic path order; source_receipts use explicitly declared compiled dependency order, reference_receipts use physics/game/limits fixture order when those fixtures are approved. SHA strings are exactly 64 lowercase hex digits. Report.source_hash applies the existing ordered receipt codec to the compiled closure. Summary.report_hash uses the same one-item compiled_bytes_identity codec with path `gate-assistance-report-v1` and the recursively sorted compact native Report JSON body, without newline. The inner receipt hashes that body and the outer digest hashes the insertion-ordered receipt. An external file receipt separately hashes exact exported report bytes; report serialization writes compact struct-order JSON followed by exactly one newline. Summary itself has no self hash. Reports have version `gate-assistance-diagnostic-v1`, summaries `gate-assistance-summary-v1`. Report.checks have unique names, deterministic order and recomputed truth; no missing, duplicate, extra or reordered census/profile/control can be masked by a passed flag. Check names, in order, are `registered_census`, `reference_outcomes`, `reference_trace_hashes`, `reference_controls`, `best_responses`, `pure_membership`, `representative_ties`, `reward_share_invariance`, `resource_limits`, `source_and_reference_identity`. Each detail string states checked scope/counts or a contextual discrepancy; Report.passed is true exactly when every recomputed check passes. This is a check schema, not a claim that any check has run.

Strict report validation checks dimensions, exact strata/order, values/ranges/counters, no duplicate profiles, hashes, receipt identities, limits, all Episode reconstruction and all response maxima/membership. A stale identity, malformed history, arithmetic overflow, operational I/O failure or cap exhaustion must return a contextual Error. Physical non-delivery remains a valid Deadline outcome. Native CLI and WASM use the same structs and kernel. Browser payloads separately use existing `lossless_value`, converting integer payload atoms to canonical decimal strings; envelope indexes/version and bounded input IDs use exact safe integers. This projection is not the canonical native Episode used for reference hashes. Browser selected episodes retain named Config and hence distinct complete hashes after renaming while physics/payoffs remain equal.

## Logical resource accounting and independent cap derivation

The three Limits apply independently to each compile(stratum) invocation, with zero starting counters and checked u64 increments; diagnostics does not reset a limit partway through a stratum. No preallocation bypass, wrapping count or operational failure converted to a losing episode is permitted. profiles counts each evaluated raw pair once, charged before evaluation. transitions counts each appended Event, including every Reset, Action and Terminal (but not Initial, observations, captures or solver comparisons), charged before the corresponding transition. These are implementation work counters, distinct from paid actions/physical credits. The reference derives the exact full count from its independently implemented event lists; no successful delivery/equilibrium count is presumed.

allocation_bytes is a portable **logical live payload** budget, not Rust size_of, allocator capacity, RSS or reserved memory. For a canonical JSON value J let L(J) be len(canonical(J)). Define a reproducible streaming compiler schedule and charge these simultaneously live components:

- Retained rows: sum L(Row) over rows appended so far; a row's trace_hash is its 64-character final fingerprint. Charge a new row before retaining it. Internal outcome interning may not reduce this logical charge.
- Retained payoff scratch: 4 bytes per completed raw pair (two signed i16 values), charged even if read back from rows rather than duplicated physically.
- Response maxima scratch: 2×(64+729) bytes, held for the whole compile. Pure membership storage: 3 bytes per retained Profile (u16+u8), accumulated during response analysis. Team/lex/best-group profile bookkeeping: 9 bytes throughout. These are explicit portable field widths, without alignment overhead.
- Per-profile episode scratch: 2×L(Episode) plus L(receipt_array) plus 64 bytes while completing the current trace (one typed logical episode representation, its canonical buffer, the exact one-item identity receipt and final hash). For L(receipt_array), canonical versus insertion key order has equal length. The scratch charge grows with committed Episode content and peaks at the completed Episode. It includes an Outcome initialized from the current state/counts at each boundary; since counters are nonnegative spending and bounded actions, completed serialization is a conservative peak only if independently verified. Use the maximum over all boundary snapshots if any length decreases. For each logical scratch boundary snapshot, delivery_round is the round of the successful Deliver Action Event already committed in that snapshot's event prefix, or null if that prefix contains no successful Deliver. Thus it is q at Deliver's AfterAction boundary, before the Terminal event, reward settlement and terminal gate closure, and remains q at Finished. Other scratch Outcome fields use only spending, reward and executed-action counts committed at that boundary; at Deliver's AfterAction boundary reward is still [0,0]. These temporary accounting projections are not completed Episodes. Drop profile scratch before beginning the next profile.
- At a row append the completed profile scratch and the newly retained row/payoff are simultaneously live. During analysis keep all rows/payoffs/maxima/bookkeeping and count pure storage. After analysis retain rows and the complete Analysis, charging L(Analysis) in addition to the above analysis-phase storage until temporary scratch is dropped. peak_bytes is the greatest logical live total across these declared phases.

This definition is intentionally source-independent and conservative about representation reuse. Its constants define logical units, not guessed required capacities. The independent author computes per-stratum maxima and the reviewer recomputes them; registered provisional limits are twice the largest independently approved requirement for each counter across the 18 strata, checked for overflow. No cap value is registered in this document. Caps and their derivation must be separately reviewed before a judged compile. Later native/WASM feasibility separately measures actual allocations/serialized full reports/browser records and may reveal constraints this logical budget does not represent. If full retained reports or actual WASM do not fit, seek a registered amendment; never silently sample/prune/shrink the census.

Report construction, serialization, source/reference receipts and browser CapturePoint copies are outside per-stratum compile accounting and must be included in the later whole-diagnostic/native/WASM feasibility evidence. Browser retains existing 64 KiB input/link, 16 MiB record, 4,096 checkpoint, one outstanding request, 60-second timeout and two-record limits. No browser census compilation is implied.

## Independent handoff and scope

The fresh numerical author receives only this sealed protocol, the approved design and fixed handoff inputs, no production source or prior expected outputs. It implements `evaluate(k,b,d,h,r)->dict`, `decode_holder(h)->tuple[str,...]` and `decode_retriever(r)->tuple[str,...]` using Python standard library. Manually derived supplied-near and delayed-suffix unit traces must precede enumeration. Preserve failure/repair evidence and every raw suffix. It derives all 18 complete tables, best responses, all pure sets/ties, selected controls and full histories; 839,808 raw outcomes and up to 90 available control histories are requirements, not measured success claims. A separate fresh numerical reviewer seals its own checks before viewing author output. No numerical handoff precedes independent protocol approval.

Capture limitations, logical versus actual memory and controller-family bounds must remain visible in findings. This experiment can establish the exact specified finite game's behavior; it cannot establish learning convergence, an equilibrium-selection mechanism, hidden mechanism discovery or general altruism. Do not change the registered physics after seeing outcomes without preserving this version and explicitly registering a separate series.
