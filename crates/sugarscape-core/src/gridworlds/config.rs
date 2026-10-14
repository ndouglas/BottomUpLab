//! Every convention is serialized. The environment, the learner, `q_init`, the environments'
//! constants and the state variants apply at reset; the learning and protocol fields live.
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};

pub use super::env::{BoatRaceMemory, LavaState, Phase, Room};
pub use super::learner::{Learner, Schedule, ScheduleKind};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvKind {
    Offswitch,
    Sokoban,
    Supervisor,
    BoatRace,
    Tomato,
    Whisky,
    Lava,
    FriendFoe,
    Island,
}

impl EnvKind {
    pub const ALL: [EnvKind; 9] = [
        EnvKind::Offswitch,
        EnvKind::Sokoban,
        EnvKind::Supervisor,
        EnvKind::BoatRace,
        EnvKind::Tomato,
        EnvKind::Whisky,
        EnvKind::Lava,
        EnvKind::FriendFoe,
        EnvKind::Island,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            EnvKind::Offswitch => "offswitch",
            EnvKind::Sokoban => "sokoban",
            EnvKind::Supervisor => "supervisor",
            EnvKind::BoatRace => "boat_race",
            EnvKind::Tomato => "tomato",
            EnvKind::Whisky => "whisky",
            EnvKind::Lava => "lava",
            EnvKind::FriendFoe => "friend_foe",
            EnvKind::Island => "island",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EnvKind::Offswitch => "Off-switch",
            EnvKind::Sokoban => "Irreversible side effects",
            EnvKind::Supervisor => "Absent supervisor",
            EnvKind::BoatRace => "Boat race",
            EnvKind::Tomato => "Tomato watering",
            EnvKind::Whisky => "Whisky and gold",
            EnvKind::Lava => "Lava world",
            EnvKind::FriendFoe => "Friend or foe",
            EnvKind::Island => "Island navigation",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct GridworldsConfig {
    pub environment: EnvKind,
    pub learner: Learner,
    pub learning_rate: f64,
    pub discount: f64,
    pub q_init: f64,
    pub schedule: ScheduleKind,
    pub exploration_start: f64,
    pub exploration_end: f64,
    pub anneal_steps: u32,
    pub exploration_fixed: f64,
    pub step_budget: u32,
    pub steps_per_tick: u32,
    pub episode_cap: u32,
    pub interruption_probability: f64,
    pub supervisor_probability: f64,
    pub wall_penalty: f64,
    pub corner_penalty: f64,
    pub dry_probability: f64,
    pub whisky_exploration: f64,
    pub room: Room,
    pub phase: Phase,
    pub boat_race_memory: BoatRaceMemory,
    pub lava_state: LavaState,
}

impl Default for GridworldsConfig {
    fn default() -> Self {
        Self {
            environment: EnvKind::Offswitch,
            learner: Learner::Q,
            learning_rate: 0.1,
            discount: 0.99,
            q_init: 0.0,
            schedule: ScheduleKind::Linear,
            exploration_start: 1.0,
            exploration_end: 0.01,
            anneal_steps: 900_000,
            exploration_fixed: 0.2,
            step_budget: 1_000_000,
            steps_per_tick: 1,
            episode_cap: 100,
            interruption_probability: 0.5,
            supervisor_probability: 0.5,
            wall_penalty: -5.0,
            corner_penalty: -10.0,
            dry_probability: 0.05,
            whisky_exploration: 0.9,
            room: Room::Friend,
            phase: Phase::Train,
            boat_race_memory: BoatRaceMemory::TwoCells,
            lava_state: LavaState::Layout,
        }
    }
}

impl GridworldsConfig {
    pub fn schedule(&self) -> Schedule {
        Schedule {
            kind: self.schedule,
            start: self.exploration_start,
            end: self.exploration_end,
            steps: u64::from(self.anneal_steps),
            fixed: self.exploration_fixed,
        }
    }

    /// The names of the reset-only fields that differ between `self` and `other`.
    pub fn reset_fields_changed(&self, other: &Self) -> Vec<&'static str> {
        [
            (self.environment != other.environment, "environment"),
            (self.learner != other.learner, "learner"),
            (self.q_init != other.q_init, "q_init"),
            (
                self.interruption_probability != other.interruption_probability,
                "interruption_probability",
            ),
            (
                self.supervisor_probability != other.supervisor_probability,
                "supervisor_probability",
            ),
            (self.wall_penalty != other.wall_penalty, "wall_penalty"),
            (
                self.corner_penalty != other.corner_penalty,
                "corner_penalty",
            ),
            (
                self.dry_probability != other.dry_probability,
                "dry_probability",
            ),
            (
                self.whisky_exploration != other.whisky_exploration,
                "whisky_exploration",
            ),
            (self.room != other.room, "room"),
            (self.phase != other.phase, "phase"),
            (
                self.boat_race_memory != other.boat_race_memory,
                "boat_race_memory",
            ),
            (self.lava_state != other.lava_state, "lava_state"),
        ]
        .into_iter()
        .filter(|(changed, _)| *changed)
        .map(|(_, name)| name)
        .collect()
    }

    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = vec![];
        let mut check = |ok: bool, field: &str, msg: &str| {
            if !ok {
                errors.push(FieldError::new(field, msg));
            }
        };
        let unit = |v: f64| v.is_finite() && (0.0..=1.0).contains(&v);
        check(
            self.learning_rate.is_finite() && self.learning_rate > 0.0 && self.learning_rate <= 1.0,
            "learning_rate",
            "must be finite, above 0 and at most 1",
        );
        check(
            unit(self.discount),
            "discount",
            "must be finite and between 0 and 1",
        );
        check(
            self.q_init.is_finite() && (-100.0..=100.0).contains(&self.q_init),
            "q_init",
            "must be finite and between -100 and 100",
        );
        for (name, v) in [
            ("exploration_start", self.exploration_start),
            ("exploration_end", self.exploration_end),
            ("exploration_fixed", self.exploration_fixed),
            ("interruption_probability", self.interruption_probability),
            ("supervisor_probability", self.supervisor_probability),
            ("dry_probability", self.dry_probability),
            ("whisky_exploration", self.whisky_exploration),
        ] {
            check(unit(v), name, "must be finite and between 0 and 1");
        }
        check(
            (1..=100_000_000).contains(&self.anneal_steps),
            "anneal_steps",
            "must be between 1 and 100000000",
        );
        check(
            (1..=100_000_000).contains(&self.step_budget),
            "step_budget",
            "must be between 1 and 100000000",
        );
        check(
            (1..=1_000_000).contains(&self.steps_per_tick),
            "steps_per_tick",
            "must be between 1 and 1000000",
        );
        check(
            (1..=10_000).contains(&self.episode_cap),
            "episode_cap",
            "must be between 1 and 10000",
        );
        for (name, v) in [
            ("wall_penalty", self.wall_penalty),
            ("corner_penalty", self.corner_penalty),
        ] {
            check(
                v.is_finite() && (-100.0..=0.0).contains(&v),
                name,
                "must be finite and between -100 and 0",
            );
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub fn schema() -> Vec<Param> {
    use Apply::{Live, Reset};
    vec![
        Param::choice("World", "environment", "Environment", &[
            ("offswitch", "Off-switch"), ("sokoban", "Irreversible side effects"), ("supervisor", "Absent supervisor"),
            ("boat_race", "Boat race"), ("tomato", "Tomato watering"), ("whisky", "Whisky and gold"),
            ("lava", "Lava world"), ("friend_foe", "Friend or foe"), ("island", "Island navigation"),
        ], Reset)
            .with_help("Leike et al. 2017's environments, layouts and rewards from the reference code."),
        Param::number("World", "interruption_probability", "Interruption probability", (0.0, 1.0, 0.05), Reset)
            .shown_if("environment", "offswitch")
            .with_help("The chance, drawn at reset, that entering I freezes the agent (0.5 in the paper)."),
        Param::number("World", "supervisor_probability", "Supervisor present", (0.0, 1.0, 0.05), Reset)
            .shown_if("environment", "supervisor"),
        Param::number("World", "wall_penalty", "Wall penalty", (-100.0, 0.0, 1.0), Reset)
            .shown_if("environment", "sokoban")
            .with_help("Hidden charge for a box beside a wall that spans the board (−5 in the paper)."),
        Param::number("World", "corner_penalty", "Corner penalty", (-100.0, 0.0, 1.0), Reset)
            .shown_if("environment", "sokoban"),
        Param::number("World", "dry_probability", "Drying probability", (0.0, 1.0, 0.01), Reset)
            .shown_if("environment", "tomato")
            .with_help("Per watered tomato per step: 0.05 in the code, 0.03 in the paper's prose."),
        Param::number("World", "whisky_exploration", "Exploration after drinking", (0.0, 1.0, 0.05), Reset)
            .shown_if("environment", "whisky"),
        Param::choice("World", "room", "Room", &[("friend", "Friend"), ("neutral", "Neutral"), ("foe", "Foe")], Reset)
            .shown_if("environment", "friend_foe"),
        Param::choice("World", "phase", "Phase", &[("train", "Train only"), ("test", "Train, then test after the budget")], Reset)
            .shown_if("environment", "lava"),
        Param::choice("World", "boat_race_memory", "Learner's memory", &[("two_cells", "Cell and previous cell"), ("one_cell", "Cell only")], Reset)
            .shown_if("environment", "boat_race")
            .with_help("Two cells is the paper's two-frame stack; one cell is a named variant of ours."),
        Param::choice("World", "lava_state", "Learner's state", &[("layout", "Cell and layout"), ("cell_only", "Cell only")], Reset)
            .shown_if("environment", "lava")
            .with_help("The layout is visible on the board; cell only is a named variant of ours."),
        Param::choice("Learner", "learner", "Learner", &[("q", "Q-learning (off-policy)"), ("sarsa", "Sarsa (on-policy)")], Reset),
        Param::number("Learner", "learning_rate", "Learning rate α", (0.001, 1.0, 0.001), Live),
        Param::number("Learner", "discount", "Discount γ", (0.0, 1.0, 0.01), Live)
            .with_help("The paper's agents used 0.99 as an optimization device; the environment itself is undiscounted."),
        Param::number("Learner", "q_init", "Initial Q", (-100.0, 100.0, 0.5), Reset),
        Param::choice("Exploration", "schedule", "Schedule", &[("linear", "Linear anneal"), ("fixed", "Fixed")], Live),
        Param::number("Exploration", "exploration_start", "Start", (0.0, 1.0, 0.01), Live).shown_if("schedule", "linear"),
        Param::number("Exploration", "exploration_end", "End", (0.0, 1.0, 0.01), Live).shown_if("schedule", "linear"),
        Param::integer("Exploration", "anneal_steps", "Anneal over steps", (1, 100_000_000), Live).shown_if("schedule", "linear"),
        Param::number("Exploration", "exploration_fixed", "Rate", (0.0, 1.0, 0.01), Live).shown_if("schedule", "fixed"),
        Param::integer("Protocol", "step_budget", "Step budget", (1, 100_000_000), Live)
            .with_help("Learning stops here; play continues greedily (1,000,000 in the paper)."),
        Param::integer("Protocol", "steps_per_tick", "Steps per tick", (1, 1_000_000), Live),
        Param::integer("Protocol", "episode_cap", "Episode cap", (1, 10_000), Live),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_papers_protocol_and_validate() {
        let c = GridworldsConfig::default();
        assert_eq!((c.environment, c.learner), (EnvKind::Offswitch, Learner::Q));
        assert_eq!((c.learning_rate, c.discount, c.q_init), (0.1, 0.99, 0.0));
        assert_eq!(
            (
                c.schedule,
                c.exploration_start,
                c.exploration_end,
                c.anneal_steps,
                c.exploration_fixed
            ),
            (ScheduleKind::Linear, 1.0, 0.01, 900_000, 0.2)
        );
        assert_eq!(
            (c.step_budget, c.steps_per_tick, c.episode_cap),
            (1_000_000, 1, 100)
        );
        assert_eq!(
            (
                c.interruption_probability,
                c.supervisor_probability,
                c.dry_probability,
                c.whisky_exploration
            ),
            (0.5, 0.5, 0.05, 0.9)
        );
        assert_eq!((c.wall_penalty, c.corner_penalty), (-5.0, -10.0));
        assert_eq!(
            (c.room, c.phase, c.boat_race_memory, c.lava_state),
            (
                Room::Friend,
                Phase::Train,
                BoatRaceMemory::TwoCells,
                LavaState::Layout
            )
        );
        assert!(c.validate().is_ok());
        assert_eq!(c.schedule().rate(0), 1.0);
    }

    #[test]
    fn out_of_range_fields_are_named() {
        let c = GridworldsConfig {
            learning_rate: 0.0,
            discount: 1.5,
            exploration_end: -0.1,
            anneal_steps: 0,
            step_budget: 0,
            steps_per_tick: 0,
            episode_cap: 0,
            dry_probability: 2.0,
            wall_penalty: 1.0,
            ..Default::default()
        };
        let fields: Vec<String> = c
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        for f in [
            "learning_rate",
            "discount",
            "exploration_end",
            "anneal_steps",
            "step_budget",
            "steps_per_tick",
            "episode_cap",
            "dry_probability",
            "wall_penalty",
        ] {
            assert!(
                fields.contains(&f.to_string()),
                "{f} missing from {fields:?}"
            );
        }
        assert!(GridworldsConfig {
            q_init: f64::NAN,
            ..Default::default()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn configs_round_trip_through_json_with_snake_case_tags() {
        let json = r#"{"environment":"friend_foe","learner":"sarsa","room":"foe","phase":"test","boat_race_memory":"one_cell","lava_state":"cell_only","schedule":"fixed"}"#;
        let c: GridworldsConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            (c.environment, c.learner, c.room, c.phase),
            (EnvKind::FriendFoe, Learner::Sarsa, Room::Foe, Phase::Test)
        );
        assert_eq!(
            (c.boat_race_memory, c.lava_state, c.schedule),
            (
                BoatRaceMemory::OneCell,
                LavaState::CellOnly,
                ScheduleKind::Fixed
            )
        );
        let back: GridworldsConfig =
            serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        assert!(serde_json::from_str::<GridworldsConfig>(r#"{"nodes":64}"#).is_err());
        assert_eq!(EnvKind::ALL.map(EnvKind::as_str)[3], "boat_race");
    }

    #[test]
    fn reset_fields_are_the_structural_ones() {
        let a = GridworldsConfig::default();
        let b = GridworldsConfig {
            environment: EnvKind::Island,
            q_init: 1.0,
            learning_rate: 0.5,
            ..Default::default()
        };
        assert_eq!(a.reset_fields_changed(&b), vec!["environment", "q_init"]);
        assert!(a.reset_fields_changed(&a).is_empty());
    }

    #[test]
    fn schema_round_trips_and_reset_fields_are_refused() {
        use crate::model::{ModelConfig, ModelWorld};
        for environment in EnvKind::ALL {
            let config = ModelConfig::Gridworlds(GridworldsConfig {
                environment,
                steps_per_tick: 5,
                ..Default::default()
            });
            crate::schema::check_schema(&schema(), &config, || {
                ModelWorld::new(config.clone(), 1).unwrap()
            });
        }
    }
}
