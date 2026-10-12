//! Every convention is serialized; `game`, `mode` and `theta_bins` apply at reset, the rest live.
use crate::{
    config::FieldError,
    schema::{Apply, Param},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Game {
    Paperclips,
    OffSwitch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Static,
    Iterated,
    Learning,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HumanRule {
    Expert,
    Equilibrium,
    Pooling,
    UniformRandom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RobotMap {
    BestResponse,
    Fixed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    Prior,
    Report,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AssistanceConfig {
    pub game: Game,
    pub mode: Mode,
    pub human_rule: HumanRule,
    pub robot_map: RobotMap,
    pub fixed_0: u32,
    pub fixed_1: u32,
    pub fixed_2: u32,
    pub theta_bins: u32,
    pub episodes_per_tick: u32,
    pub respond_every: u32,
    pub fictitious_weight: f64,
    pub protocol: Protocol,
    pub mu: f64,
    pub sigma: f64,
    pub beta: f64,
    pub actions: u32,
    pub observation_noise: f64,
    pub quadrature_density: u32,
}

impl Default for AssistanceConfig {
    fn default() -> Self {
        Self {
            game: Game::Paperclips,
            mode: Mode::Iterated,
            human_rule: HumanRule::Expert,
            robot_map: RobotMap::BestResponse,
            fixed_0: 0,
            fixed_1: 1,
            // Not 2: the schema check probes each integer at its default plus one.
            fixed_2: 1,
            theta_bins: 92,
            episodes_per_tick: 1000,
            respond_every: 1,
            fictitious_weight: 1.0,
            protocol: Protocol::Prior,
            mu: 0.0,
            sigma: 1.0,
            beta: 0.0,
            actions: 1,
            observation_noise: 1.0,
            quadrature_density: 8,
        }
    }
}

impl AssistanceConfig {
    pub fn bins(&self) -> usize {
        self.theta_bins as usize
    }
    pub fn fixed_map(&self) -> super::paperclips::Map {
        [self.fixed_0 as u8, self.fixed_1 as u8, self.fixed_2 as u8]
    }
    pub fn validate(&self) -> Result<(), Vec<FieldError>> {
        let mut errors = vec![];
        let mut check = |ok: bool, field: &str, msg: &str| {
            if !ok {
                errors.push(FieldError::new(field, msg));
            }
        };
        for (name, v) in [
            ("fixed_0", self.fixed_0),
            ("fixed_1", self.fixed_1),
            ("fixed_2", self.fixed_2),
        ] {
            check(v <= 2, name, "must be a deployment index 0, 1 or 2");
        }
        check(
            (3..=1000).contains(&self.theta_bins),
            "theta_bins",
            "must be between 3 and 1000",
        );
        check(
            (1..=100_000).contains(&self.episodes_per_tick),
            "episodes_per_tick",
            "must be between 1 and 100000",
        );
        check(
            (1..=1000).contains(&self.respond_every),
            "respond_every",
            "must be between 1 and 1000",
        );
        check(
            self.fictitious_weight.is_finite()
                && self.fictitious_weight > 0.0
                && self.fictitious_weight <= 1000.0,
            "fictitious_weight",
            "must be finite, positive and at most 1000",
        );
        check(
            self.mu.is_finite() && (-10.0..=10.0).contains(&self.mu),
            "mu",
            "must be finite and between -10 and 10",
        );
        check(
            self.sigma.is_finite() && (0.01..=10.0).contains(&self.sigma),
            "sigma",
            "must be finite and between 0.01 and 10",
        );
        check(
            self.beta == 0.0 || (self.beta.is_finite() && (0.01..=100.0).contains(&self.beta)),
            "beta",
            "must be 0 (rational) or between 0.01 and 100",
        );
        check(
            (1..=64).contains(&self.actions),
            "actions",
            "must be between 1 and 64",
        );
        check(
            self.observation_noise.is_finite() && (0.01..=10.0).contains(&self.observation_noise),
            "observation_noise",
            "must be finite and between 0.01 and 10",
        );
        check(
            (4..=64).contains(&self.quadrature_density),
            "quadrature_density",
            "must be between 4 and 64",
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
        Param::choice("Game", "game", "Game", &[("paperclips", "Paperclips"), ("off_switch", "Off-switch")], Reset)
            .with_help("The paperclip game (CIRL 2016, §3.3) or the off-switch game (IJCAI 2017)."),
        Param::choice("Paperclips", "mode", "Mode", &[("static", "Static"), ("iterated", "Iterated best response"), ("learning", "Fictitious play")], Reset)
            .shown_if("game", "paperclips")
            .with_help("Static plays one fixed pair; the dynamics are ours."),
        Param::choice("Paperclips", "human_rule", "Human rule", &[("expert", "Expert"), ("equilibrium", "Teaching (41/92, 51/92)"), ("pooling", "Always one of each"), ("uniform_random", "Uniform random")], Live)
            .shown_if("game", "paperclips")
            .with_help("The human's rule in static mode, or the start of a dynamic. In learning mode it applies at reset."),
        Param::choice("Paperclips", "robot_map", "Robot map", &[("best_response", "Best response"), ("fixed", "Fixed")], Live)
            .shown_if("game", "paperclips"),
        Param::integer("Paperclips", "fixed_0", "After (0, 2)", (0, 2), Live).shown_if("game", "paperclips").and_shown_if("robot_map", "fixed")
            .with_help("Deployment index: 0 = (0, 90), 1 = (50, 50), 2 = (90, 0)."),
        Param::integer("Paperclips", "fixed_1", "After (1, 1)", (0, 2), Live).shown_if("game", "paperclips").and_shown_if("robot_map", "fixed"),
        Param::integer("Paperclips", "fixed_2", "After (2, 0)", (0, 2), Live).shown_if("game", "paperclips").and_shown_if("robot_map", "fixed"),
        Param::integer("Paperclips", "theta_bins", "θ bins", (3, 1000), Reset).shown_if("game", "paperclips")
            .with_help("92 makes the paper's thresholds bin edges."),
        Param::integer("Paperclips", "respond_every", "Respond every", (1, 1000), Live).shown_if("game", "paperclips").and_shown_if("mode", "iterated")
            .with_help("Ticks between best-response rounds."),
        Param::number("Paperclips", "fictitious_weight", "Observation weight", (0.01, 1000.0, 0.01), Live).shown_if("game", "paperclips").and_shown_if("mode", "learning")
            .with_help("Weight of each new observation in the frequency tables; 1 is classic fictitious play."),
        Param::choice("Off-switch", "protocol", "Protocol", &[("prior", "The paper's game"), ("report", "Noisy report (Figure 4)")], Live)
            .shown_if("game", "off_switch")
            .with_help("Prior: U_a is drawn from the belief. Report: a N(0, 1) population, a noisy report, the posterior mean; our reconstruction."),
        Param::number("Off-switch", "mu", "Belief mean μ", (-10.0, 10.0, 0.05), Live).shown_if("game", "off_switch"),
        Param::number("Off-switch", "sigma", "Belief s.d. σ", (0.01, 10.0, 0.01), Live).shown_if("game", "off_switch"),
        Param::number("Off-switch", "beta", "Human temperature β", (0.0, 100.0, 0.01), Live).shown_if("game", "off_switch")
            .with_help("0 is the rational human; larger is noisier (eq. 5)."),
        Param::integer("Off-switch", "actions", "Candidate actions", (1, 64), Live).shown_if("game", "off_switch").and_shown_if("protocol", "report"),
        Param::number("Off-switch", "observation_noise", "Report noise", (0.01, 10.0, 0.01), Live).shown_if("game", "off_switch").and_shown_if("protocol", "report"),
        Param::integer("Off-switch", "quadrature_density", "Quadrature density", (4, 64), Live).shown_if("game", "off_switch")
            .with_help("Trapezoid points per smallest scale of σ and β. Cost per episode under the report protocol grows as σ/β."),
        Param::integer("Episodes", "episodes_per_tick", "Episodes per tick", (1, 100_000), Live),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_the_literal_rules_and_validate() {
        let c = AssistanceConfig::default();
        assert_eq!(
            (c.game, c.mode, c.human_rule, c.robot_map),
            (
                Game::Paperclips,
                Mode::Iterated,
                HumanRule::Expert,
                RobotMap::BestResponse
            )
        );
        assert_eq!(
            (
                c.theta_bins,
                c.episodes_per_tick,
                c.respond_every,
                c.fictitious_weight
            ),
            (92, 1000, 1, 1.0)
        );
        assert_eq!(
            (
                c.protocol,
                c.mu,
                c.sigma,
                c.beta,
                c.actions,
                c.observation_noise,
                c.quadrature_density
            ),
            (Protocol::Prior, 0.0, 1.0, 0.0, 1, 1.0, 8)
        );
        assert!(c.validate().is_ok());
        assert_eq!(c.fixed_map(), [0, 1, 1]);
    }

    #[test]
    fn out_of_range_fields_are_named() {
        let c = AssistanceConfig {
            fixed_2: 3,
            theta_bins: 2,
            sigma: 0.0,
            beta: 0.005,
            actions: 0,
            quadrature_density: 3,
            ..Default::default()
        };
        let fields: Vec<String> = c
            .validate()
            .unwrap_err()
            .into_iter()
            .map(|e| e.field)
            .collect();
        for f in [
            "fixed_2",
            "theta_bins",
            "sigma",
            "beta",
            "actions",
            "quadrature_density",
        ] {
            assert!(
                fields.contains(&f.to_string()),
                "{f} missing from {fields:?}"
            );
        }
        assert!(AssistanceConfig {
            beta: 0.0,
            ..Default::default()
        }
        .validate()
        .is_ok());
        assert!(AssistanceConfig {
            mu: f64::NAN,
            ..Default::default()
        }
        .validate()
        .is_err());
    }

    #[test]
    fn configs_round_trip_through_json_with_snake_case_tags() {
        let json = r#"{"game":"off_switch","mode":"learning","human_rule":"uniform_random","robot_map":"fixed","protocol":"report"}"#;
        let c: AssistanceConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            (c.game, c.mode, c.human_rule, c.robot_map, c.protocol),
            (
                Game::OffSwitch,
                Mode::Learning,
                HumanRule::UniformRandom,
                RobotMap::Fixed,
                Protocol::Report
            )
        );
        let back: AssistanceConfig =
            serde_json::from_str(&serde_json::to_string(&c).unwrap()).unwrap();
        assert_eq!(back, c);
        assert!(serde_json::from_str::<AssistanceConfig>(r#"{"nodes":64}"#).is_err());
    }

    #[test]
    fn schema_round_trips_and_only_game_mode_and_bins_apply_at_reset() {
        use crate::model::{ModelConfig, ModelWorld};
        for config in [
            ModelConfig::Assistance(AssistanceConfig::default()),
            ModelConfig::Assistance(AssistanceConfig {
                game: Game::OffSwitch,
                ..Default::default()
            }),
        ] {
            crate::schema::check_schema(&schema(), &config, || {
                ModelWorld::new(config.clone(), 1).unwrap()
            });
        }
    }
}
