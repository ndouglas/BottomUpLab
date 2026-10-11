# Gate Assistance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task. The user has already selected fresh implementers and independent reviewers. Steps use checkbox syntax for tracking.

**Goal:** Build and visualize the exact private-payoff gate schedule game, retaining all equilibria and the information-matched team benchmark.

**Architecture:** A sibling native gate_assistance laboratory owns validated types, local execution, physical chronology, canonical traces, exhaustive payoff tables, and checked reporting. A bounded Rust adapter supplies the existing WASM episode worker and a gate-specific renderer in the existing Episodes shell. Independent protocol/numerical gates precede production implementation and frozen scientific collection.

**Tech Stack:** Existing Rust/serde/clap workspace, Python standard-library independent references, existing SHA receipt codec, wasm-bindgen/wasm-pack, TypeScript/Vite/Vitest and DOM/SVG. No new dependency, JavaScript physics, general planner, or navigation shell.

**Spec:** [Approved gate design](../specs/2026-10-11-gate-assistance-design.md), commit 161a664, approved by the user “LGTM!” on 2026-10-11.

**Status:** Execution approved by the user’s “LGTM!” on 2026-10-11. Tasks 1–2 are complete with independently approved protocol/references and kernel. Task 3 is beginning; no scientific measurement has begun.

## Global Constraints

- Fixed graph P–O–C, with gated O–C edge; two Agents may share a cell. Holder starts P or O; Retriever starts O. One token starts C, delivery occurs O. Capabilities and initial configuration are supplied.
- Six rounds, Holder then Retriever. Close the gate freely at each round start. Hold succeeds only at P, opening the edge for the remainder of that round. Closing cannot erase cargo or crush an Agent.
- Each Agent starts with 32 credits. Wait costs zero and consumes its opportunity. Travel costs one even when blocked; Hold costs k even when blocked; Retriever Advance costs one even when blocked and executes exactly one own-state-derived primitive.
- Delivery grants total reward 12, split b / (12−b), once, then terminates and closes freely. Unexecuted suffixes are not charged. Deadline is a valid completed non-delivery episode with zero reward and actual spending retained; operational errors are separate.
- 729 Holder words × 64 Retriever words = 46,656 raw pairs per stratum. Most-significant-round-first IDs; Holder digits Wait/Travel/Hold = 0/1/2, Retriever Wait/Advance = 0/1. No strategy pruning or merging by realized trace.
- k∈{1,2,4}, b∈{0,3,6}, distance∈{0,1}: 18 strata, 839,808 raw outcomes, up to 90 control trajectories. Preserve all pure equilibria and private payoff ties; no mixed-equilibrium substitution or invented selection mechanism.
- Team selection: maximize group net, minimize total spending, earliest delivery (none after six), then Holder ID and Retriever ID. Equilibrium tests use only private payoff maxima, with no secondary tie criterion.
- Only own schedule, committed local observations/history, own feedback and public clock/termination enter execution. Observe initially and before/after own actions; peer actions/browser capture do not add passive observations. Own terminal settlement is separate.
- Team and individual controls share public initial information, local interfaces, budgets, actions and chronology. Other schedules, actual peer private history and evaluator state are unavailable to execution.
- Browser default is labeled Supplied minimal assistance at k=1,b=3,d=0. One selected episode, 64 KiB input/link, 16 MiB record, 4,096 checkpoints, one request, 60-second worker timeout, two retained records. Evaluator payload is Researcher-only at every gate checkpoint, including terminal/comparison.
- New namespace; preserve prior engines, first outputs, guide prefixes, historical source identities and evidence. Authenticate necessary common source-guard changes; do not relabel historical measurements or broadly ignore identity differences.
- Existing .claude/worktrees/crowd, branch crowd. Inspect Git before mutations. Never stage .claude/, papers/, survey/out/ or ignored evidence; no recreate/stash/discard/force/hook bypass. Agent and American spelling in code/docs.
- Fresh producer/independent review per task and independent precollection/whole-increment reviews. One product writer/build owner. Root owns orchestration, evidence, tracking and Git. Preserve the worktree and evidence after completion.
- Rustfmt only edited files with child traversal disabled, never cargo fmt in survey. Workspace formatting checks are permitted. Stop/reassess after three failed attempts. Preserve all failed gates and superseded artifacts.
- Commit working increments, safely merge/push completed work, and verify exact CI/Pages. Frozen first/repeat use exclusive paths. Postmeasurement scientific revisions require a separate registered series; delivery changes require a separate manifest.

## Review Focus

1. Early delivery leaves unexecuted suffixes, but unilateral deviations can activate them: retain all raw strategies and test the explicit suffix witness in Task 3.
2. Holder observation changes must not refresh the passive Retriever, and browser capture must not alter knowledge: Task 2 chronology tests and Task 6 all-checkpoint DOM checks.
3. Tied private payoffs must preserve all equilibria, while team ties have a distinct selection rule: Task 3 synthetic-table tests and independent full census.
4. Duplicate nested JSON keys, Boolean/float/string IDs, forged hashes/classification and edited unblurred input must reject contextually without replacing a retained record: Tasks 4–6.
5. Terminal/comparison Actor panes must not reveal the peer schedule or evaluator payload, and unmatched clocks must remain unavailable: Tasks 5–6, including completed and deadline episodes.

---

## Files and stages

New laboratory files below are under crates/sugarscape-core/src/gate_assistance/.

| Files | Responsibility |
| --- | --- |
| docs/superpowers/protocols/2026-10-11-gate-assistance.md | Normative wire/physics/trace protocol, reference receipts, caps and final freeze |
| mod.rs, types.rs, controller.rs, kernel.rs, observation.rs | Checked configuration/words, own-only primitive choice, authoritative transitions and local history |
| canonical.rs | Sorted exact JSON and existing SHA receipt-codec reuse |
| budget.rs, game.rs, metrics.rs | Bounded complete payoff matrix, best responses, all equilibria, controls and matched metrics |
| diagnostics.rs, diagnostics/protocol.rs, diagnostics/references.rs | Full ordered census, fresh integrity, compact measured summary and provenance |
| tests/{mod,types,kernel,observation,canonical,game,metrics,diagnostics}.rs | Owning behavioral/reference/tamper tests |
| tests/fixtures/{kernel,game,limits}-reference.json | Compact independently approved witnesses/metadata, linking full ignored tables |
| crates/sugarscape-cli/src/gate_assistance.rs, tests/gate_assistance.rs, src/main.rs | Fixed gate-assistance diagnose and checked output |
| crates/sugarscape-core/src/browser_experiments/gate/{mod,projection,duplicate}.rs | Strict selected input, native capture/projection, gate-only nested-key rejection |
| Browser {mod,input,catalog}.rs and tests/gate.rs | Additive seventeenth descriptor, dispatch, strict input and import |
| web/src/episodes/{gate-projection,gate-view,gate-comparison}.ts | Allowlisted native gate rendering and exact matched comparisons |
| web/src/episodes/{gate-projection,gate-parity,gate-input,gate-comparison}.test.ts, gate.browser-test.ts | DOM, actual Node WASM/native, input/comparison and actual browser acceptance |
| Browser recorded.rs, fixtures/gate-recorded.json, web recorded.ts/tests | Measured compact census registration, separate from episode runs |
| docs/gate-assistance.md, docs/{roadmap,papers,reproducibility}.md | Correct findings, source scope, stable claim IDs and next milestone |

Add only the gate export to core lib.rs. Common browser/TypeScript types, controls, view, catalog, projection and source guards change only at named integration seams. Preserve every original study. F5's explicit fixture selector/mutation test changes only for newly compiled gate JSON inputs; W1 closure must be inspected accurately, not assumed universal.

On execution approval, create an owned IMPLEMENTATION_PLAN.md with five stages and Goal/Success Criteria/Tests/Status: (1) protocol/reference; (2) kernel/game; (3) diagnostic; (4) adapter/viewer; (5) frozen collection/delivery. If a foreign tracker exists, use docs/superpowers/plans/2026-10-11-gate-assistance/IMPLEMENTATION_PLAN.md and record ownership. Retained plan workspace is the skill-generated .superpowers/sdd/2026-10-11-gate-assistance/. Check its identity/progress ledger before dispatching; never redispatch completed tasks or read sibling ledgers as this plan's progress.

Root archives status/HEAD, locks/manifests, old scientific source receipts/first reports, guide prefixes, current source guards, catalog IDs, and all 16 same-target default native/WASM records before integration changes. Every common identity delta must be enumerated and authenticated. Preserve original exports; where additive common dependencies legitimately change current rules_identity, prove complete record-body equality excluding only that authenticated identity field, retain both identities and reject stale/unknown imports. Do not claim whole-export byte equality when that field differs.

## Shared contracts

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

## Task 1: Declarative protocol, independent census and preservation

**Files:** protocol document; three compact reference fixtures; ignored reference programs/tables, sealed inputs/reviews, caps and preservation inventories. No production source change.

**Consumes:** approved spec and actual current shared wire/identity conventions. **Produces:** reviewed exact serde/event/clock/canonical-trace schema, independently derived complete 18 tables/all best responses/pure sets/ties/control histories, compact witnesses and approved provisional limits.

- [ ] **Step 1: Root records ownership, base and original bytes.** Verify existing crowd, index/foreign work and owned tracker; snapshot all 16 catalog/default records on both native and actual WASM targets plus old source/guide/reference receipts. Do not rebuild or overwrite an original executable until its before exports and immutable copy/hash are retained. One build owner.
- [ ] **Step 2: Source-informed protocol producer and independent protocol review.** Freeze every event/observation/terminal order and field, including pre/post-own sensing, public phases, blocked charges, settlement/closure order and canonical exclusions. Finalize IDs, safe integers, duplicate rejection and report/control order. Canonical Episode excludes report classification, compiler statistics, source/reference receipts and self hashes, which live in separate envelopes; it includes actual config/profile/events/local histories/outcome. Inspect the existing compiled_bytes_identity codec without modifying old scientific files. Define trace identity as its existing one-item receipt codec: inner SHA256 of sorted exact UTF-8 Episode JSON; outer SHA256 of compact receipt array with fields path,bytes,sha256 in that order and path gate-assistance-trace-v1. Python derives exactly those bytes. This domain-bound complete trace fingerprint avoids copied SHA code/new dependencies; document it accurately, not as the naked JSON digest. Independent review precedes numerical handoff.
- [ ] **Step 3: Fresh source-blind derivation.** Author receives only sealed spec/protocol/canonical inputs and creates a Python standard-library reference.py with evaluate(k,b,d,h,r)->dict returning the complete protocol Episode, decode_holder(h)->tuple[str,...], and decode_retriever(r)->tuple[str,...]. Author independently implements the transition rules from the protocol, with no production imports/source access. The exact output/hash harness is:

        import hashlib, json
        from pathlib import Path
        from reference import evaluate
        def trace_hash(episode):
            body = json.dumps(episode,sort_keys=True,separators=(',',':'),
                              ensure_ascii=False,allow_nan=False).encode('utf-8')
            receipt = [{'path':'gate-assistance-trace-v1','bytes':len(body),
                        'sha256':hashlib.sha256(body).hexdigest()}]
            return hashlib.sha256(json.dumps(receipt,separators=(',',':'),
                       ensure_ascii=False,allow_nan=False).encode('utf-8')).hexdigest()
        out = Path('full-reference.jsonl').open('x',encoding='utf-8')
        for k in (1, 2, 4):
            for b in (0, 3, 6):
                for d in (0, 1):
                    for h in range(729):
                        for r in range(64):
                            episode = evaluate(k,b,d,h,r)
                            row = {'stratum':[k,b,d],'profile':[h,r],
                                   'outcome':episode['outcome'],
                                   'trace_hash':trace_hash(episode)}
                            out.write(json.dumps(row,separators=(',',':'))+'\n')
        out.close()

  Derive response maxima by columns/rows, all pure membership, all team/control selections, action/cost metrics and resource estimates. Preserve every raw suffix. Full tables remain ignored; compact fixtures bind their hashes and complete witnesses. Unit-test the reference using manually derived supplied-near and delayed-suffix traces before enumeration. Run python3 -m unittest test_reference -v from the sealed reference directory; retain true failure/repair/pass evidence. Complete control/witness Episodes are stored separately, with raw rows linking their trace hashes.
- [ ] **Step 4: Fresh numerical review with pre-author seal.** Reviewer derives its own physics/tie/suffix/settlement checks before reading author output, compares all 839,808 results, response maxima and pure sets, and verifies 90 control availability/history records. Seal input/read-access/source receipts. Keep provisional null review receipts outside strict final fixtures; self-review cannot replace independent approval.
- [ ] **Step 5: Register provisional work/allocation caps.** Use doubled largest independently derived per-stratum profile/transition/logical-allocation requirements, with explicit counter definition and scope. Observe native/WASM feasibility later before final freeze; caps are not physical credits, RSS or reserved memory. Register the pre-mortem and forbid success-driven sweeps. No guessed scientific optimum or equilibrium count belongs in this plan.
- [ ] **Step 6: Commit only reviewed compact artifacts.** Root stages the protocol/three fixtures, after actual independent approvals and hash checks. Keep full references/evidence ignored. Commit docs(gate): register independent assistance game reference.

## Task 2: Integer kernel, own-only execution and canonical traces

**Files:** core lib.rs additive export; gate mod.rs/types.rs/controller.rs/kernel.rs/observation.rs/canonical.rs and owning tests. **Consumes:** sealed Task 1 protocol/fixtures. **Produces:** Config/Stratum/Profile/words/View/Event/LocalEntry/Episode/Outcome/Error, run_episode, run_with_capture and trace_hash. No solver/table classification in View.

- [ ] **Step 1: Write behavior RED tests.** Include these independently hand-checkable examples, plus strict type/boundary/duplicate tests:

        #[test]
        fn supplied_near_matches_manual_physics()->Result<(),Error> {
        let cfg = Config { stratum: Stratum::new(1,3,0)?, ids: Ids::default() };
        let e = run_episode(&cfg, Profile::new(540,60)?)?;
        assert_eq!(e.outcome.delivery_round, Some(4));
        assert_eq!(e.outcome.spent, [2,4]);
        assert_eq!(e.outcome.reward, [3,9]);
        assert_eq!(e.outcome.net, [1,5]);
        let solo = run_episode(&cfg, Profile::new(0,63)?)?;
        assert_eq!(solo.outcome.delivery_round, None);
        assert_eq!(solo.outcome.spent, [0,6]);
        Ok(())
        }

  Separate named tests pin Hold away from P charged/blocked, Travel from P charged/blocked, cargo surviving closure, no collision cost, pickup/delivery once, all-Hold early termination charging only four rounds, no post-terminal action, passive observations unchanged and capture on/off equality. Test Role renaming, Unicode/escaped IDs and raw IDs 729/64 rejecting.
- [ ] **Step 2: Run RED.** cargo test -p sugarscape-core --lib gate_assistance::tests:: -- --nocapture; retain actual missing-module/behavior failures, not a build syntax failure presented as behavioral RED.
- [ ] **Step 3: Implement the pure controller and transactional kernel.** The Retriever choice is exactly:

        match (view.own_cell, view.own_cargo) {
            (Cell::O, false) => Primitive::Move(Cell::C),
            (Cell::C, false) => Primitive::Pickup,
            (Cell::C, true) => Primitive::Move(Cell::O),
            (Cell::O, true) => Primitive::Deliver,
            _ => return Err(Error::InvalidHistory("Retriever own phase".into())),
        }

  Apply this only for an Advance token; Wait returns Primitive::Wait. Holder tokens map to Wait, Move(P), OperatePlate. choose reads no actual peer/world. Kernel validates/charges before state mutations and appends normative histories at the declared phases. Capture hooks receive immutable points after native commits; no capture-generated observation. Implement strict constructors and word ID round-trips for all 729/64 words. Emit exact typed block reasons, not platform-dependent error strings inside scientific traces.
- [ ] **Step 4: Implement canonical trace identity.** Recursively sort objects, preserve array order/UTF-8, reject noninteger numbers, serialize the scientific Episode, then call unchanged crate::surface_discovery::compiled_bytes_identity with the single domain path above. Map its error into Gate Error. Bind that exact reused helper and transitive SHA source files in gate source inventory; do not alter their visibility or old source. Compare complete canonical bytes/fingerprints with Python witnesses, including Unicode, control escapes, signed zero net, null delivery and different unexecuted word IDs.
- [ ] **Step 5: GREEN and review.** Same focused tests plus complete independent kernel/control/chronology witnesses; edited-file rustfmt --config skip_children=true and workspace all-target/all-feature Clippy as appropriate under one owner. Confirm old scientific source hashes unchanged except the explicitly additive lib export/identity seam. Fresh spec/quality reviewer sees exact task diff/receipts; resolve findings before root commit feat(gate): execute paid assistance with local histories.

## Task 3: Complete payoff game, all equilibria and resource ownership

**Files:** budget.rs/game.rs/metrics.rs, mod.rs exports and tests/game.rs/tests/metrics.rs. **Consumes:** run_episode, trace_hash, Outcome and reference tables. **Produces:** Limits/CompileStats/GameTable/Analysis/Selector/Classification, compile/resolve/classify and complete matched metrics.

- [ ] **Step 1: Write RED for off-path and tie behavior.** The explicit physical-event witness, using the Task2 cfg with k=1,b=3,d=0, is:

        #[test]
        fn deviation_activates_an_unexecuted_suffix()->Result<(),Error> {
        let cfg = Config { stratum: Stratum::new(1,3,0)?, ids: Ids::default() };
        let a = run_episode(&cfg, Profile::new(540,60)?)?;
        let b = run_episode(&cfg, Profile::new(546,60)?)?;
        assert_eq!(a.events, b.events); // complete Episodes still differ in input words
        assert_eq!(run_episode(&cfg,Profile::new(540,15)?)?.outcome.delivery_round,None);
        assert_eq!(run_episode(&cfg,Profile::new(546,15)?)?.outcome.delivery_round,Some(6));
        Ok(())
        }

  The algebraic tie case is complete and independent of physics:

        #[test]
        fn all_private_payoff_ties_are_retained()->Result<(),Error> {
        let responses = response_sets(2,2,&[[0,0],[0,0],[0,0],[0,0]])?;
        assert_eq!(responses.pure, vec![Profile::new(0,0)?,Profile::new(0,1)?,
                                       Profile::new(1,0)?,Profile::new(1,1)?]);
        Ok(())
        }

  Add literal nonzero integer matrices checking unique private responses, team ties preferring lower spending before delivery time, pure availability/ranges not becoming representative averages, and regrets using each role's own unilateral maximum. Include response_sets(2,2,&[[1,-1],[-1,1],[-1,1],[1,-1]]) returning an empty pure set, without a fake profile. Define expected matrices/sets directly; do not call compile to generate expected values.
- [ ] **Step 2: Run RED.** cargo test -p sugarscape-core --lib gate_assistance::tests::game -- --nocapture and the metrics filter, with retained exact statuses.
- [ ] **Step 3: Compile every raw pair once and derive responses.** Use the following row contract:

        for h in 0..729u16 {
            for r in 0..64u8 {
                let episode = run_episode(&default_id_config, Profile::new(h,r)?)?;
                rows.push(Row { trace_hash: trace_hash(&episode)?, outcome: episode.outcome });
            }
        }
        // First scan: maxima against each fixed opponent word.
        // Second scan: retain iff net[0]==holder_best[r] && net[1]==retriever_best[h].

  Implement checked allocation/counters before retaining rows; one budget owns the complete compile. Profiles/transitions accumulate, temporary leases release, peak live bytes remain. Include nested event/history/canonical scratch, hashes, any projected payoff buffer and table/analysis vectors in accounting; do not multiply a fresh per-row cap or reset cumulative counts. Test atomic failed reservations, final-row limits, early delivery work, output retained charges and no silent pruning. Logical counters/charged peak estimates may enter the deterministic report; wall time, OS RSS and observed WASM/Node memory stay in separate engineering/status receipts so repeat bytes do not depend on host timing. Avoid global caches initially; any feasibility-motivated public-stratum cache must be bounded/accounted/reviewed, never mutable simulation state or keyed by names.
- [ ] **Step 4: Implement exact selections/classification/contrasts.** SoloAttempt=(0,63); SuppliedHelp near=(540,60), far=(423,30). resolve unavailable pure representatives as None. Preserve all private payoff ties; team/best-group-pure ranking uses the declared tuple. classify computes checked per-role regret and team gap. Generate comparisons on one k/b/d axis at a time, keeping other inputs fixed; report all role values and all equilibrium ranges without survivor or arbitrary-selection means.
- [ ] **Step 5: Verify full engineering equality.** Compare all 18×46,656 current rows/outcomes/trace fingerprints, response maxima, pure sets, rankings, controls and metrics with sealed independent full tables. Stream comparison, retain paths/sizes/hashes/stats/elapsed/RSS and every failure. This is engineering verification, not scientific first collection. Compact fixture tests plus full ignored reference evidence are both required.
- [ ] **Step 6: Fresh review and commit.** Focused tests, scoped format/Clippy and source receipts; reviewer audits suffix witness, matrix indices, numeric ranking and full compile budget. Root commits feat(gate): audit every private-payoff equilibrium and team plan.

## Task 4: Current-source diagnostic and fixed CLI

**Files:** diagnostics.rs/protocol.rs/references.rs, tests/diagnostics.rs, CLI gate_assistance.rs/main.rs/tests. **Consumes:** complete game/kernel and independent references. **Produces:** strict Report/Panel/Summary schemas, registered_strata, diagnose/report_integrity/reconstruct_episode/summary_json, gate-assistance diagnose.

- [ ] **Step 1: RED tamper/transport tests.** Pin missing/duplicate/reordered strata and raw rows, h/r transposition, forged hashes/net/regret/pure sets/representatives/stats/references, null unavailable examples, Boolean/float/string IDs and duplicate nested fields. A saved passed flag must not authorize a changed payload. Test CLI rejecting scientific flags, output write failure and flush failure; do not add an ignored successful diagnostic test before frozen collection.
- [ ] **Step 2: Run RED.** cargo test -p sugarscape-core --lib gate_assistance::tests::diagnostics -- --nocapture; cargo test -p sugarscape-cli --test gate_assistance. Preserve actual failure evidence.
- [ ] **Step 3: Implement strict ordered report and fresh integrity.** Store every raw pair mapping/hash, all 18 tables/maxima/pure membership/control availability, resource stats and scientific source/reference receipts. Interning must reconstruct exactly. Integrity recompiles each current stratum under registered limits, reconstructs canonical Episodes, checks all rows/maxima/metrics and independent full-table digest/compact witnesses; never trust cached flags. Portable runtime metadata binds sealed reference receipts, while root independently rehashes actual ignored files before collection. Operational/resource errors propagate; genuine scientific inequality returns false with explicit failed checks.
- [ ] **Step 4: Implement output and summary boundaries.** Reuse typed strict serde for report fields; no float tolerance/implicit coercion. New CLI accepts exactly diagnose, serializes plus one newline with checked write_all/flush; success 0, well-formed integrity failure emitted with exit 2, I/O/operational failure exit 1. Summary revalidates retained report and preserves every stratum/range/control/provenance, omitting full payoff matrices/histories. Define report-content hash excluding the CLI newline and self-references; retain the emitted-file hash separately. Source inventory includes all new Rust/embedded JSON/shared SHA dependencies and Cargo/compiler/binary closure at freeze.
- [ ] **Step 5: GREEN and release review.** Same scopes, workspace Clippy, edited formatting and cargo build --release -p sugarscape-cli. Release executable is target/release/sugarscape. Reviewer verifies complete-census checks and error paths; root commits feat(gate): report the frozen exhaustive assistance game. No successful scientific diagnose invocation yet.

## Task 5: Native browser adapter, strict import and actual WASM feasibility

**Files:** browser gate/mod.rs/projection.rs/duplicate.rs; mod.rs/input.rs/catalog.rs/record.rs/equivalence.rs; tests/gate.rs/tests/mod.rs; WASM experiment_view_tests.rs; TypeScript StudyId/family unions and exact historical guards only where needed. Named catalog assumptions: core tests/discovery.rs, spatial_fixed.rs, spatial_burrow.rs, CLI experiment_view.rs and WASM experiment_view_tests.rs; audit every catalog-derived count/order assertion before editing. F5 selector/mutation test only for gate embedded JSON.

**Consumes:** native tables/kernel/report identity and existing bounded record. **Produces:** StudyId::GateAssistance, family Gate (TS 'gate'), strict registered Input selector/raw-profile form and native EpisodeRecord. Own local projection contains actual dated map data; Researcher contains truth/words/classification.

- [ ] **Step 1: Capture original defaults/identities, then RED.** Tests cover exact first16 catalog ordering plus appended gate descriptor, default SuppliedHelp, every18 stratum/5 selectors and raw endpoints, unavailable selector errors, normalized/raw input quotas, duplicate keys including nested payload, unsafe numeric types, forged current records and edited identity. Preserve all16 original native and actual WASM defaults before common source changes.
- [ ] **Step 2: Wire native authority.** Add Input::GateAssistance { stratum, selector-or-profile, ids } with mutually exclusive selector/raw choice, fixed protocol constants and contextual rejection. Add the family/descriptor/dispatch without repurposing a legacy spatial variant. Catalog metadata is static; catalog loading does not compile games or run a diagnostic. Resolve representatives against a single current-stratum compile; raw cases still receive exact current classification. Names are not compile keys. run_with_capture emits native-owned checkpoints; complete payload retains scientific Episode and evaluator classification separately. Import reruns/reconstructs the full current record and rejects edits. No JS simulation or whole-census worker request.
- [ ] **Step 3: Gate-only unique-key checks and exact quotas.** Add a recursive serde Visitor detecting duplicate object keys before gate input/record Value conversion; preserve all existing legacy parsing behavior. Nested arrays/objects and duplicate study tags are tested with raw strings, not already-parsed JS objects. Count actual serialized setup/checkpoint/envelope bytes before retaining, include initial/terminal/commas and temporary conversion reservations; use existing 64KiB/16MiB/4096 limits. Test large but valid names without invented name-length caps, resource failure without partial successful records, and harmless rename equivalence.
- [ ] **Step 4: Audit identity seams/collector closure.** Gate rules_identity binds its actual native/adapter/shared-helper/reference closure. Enumerate current dependencies and source deltas before/after additive browser/lib/TypeScript changes. Update only exact authenticated old/common byte projections in historical source tests; reject tampered known paths and leave unknown paths unchanged. Retain complete original defaults and prove body equality across declared current identity changes. Add every compiled gate reference JSON to F5 explicit FILES and its isolated mutation test; inspect W1 selected membership and compiled stamps accurately. Never run unrelated collector science.
- [ ] **Step 5: Actual Node WASM and native parity.** wasm-pack test --node crates/sugarscape-wasm with matching wasm-bindgen runner; test all available 18×5 selected cases and raw endpoints/witnesses, complete export/import equality, exact integer atoms, max valid input and errors. Measure cold compile/import latency, wire bytes/checkpoints, compiler accounting, native RSS, WASM linear memory and Node RSS separately. Do not count zero host WASM tests. Keep product 60s bound separate from explicit 30s/60s/120s host integration-test allowances. If limits do not fit, retain evidence and review a design/cap amendment before collection.
- [ ] **Step 6: GREEN, review and commit.** Native focused/browser/CLI catalog tests, actual Node, scoped format/Clippy/typecheck/release receipts and legacy body/identity audit. Fresh reviewer checks observer side effects/retention/input/source scopes. Root commits feat(gate): expose bounded native assistance episodes in WASM. Real pending cancellation/timeout remains a shared Task6 gate until actual browser receipt closes it.

## Task 6: Gate viewer, comparison and real-browser acceptance

**Files:** web gate-projection.ts/gate-view.ts/gate-comparison.ts and four owning .test.ts files/gate.browser-test.ts; types.ts/catalog.ts/controls.ts/view.ts/projection.ts/styles only named seams. **Consumes:** native checkpoints/family Gate/input/classification. **Produces:** existing Episodes gate renderer, strict controls and matched pairs, current actual served/browser proof.

- [ ] **Step 1: RED boundary/input/clock tests.** Use native-backed fixtures, including supplied near/far, idle/deadline and a non-equilibrium raw pair. Test this boundary directly:

        for (const kind of ['agent'] as const) {
          expect(resultPayload(record, lastIndex, {kind, agent:'Agent-A'})).toBeNull();
        }
        expect(resultPayload(record,lastIndex,{kind:'researcher'})).toEqual(record.payload);

  Repeat all checkpoints, Agent-B/public-only absent-local views, terminal comparisons, passive stale-map timestamps, unseen gate unavailable versus inferred closure, and Researcher positive controls. The test fixtures define record/lastIndex from native exports, not invented privileged Agent state. Pin unblurred Run/Share IDs, duplicate raw JSON import, invalid/replaced input preservation and unmatched stages returning unavailable.
- [ ] **Step 2: Implement allowlisted presentation.** New Gate family selects the gate renderer inside the current shell. Draw fixed graph/plate/gate edge/resource and committed native local states; label observations/inferences/dates separately. Display schedule token versus paid primitive, role-owned costs and known own settlement. Extend existing resultPayload's discovery privacy branch to gate; do not alter legacy terminal-Agent behavior. Inputs and full evaluator words outside Agent panes are explicitly researcher controls, never copied into local View. No actual peer schedule/classification in an Actor pane at any stage.
- [ ] **Step 3: Matched comparison integration.** Key on stratum/IDs/fixed protocol; change only declared selector/raw pair. Use exact public round/role/stage matching; missing checkpoints stay unavailable, with no invented waits. Present raw-vs-control and distinct team/equilibrium/fixed-script labels. Sequential pairs retain at most two records. Native authority supplies classification; UI does not compute equilibria or utility.
- [ ] **Step 4: GREEN unit/parity/typecheck/build.** npm test -- src/episodes/gate-projection.test.ts src/episodes/gate-input.test.ts src/episodes/gate-comparison.test.ts src/episodes/gate-parity.test.ts; npm run typecheck; npm run build. Give real-engine parity cases explicit bounded integration deadlines following current tests; keep pure DOM tests on defaults. Full web tests run on final source. Refresh any old guard whose actual current byte change is authenticated; retain originals.
- [ ] **Step 5: Actual browser acceptance.** Use the installed browser skill and owned QA tab/server only. Capture real worker run/import/export/share and complete native-equal records; inspect every checkpoint for completed near/far and deadline cases under both Agents/Researcher, terminal/public-only/comparison positives/negatives. Verify actual gate rendering/local map timestamps/no observer mutation, raw input before blur, summary/episode separation once summary arrives in Task7, real pending cancel/dispose/fresh-worker success and actual60s timeout. Network-delay fixtures may control resource availability only while forwarding unchanged production bytes and clearly labeling initialization timeout; never fake timer/outcomes. Capture narrow layout/keyboard/reduced motion. Bind browser-observed served assets/current WASM/source, disclose unavailable worker-network-body capture accurately. Restore hooks/owned tabs and retain evidence.
- [ ] **Step 6: Fresh review and commit.** Complete source-bound report with exact tests/receipts/limitations and shared Task5 cancellation closure. Reviewer independently checks DOM masks/current source. Root commits feat(gate): visualize assistance costs and equilibrium comparisons. Stage4 cannot close without actual browser evidence; no scientific collection yet.

## Task 7: Freeze, first/repeat, measured summary and exact delivery

**Files:** protocol freeze footer; browser recorded.rs/fixtures/gate-recorded.json; web recorded.ts/recorded.test.ts/recorded-parity.test.ts; docs/gate-assistance.md/roadmap.md/papers.md/reproducibility.md; F5 selector/mutation test for new summary JSON; owned tracker/plan closure. All large evidence stays ignored.

**Consumes:** independently reviewed Tasks1–6 and complete engineering equality. **Produces:** immutable full first/repeat, independent census validation, correct measured guide/compact summary/browser registration, final review/integration/exact CI/Pages.

- [ ] **Step 1: Source-bearing incoming integration before freeze.** Inspect main/remotes/worktrees/status. If main moved, normal incoming merge into crowd preserving foreign files; named conflicts get producer/review and affected source/parity/browser refresh. Docs-only changes still receive inventory/diff checks. Do not collect on a stale source binding. Record first-measurement versus later delivery scopes explicitly.
- [ ] **Step 2: Run required gates on recorded source with one build owner.** Retain each exact command/cwd/exit/count/skip/log and stop on failure. Commands:

        cargo fmt --all --check
        cargo clippy --workspace --all-targets --all-features -- -D warnings
        cargo test --workspace
        wasm-pack test --node crates/sugarscape-wasm
        cargo build --release -p sugarscape-cli

  In web: npm run build, npm run typecheck, npm test. Root: python3 -m unittest discover -s studio/tests -t studio -v. Run affected F5/W1 source tests/stamps, not unrelated science. Rebuilt runtime/served bytes must match current actual browser proof; refresh dependent captures if bytes change. Retain inherited packaging notices/browser-only skips; no blanket warning-free claim or test disabling.
- [ ] **Step 3: Fresh precollection review and commit.** Bind full source/lock/toolchain/native/WASM/served artifacts, all references/schema/caps/census and collector/browser scopes in an immutable manifest. Exclude itself/mutable footer and avoid commit/hash fixed-point claims; footer binds manifest SHA, root release binds actual later frozen commit. Independently rehash real external artifacts and old-source/guide/default preservation. Review all839808 engineering comparisons, all pure ties/suffixes/controls, objective information parity, caps and actual WASM/browser limits. Resolve findings and commit exact reviewed freeze before any successful diagnostic invocation. Internal release follows existing authorization, not a redundant user permission request.
- [ ] **Step 4: Exclusive collection and one repeat.** Use absolute frozen target/release/sugarscape and exact unchanged source/settings/limits. The reviewed runner uses:

        with (evidence/'first-gate-assistance.json').open('xb') as out:
            with (evidence/'first-gate-assistance.stderr').open('xb') as err:
                result = subprocess.run([str(binary),'gate-assistance','diagnose'],
                                        stdout=out,stderr=err,check=False)

  Record exclusive status even on failure; preserve failed stdout/stderr. Repeat exactly once into new exclusive paths after hash rechecks, compare full bytes. No zero/adverse/no-cooperation outcome authorizes extra runs. A corrected scientific source requires a separately registered retained series.
- [ ] **Step 5: Independent streaming validation and retained-first CLI harness.** A standard-library validator does not invoke the engine. Stream all18×46656 first rows against sealed outcomes/complete trace fingerprints, check ordered indices/maxima/all pure membership/ties/metrics/all available full control histories, canonical hashes and caps/source/reference receipts. Verify report stdout status and emitted-file hash; CLI success harness consumes retained first/status, not another diagnose. Test chunked numbers/escapes/Unicode, duplicate keys/trailing data, integer-vs-Boolean/float rejection and changed rows before release using sealed engineering witnesses.
- [ ] **Step 6: Generate native summary and publish complete findings.** summary_json freshly validates retained first; compare every compact field independently. Embed only measured compact summary; register gate_assistance in current recorded loader without recomputing science. Keep full raw tables out of browser bundle. Add native/actual Node WASM/DOM recorded parity tests before registration, update only the new F5 summary fixture entry with RED mutation evidence, refresh actual stamps. Capture actual served summary/native/export equality, every18 stratum/equilibrium-range/control row and unchanged summary after selected runs. Preserve old recorded values and Actor masks. Guide/register/catalog/roadmap give all strata, payoff ranges/regrets/team gaps, costly blocked attempts/no-delivery cases and multiplicity; no claim equilibrium selection, learned cooperation or unrestricted affordance discovery. Reserve stable claim IDs GATE-1 (physical/cost correctness), GATE-2 (private incentives/equilibria), GATE-3 (team gap/coordination) with conclusions scoped to measured checks, not predefined successes.
- [ ] **Step 7: Separate delivery manifest and dependent gates.** Archive original runtime/source binding before summary rebuild. Record exact postmeasurement delivery source/artifact delta and unchanged physics/settings/first outputs. Refresh native/WASM/import/source/collector/browser/recorded tests and full gates only when those dependencies change or failures demand it. Do not relabel delivery binaries as measurement binaries. Retain all failed/superseded evidence and every earlier guide prefix; source-bound helper code/canonical protocol stays frozen.
- [ ] **Step 8: Whole-increment review, commit and normal integration.** Fresh most-capable reviewer checks complete code/evidence/first/reference/summary/guide/hash matches and all ledger decisions. One scoped fix wave if needed, affected gates, independent re-review. Root commits reviewed findings/registration and records local plan progress. Inspect main again; preserve foreign work, normal merge/fast-forward/push without force and verify exact CI/Pages. CI regressions receive systematic diagnosis, producer-owned minimal fixes, scoped review and explicit delivery deltas, never scientific reruns or disabling tests.
- [ ] **Step 9: Honest tracker closure and final remote proof.** Keep Stage5 In Progress while its required CI/Pages are pending/failed. After actual successful integration verification, mark all owned stages Complete, snapshot/remove only that tracker, close plan metadata and commit under standing authorization. Push any final metadata commit and check its exact CI/Pages too; then write completion only into ignored ledger/handoff, with no further tracked bookkeeping loop. Preserve this entire worktree/evidence and report every chronological Ruling with cost if wrong.

## Self-review and execution handoff

Coverage: Task1 protocol/independence/source receipt/caps; Task2 physics/clock/local knowledge/canonical identity; Task3 all raw schedules/deviations/equilibria/metrics/budget; Task4 checked full diagnostic/CLI; Task5 exact bounded adapter/import/actual WASM/compatibility; Task6 visible maps/privacy/input/comparisons/real browser; Task7 freeze/census validation/recorded summary/findings/final CI. Every Review Focus has tests in its owning tasks. No numerical optimum or pure-equilibrium count is chosen without the independent reference.

The seven tasks map to five owned stages. Source-informed protocol and source-blind numerical authors are separate from production implementers. No prior plan/evidence workspace is reused or deleted. Shared interfaces above are the handoff names; the normative wire is reviewed in Task1 before any production consumer. Bounds/cap feasibility can reject the design before scientific collection; changes require an explicit reviewed amendment, not silent simplification.

The user has already chosen subagent-driven execution. This written plan requests review before implementation. After approval, execution continues task-by-task through independent reviews and exact delivery under the preserved authorization, without repeated continuation prompts.
