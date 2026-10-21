//! The hybrid kind's configuration: the world, the structure, the bots, the imitation rule and
//! each world's game. Game parameters apply live; structure, population, framework, θ, the step
//! unit and the time limit apply on reset (amendment 13).

use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};

pub use super::booker::BotKind;
pub use super::engine::Accumulate;
pub use super::graph::{Neighborhood, Placement};
pub use super::guo::BotAction;
pub use super::si::{Division, Framework, MixedDraw};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum World {
    Booker,
    Guo,
    Si,
    Coloring,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Structure {
    WellMixed,
    Lattice,
    ErdosRenyi,
    BarabasiAlbert,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bots {
    /// `bot_count` bots, placed by `placement`.
    Count,
    /// round(`bot_fraction` · N) bots, placed by `placement`.
    Fraction,
    /// Each node a bot with probability `bot_fraction` (Si et al.'s code).
    Bernoulli,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepUnit {
    /// The paper's step: one update (Booker), one comparison (Guo, well-mixed), N updates (Guo
    /// on a graph), one round (Si), one second (coloring).
    Paper,
    /// N updates for Booker and well-mixed Guo (`sweep_steps`); the paper's step elsewhere.
    Sweep,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct HybridConfig {
    pub world: World,
    pub structure: Structure,
    pub side: u32,
    pub nodes: u32,
    pub neighborhood: Neighborhood,
    pub periodic: bool,
    pub mean_degree: f64,
    pub links: u32,
    /// The graph's own seed (Booker's ten networks); None draws the graph from the run's seed.
    pub graph_seed: Option<u64>,
    pub bots: Bots,
    pub bot_count: u32,
    pub bot_fraction: f64,
    pub placement: Placement,
    pub beta: f64,
    pub step: StepUnit,
    pub accumulate: Accumulate,
    pub initial_cooperation: f64,
    pub bot_kind: BotKind,
    pub b: f64,
    pub c: f64,
    pub t: f64,
    pub s: f64,
    pub phi: f64,
    pub bot_action: BotAction,
    pub r: f64,
    pub theta: f64,
    pub framework: Framework,
    pub mixed_draw: MixedDraw,
    pub division: Division,
    pub noise: f64,
    pub errant: f64,
    pub reaction: f64,
    pub bot_interval: f64,
    pub time_limit: f64,
}

/// Booker et al.'s lattice (Figs. 4–6): 50 × 50, open, von Neumann; 10% Samaritan bots; β = 0.1;
/// b = 2, c = 1; humans start C with probability ½.
impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            world: World::Booker,
            structure: Structure::Lattice,
            side: 50,
            nodes: 1000,
            neighborhood: Neighborhood::VonNeumann,
            periodic: false,
            mean_degree: 4.0,
            links: 2,
            graph_seed: None,
            bots: Bots::Fraction,
            bot_count: 0,
            bot_fraction: 0.1,
            placement: Placement::Random,
            beta: 0.1,
            step: StepUnit::Paper,
            accumulate: Accumulate::Sum,
            initial_cooperation: 0.5,
            bot_kind: BotKind::Samaritan,
            b: 2.0,
            c: 1.0,
            t: 1.4,
            s: 0.5,
            phi: 0.5,
            bot_action: BotAction::Redraw,
            r: 0.1,
            theta: 0.9,
            framework: Framework::Mixed,
            mixed_draw: MixedDraw::PerInteraction,
            division: Division::Integer,
            noise: 0.1,
            errant: 0.041,
            reaction: 1.59,
            bot_interval: 1.5,
            time_limit: 300.0,
        }
    }
}

pub fn booker_defaults() -> HybridConfig {
    HybridConfig::default()
}

/// Guo et al.'s Fig. 6 lattice: 100 × 100, periodic, von Neumann; 4,871 bots (y = 0.95) at
/// random; K = 10; snowdrift T = 1.4, S = 0.5; φ = 0.5.
pub fn guo_defaults() -> HybridConfig {
    HybridConfig {
        world: World::Guo,
        side: 100,
        nodes: 10_000,
        periodic: true,
        bots: Bots::Count,
        bot_count: 4871,
        beta: 10.0,
        ..Default::default()
    }
}

/// Si et al.'s Fig. 3 lattice as the code runs it: 100 × 100, periodic, Moore; each node a bot
/// with probability 0.3; θ = 0.9; r = 0.1; β = 1/κ = 10; the mixed framework.
pub fn si_defaults() -> HybridConfig {
    HybridConfig {
        world: World::Si,
        side: 100,
        nodes: 10_000,
        neighborhood: Neighborhood::Moore,
        periodic: true,
        bots: Bots::Bernoulli,
        bot_fraction: 0.3,
        beta: 10.0,
        ..Default::default()
    }
}

/// Shirado and Christakis's game: a 20-node Barabási–Albert graph with two links per new node,
/// three bots on the highest-degree nodes with 10% noise, the reconstructed humans' errant rate
/// 0.041 and mean decision time 1.59 s, a 300 s limit.
pub fn coloring_defaults() -> HybridConfig {
    HybridConfig {
        world: World::Coloring,
        structure: Structure::BarabasiAlbert,
        nodes: 20,
        links: 2,
        bots: Bots::Count,
        bot_count: 3,
        placement: Placement::Hubs,
        ..Default::default()
    }
}

impl HybridConfig {
    /// The node count: side² on a lattice, `nodes` otherwise.
    pub fn size(&self) -> usize {
        if self.structure == Structure::Lattice {
            (self.side as usize).pow(2)
        } else {
            self.nodes as usize
        }
    }

    /// How many bots are placed (None for a Bernoulli draw).
    pub fn bot_total(&self) -> Option<usize> {
        match self.bots {
            Bots::Count => Some(self.bot_count as usize),
            Bots::Fraction => Some((self.bot_fraction * self.size() as f64).round() as usize),
            Bots::Bernoulli => None,
        }
    }

    /// The names of the reset-only fields that differ between `self` and `other`.
    pub fn reset_fields_changed(&self, other: &Self) -> Vec<&'static str> {
        [
            (self.world != other.world, "world"),
            (self.structure != other.structure, "structure"),
            (self.side != other.side, "side"),
            (self.nodes != other.nodes, "nodes"),
            (self.neighborhood != other.neighborhood, "neighborhood"),
            (self.periodic != other.periodic, "periodic"),
            (self.mean_degree != other.mean_degree, "mean_degree"),
            (self.links != other.links, "links"),
            (self.graph_seed != other.graph_seed, "graph_seed"),
            (self.bots != other.bots, "bots"),
            (self.bot_count != other.bot_count, "bot_count"),
            (self.bot_fraction != other.bot_fraction, "bot_fraction"),
            (self.placement != other.placement, "placement"),
            (self.step != other.step, "step"),
            (
                self.initial_cooperation != other.initial_cooperation,
                "initial_cooperation",
            ),
            (self.bot_kind != other.bot_kind, "bot_kind"),
            (self.theta != other.theta, "theta"),
            (self.framework != other.framework, "framework"),
            (self.time_limit != other.time_limit, "time_limit"),
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
        let within = |v: f64, lo: f64, hi: f64| v.is_finite() && (lo..=hi).contains(&v);
        match self.world {
            World::Booker => check(
                self.structure != Structure::WellMixed,
                "structure",
                "Booker's runs need a graph; the well-mixed case is the exact chain, drawn as the reference",
            ),
            World::Coloring => check(
                self.structure == Structure::BarabasiAlbert && self.links <= 2 && self.nodes <= 200,
                "structure",
                "the coloring game runs on a Barabási–Albert graph of at most 200 nodes with one or two links per new node",
            ),
            World::Guo | World::Si => {}
        }
        if self.structure == Structure::Lattice {
            check(
                (3..=400).contains(&self.side),
                "side",
                "must be between 3 and 400",
            );
        } else {
            check(
                (2..=200_000).contains(&self.nodes),
                "nodes",
                "must be between 2 and 200000",
            );
        }
        check(
            within(self.mean_degree, 0.1, 50.0),
            "mean_degree",
            "must be between 0.1 and 50",
        );
        check(
            (1..=10).contains(&self.links)
                && (self.structure != Structure::BarabasiAlbert
                    || (self.links as usize) < self.size()),
            "links",
            "must be between 1 and 10 and below the node count",
        );
        check(
            within(self.bot_fraction, 0.0, 1.0),
            "bot_fraction",
            "must be between 0 and 1",
        );
        if let Some(bots) = self.bot_total() {
            check(
                bots < self.size(),
                if self.bots == Bots::Count {
                    "bot_count"
                } else {
                    "bot_fraction"
                },
                "must leave at least one human",
            );
        }
        check(
            within(self.beta, 0.0, 1000.0),
            "beta",
            "must be between 0 and 1000",
        );
        check(
            within(self.initial_cooperation, 0.0, 1.0),
            "initial_cooperation",
            "must be between 0 and 1",
        );
        check(within(self.b, 0.0, 100.0), "b", "must be between 0 and 100");
        check(
            within(self.c, 0.01, 10.0),
            "c",
            "must be between 0.01 and 10",
        );
        check(
            within(self.t, -10.0, 10.0),
            "t",
            "must be between -10 and 10",
        );
        check(
            within(self.s, -10.0, 10.0),
            "s",
            "must be between -10 and 10",
        );
        check(within(self.phi, 0.0, 1.0), "phi", "must be between 0 and 1");
        check(within(self.r, 0.0, 1.0), "r", "must be between 0 and 1");
        check(
            within(self.theta, 0.0, 1.0),
            "theta",
            "must be between 0 and 1",
        );
        check(
            within(self.noise, 0.0, 1.0),
            "noise",
            "must be between 0 and 1",
        );
        check(
            within(self.errant, 0.0, 1.0),
            "errant",
            "must be between 0 and 1",
        );
        check(
            within(self.reaction, 0.01, 60.0),
            "reaction",
            "must be between 0.01 and 60 seconds",
        );
        check(
            within(self.bot_interval, 0.01, 60.0),
            "bot_interval",
            "must be between 0.01 and 60 seconds",
        );
        check(
            within(self.time_limit, 1.0, 3600.0),
            "time_limit",
            "must be between 1 and 3600 seconds",
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
        Param::choice("World", "world", "World", &[("booker", "Booker et al.: Samaritan or discriminatory bots"), ("guo", "Guo et al.: bots that cooperate with probability φ"), ("si", "Si et al.: cooperative bots in three frameworks"), ("coloring", "Shirado & Christakis: noisy bots in a coloring game")], Reset)
            .with_help("Each world opens on its paper's setup from the presets; switching here keeps the other fields."),
        Param::choice("Structure", "structure", "Structure", &[("well_mixed", "Well-mixed"), ("lattice", "Square lattice"), ("erdos_renyi", "Erdős–Rényi"), ("barabasi_albert", "Barabási–Albert")], Reset),
        Param::integer("Structure", "side", "Lattice side", (3, 400), Reset).shown_if("structure", "lattice"),
        Param::choice("Structure", "neighborhood", "Neighborhood", &[("von_neumann", "Von Neumann (4)"), ("moore", "Moore (8)")], Reset).shown_if("structure", "lattice"),
        Param::bool("Structure", "periodic", "Wrap around", Reset).shown_if("structure", "lattice"),
        Param::integer("Structure", "nodes", "Nodes", (2, 200_000), Reset)
            .with_help("Ignored on a lattice, where the side sets the size."),
        Param::number("Structure", "mean_degree", "Mean degree", (0.1, 50.0, 0.1), Reset).shown_if("structure", "erdos_renyi"),
        Param::integer("Structure", "links", "Links per new node (m)", (1, 10), Reset).shown_if("structure", "barabasi_albert"),
        Param::choice("Bots", "bots", "Bots by", &[("count", "Count"), ("fraction", "Fraction of the nodes"), ("bernoulli", "Each node with this probability (Si's code)")], Reset),
        Param::integer("Bots", "bot_count", "Bot count", (0, 200_000), Reset).shown_if("bots", "count"),
        Param::number("Bots", "bot_fraction", "Bot fraction", (0.0, 1.0, 0.01), Reset)
            .with_help("A fraction of the nodes, or each node's probability of being a bot."),
        Param::choice("Bots", "placement", "Placement", &[("random", "Random"), ("hubs", "Highest degree first"), ("leaves", "Lowest degree first")], Reset)
            .with_help("Ignored for a Bernoulli draw."),
        Param::number("Imitation", "beta", "Selection strength β", (0.0, 1000.0, 0.01), Live)
            .with_help("The Fermi rule 1/(1 + e^{−β(π_model − π_focal)}): Booker's β, Guo's K, Si's 1/κ."),
        Param::choice("Imitation", "step", "Step", &[("paper", "The paper's step"), ("sweep", "N updates (a Monte Carlo step)")], Reset),
        Param::choice("Imitation", "accumulate", "Payoffs", &[("sum", "Summed over neighbors"), ("mean", "Mean over neighbors")], Live),
        Param::number("Imitation", "initial_cooperation", "Humans cooperating at the start", (0.0, 1.0, 0.01), Reset),
        Param::choice("Booker", "bot_kind", "Bots", &[("samaritan", "Samaritan (always C)"), ("discriminatory", "Discriminatory (mirror the partner)"), ("malicious", "Malicious (always D)")], Reset),
        Param::number("Booker", "b", "Benefit b", (0.0, 100.0, 0.1), Live),
        Param::number("Booker", "c", "Cost c", (0.01, 10.0, 0.01), Live),
        Param::number("Guo", "t", "Temptation T", (-10.0, 10.0, 0.01), Live),
        Param::number("Guo", "s", "Sucker's payoff S", (-10.0, 10.0, 0.01), Live),
        Param::number("Guo", "phi", "Bot cooperation φ", (0.0, 1.0, 0.01), Live),
        Param::choice("Guo", "bot_action", "Bot action", &[("redraw", "Redrawn when chosen (Algorithm II)"), ("frozen", "Drawn once"), ("per_interaction", "Drawn for every game")], Live),
        Param::number("Si", "r", "Dilemma strength r", (0.0, 1.0, 0.01), Live),
        Param::number("Si", "theta", "Bot strategy θ", (0.0, 1.0, 0.01), Reset),
        Param::choice("Si", "framework", "Framework", &[("discrete", "Discrete"), ("continuous", "Continuous"), ("mixed", "Mixed")], Reset),
        Param::choice("Si", "mixed_draw", "Mixed actions", &[("per_interaction", "Drawn for every game (the code)"), ("per_round", "Drawn once a round (Fig. A1)")], Live).shown_if("framework", "mixed"),
        Param::choice("Si", "division", "Well-mixed payoff", &[("integer", "As the code runs it (integer division)"), ("float", "The stated game (Eq. 1)")], Live)
            .with_help("The code's (N − 1 − c)/(N − 1) is integer division, so S and P drop out unless nobody else cooperates."),
        Param::number("Coloring", "noise", "Bot noise", (0.0, 1.0, 0.01), Live),
        Param::number("Coloring", "errant", "Human errant rate (ours)", (0.0, 1.0, 0.001), Live),
        Param::number("Coloring", "reaction", "Human mean decision time (s, ours)", (0.01, 60.0, 0.01), Live),
        Param::number("Coloring", "bot_interval", "Bot decision interval (s)", (0.01, 60.0, 0.01), Live),
        Param::number("Coloring", "time_limit", "Time limit (s)", (1.0, 3600.0, 1.0), Reset),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_world_opens_on_its_papers_setup() {
        let b = booker_defaults();
        assert_eq!(
            (b.world, b.structure, b.side, b.neighborhood, b.periodic),
            (
                World::Booker,
                Structure::Lattice,
                50,
                Neighborhood::VonNeumann,
                false
            )
        );
        assert_eq!(
            (b.bot_kind, b.b, b.c, b.beta, b.bot_total(), b.step),
            (
                BotKind::Samaritan,
                2.0,
                1.0,
                0.1,
                Some(250),
                StepUnit::Paper
            )
        );
        let g = guo_defaults();
        assert_eq!(
            (
                g.world,
                g.side,
                g.periodic,
                g.bot_total(),
                g.beta,
                g.t,
                g.s,
                g.bot_action
            ),
            (
                World::Guo,
                100,
                true,
                Some(4871),
                10.0,
                1.4,
                0.5,
                BotAction::Redraw
            )
        );
        let s = si_defaults();
        assert_eq!(
            (
                s.world,
                s.neighborhood,
                s.periodic,
                s.bots,
                s.bot_fraction,
                s.theta,
                s.r
            ),
            (
                World::Si,
                Neighborhood::Moore,
                true,
                Bots::Bernoulli,
                0.3,
                0.9,
                0.1
            )
        );
        assert_eq!(
            (s.framework, s.mixed_draw, s.division, s.beta),
            (
                Framework::Mixed,
                MixedDraw::PerInteraction,
                Division::Integer,
                10.0
            )
        );
        let c = coloring_defaults();
        assert_eq!(
            (
                c.world,
                c.structure,
                c.nodes,
                c.links,
                c.bot_total(),
                c.placement
            ),
            (
                World::Coloring,
                Structure::BarabasiAlbert,
                20,
                2,
                Some(3),
                Placement::Hubs
            )
        );
        assert_eq!(
            (c.noise, c.errant, c.reaction, c.bot_interval, c.time_limit),
            (0.1, 0.041, 1.59, 1.5, 300.0)
        );
        for cfg in [b, g, s, c] {
            assert!(cfg.validate().is_ok(), "{cfg:?}");
        }
    }

    #[test]
    fn out_of_range_and_inconsistent_fields_are_named() {
        let bad = HybridConfig {
            side: 2,
            beta: -1.0,
            phi: 2.0,
            r: 1.5,
            theta: -0.1,
            c: 0.0,
            noise: 2.0,
            reaction: 0.0,
            time_limit: 0.0,
            initial_cooperation: 2.0,
            ..booker_defaults()
        };
        let fields: Vec<String> = bad
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        for f in [
            "side",
            "beta",
            "phi",
            "r",
            "theta",
            "c",
            "noise",
            "reaction",
            "time_limit",
            "initial_cooperation",
        ] {
            assert!(
                fields.contains(&f.to_string()),
                "{f} missing from {fields:?}"
            );
        }
        let wm = HybridConfig {
            structure: Structure::WellMixed,
            ..booker_defaults()
        };
        assert_eq!(
            wm.validate().unwrap_err()[0].field,
            "structure",
            "Booker's well-mixed case is the exact chain"
        );
        let lattice_coloring = HybridConfig {
            structure: Structure::Lattice,
            ..coloring_defaults()
        };
        assert_eq!(
            lattice_coloring.validate().unwrap_err()[0].field,
            "structure"
        );
        let crowded = HybridConfig {
            bots: Bots::Count,
            bot_count: 2500,
            ..booker_defaults()
        };
        assert_eq!(crowded.validate().unwrap_err()[0].field, "bot_count");
        let all = HybridConfig {
            bot_fraction: 1.0,
            ..booker_defaults()
        };
        assert_eq!(all.validate().unwrap_err()[0].field, "bot_fraction");
        assert!(
            HybridConfig {
                bot_fraction: 1.0,
                bots: Bots::Bernoulli,
                ..si_defaults()
            }
            .validate()
            .is_ok(),
            "a Bernoulli draw may leave no humans"
        );
        assert!(HybridConfig {
            b: f64::NAN,
            ..booker_defaults()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn configs_round_trip_through_json_with_snake_case_tags() {
        let json = r#"{"world":"si","structure":"well_mixed","nodes":500,"bots":"bernoulli","framework":"discrete","division":"float","mixed_draw":"per_round","bot_kind":"discriminatory","bot_action":"per_interaction","neighborhood":"moore","placement":"leaves","step":"sweep","accumulate":"mean","graph_seed":3}"#;
        let c: HybridConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            (c.world, c.structure, c.framework, c.division, c.mixed_draw),
            (
                World::Si,
                Structure::WellMixed,
                Framework::Discrete,
                Division::Float,
                MixedDraw::PerRound
            )
        );
        assert_eq!(
            (
                c.bot_kind,
                c.bot_action,
                c.placement,
                c.step,
                c.accumulate,
                c.graph_seed
            ),
            (
                BotKind::Discriminatory,
                BotAction::PerInteraction,
                Placement::Leaves,
                StepUnit::Sweep,
                Accumulate::Mean,
                Some(3)
            )
        );
        let back: HybridConfig = serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        assert!(serde_json::from_str::<HybridConfig>(r#"{"proxy_a":true}"#).is_err());
    }

    #[test]
    fn reset_fields_are_the_structure_and_the_population() {
        let a = booker_defaults();
        let b = HybridConfig {
            beta: 5.0,
            b: 3.0,
            side: 40,
            framework: Framework::Discrete,
            ..booker_defaults()
        };
        assert_eq!(a.reset_fields_changed(&b), vec!["side", "framework"]);
        assert!(a.reset_fields_changed(&a).is_empty());
    }

    #[test]
    fn schema_round_trips_and_reset_fields_are_refused() {
        use crate::model::{ModelConfig, ModelWorld};
        for config in [
            booker_defaults(),
            guo_defaults(),
            si_defaults(),
            coloring_defaults(),
        ] {
            let config = ModelConfig::Hybrid(config);
            crate::schema::check_schema(&schema(), &config, || {
                ModelWorld::new(config.clone(), 1).unwrap()
            });
        }
    }
}
