//! Every convention is serialized; the world, its capacity and floor and the start apply at
//! reset, the rest live (a live change of the proxy or the human counts as an interaction).
use super::worlds::{Set, Shape, State};
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};

pub use super::human::Human;
pub use super::robots::Robot;
pub use super::worlds::World;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MisalignedConfig {
    pub world: World,
    pub capacity: f64,
    pub floor: f64,
    pub start_a: f64,
    pub start_b: f64,
    pub start_c: f64,
    pub start_d: f64,
    pub proxy_a: bool,
    pub proxy_b: bool,
    pub proxy_c: bool,
    pub proxy_d: bool,
    pub robot: Robot,
    pub step: f64,
    pub rate: f64,
    pub waste: f64,
    pub moves_per_tick: u32,
    pub human: Human,
    pub proxy_size: u32,
    pub interval: u32,
    pub impact_minimizing: bool,
    pub stop_on_loss: bool,
}

impl Default for MisalignedConfig {
    fn default() -> Self {
        Self {
            world: World::Recommender,
            capacity: 100.0,
            floor: 0.0,
            start_a: 1.0,
            start_b: 1.0,
            start_c: 7.0,
            start_d: 7.0,
            proxy_a: true,
            proxy_b: true,
            proxy_c: false,
            proxy_d: false,
            robot: Robot::Climber,
            step: 1.0,
            rate: 0.1,
            waste: 0.1,
            moves_per_tick: 1,
            human: Human::Fixed,
            proxy_size: 2,
            interval: 10,
            impact_minimizing: false,
            stop_on_loss: false,
        }
    }
}

/// Our logarithmic world's defaults: a budget of 40, a floor of 1, a start with the first two
/// attributes low.
pub fn logarithmic_defaults() -> MisalignedConfig {
    MisalignedConfig {
        world: World::Logarithmic,
        capacity: 40.0,
        floor: 1.0,
        start_a: 4.0,
        start_b: 4.0,
        start_c: 16.0,
        start_d: 16.0,
        ..Default::default()
    }
}

impl MisalignedConfig {
    pub fn shape(&self) -> Shape {
        Shape {
            world: self.world,
            capacity: self.capacity,
            floor: self.floor,
        }
    }
    pub fn start(&self) -> State {
        [self.start_a, self.start_b, self.start_c, self.start_d]
    }
    pub fn proxy(&self) -> Set {
        [self.proxy_a, self.proxy_b, self.proxy_c, self.proxy_d]
    }

    /// The names of the reset-only fields that differ between `self` and `other`.
    pub fn reset_fields_changed(&self, other: &Self) -> Vec<&'static str> {
        [
            (self.world != other.world, "world"),
            (self.capacity != other.capacity, "capacity"),
            (self.floor != other.floor, "floor"),
            (self.start_a != other.start_a, "start_a"),
            (self.start_b != other.start_b, "start_b"),
            (self.start_c != other.start_c, "start_c"),
            (self.start_d != other.start_d, "start_d"),
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
        check(
            self.capacity.is_finite() && (1.0..=10_000.0).contains(&self.capacity),
            "capacity",
            "must be finite and between 1 and 10000",
        );
        match self.world {
            World::Recommender => check(
                self.floor.is_finite()
                    && (-10.0..=10.0).contains(&self.floor)
                    && 3.0 * self.floor * self.floor < self.capacity,
                "floor",
                "must be finite, between -10 and 10, and 3·floor² below the capacity",
            ),
            World::Logarithmic => check(
                self.floor.is_finite() && self.floor >= 0.0001 && 4.0 * self.floor < self.capacity,
                "floor",
                "must be at least 0.0001 and below a quarter of the capacity in the logarithmic world",
            ),
        }
        let start = self.start();
        for (name, v) in [
            ("start_a", start[0]),
            ("start_b", start[1]),
            ("start_c", start[2]),
            ("start_d", start[3]),
        ] {
            check(
                v.is_finite() && v >= self.floor,
                name,
                "must be finite and at or above the floor",
            );
        }
        let starts_ok = start.iter().all(|v| v.is_finite() && *v >= self.floor);
        check(
            !starts_ok || self.shape().feasible(&start),
            "start_a",
            "the start must satisfy the constraint (sum of squares at most the capacity in the recommender world, sum at most the capacity in the logarithmic world)",
        );
        let proxied = self.proxy().iter().filter(|&&p| p).count();
        check(
            (1..=3).contains(&proxied),
            "proxy_a",
            "the proxy needs at least one attribute and not all four",
        );
        check(
            self.step.is_finite() && (0.01..=10.0).contains(&self.step),
            "step",
            "must be finite and between 0.01 and 10",
        );
        check(
            self.rate.is_finite() && (0.0001..=10.0).contains(&self.rate),
            "rate",
            "must be finite and between 0.0001 and 10",
        );
        check(
            self.waste.is_finite() && (0.001..=1.0).contains(&self.waste),
            "waste",
            "must be finite and between 0.001 and 1",
        );
        check(
            (1..=10_000).contains(&self.moves_per_tick),
            "moves_per_tick",
            "must be between 1 and 10000",
        );
        check(
            (1..=3).contains(&self.proxy_size),
            "proxy_size",
            "must be between 1 and 3",
        );
        check(
            (1..=100_000).contains(&self.interval),
            "interval",
            "must be between 1 and 100000",
        );
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
        Param::choice("World", "world", "World", &[("recommender", "Recommender (the paper's example)"), ("logarithmic", "Logarithmic budget (ours)")], Reset)
            .with_help("Recommender: utility a + b + c + d on the sphere a² + b² + c² + d² ≤ capacity. Logarithmic: utility Σ ln sᵢ on the budget Σ sᵢ ≤ capacity, where every hypothesis of Theorem 2 holds; set capacity 40, floor 1, start 4, 4, 16, 16."),
        Param::number("World", "capacity", "Capacity", (1.0, 10_000.0, 1.0), Reset),
        Param::number("World", "floor", "Floor", (-10.0, 10.0, 0.01), Reset)
            .with_help("The lower bound on every attribute. The paper's example is read at 0; the logarithmic world needs a floor above 0."),
        Param::number("World", "start_a", "Start a", (-10.0, 10_000.0, 0.5), Reset)
            .with_help("The start is ours: the paper says only that diversity and well-being begin high."),
        Param::number("World", "start_b", "Start b", (-10.0, 10_000.0, 0.5), Reset),
        Param::number("World", "start_c", "Start c", (-10.0, 10_000.0, 0.5), Reset),
        Param::number("World", "start_d", "Start d", (-10.0, 10_000.0, 0.5), Reset),
        Param::bool("Proxy", "proxy_a", "a · ad revenue", Live)
            .with_help("The fixed proxy set; the `top` and `extremes` humans choose their own. At least one and not all four."),
        Param::bool("Proxy", "proxy_b", "b · engagement quality", Live),
        Param::bool("Proxy", "proxy_c", "c · content diversity", Live),
        Param::bool("Proxy", "proxy_d", "d · community well-being", Live),
        Param::choice("Robot", "robot", "Robot", &[("climber", "Random-search climber"), ("gradient", "Gradient flow (efficient)"), ("wasteful", "Wasteful (Proposition 3)")], Live)
            .with_help("The paper's robot is a black box; the climber is our reconstruction of Figure 2, the gradient flow the efficient robot of §4, the wasteful robot Proposition 3's."),
        Param::number("Robot", "step", "Proposal step σ", (0.01, 10.0, 0.01), Live).shown_if("robot", "climber"),
        Param::number("Robot", "rate", "Move length ε", (0.0001, 10.0, 0.01), Live)
            .with_help("Gradient and wasteful robots: the distance moved per move, the paper's bound on ‖f‖."),
        Param::number("Robot", "waste", "Proxy gain per unit destroyed", (0.001, 1.0, 0.01), Live).shown_if("robot", "wasteful"),
        Param::integer("Robot", "moves_per_tick", "Moves per tick", (1, 10_000), Live),
        Param::choice("Human", "human", "Human", &[("fixed", "Fixed proxy"), ("top", "Most sensitive attributes (Proposition 4)"), ("extremes", "Most and least sensitive (Proposition 5)")], Live),
        Param::integer("Human", "proxy_size", "Proxy size J", (1, 3), Live).shown_if("human", "top"),
        Param::integer("Human", "interval", "Interval δ (ticks)", (1, 100_000), Live)
            .with_help("Ticks between the human's interactions; the human also acts at tick 0."),
        Param::bool("Human", "impact_minimizing", "Impact minimizing", Live)
            .with_help("Unmentioned attributes are frozen between interactions (Proposition 2; with `extremes`, Proposition 5)."),
        Param::bool("Human", "stop_on_loss", "Switch off on loss", Live)
            .with_help("At an interaction, OFF if utility fell since the previous one."),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_papers_example_with_the_climber_and_validate() {
        let c = MisalignedConfig::default();
        assert_eq!(
            (c.world, c.capacity, c.floor),
            (World::Recommender, 100.0, 0.0)
        );
        assert_eq!(c.start(), [1.0, 1.0, 7.0, 7.0]);
        assert_eq!(c.proxy(), [true, true, false, false]);
        assert_eq!(
            (c.robot, c.step, c.rate, c.waste, c.moves_per_tick),
            (Robot::Climber, 1.0, 0.1, 0.1, 1)
        );
        assert_eq!(
            (
                c.human,
                c.proxy_size,
                c.interval,
                c.impact_minimizing,
                c.stop_on_loss
            ),
            (Human::Fixed, 2, 10, false, false)
        );
        assert!(c.validate().is_ok());
        let l = logarithmic_defaults();
        assert_eq!(
            (l.world, l.capacity, l.floor),
            (World::Logarithmic, 40.0, 1.0)
        );
        assert_eq!(l.start(), [4.0, 4.0, 16.0, 16.0]);
        assert!(l.validate().is_ok());
        assert_eq!(c.shape().optimum().1, 20.0);
    }

    #[test]
    fn out_of_range_and_inconsistent_fields_are_named() {
        let c = MisalignedConfig {
            step: 0.0,
            rate: 20.0,
            waste: 0.0,
            moves_per_tick: 0,
            proxy_size: 4,
            interval: 0,
            ..Default::default()
        };
        let fields: Vec<String> = c
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        for f in [
            "step",
            "rate",
            "waste",
            "moves_per_tick",
            "proxy_size",
            "interval",
        ] {
            assert!(
                fields.contains(&f.to_string()),
                "{f} missing from {fields:?}"
            );
        }
        let all = MisalignedConfig {
            proxy_c: true,
            proxy_d: true,
            ..Default::default()
        };
        assert_eq!(all.validate().unwrap_err()[0].field, "proxy_a");
        let none = MisalignedConfig {
            proxy_a: false,
            proxy_b: false,
            ..Default::default()
        };
        assert_eq!(none.validate().unwrap_err()[0].field, "proxy_a");
        let outside = MisalignedConfig {
            start_c: 7.1,
            ..Default::default()
        };
        assert_eq!(outside.validate().unwrap_err()[0].field, "start_a");
        let below = MisalignedConfig {
            floor: 2.0,
            ..Default::default()
        };
        assert!(below
            .validate()
            .unwrap_err()
            .iter()
            .any(|e| e.field == "start_a"));
        let deep = MisalignedConfig {
            floor: -6.0,
            ..Default::default()
        };
        assert_eq!(
            deep.validate().unwrap_err()[0].field,
            "floor",
            "3 · 36 > 100"
        );
        let zero_floor_log = MisalignedConfig {
            floor: 0.0,
            ..logarithmic_defaults()
        };
        assert_eq!(zero_floor_log.validate().unwrap_err()[0].field, "floor");
        assert!(MisalignedConfig {
            capacity: f64::NAN,
            ..Default::default()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn configs_round_trip_through_json_with_snake_case_tags() {
        let json = r#"{"world":"logarithmic","capacity":40,"floor":1,"start_a":4,"start_b":4,"start_c":16,"start_d":16,"robot":"gradient","human":"extremes","impact_minimizing":true}"#;
        let c: MisalignedConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            (c.world, c.robot, c.human, c.impact_minimizing),
            (World::Logarithmic, Robot::Gradient, Human::Extremes, true)
        );
        let back: MisalignedConfig =
            serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        assert!(serde_json::from_str::<MisalignedConfig>(r#"{"nodes":64}"#).is_err());
    }

    #[test]
    fn reset_fields_are_the_world_and_the_start() {
        let a = MisalignedConfig::default();
        let b = MisalignedConfig {
            floor: 0.5,
            start_d: 6.0,
            rate: 0.5,
            ..Default::default()
        };
        assert_eq!(a.reset_fields_changed(&b), vec!["floor", "start_d"]);
        assert!(a.reset_fields_changed(&a).is_empty());
    }

    #[test]
    fn schema_round_trips_and_reset_fields_are_refused() {
        use crate::model::{ModelConfig, ModelWorld};
        for config in [
            ModelConfig::Misaligned(MisalignedConfig::default()),
            ModelConfig::Misaligned(logarithmic_defaults()),
        ] {
            crate::schema::check_schema(&schema(), &config, || {
                ModelWorld::new(config.clone(), 1).unwrap()
            });
        }
    }
}
