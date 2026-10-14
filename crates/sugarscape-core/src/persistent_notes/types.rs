use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Role {
    Writer,
    Reader,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Cell {
    P,
    O,
    C,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Good {
    Left,
    Right,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Note {
    Blank,
    NoteLeft,
    NoteRight,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Arm {
    Retained,
    Erased,
    NoNote,
    NoAcquireNoNote,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WriterProgram {
    ObserveWrite,
    ObserveSilent,
    Silent,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ReaderRoutine {
    BlindGuess,
    InspectFirst,
    ReadAdaptive,
    Oracle,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BlankChoice {
    Inspect,
    GuessLeft,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Phase {
    Writer,
    Departure,
    Reader,
    Terminal,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TerminalReason {
    Delivered,
    Deadline,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventKind {
    Action,
    Departure,
    Settlement,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ActionStatus {
    Applied,
    Blocked,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BlockReason {
    WrongLocation,
    NoKey,
    NoRecipient,
    CargoOccupied,
    NoCargo,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PrimitiveKind {
    Wait,
    Move,
    InspectManifest,
    PassKey,
    WriteNote,
    ReadNote,
    Pickup,
    Deliver,
}
impl Role {
    pub fn index(self) -> usize {
        match self {
            Self::Writer => 0,
            Self::Reader => 1,
        }
    }
}
impl Good {
    pub fn index(self) -> usize {
        match self {
            Self::Left => 0,
            Self::Right => 1,
        }
    }
}
pub type EpisodeKey = (u64, u64, u64, u64, u64, Arm, ReaderRoutine, Good);
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ratio {
    pub numerator: i64,
    pub denominator: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stratum {
    pub acquisition_cost: u64,
    pub write_cost: u64,
    pub note_read_cost: u64,
    pub manifest_inspection_cost: u64,
    pub writer_share: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ids {
    pub writer: String,
    pub reader: String,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub stratum: Stratum,
    pub arm: Arm,
    pub routine: ReaderRoutine,
    pub request: Good,
    pub ids: Ids,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicSettings {
    pub acquisition_cost: u64,
    pub write_cost: u64,
    pub note_read_cost: u64,
    pub manifest_inspection_cost: u64,
    pub own_share: u64,
    pub initial_credits: u64,
    pub writer_opportunities: u64,
    pub reader_opportunities: u64,
    pub correct_total_reward: u64,
    pub request_prior: [Ratio; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Primitive {
    pub kind: PrimitiveKind,
    pub cell: Option<Cell>,
    pub note: Option<Note>,
    pub good: Option<Good>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Clock {
    pub boundary: u64,
    pub phase: Phase,
    pub writer_slot: u64,
    pub reader_slot: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicStatus {
    pub writer_active: bool,
    pub reader_active: bool,
    pub terminal: bool,
    pub terminal_reason: Option<TerminalReason>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Participant {
    pub role: Role,
    pub cell: Option<Cell>,
    pub active: bool,
    pub key: bool,
    pub cargo: Option<Good>,
    pub credits: u64,
    pub spent: u64,
    pub reward: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lineage {
    pub writer: Role,
    pub observed_request: Good,
    pub observation_clock: Clock,
    pub write_clock: Clock,
    pub symbol: Note,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Physical {
    pub request: Good,
    pub field: Note,
    pub key_owner: Role,
    pub participants: [Participant; 2],
    pub goods: [bool; 2],
    pub lineage: Option<Lineage>,
    pub settled: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapCell {
    pub cell: Cell,
    pub sensed_at: Clock,
    pub occupants: Vec<Role>,
    pub key_holder: Option<Role>,
    pub goods: Vec<Good>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalMap {
    pub sensed_at: Clock,
    pub cells: Vec<MapCell>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadRecord {
    pub symbol: Note,
    pub clock: Clock,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PossessionReceipt {
    pub clock: Clock,
    pub key: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feedback {
    pub clock: Clock,
    pub primitive: Primitive,
    pub status: ActionStatus,
    pub block_reason: Option<BlockReason>,
    pub cost: u64,
    pub observed_request: Option<Good>,
    pub read_symbol: Option<Note>,
    pub picked_good: Option<Good>,
    pub reward: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct View {
    pub role: Role,
    pub writer_program: Option<WriterProgram>,
    pub reader_routine: Option<ReaderRoutine>,
    pub public: PublicSettings,
    pub clock: Clock,
    pub status: PublicStatus,
    pub cell: Option<Cell>,
    pub key: bool,
    pub cargo: Option<Good>,
    pub credits: u64,
    pub spent: u64,
    pub reward: u64,
    pub known_request: Option<Good>,
    pub belief: [Ratio; 2],
    pub map: LocalMap,
    pub note_read: Option<ReadRecord>,
    pub key_receipt: Option<PossessionReceipt>,
    pub history: Vec<Feedback>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub kind: EventKind,
    pub clock: Clock,
    pub actor: Option<Role>,
    pub feedback: Option<Feedback>,
    pub erased: bool,
    pub terminal_reason: Option<TerminalReason>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Checkpoint {
    pub clock: Clock,
    pub public: PublicStatus,
    pub physical: Physical,
    pub own: [View; 2],
    pub event: Option<Event>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Costs {
    pub acquisition: [u64; 2],
    pub writing: [u64; 2],
    pub reading: [u64; 2],
    pub inspection: [u64; 2],
    pub movement: [u64; 2],
    pub handoff: [u64; 2],
    pub pickup: [u64; 2],
    pub delivery: [u64; 2],
    pub wait: [u64; 2],
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub attempted: [u64; 2],
    pub successful: [u64; 2],
    pub blocked: [u64; 2],
    pub reads: u64,
    pub nonblank_reads: u64,
    pub inspections: [u64; 2],
    pub writes: u64,
    pub key_passes: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Outcome {
    pub correct: bool,
    pub completed: bool,
    pub deadline: bool,
    pub terminal_reason: TerminalReason,
    pub completion_clock: Option<Clock>,
    pub selected_good: Option<Good>,
    pub belief: [Ratio; 2],
    pub spent: [u64; 2],
    pub reward: [u64; 2],
    pub net: [i64; 2],
    pub group_net: i64,
    pub costs: Costs,
    pub counts: Counts,
    pub note_survived: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Episode {
    pub key: EpisodeKey,
    pub config: Config,
    pub checkpoints: Vec<Checkpoint>,
    pub outcome: Outcome,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidInput(String),
    InvalidHistory(String),
    InvalidReport(String),
    ResourceLimit { resource: String, limit: u64 },
    ArithmeticOverflow,
    Io(String),
}
impl Error {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidInput(_) => "InvalidInput",
            Self::InvalidHistory(_) => "InvalidHistory",
            Self::InvalidReport(_) => "InvalidReport",
            Self::ResourceLimit { .. } => "ResourceLimit",
            Self::ArithmeticOverflow => "ArithmeticOverflow",
            Self::Io(_) => "Io",
        }
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {}
impl Config {
    pub fn example(arm: Arm, routine: ReaderRoutine, request: Good) -> Self {
        Self {
            stratum: Stratum {
                acquisition_cost: 1,
                write_cost: 1,
                note_read_cost: 1,
                manifest_inspection_cost: 4,
                writer_share: 0,
            },
            arm,
            routine,
            request,
            ids: Ids {
                writer: "Agent-A".into(),
                reader: "Agent-B".into(),
            },
        }
    }
    pub fn key(&self) -> EpisodeKey {
        let s = &self.stratum;
        (
            s.acquisition_cost,
            s.write_cost,
            s.note_read_cost,
            s.manifest_inspection_cost,
            s.writer_share,
            self.arm,
            self.routine,
            self.request,
        )
    }
}
impl Primitive {
    pub fn new(kind: PrimitiveKind) -> Self {
        Self {
            kind,
            cell: None,
            note: None,
            good: None,
        }
    }
}
impl Ratio {
    pub const HALF: Self = Self {
        numerator: 1,
        denominator: 2,
    };
    pub const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };
    pub const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };
}
