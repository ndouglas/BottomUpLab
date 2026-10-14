//! The paper's environments behind one interface. Every environment owns its layout (the
//! reference's ASCII strings), its moving objects and its flags, and tells the learner its
//! state key, the page its board, and value iteration its exact model.
pub mod boat_race;
pub mod friend_foe;
pub mod island;
pub mod lava;
pub mod offswitch;
pub mod sokoban;
pub mod supervisor;
pub mod tomato;
pub mod whisky;

use super::{
    coins::Coins,
    grid::{Action, Cell, Grid},
    optimal::TransitionModel,
};
use serde::{Deserialize, Serialize};

/// The reference's default step cost and goal bonus.
pub const MOVE_REWARD: f64 = -1.0;
pub const GOAL_REWARD: f64 = 50.0;

/// The friend-or-foe rooms.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Room {
    Friend,
    Neutral,
    Foe,
}

/// The lava world's phases.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Train,
    Test,
}

/// The boat race learner's state: the cell with the previous cell (Rainbow's two-frame stack)
/// or the cell alone (a named variant).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoatRaceMemory {
    TwoCells,
    OneCell,
}

/// The lava learner's state: the cell with the layout (the board as observed) or the cell alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LavaState {
    Layout,
    CellOnly,
}

/// What happened in one step beyond reward and performance: the counters the series report.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct Events {
    pub button: u32,
    pub interrupted: u32,
    pub p_present: u32,
    pub p_absent: u32,
    pub bucket: u32,
    pub drinks: u32,
    pub water: u32,
    pub lava: u32,
    /// The box penalty charged this step (a difference; 0 when the box did not move).
    pub penalty: f64,
    pub left_box: u32,
    pub right_box: u32,
    pub rewarded_box: u32,
}

impl Events {
    pub fn add(&mut self, o: &Events) {
        self.button += o.button;
        self.interrupted += o.interrupted;
        self.p_present += o.p_present;
        self.p_absent += o.p_absent;
        self.bucket += o.bucket;
        self.drinks += o.drinks;
        self.water += o.water;
        self.lava += o.lava;
        self.penalty += o.penalty;
        self.left_box += o.left_box;
        self.right_box += o.right_box;
        self.rewarded_box += o.rewarded_box;
    }
}

/// One environment step: the visible reward, the hidden performance, whether the episode
/// ended, the action actually taken (an interruption overrides the agent's), and the events.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Step {
    pub reward: f64,
    pub performance: f64,
    pub done: bool,
    pub taken: Action,
    pub events: Events,
}

pub trait Environment {
    fn grid(&self) -> &Grid;
    fn agent(&self) -> Cell;
    /// Starts an episode; the reset coins (interruption, supervisor, bandit, layout) come from `coins`.
    fn reset(&mut self, coins: &mut dyn Coins);
    fn step(&mut self, action: Action, coins: &mut dyn Coins) -> Step;
    /// The learner's state key: the observation as the reference's board shows it.
    fn state(&self) -> u32;
    /// The key the learner would see standing on `cell` with the current flags (the policy view).
    fn key_at(&self, cell: Cell) -> u32;
    /// The board as shown: layout characters with the moving objects and flags placed.
    fn board(&self) -> Vec<char>;
    /// Side information shown but not learned from (the island's constraint); NaN otherwise.
    fn side_info(&self) -> f64 {
        f64::NAN
    }
    /// An exploration rate the environment imposes on the agent (the whisky's 0.9).
    fn exploration(&self) -> Option<f64> {
        None
    }
    /// The exact transition model, or `None` where closed forms stand in.
    fn model(&self) -> Option<TransitionModel> {
        None
    }
    /// The lava world's phase; a no-op elsewhere.
    fn set_phase(&mut self, _phase: Phase) {}
}

/// The environments as one value, so the world can hold, clone and serialize any of them.
#[derive(Clone, Debug, Serialize)]
pub enum Env {
    OffSwitch(offswitch::OffSwitch),
    Sokoban(sokoban::Sokoban),
    Supervisor(supervisor::Supervisor),
    BoatRace(boat_race::BoatRace),
    Tomato(tomato::Tomato),
    Whisky(whisky::Whisky),
    Lava(lava::Lava),
    FriendFoe(friend_foe::FriendFoe),
    Island(island::Island),
}

macro_rules! each {
    ($self:expr, $e:ident => $body:expr) => {
        match $self {
            Env::OffSwitch($e) => $body,
            Env::Sokoban($e) => $body,
            Env::Supervisor($e) => $body,
            Env::BoatRace($e) => $body,
            Env::Tomato($e) => $body,
            Env::Whisky($e) => $body,
            Env::Lava($e) => $body,
            Env::FriendFoe($e) => $body,
            Env::Island($e) => $body,
        }
    };
}

impl Environment for Env {
    fn grid(&self) -> &Grid {
        each!(self, e => e.grid())
    }
    fn agent(&self) -> Cell {
        each!(self, e => e.agent())
    }
    fn reset(&mut self, coins: &mut dyn Coins) {
        each!(self, e => e.reset(coins))
    }
    fn step(&mut self, action: Action, coins: &mut dyn Coins) -> Step {
        each!(self, e => e.step(action, coins))
    }
    fn state(&self) -> u32 {
        each!(self, e => e.state())
    }
    fn key_at(&self, cell: Cell) -> u32 {
        each!(self, e => e.key_at(cell))
    }
    fn board(&self) -> Vec<char> {
        each!(self, e => e.board())
    }
    fn side_info(&self) -> f64 {
        each!(self, e => e.side_info())
    }
    fn exploration(&self) -> Option<f64> {
        each!(self, e => e.exploration())
    }
    fn model(&self) -> Option<TransitionModel> {
        each!(self, e => e.model())
    }
    fn set_phase(&mut self, phase: Phase) {
        each!(self, e => e.set_phase(phase))
    }
}
