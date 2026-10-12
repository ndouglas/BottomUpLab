# Key delegation protocol 1

Status: declarative registration awaiting independent protocol review. No key production implementation, numerical reference, measured result or numeric resource cap is supplied by this document. The approved [design](../specs/2026-10-11-key-delegation-design.md) and [implementation plan](../plans/2026-10-12-key-delegation.md) govern the increment. Preserve Gate Assistance and every prior laboratory's scientific source and evidence.

## Representation and strict validation

Native wire values use JSON objects, arrays, UTF-8 strings, Boolean atoms, exact base-ten integer number atoms and explicit null. Every field listed below is required, including nullable fields, unless the raw selected-input rule explicitly specifies absence. Structs must be objects, never positional arrays. Reject unknown keys and duplicate decoded keys at every nesting level before map construction can erase duplicates, including keys spelled with different equivalent JSON escapes. Reject floats, exponent forms, decimal strings in native integer fields, Boolean-to-integer coercion, negative zero, leading zeros, non-finite numbers, wrong array lengths, malformed tagged variants and out-of-range values. Canonical input integer syntax is `0` or `-?[1-9][0-9]*`, with sign prohibited on unsigned fields. Object key input order is immaterial. UTF-8 and decoded strings must be valid Unicode scalar sequences; reject lone surrogates. Do not normalize Unicode.

Schema notation below is declarative: `u8`, `u16`, `u64`, `i16`, `i64` denote checked exact integer ranges of those widths; semantic bounds narrow them. `bool` is a JSON Boolean, `String` a string, `[T;N]` an exact-length array, `Vec<T>` an ordered array, and `Option<T>` a required field containing T or null. All integer operations use checked arithmetic; errors are operational. Native u64 receipt/count values remain numbers. Browser payload projection uses the existing lossless decimal-string conversion for all payload integers; input IDs and envelope version/index remain safe exact numbers. Any JSON integer interpreted as a JavaScript number must be within ±9007199254740991. Decimal-string browser atoms use canonical signed/unsigned decimal syntax with no plus or leading zeros. Browser projection is a distinct wire layer and never changes native canonical bytes.

Unit enums are strings with exact PascalCase spellings. Newtype enums are one-key objects, e.g. `{"Move":"C"}`, `{"Giver":"Pass"}`, `{"Blocked":"TransferDisabled"}`, `{"Observed":{...}}`. Struct variants are one-key objects whose value is a strict object containing precisely its listed fields. Options serialize as their value or null, never absent. Native fields and enum variants are case-sensitive. Role-indexed pairs are always Giver, Receiver; cells always P, O, C; occupants ordered by role, never Agent ID. Arrays preserve order.

Agent IDs are opaque nonempty distinct Unicode strings. Whitespace-only is nonempty. Default Giver is `Agent-A`, Receiver `Agent-B`. IDs are never parsed as schedule IDs or ordered to determine execution. Selected input byte limits bound label size; the native domain imposes no additional hidden length cap. Opaque consistent renaming changes name-bearing trace hashes but preserves physics, decisions and payoff classifications.

## Domain, kernel and local interface schema

The following names and signatures are normative. Validated domain structs have private fields and read-only accessors; field declarations here specify their serialized shape, not public mutation.

```rust
Role = Giver | Receiver;
InitialOwner = Giver | Receiver;
Cell = P | O | C;
GiverToken = Wait | Approach | Pass;
ReceiverToken = Wait | Advance;
Token = Giver(GiverToken) | Receiver(ReceiverToken);
OwnWord = Giver([GiverToken;6]) | Receiver([ReceiverToken;6]);
Intent = Wait | Move(Cell) | Pass | Pickup | Deliver;
Termination = Delivered | Deadline;
Phase = Initial | BeforeAction | AfterAction | Finished;
ResourceLocation = Cache | Carried | Delivered;
BlockReason = AlreadyAtOutside | TransferDisabled | NotKeyOwner |
              NotColocated | NoToken | WrongPosition | EmptyHands | FullHands;
ActionOutcome = Success | Blocked(BlockReason);

Stratum { transfer_enabled:bool, transfer_cost:u8, giver_share:u8, distance:u8 }
Ids { giver:String, receiver:String }
Config { stratum:Stratum, ids:Ids, initial_owner:InitialOwner }
Profile { giver:u16, receiver:u8 }
Clock { round:u8, phase:Phase, role:Option<Role> }
PhysicalState {
  positions:[Cell;2], receiver_cargo:bool, resource:ResourceLocation,
  key_owner:Role, spent:[u8;2], reward:[u8;2]
}
EventKind = Action { role:Role, token:Token, intent:Intent,
                     result:ActionOutcome, paid:u8 } | Terminal(Termination);
Event { clock:Clock, kind:EventKind, before:PhysicalState, after:PhysicalState }
VisibleOccupant { role:Role, id:String, cargo:bool, key:bool }
VisibleCell { cell:Cell, occupants:Vec<VisibleOccupant>, loose_token:bool }
Observation { clock:Clock, cells:Vec<VisibleCell> }
OwnershipReceipt { clock:Clock, counterpart:String, before:bool, after:bool }
LocalEntry = Public { clock:Clock, termination:Option<Termination> } |
             Observed(Observation) |
             OwnAction { clock:Clock, token:Token, intent:Intent,
                         result:ActionOutcome, paid:u8 } |
             Ownership(OwnershipReceipt) |
             Settlement { clock:Clock, reward:u8, net:i16 };
View {
  role:Role, word:OwnWord, own_cell:Cell, own_cargo:bool, own_key:bool,
  credits:u8, own_reward:u8, own_net:i16, clock:Clock,
  history:Vec<LocalEntry>, last_observation:Observation
}
Outcome {
  spent:[u8;2], reward:[u8;2], net:[i16;2], group_net:i16,
  delivery_round:Option<u8>, giver_attempts:[u8;3], giver_blocked:[u8;3],
  receiver_attempts:[u8;4], receiver_blocked:[u8;4], receiver_waits:u8,
  transfer_count:u8, approach_spent:u8, pass_spent:u8, unused_credits:[u8;2]
}
Episode {
  version:u16, config:Config, profile:Profile, words:[OwnWord;2],
  events:Vec<Event>, histories:[Vec<LocalEntry>;2], outcome:Outcome
}
CapturePoint { clock:Clock, views:[View;2], physical:PhysicalState }
Error = InvalidInput(String) | InvalidHistory(String) | InvalidReport(String) |
        ResourceLimit { resource:String, limit:u64 } | ArithmeticOverflow | Io(String);

Stratum::new(e:bool,t:u8,b:u8,d:u8) -> Result<Stratum,Error>;
Ids::new(giver:String,receiver:String) -> Result<Ids,Error>;
Config::new(stratum:Stratum,ids:Ids,initial_owner:InitialOwner) -> Result<Config,Error>;
Profile::new(giver:u16,receiver:u8) -> Result<Profile,Error>;
choose(&View) -> Result<Intent,Error>;
run_episode(&Config,Profile) -> Result<Episode,Error>;
run_with_capture(&Config,Profile,&mut dyn FnMut(&CapturePoint)->Result<(),Error>)
  -> Result<Episode,Error>;
trace_hash(&Episode) -> Result<String,Error>;
```

Stratum domains are e false/true, t∈{1,2,4}, b∈{0,3,6}, d∈{0,1}. Profile domains are Giver 0..728 and Receiver 0..63. Episode.version is exactly 1. Encode words in radix 3 and 2 respectively, most-significant-round first, exactly six digits including leading zeros; Giver 0/1/2 = Wait/Approach/Pass, Receiver 0/1 = Wait/Advance. Episode.words must equal both decoded profile words, in role order, even if suffixes never execute. Preserve every raw schedule separately.

Round zero occurs only at Initial with role null. BeforeAction and AfterAction use round 1..6 and the active role; Finished uses round 1..6 and role null. No reset, opening, closing or postterminal phase exists. Observations and entries obey the chronology below, not merely individual field ranges. All executed counts are 0..6, transfer_count 0..1, paid 0..4, spending Giver 0..24 and Receiver 0..6, rewards Giver 0..6 and Receiver 0..12, unused credits 8..32 and 26..32. Private nets are within [-24,6] and [-6,12], group within [-30,12]; reachable values must also exactly reconstruct. Delivery round is null for Deadline, otherwise 4..6. InitiallyEquipped shares these bounds. approach_spent is 0..6 and pass_spent 0..24; both must satisfy their exact count formulas. Episode.events has 9..13 entries under reachable six-round execution; each history has at most 47 entries and each word exactly six tokens. A capture sequence has 18..26 points (Initial, two per executed slot, Finished). Visible cells have length 2 or 3 exactly as location specifies, occupants length 0..2 with unique roles/IDs. No loose/cargo duplication is valid: Cache iff loose resource exists and cargo false; Carried iff Receiver cargo true; Delivered iff cargo false and successful delivery committed. Giver cargo is always false. Key owner is always exactly one role, including after delivery. Giver positions P/O and Receiver O/C only.

Own net always equals own reward minus own spending, including before settlement. Credits are 32 minus spending and never increased by reward. Public initial knowledge is the declared topology, capabilities, budgets, horizon, stratum, positions, resource and initial owner. This is supplied immutable knowledge, not a live global sensor. `choose` is pure and called only on a valid active BeforeAction View; it selects the round's own token and derives Advance from own position/cargo. It cannot inspect Config for dynamic world state, the peer word/credits/history, full events, compiler analysis or capture physical state. The static model rules require no peer field in View. Invalid role/word/clock/state/history combinations return InvalidHistory; never fabricate a valid blocked result for a corrupt world.

## Physics and exact chronology

The graph is P–O–C. P–O is always traversable; both directions of O–C require the moving Agent's actual key possession. Initial positions are [O,O] at d=0, [P,O] at d=1. Resource starts Cache, cargo false, spending/reward zero. InitialOwner selects the sole owner; main census always Giver. No global gate-open state, Hold, dropping, copying, remote transfer, theft or return action exists.

Every action is a checked transaction. Validate preconditions, arithmetic, candidate state, event/history payloads and resource charges before authoritative mutation. A resource error or failed capture aborts with Error and cannot return a successful partial Episode. Pass commits ownership and both own receipts as one indivisible transaction; no observer or capture sees an intermediate ownerless/duplicate-owner state or only one receipt.

1. Initial clock `{round:0,phase:"Initial",role:null}`: append Public(clock,null) to both histories, then Observed(initial local map) to each, role order. Emit Initial capture after both observations. There is no Initial Event.
2. For round q=1..6, Giver then Receiver, set BeforeAction(q,role), append Public(clock,null) to both histories, append Observed only to acting Agent's history, then emit BeforeAction capture. Choose from the active View. There is no extra round-start transaction or capture.
3. Snapshot before. Compute declared charge and success/block reason against pre-action state, apply own charge even when blocked, and apply successful physical primitive. Set AfterAction(q,role). Append Action Event with exact before/after snapshots. Append Public(AfterAction,null) to both histories, then OwnAction to actor. If this is successful Pass, append Ownership to Giver then Receiver, with the same AfterAction clock, counterpart's exact configured ID, and own Boolean possession before/after (Giver true→false, Receiver false→true). These two receipts are committed before Giver's postaction sensing. Then append Observed only to actor and emit AfterAction capture. Receipt delivery never refreshes Receiver's last_observation, cells, or sensing timestamp; its own_key does immediately reflect its receipt. Failed Pass has no Ownership entry in either history and no Receiver notification except the ordinary public clock.
4. Successful Deliver in step 3 consumes Receiver cargo and sets resource Delivered. Its Action after snapshot, postaction observation and AfterAction capture still have reward [0,0]. Immediately afterward finish Delivered in q. Otherwise finish Deadline after Receiver round-six AfterAction. Unexecuted slots have no charge/event/observation/count.
5. Finished(q,null): snapshot before; set reward [b,12−b] once for Delivered or retain [0,0] for Deadline, without any key/position/resource change. Append one Terminal Event. Append Public(Finished,termination) to both, then Settlement(Finished,own reward,own net) to each, Giver first, including zeros. Emit Finished capture; no terminal observation. Outcome derives from committed events and terminal state. There is exactly one Terminal event and settlement per Agent.

Every capture has both Views.clock equal its public clock; history is the exact committed own prefix, last_observation remains the most recent actual Observed entry with its original timestamp. Own state and own ledgers reflect current authoritative own values. The passive Agent receives public clock and any narrowly specified ownership or settlement entry, never a refreshed map or peer action/ledger. Read-only capture/replay adds no sensing or receipts. Views/capture copies are excluded from Episode; the histories they expose occur exactly once in Episode.

Wait succeeds at cost zero without physical change. Approach costs one: P→O succeeds; at O block AlreadyAtOutside. Pass costs t: check permission, then Giver possession, then co-location, in that order; first failure is TransferDisabled, NotKeyOwner or NotColocated. Success changes key_owner Giver→Receiver and nothing else physical. Receiver acceptance consumes no slot or credit. Later Pass attempts pay t and fail; disabled Pass pays t and never transfers even if otherwise eligible.

Receiver Advance costs one and performs exactly one primitive using own state: (O,false)→Move(C); (C,false)→Pickup; (C,true)→Move(O); (O,true)→Deliver. Move crosses only when Receiver owns the key; otherwise block NotKeyOwner. Pickup changes cargo false→true and resource Cache→Carried. Deliver changes cargo true→false and resource Carried→Delivered. For direct intent/history validation, role/capability/legal-destination violations are operational errors; then physical checks occur position (WrongPosition), cargo (Pickup FullHands or Deliver EmptyHands), resource (Pickup NoToken), possession for crossing (NotKeyOwner). WrongPosition/FullHands/EmptyHands/NoToken cannot arise in a valid complete episode with the supplied controller, but their diagnostic spellings are fixed. Registered schedules never exhaust budgets; exceeding 32 is an operational invalid state, never automatic Wait or an extra failure mechanism.

Blocked attempts preserve position/cargo/resource/owner/reward, changing only actor spending. Giver attempt/blocked arrays order Wait,Approach,Pass. Receiver arrays order outbound Move(C),Pickup,return Move(O),Deliver; Receiver Wait uses receiver_waits. Wait never blocks. Successful counts are attempts minus blocked. transfer_count equals successful Pass count. approach_spent equals Approach attempts; pass_spent=t×Pass attempts; their sum equals Giver spending. Receiver spending equals sum of its four attempts. Outcome reward/spent match terminal snapshots; net=reward−spent; group_net=sum(net); unused_credits=32−spent.

## Local sensing

Static distance≤1 defines visibility, even across the locked edge: at P cells [P,O], at O [P,O,C], at C [O,C]. Include empty visible cells. Occupants expose role, configured identity, actual cargo and actual own key possession only, in role order. loose_token is true exactly at C when resource Cache. A visible peer's possession is dated sensed evidence, not a continuing remote owner feed. Observation contains no global owner, peer word/credits/reward/net/history, gate-open field or evaluator field. No inference is inserted into Observed entries. Visible occupants can include both Agents without blocking motion.

Receipt counterpart identity is supplied and narrowly scoped; no peer spending/reward/word is added. Current own possession may therefore disagree with a stale self-occupant image until the next own sensing; retain both timestamps and label the receipt separately in presentation. Settlement similarly updates each own ledger without sensing or revealing peer finances. Unknown-Agent views are empty masks even at termination and in comparisons.

## Complete Episode identity and receipts

Canonicalize the entire validated native Episode recursively by sorting object keys lexicographically by Unicode scalar value, retaining every array order. Emit compact JSON with comma/colon separators, exact integer number atoms, lowercase true/false/null, UTF-8 with no BOM or final newline. Strings encode Unicode directly, escaping quote/backslash and controls only; use short escapes for backspace/tab/newline/formfeed/carriage return and lowercase `\u00xx` for other controls. No slash or non-ASCII escaping. All field names are ASCII, so Rust lexical ordering and Python sorting agree.

Include version, every Config field including IDs/InitialOwner, full Profile and words, every event/snapshot, both full local histories including ownership/settlement entries, and complete Outcome. Exclude only data not in Episode: classifications, selectors, compiler stats, limits, capture/prefix copies, source/reference/toolchain/artifact/report receipts, elapsed time, actual memory and the trace hash itself. Never exclude unexecuted words, IDs, zero settlement or null fields.

`Receipt { path:String, bytes:u64, sha256:String }` describes raw exact bytes. SHA strings are exactly 64 lowercase hexadecimal digits. Path labels are nonempty relative ASCII slash-separated names without empty, dot or dot-dot components, backslash, controls, quotes or absolute prefix. Unique path labels within each collection; order is declared, never sorted implicitly. For ordered receipts serialize the compact array with receipt object field insertion order `path,bytes,sha256` (not recursively sorted) and hash those bytes once with SHA-256. This outer identity differs from each raw content digest.

For trace identity, canonical body B yields the single receipt `{"path":"key-delegation-trace-v1","bytes":len(B),"sha256":SHA256(B)}`; hash its one-item receipt array as above. `len` counts UTF-8 bytes. The source-blind implementation uses only standard-library JSON and hashlib; exact schema validation precedes canonicalization. Canonical Unicode/control-string witnesses and both digest layers must agree across native/WASM/reference. For report-content identity substitute canonical full Report and path `key-delegation-report-v1`. Exported CLI JSON has one trailing newline, whose raw file receipt remains separate. Summary has no self hash.

## Game, selectors and analysis schema

```rust
Limits { profiles:u64, transitions:u64, allocation_bytes:u64 }
CompileStats { profiles:u64, transitions:u64, peak_bytes:u64 }
Row { outcome:Outcome, trace_hash:String }
Classification {
  giver_regret:i16, receiver_regret:i16, is_pure:bool, team_gap:i16,
  outcome:Outcome
}
Analysis {
  giver_best:Vec<i16>, receiver_best:Vec<i16>, pure:Vec<Profile>,
  team:Profile, lex_pure:Option<Profile>, best_group_pure:Option<Profile>,
  solo_best:i16
}
GameTable { stratum:Stratum, rows:Vec<Row>, analysis:Analysis, stats:CompileStats }
Selector = SoloAttempt | SuppliedHandoff | TeamOptimal | NashLex |
           NashBestGroup | InitiallyEquipped;
Selection { initial_owner:InitialOwner, profile:Profile,
            classification:Option<Classification> }
ClassificationScope = main | changed_initial_control; // exact string values
ControlRecord {
  selector:Selector, classification_scope:ClassificationScope,
  selection:Option<Selection>, episode:Option<Episode>, unavailable:Option<String>
}
Representative { profile:Profile, classification:Classification }
PayoffRanges { giver:Option<[i16;2]>, receiver:Option<[i16;2]>, group:Option<[i16;2]> }
EquilibriumSummary {
  pure_count:u64, delivering_pure_count:u64, ranges:PayoffRanges,
  range_unavailable:Option<String>, delivery_round_range:Option<[u8;2]>,
  delivery_unavailable:Option<String>, team_gap_range:Option<[i16;2]>,
  lex:Option<Representative>, best_group:Option<Representative>
}
SummaryControl {
  selector:Selector, classification_scope:ClassificationScope,
  selection:Option<Selection>, outcome:Option<Outcome>, unavailable:Option<String>
}
SummaryStratum {
  stratum:Stratum, stats:CompileStats, equilibrium:EquilibriumSummary,
  team:Representative, solo_best:i16, controls:Vec<SummaryControl>
}
Limits::registered() -> Result<Limits,Error>;
compile(Stratum,&Limits) -> Result<GameTable,Error>;
classify(&GameTable,Profile) -> Result<Classification,Error>;
resolve(&GameTable,Selector) -> Result<Option<Selection>,Error>;
```

Enumerate strata with e false,true outer; t 1,2,4; b 0,3,6; d 0,1 inner. There are exactly 36, each with all 729×64=46,656 rows, Giver ID outer and Receiver inner, index `64*g+r`; total 1,679,616 raw pairs. Main compile always starts Giver owner and uses default IDs. No seeds, sampling, pruning, adaptive grid, outcome merging or behavioral weighting. InitiallyEquipped is separate execution, never a main-table row or equilibrium classifier input.

giver_best[r] is max Giver net over all g with that r (64 entries); receiver_best[g] is max Receiver net over all r with that g (729 entries). Regrets are these maxima minus own net, nonnegative. is_pure iff both regrets zero. pure retains all ties in (g,r) order, including identical realized traces and unexecuted suffixes. team_gap is maximum group net minus row group_net, nonnegative. Regrets/team_gap fit i16 with semantic upper bound 60. Pure membership has no spending/delivery tie-break.

Team representative minimizes `(-group_net, total_spent, delivery_round_or_7, giver, receiver)`. lex_pure is first pure Profile or null; best_group_pure minimizes team ordering within pure or null. solo_best is maximum Receiver net over rows with Giver ID zero, distinct from SoloAttempt's payoff. Names never enter ranking. Team and individual execution have the same supplied initial knowledge and local interface; solver objective grants no extra execution sensor.

Control order is the Selector order shown. SoloAttempt is Giver Wait×6, Receiver Advance×6. SuppliedHandoff near is Giver [Pass,Wait,Wait,Wait,Wait,Wait], Receiver [Advance,Advance,Advance,Advance,Wait,Wait]; far is Giver [Approach,Pass,Wait,Wait,Wait,Wait], Receiver [Wait,Advance,Advance,Advance,Advance,Wait]. Disabled permission retains these exact words. TeamOptimal/NashLex/NashBestGroup select the respective Analysis profiles. InitiallyEquipped always changes initial owner to Receiver, uses Giver Wait×6 and Receiver [Advance,Advance,Advance,Advance,Wait,Wait], with classification null and scope changed_initial_control. All main selections use owner Giver, non-null classification and scope main.

Available ControlRecord has selection and episode present, unavailable null, and matching Config/profile/Outcome; SummaryControl retains the same Selection and Outcome without history. Only NashLex/NashBestGroup may be unavailable, exactly for an empty pure set, with selection/episode/outcome null and unavailable `no_pure_equilibrium`. Scope remains main. InitiallyEquipped remains available and must never be labeled equilibrium/pure/team-gap qualified; its classification is explicitly null in Selection and browser payload.

Pure ranges are min/max over all pure rows, not distinct outcomes. Empty pure set has zero counts, all ranges/representatives/team_gap_range null and range_unavailable `no_pure_equilibrium`; otherwise all are present and range_unavailable null. delivery_round_range uses delivering pure rows only; if empty, null with delivery_unavailable `no_delivering_pure_equilibrium`, otherwise range present and reason null. Counts are 0..46,656. Range endpoints are ordered and reconstruct exactly; main analysis/team/controls are recomputed from rows rather than trusted. All six control records are retained per stratum, exactly 216 in report and summary.

## Contrasts and complete reporting schema

```rust
ContrastAxis = TransferPermission | TransferCost | GiverShare | Distance;
OutcomeDelta {
  spent:[i16;2], reward:[i16;2], net:[i16;2], group_net:i16,
  delivery_round:Option<i16>, delivery_unavailable:Option<String>,
  giver_attempts:[i16;3], giver_blocked:[i16;3],
  receiver_attempts:[i16;4], receiver_blocked:[i16;4], receiver_waits:i16,
  transfer_count:i16, approach_spent:i16, pass_spent:i16, unused_credits:[i16;2]
}
ClassificationDelta { giver_regret:i16, receiver_regret:i16, team_gap:i16 }
FixedWordDelta { profile:Profile, delta:OutcomeDelta }
FixedWordSummary {
  profiles:u64, delta_hash:String, physical_equal_count:u64,
  spending_equal_count:u64, group_net_equal_count:u64,
  delivery_equal_count:u64, share_rule_count:Option<u64>
}
ControlContrast {
  selector:Selector, lower:SummaryControl, upper:SummaryControl,
  delta:Option<OutcomeDelta>, classification_delta:Option<ClassificationDelta>,
  classification_unavailable:Option<String>, unavailable:Option<String>
}
RangeDelta { giver:[i16;2], receiver:[i16;2], group:[i16;2], team_gap:[i16;2] }
EquilibriumContrast {
  lower:EquilibriumSummary, upper:EquilibriumSummary,
  pure_count:i64, delivering_pure_count:i64,
  range_delta:Option<RangeDelta>, range_unavailable:Option<String>,
  delivery_round_range_delta:Option<[i16;2]>, delivery_unavailable:Option<String>
}
Contrast {
  axis:ContrastAxis, lower:Stratum, upper:Stratum,
  fixed_words:FixedWordSummary, controls:Vec<ControlContrast>,
  equilibrium:EquilibriumContrast
}
Check { name:String, passed:bool, detail:String }
Panel { stratum:Stratum, rows:Vec<Row>, analysis:Analysis,
        controls:Vec<ControlRecord>, stats:CompileStats }
Report {
  version:String, source_hash:String, source_receipts:Vec<Receipt>,
  reference_receipts:Vec<Receipt>, limits:Limits, panels:Vec<Panel>,
  contrasts:Vec<Contrast>, checks:Vec<Check>, passed:bool
}
Summary {
  version:String, report_hash:String, source_hash:String,
  source_receipts:Vec<Receipt>, reference_receipts:Vec<Receipt>, limits:Limits,
  settings:Vec<SummaryStratum>, contrasts:Vec<Contrast>, checks:Vec<Check>, passed:bool
}
diagnose(&Limits) -> Result<Report,Error>;
report_integrity(&Report,&Limits) -> Result<bool,Error>;
summary_json(&Report,&Limits) -> Result<String,Error>;
```

Report.version is `key-delegation-diagnostic-v1`; Summary.version `key-delegation-summary-v1`. Both contain all 36 strata in registered order, every six-control array in Selector order, and all 84 contrasts. No omitted/duplicated/reordered panel, row, selector, profile, receipt or contrast is tolerated. Explicit nullable fields stay present in report and summary. Limits passed to validation must equal registered Limits and stored Limits. Check/detail strings are diagnostic text, not additional scientific values; detail is required nonempty Unicode, does not determine validity, and never hides a failed recomputation. Hashes follow the exact lowercase 64-hex type; receipt sizes, counters and caps are checked u64, with positive caps. No elapsed-time/RSS field occurs in scientific Report.

Contrast order is axis order above, then scan lower strata in registered order and upper strata in registered order, retaining only matched pairs with unchanged other coordinates. Permission pairs false→true; price pairs only 1→2 and 2→4; share pairs only 0→3 and 3→6; distance 0→1. Thus 18+24+24+18=84; price 1→4 and share 0→6 do not enter this adjacent design. All deltas are upper minus lower, componentwise and checked. Numeric delta fields are bounded by endpoint domains, counts differences fit i64, and no unsigned subtraction is used.

OutcomeDelta includes every Outcome field; all arrays keep original order. delivery_round delta exists iff both endpoints delivered; otherwise null and delivery_unavailable `delivery_round_unavailable`. When both deliver, reason is null. delivery_equal_count requires equal Option<round>, so two nulls count equal; group_net/spending equal counts compare exact values. Delta includes transfer/action counts, cost components and unused credits, not only payoffs. No Boolean is subtracted: each endpoint retains is_pure and classifications.

For each contrast, exhaustively form the ordered 46,656 FixedWordDelta array from same-profile endpoint rows. Compute its canonical bytes and outer one-item receipt identity with path `key-delegation-fixed-word-deltas-v1`. `delta_hash` is that outer identity. The lower/upper strata and axis are retained explicitly in Contrast; for unambiguous domain binding the hashed body is the strict object `{axis:ContrastAxis,lower:Stratum,upper:Stratum,rows:Vec<FixedWordDelta>}`, not an unlabelled array. Full arrays are retained in both independent numerical derivations before comparison. Production Report need not duplicate them: its two complete endpoint panels reconstruct every value and the identity. No sampling or digest-only numerical comparison is accepted.

physical_equal_count compares complete ordered Action/Terminal trajectories projected to each event's clock and, for Actions, role/token/intent/result, plus before/after positions, receiver_cargo, resource and key_owner. Strip spent/reward/paid and ignore Config, Episode Outcome, histories, names and raw words only for this physical comparison. Terminal kind remains included. These exclusions are confined to this explicitly named physical projection, never whole-Episode trace hashing. Counts are 0..46,656, profiles exactly 46,656. Fixed-share contrasts additionally require share_rule_count=46,656: for each pair, physical projection, spending, attempts/blocks/transfers, delivery and group net equal; if delivered, reward and net changes are [+Δb,−Δb]; if not delivered, both are [0,0]. For non-share axes share_rule_count is null. Every fixed-word count is reported for every axis even when no equality is predicted; never silently treat permission or distance as mere reward changes. Full hashes normally differ across changed Configs even when physical trajectories agree.

Control endpoints are separately reselected under each stratum. Each ControlContrast preserves both SummaryControls; selector agrees at all three locations. If both available, delta exists and unavailable null. If either unavailable, delta null and unavailable `lower_control_unavailable`, `upper_control_unavailable` or `both_controls_unavailable`. Available main/main endpoints have classification_delta present and classification_unavailable null; changed-initial endpoints have classification_delta null and classification_unavailable `changed_initial_control`. Missing main endpoints use the same unavailable reason for classification_unavailable. Do not mix InitiallyEquipped with a main row or use it as a matched transfer estimate. Reselected changes are not fixed-policy causal effects even if selected words happen to coincide.

EquilibriumContrast retains both complete equilibrium summaries and always reports upper-minus-lower pure and delivering counts. If both pure sets are nonempty, range_delta subtracts corresponding min endpoints and max endpoints separately, including team_gap, and range_unavailable is null. Otherwise range_delta is null and reason `lower_pure_set_empty`, `upper_pure_set_empty` or `both_pure_sets_empty`. Delivery range delta requires both delivering subsets nonempty, otherwise null with reason `lower_delivering_pure_set_empty`, `upper_delivering_pure_set_empty` or `both_delivering_pure_sets_empty`. These compare range endpoints, not arbitrarily paired equilibria.

Report checks have unique exact names in this order: `registered_census`, `reference_outcomes`, `reference_trace_hashes`, `reference_controls`, `best_responses`, `pure_membership`, `representative_ties`, `fixed_word_contrasts`, `reward_share_invariance`, `resource_limits`, `source_and_reference_identity`. Report.passed is true iff all freshly recomputed checks pass. A false stored flag cannot turn invalid evidence into a valid different result; a true flag or supplied hash is never authority. Integrity checks current source/reference receipts, replayed complete Episodes and traces, each raw row, every analysis/control/contrast and arithmetic/cap value against sealed independent references. Semantic disagreement returns false; malformed/stale identity, invalid schema/history, arithmetic/cap/I/O errors return contextual Error. The summary requires successful fresh integrity reconstruction of the retained Report; it does not run `diagnose` again or merely trust serialized checks/classification.

## Portable work and payload accounting

Limits are per `compile(stratum)`, counters start at zero, never reset within a stratum. `profiles` charges one before each of the 46,656 evaluations. `transitions` charges one before every appended Event, including Action and Terminal, excluding Initial, observations, receipts, capture and solver comparisons. A successful Pass is one Event with two required own-history receipts; receipt bytes are charged below. In-world credits, event work, logical payload, actual allocator memory and elapsed time remain separate units.

For any schema value J, L(J) is exact UTF-8 length of its recursively sorted canonical JSON. Let A(n) be sum L(Row) over n retained rows, P(n)=4n bytes for two signed i16 payoff values per row, M=2(64+729) bytes for response maxima, and B=9 bytes for team/lex/best Profile bookkeeping (3 bytes each). M+B remain charged throughout a compile, including initial state. Interning or reusing rows/payoffs may not reduce this logical charge. Pure membership Q(k)=3k bytes is charged as each pure Profile is appended during analysis. These portable packed widths intentionally omit language/allocator alignment and capacity.

For profile j, enumerate accounting boundaries after initial commit, each BeforeAction commit, each complete AfterAction commit including both Pass receipts before capture, and Finished commit. Construct a partial Episode with unchanged version/Config/profile/words, exact committed event/history prefixes, and current Outcome counters/state. Its delivery_round becomes q as soon as successful Deliver Action commits (before settlement); null before that. Rewards/net/unused credits reflect current state at that boundary. These accounting snapshots are not valid completed Episodes and cannot be imported or accepted as scientific outcomes.

At each boundary E, scratch S(E)=2L(E)+L(T(E))+64, where T(E) is the one-item trace receipt array over E's canonical bytes with path `key-delegation-trace-v1`, exact length and 64-character raw SHA. L(T) means its compact receipt-codec length; its key order does not affect byte count. One L(E) charges typed logical Episode payload, the other the canonical buffer, plus receipt and final 64-byte hex hash. Partial scratch hashes need not be computed merely to count fixed hex width. Candidate boundaries must be charged before mutation; capture never creates an additional boundary. Use the maximum over all listed boundaries, not an assumption that final JSON length is greatest (null→digit and settlement changes can shorten fields).

Before each profile j, retain A(j−1)+P(j−1)+M+B and no earlier profile scratch. At each boundary charge this base+S(E). At row append, final completed scratch and new row/payoff are simultaneously live, so charge A(j)+P(j)+M+B+S(final). Then drop scratch before the next profile. Analysis charges A(N)+P(N)+M+B+Q(k) for every appended pure member, and finally that same total plus L(Analysis) while temporary analysis storage is still live. After final analysis scratch is dropped, rows and Analysis remain. `peak_bytes` is the maximum across these initial, boundary, append and analysis totals. All sums/products/increments are checked u64 and prospective limit failures occur before retaining/mutating the candidate. A cap failure returns Error, never a partially valid GameTable or non-delivery row.

The independent author/reviewer derive every per-stratum `CompileStats`, boundary/append/analysis maxima and their witnesses from protocol-only episodes/serialization. `Limits::registered()` reads exactly twice the largest independently approved requirement across all 36 strata for each of profiles, transitions, peak_bytes, with checked multiplication. This is a formula and safety factor, not permission to guess numeric values or tune after judged results. No numerical caps are fixed here. Independent limits-reference seals them before production compilation/collection. A later feasibility failure preserves evidence and requires reviewed amendment; do not silently enlarge caps, sample, prune or loosen timeouts.

Per-stratum compile accounting excludes controls outside compile, report construction/serialization, source/reference receipt construction, actual allocations and browser capture copies. Whole-diagnostic feasibility must independently retain actual artifact size, memory and elapsed time evidence and show complete first/repeat capture fits; these engineering quantities are not claims established by logical caps. Browser selected records keep 64 KiB input/link, 16 MiB record, 4,096 checkpoints, one outstanding request, 60-second worker timeout and two retained records. Enforce projected record/checkpoint size before append, including complete history copies and lossless conversion. No exception for the new study and no whole diagnostic in a selected request.

## Compiled closure and selected browser boundary

New scientific source lives only under `crates/sugarscape-core/src/key_delegation/`. `SOURCE_INPUTS` embeds the following exact repository-relative labels and raw bytes, in this order:

1. `crates/sugarscape-core/src/key_delegation/mod.rs`
2. `crates/sugarscape-core/src/key_delegation/types.rs`
3. `crates/sugarscape-core/src/key_delegation/validation.rs`
4. `crates/sugarscape-core/src/key_delegation/controller.rs`
5. `crates/sugarscape-core/src/key_delegation/observation.rs`
6. `crates/sugarscape-core/src/key_delegation/kernel.rs`
7. `crates/sugarscape-core/src/key_delegation/canonical.rs`
8. `crates/sugarscape-core/src/key_delegation/sha256.rs`
9. `crates/sugarscape-core/src/key_delegation/budget.rs`
10. `crates/sugarscape-core/src/key_delegation/game.rs`
11. `crates/sugarscape-core/src/key_delegation/metrics.rs`
12. `crates/sugarscape-core/src/key_delegation/diagnostics.rs`
13. `crates/sugarscape-core/src/key_delegation/source_inputs.rs`
14. `crates/sugarscape-core/src/key_delegation/protocol.rs`
15. `crates/sugarscape-core/src/key_delegation/references.rs`
16. `Cargo.toml`
17. `crates/sugarscape-core/Cargo.toml`
18. `Cargo.lock`

`REFERENCE_INPUTS` independently embeds exactly `crates/sugarscape-core/src/key_delegation/tests/fixtures/kernel-reference.json`, `game-reference.json`, `limits-reference.json` in that order (the latter two share the same full directory prefix). Authenticate these separately in Report.reference_receipts and browser identity; separate does not mean omitted dependency binding. SOURCE_INPUTS source_hash is the ordered outer receipt identity of the 18 source entries. References' outer identity is recomputed from all three exact fixture receipts. Source/protocol reference code cannot store a computed hash of itself. Runtime computes actual raw SHA-256 locally in the new sha256.rs, adapting the existing portable algorithm without editing the original. Independent known-answer tests bind its behavior. No new host generator is authorized; core build.rs and build_support/surface_discovery.rs remain byte-unchanged frozen Gate inputs. No derived receipt is included in its own hash.

External freeze additionally binds this protocol, approved design/plan, all independent derivations and reviews, protocol constants, crate facades, CLI/WASM adapters, toolchains, targets, flags, dependency artifacts, native executable, original WASM and generated JS/WASM/package/declarations separately. Audit actual includes/imports before freeze; any extra consumed source/data requires reviewed explicit closure extension. Actual source guard readers must include new Rust files (including sha256.rs), fixtures and manifests/lock through their existing rules or narrowly reviewed registration. Do not exempt a new family broadly or alter historical source identities/results.

Raw selected input is a strict object with exactly `study:"key_delegation"`, `stratum:Stratum`, `ids:Ids`, and exactly one present `selector:Selector` or `profile:Profile`. The unused choice field must be absent, not null; a present null in either choice rejects. IDs/stratum are required, with catalog defaults explicitly supplied. No `initial_owner`, classification, outcome, limits, passed flag, peer hidden state or extra field is accepted from public raw input. Only InitiallyEquipped selects Receiver ownership; raw profiles always start Giver owner. Default is e=true,t=1,b=3,d=0, SuppliedHandoff, Agent-A/Agent-B, labeled supplied minimal handoff. Existing study defaults remain unchanged.

Use existing EpisodeRecord envelope (`kind`, `version`, `study`, `rules_identity`, `input`, `semantics`, `checkpoints`, `payload`) and existing checkpoint/one-request/cancel contract. The new payload is strict:

```rust
KeyPayload {
  episode:Episode, classification:Option<Classification>,
  classification_scope:ClassificationScope, compiler_stats:CompileStats,
  trace_hash:String
}
```

For all main selected episodes classification present/scope main; InitiallyEquipped classification null/scope changed_initial_control prominently identifies changed initial ownership. compiler_stats describes the main stratum compile used for selector availability/evaluation, never an InitiallyEquipped equilibrium census. KeyPayload is converted through the existing lossless payload projection. Private checkpoint bodies contain only the matching local View projection and supplied information; Researcher sees CapturePoint physical state and evaluator data. Unknown Agent masks remain empty of both peer and evaluator data. Import validates raw duplicate keys and exact wire shape then reconstructs current native episode/checkpoints, masks, identity and classification; lossless import is not permission to trust supplied hashes.

Browser identity is exactly `key-delegation:protocol-1:browser-key-1:native-source-sha256:{native}:adapter-source-sha256:{adapter}:reviewed-reference-sha256:{references}`, with actual outer component digests. Adapter receipt inventory order is browser_experiments/key/{mod.rs,projection.rs,duplicate.rs}, browser_experiments/{mod.rs,input.rs,catalog.rs,record.rs,equivalence.rs,wire.rs,recorded.rs}, config.rs, lib.rs, each prefixed `crates/sugarscape-core/src/`. Bind further actual adapter dependencies explicitly if implementation needs them. Browser changes can change current adapter metadata only under independent review; old scientific engines, generated fixtures and frozen receipts stay byte-preserved.

## Exactly three sealed reference fixtures

References are generated only after fresh protocol approval, by a source-blind numerical author and separately sealed independent reviewer. Neither sees production key source (none exists at handoff) or the other's expected results before sealing their own. Preserve first failures, every revision, full row arrays and all independently derived delta arrays in the new ignored workspace. Compare every field, not aggregate hashes alone. Production tests must not generate their own oracle. Exactly three final JSON fixtures are accepted under the named paths; no fourth generated fixture or unreviewed preferred constants.

The following fixture schemas are strict, with the same atom/range/ordering/null rules and Receipt type above:

```rust
GoverningInputs { protocol:Receipt, design:Receipt, interfaces:Receipt }
KernelCase { name:String, config:Config, profile:Profile,
             episode:Episode, trace_hash:String, boundaries:Vec<BoundaryWitness> }
BoundaryWitness { clock:Clock, event_count:u8, history_lengths:[u16;2],
                  views:[View;2], physical:PhysicalState, scratch_bytes:u64 }
KernelReference { version:String, inputs:GoverningInputs, cases:Vec<KernelCase>,
                  independent_artifacts:Vec<Receipt> }
GameReferencePanel {
  stratum:Stratum, row_receipt:Receipt, row_identity:String,
  analysis:Analysis, controls:Vec<ControlRecord>, stats:CompileStats
}
GameReference { version:String, inputs:GoverningInputs,
                panels:Vec<GameReferencePanel>, contrasts:Vec<Contrast>,
                independent_artifacts:Vec<Receipt> }
PeakWitness { phase:String, profile:Option<Profile>, clock:Option<Clock>,
              retained_rows:u64, pure_profiles:u64, scratch_bytes:u64,
              live_bytes:u64 }
LimitPanel { stratum:Stratum, stats:CompileStats, peak:PeakWitness,
             boundary_receipt:Receipt }
LimitsReference { version:String, inputs:GoverningInputs,
                  panels:Vec<LimitPanel>, limits:Limits,
                  independent_artifacts:Vec<Receipt> }
```

Fixture versions are respectively `key-delegation-kernel-reference-v1`, `key-delegation-game-reference-v1`, `key-delegation-limits-reference-v1`. Inputs attest exact sealed governing raw bytes; they cannot be rewritten to pretend an amended protocol is the original handoff. Artifact receipts attest retained external numerical bytes, not compilation of ignored evidence. independent_artifacts is nonempty, unique-path, in lexicographic path order and excludes the fixture itself to avoid cycles. Receipt paths use stable repository-relative evidence labels. Root independently rehashes actual external files before acceptance/freeze; runtime needs only the compiled three fixtures and their attested values, not ignored filesystem access.

Game panels are all 36 ordered strata. row_receipt attests each full canonical ordered Row array, body without newline; row_identity uses its exact registered receipt path and the one-item outer codec. The full independent arrays remain retained and compared value-for-value, while fixture holds their receipts plus complete Analysis, six complete control histories and stats. All 84 Contrasts are included; full delta arrays are attested in independent_artifacts. A row receipt must not be confused with its outer identity. Endpoint panels and stratum labels prevent accidental cross-panel reuse.

Kernel cases precede full enumeration and include manually reasoned supplied near/far, disabled paid Pass, absent co-location, post-transfer repeated Pass, locked outbound, blocked Approach, all-Wait deadline, late transfer/deadline, successful delivery, unexecuted suffix, receiver initial ownership, Unicode/escaped IDs and passive-receipt-before-next-sensing chronology. Case names unique nonempty; exact case inputs and count are sealed by the independent derivations, not retrospectively selected to agree with production. Every boundary records exact local prefixes/state and portable scratch count; event_count≤13, history lengths fit u16. All intended block paths that require corrupt unreachable state are validated as rejection cases in engineering tests, not fabricated valid episodes.

Limits panels retain all 36 stats, exact logical peak witness and receipt of the full independently retained boundary/append/analysis accounting. Peak phase is one of `initial`, `before_action`, `after_action`, `finished`, `row_append`, `analysis`; profile/clock present for initial/action/finished/append profile stages, both null for analysis. On ties choose earliest streaming schedule point. row_append clock is Finished; before/after action use matching clock; initial uses Initial. retained_rows counts rows already live at that point, including just-added row on append; pure_profiles is zero outside analysis. scratch_bytes is S(E) at profile boundaries/append, zero in analysis. live_bytes is exact full formula value, stats.peak_bytes agrees. Positive registered limits follow the double-maximum formula, not measured elapsed time. No cap constants or new scientific result are asserted by these schemas.

Before numerical handoff root seals reviewed bytes and preserves independent review. Standard-library arithmetic checks establish 729×64=46,656; 2×3×3×2×46,656=1,679,616; 36×6=216; 18+24+24+18=84; and for every registered t,b,d, `(b−t−d)+(8−b)=8−t−d`. These identities are prospective checks, not a substitute for independent full derivation or scientific measurement. All-Wait, payoff indifference, non-delivery and competing equilibria remain valid results; reference derivation must test predictions rather than encode desired conclusions.

Only later reviewed freeze authorizes exactly one key first diagnostic and one repeat. Engineering selected exports and tests do not authorize additional whole diagnostics. Any changed scientific rule after results is a separate registered series. Findings must distinguish faithful finite-game computation from mechanism, generality, learning or equilibrium-selection claims.
