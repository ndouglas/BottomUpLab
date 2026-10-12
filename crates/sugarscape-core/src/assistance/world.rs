//! The assistance world: a tick samples `episodes_per_tick` games, then the paperclip dynamics
//! respond; a snapshot at tick 0 carries the exact values before any sampling.
use super::{
    config::*,
    dynamics::{self, Tables},
    offswitch::{self, Belief, Choice, Incentive, Surface},
    paperclips::{self, Map, Rule},
    stats::AssistanceSnapshot,
    view,
};
use crate::{
    config::FieldError,
    model::{wrong_model, Model, ModelConfig, ModelKind},
    rng::{self, SimRng},
    stats::{Series, Stats},
};
use rand::Rng;
use serde::Serialize;
use std::fmt::Write;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PairChange {
    pub tick: u64,
    pub round: u64,
    pub rule: Rule,
    pub map: Map,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    pub config: AssistanceConfig,
    pub seed: u64,
    pub tick: u64,
    pub rule: Rule,
    pub map: Map,
    pub thresholds: (f64, f64),
    pub rounds: u64,
    pub stable_rounds: u64,
    pub distance: u32,
    pub history: Vec<PairChange>,
    pub equilibria: Vec<(Rule, Map)>,
    pub optimal_map: Map,
    pub optimal_rule: Vec<(f64, f64, u8)>,
    pub optimal_value: f64,
    pub teaching_thresholds: (f64, f64),
    pub expert_posterior_mean: f64,
    pub incentive: Incentive,
    pub surface: Vec<f64>,
}

#[derive(Clone)]
pub struct AssistanceWorld {
    pub config: AssistanceConfig,
    pub tick: u64,
    pub stats: Stats<AssistanceSnapshot>,
    seed: u64,
    rng: SimRng,
    random_rule: Rule,
    rule: Rule,
    map: Map,
    tables: Tables,
    rounds: u64,
    stable_rounds: u64,
    history: Vec<PairChange>,
    last_human: Option<u8>,
    incentive: Incentive,
    surface: Surface,
    equilibria: Vec<(Rule, Map)>,
}

impl AssistanceWorld {
    pub fn new(config: AssistanceConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let bins = config.bins();
        let random_rule: Rule = (0..bins).map(|_| rng.gen_range(0..3u32) as u8).collect();
        let belief = Belief {
            mu: config.mu,
            sigma: config.sigma,
        };
        let incentive = offswitch::incentive(belief, config.beta, config.quadrature_density);
        let surface = offswitch::surface(config.mu, config.quadrature_density);
        let mut w = AssistanceWorld {
            tables: Tables::uniform(bins),
            equilibria: paperclips::mutual_best_responses(bins),
            config,
            tick: 0,
            stats: Stats::default(),
            seed,
            rng,
            random_rule,
            rule: vec![],
            map: [0; 3],
            rounds: 0,
            stable_rounds: 0,
            history: vec![],
            last_human: None,
            incentive,
            surface,
        };
        w.restart_pair();
        let initial = w.exact_snapshot();
        w.stats.push(initial);
        Ok(w)
    }

    pub fn rule(&self) -> &[u8] {
        &self.rule
    }
    pub fn map(&self) -> Map {
        self.map
    }
    pub fn tables(&self) -> &Tables {
        &self.tables
    }
    pub fn incentive(&self) -> &Incentive {
        &self.incentive
    }
    pub fn surface(&self) -> &Surface {
        &self.surface
    }

    fn start_rule(&self) -> Rule {
        let bins = self.config.bins();
        match self.config.human_rule {
            HumanRule::Expert => paperclips::expert_rule(bins),
            HumanRule::Equilibrium => paperclips::equilibrium_rule(bins),
            HumanRule::Pooling => paperclips::pooling_rule(bins),
            HumanRule::UniformRandom => self.random_rule.clone(),
        }
    }

    fn start_map(&self, rule: &[u8]) -> Map {
        match (self.config.mode, self.config.robot_map) {
            (Mode::Learning, _) => dynamics::robot_reply_to_tables(&self.tables),
            (_, RobotMap::Fixed) => self.config.fixed_map(),
            (_, RobotMap::BestResponse) => paperclips::robot_best_reply(rule),
        }
    }

    /// The pair at its start (at reset, or after a live change of a start field), recorded.
    fn restart_pair(&mut self) {
        self.rule = self.start_rule();
        self.map = self.start_map(&self.rule.clone());
        self.stable_rounds = 0;
        self.history.push(PairChange {
            tick: self.tick,
            round: self.rounds,
            rule: self.rule.clone(),
            map: self.map,
        });
    }

    /// One best-response round: a changed pair is recorded, an unchanged one counts as stable.
    fn round(&mut self, rule: Rule, map: Map) {
        self.rounds += 1;
        if rule == self.rule && map == self.map {
            self.stable_rounds += 1;
        } else {
            self.stable_rounds = 0;
            self.rule = rule;
            self.map = map;
            self.history.push(PairChange {
                tick: self.tick,
                round: self.rounds,
                rule: self.rule.clone(),
                map: self.map,
            });
        }
    }

    /// R's posterior over θ-bins after the latest observed H action (the prior before any).
    pub fn posterior(&self) -> Vec<f64> {
        match self.last_human {
            None => vec![1.0 / self.config.bins() as f64; self.config.bins()],
            Some(a) if self.config.mode == Mode::Learning => {
                dynamics::robot_posterior_from_tables(&self.tables, a)
            }
            Some(a) => paperclips::posterior(&self.rule, a),
        }
    }

    /// The exact fields of a snapshot: the pair for paperclips, the incentive for the off-switch.
    fn exact_snapshot(&self) -> AssistanceSnapshot {
        let mut s = AssistanceSnapshot {
            tick: self.tick,
            ..Default::default()
        };
        match self.config.game {
            Game::Paperclips => {
                let (lo, hi) = paperclips::thresholds_of(&self.rule);
                s.threshold_low = lo;
                s.threshold_high = hi;
                s.map_0 = f64::from(self.map[0]);
                s.map_1 = f64::from(self.map[1]);
                s.map_2 = f64::from(self.map[2]);
                s.stable_rounds = self.stable_rounds as f64;
                s.distance = f64::from(paperclips::distance(&self.rule, &self.map));
            }
            Game::OffSwitch => {
                let i = &self.incentive;
                s.delta_exact = i.delta;
                s.prob_correct = i.prob_correct;
                s.rationality = i.rationality;
                s.value_act = i.value_act;
                s.value_wait = i.value_wait;
                s.value_off = i.value_off;
            }
        }
        s
    }

    pub fn step(&mut self) {
        match self.config.game {
            Game::Paperclips => self.step_paperclips(),
            Game::OffSwitch => self.step_off_switch(),
        }
    }

    fn step_paperclips(&mut self) {
        let n = self.config.episodes_per_tick;
        let (mut reward, mut regret) = (0.0, 0.0);
        for _ in 0..n {
            let e = dynamics::paperclip_episode(&mut self.rng, &self.rule, &self.map);
            reward += e.reward;
            regret += e.regret;
            if self.config.mode == Mode::Learning {
                self.tables
                    .observe(e.bin, e.human, e.robot, self.config.fictitious_weight);
            }
            self.last_human = Some(e.human);
        }
        self.tick += 1;
        match self.config.mode {
            Mode::Static => {}
            Mode::Iterated => {
                if self
                    .tick
                    .is_multiple_of(u64::from(self.config.respond_every))
                {
                    let rule = paperclips::human_best_reply(&self.map, self.config.bins());
                    let map = paperclips::robot_best_reply(&rule);
                    self.round(rule, map);
                }
            }
            Mode::Learning => {
                let rule = dynamics::human_reply_to_tables(&self.tables);
                let map = dynamics::robot_reply_to_tables(&self.tables);
                self.round(rule, map);
            }
        }
        let mut s = self.exact_snapshot();
        s.reward = reward / f64::from(n);
        s.regret = regret / f64::from(n);
        self.stats.push(s);
    }

    fn step_off_switch(&mut self) {
        let n = self.config.episodes_per_tick;
        let c = self.config.clone();
        let belief = Belief {
            mu: c.mu,
            sigma: c.sigma,
        };
        let (mut u_sum, mut allowed_u, mut value) = (0.0, 0.0, 0.0);
        let mut shares = [0u32; 3];
        let (mut delta, mut prob_correct, mut rationality, mut value_act, mut value_wait) =
            (0.0, 0.0, 0.0, 0.0, 0.0);
        for _ in 0..n {
            let e = match c.protocol {
                Protocol::Prior => {
                    dynamics::prior_episode(&mut self.rng, belief, c.beta, &self.incentive)
                }
                Protocol::Report => dynamics::report_episode(
                    &mut self.rng,
                    c.sigma,
                    c.beta,
                    c.observation_noise,
                    c.actions,
                    c.quadrature_density,
                ),
            };
            u_sum += e.u;
            if e.allowed {
                allowed_u += e.u;
            }
            value += e.value;
            shares[match e.choice {
                Choice::Act => 0,
                Choice::Wait => 1,
                Choice::Off => 2,
            }] += 1;
            delta += e.incentive.delta;
            prob_correct += e.incentive.prob_correct;
            rationality += e.incentive.rationality;
            value_act += e.incentive.value_act;
            value_wait += e.incentive.value_wait;
        }
        self.tick += 1;
        let div = f64::from(n);
        let s = AssistanceSnapshot {
            tick: self.tick,
            delta_exact: delta / div,
            delta_empirical: allowed_u / div - (u_sum / div).max(0.0),
            prob_correct: prob_correct / div,
            rationality: rationality / div,
            value_act: value_act / div,
            value_wait: value_wait / div,
            value_off: 0.0,
            value: value / div,
            share_act: f64::from(shares[0]) / div,
            share_wait: f64::from(shares[1]) / div,
            share_off: f64::from(shares[2]) / div,
            ..Default::default()
        };
        self.stats.push(s);
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }

    pub fn outcome(&self) -> Outcome {
        let (optimal_map, optimal_rule, optimal_value) = paperclips::optimal_joint();
        let expert = paperclips::expert_rule(self.config.bins());
        Outcome {
            config: self.config.clone(),
            seed: self.seed,
            tick: self.tick,
            rule: self.rule.clone(),
            map: self.map,
            thresholds: paperclips::thresholds_of(&self.rule),
            rounds: self.rounds,
            stable_rounds: self.stable_rounds,
            distance: paperclips::distance(&self.rule, &self.map),
            history: self.history.clone(),
            equilibria: self.equilibria.clone(),
            optimal_map,
            optimal_rule: optimal_rule
                .into_iter()
                .map(|(lo, hi, a)| (lo.to_f64(), hi.to_f64(), a))
                .collect(),
            optimal_value: optimal_value.to_f64(),
            teaching_thresholds: (41.0 / 92.0, 51.0 / 92.0),
            expert_posterior_mean: paperclips::posterior_mean(&paperclips::posterior(&expert, 0)),
            incentive: self.incentive,
            surface: self.surface.cells.iter().map(|c| c.delta).collect(),
        }
    }
}

impl Model for AssistanceWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Assistance(self.config.clone())
    }
    fn run(&mut self, ticks: u32) {
        AssistanceWorld::run(self, ticks)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        2
    }
    fn fingerprint(&self) -> u64 {
        let state = serde_json::json!({
            "config": self.config, "seed": self.seed, "tick": self.tick, "rule": self.rule, "map": self.map,
            "tables": self.tables, "rounds": self.rounds, "stable_rounds": self.stable_rounds,
            "history": self.history, "last_human": self.last_human,
        });
        let bytes = serde_json::to_vec(&state).unwrap();
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for byte in bytes {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        let mut stream = self.rng.clone();
        for _ in 0..2 {
            for byte in stream.gen::<u64>().to_le_bytes() {
                h ^= u64::from(byte);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
        }
        h
    }
    fn size(&self) -> (u32, u32) {
        view::size(self)
    }
    fn render(&self, mode: &str, _layer: &str, buf: &mut Vec<u8>) -> Result<(), String> {
        view::render(self, mode, buf)
    }
    fn latest_json(&self) -> String {
        serde_json::to_string(&self.stats.latest()).unwrap()
    }
    fn series_names(&self) -> Vec<String> {
        super::series_names()
    }
    fn series(&self, name: &str) -> Option<Vec<f64>> {
        self.stats.series(name)
    }
    fn latest_value(&self, name: &str) -> Option<f64> {
        self.stats.latest().and_then(|s| s.value(name))
    }
    fn series_csv(&self) -> String {
        crate::export::history_csv(&self.series_names(), self.stats.history())
    }
    fn agents_csv(&self) -> String {
        let mut out = String::new();
        match self.config.game {
            Game::Paperclips => {
                out.push_str("bin,theta,human_action,deployment,posterior\n");
                let post = self.posterior();
                for (k, &a) in self.rule.iter().enumerate() {
                    writeln!(
                        out,
                        "{k},{},{a},{},{}",
                        paperclips::bin_mid(k, self.rule.len()),
                        self.map[a as usize],
                        post[k]
                    )
                    .unwrap();
                }
            }
            Game::OffSwitch => {
                out.push_str("sigma,beta,delta,rationality,prob_correct,choice\n");
                for (b, &beta) in offswitch::BETAS.iter().enumerate() {
                    for (s, &sigma) in offswitch::SIGMAS.iter().enumerate() {
                        let c = &self.surface.cells[b * offswitch::SIGMAS.len() + s];
                        writeln!(
                            out,
                            "{sigma},{beta},{},{},{},{:?}",
                            c.delta,
                            c.rationality,
                            c.prob_correct,
                            offswitch::choice(c)
                        )
                        .unwrap();
                    }
                }
            }
        }
        out
    }
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        view::inspect(self, x, y)
    }
    fn locate(&self, _id: u64) -> Option<(u32, u32)> {
        None
    }
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Assistance(next) = next else {
            return Err(wrong_model(ModelKind::Assistance, &next));
        };
        next.validate()?;
        let errors: Vec<FieldError> = [
            (next.game != self.config.game, "game"),
            (next.mode != self.config.mode, "mode"),
            (next.theta_bins != self.config.theta_bins, "theta_bins"),
        ]
        .into_iter()
        .filter(|(changed, _)| *changed)
        .map(|(_, field)| FieldError::new(field, "changes only on reset"))
        .collect();
        if !errors.is_empty() {
            return Err(errors);
        }
        let old = std::mem::replace(&mut self.config, next);
        let c = self.config.clone();
        if c.mode != Mode::Learning
            && (c.human_rule, c.robot_map, c.fixed_map())
                != (old.human_rule, old.robot_map, old.fixed_map())
        {
            self.restart_pair();
        }
        if (c.mu, c.sigma, c.beta, c.quadrature_density)
            != (old.mu, old.sigma, old.beta, old.quadrature_density)
        {
            self.incentive = offswitch::incentive(
                Belief {
                    mu: c.mu,
                    sigma: c.sigma,
                },
                c.beta,
                c.quadrature_density,
            );
        }
        if (c.mu, c.quadrature_density) != (old.mu, old.quadrature_density) {
            self.surface = offswitch::surface(c.mu, c.quadrature_density);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assistance::paperclips::{equilibrium_rule, expert_rule};

    fn world(c: AssistanceConfig) -> AssistanceWorld {
        AssistanceWorld::new(c, 1).unwrap()
    }

    #[test]
    fn a_static_expert_world_plays_the_papers_pair_and_records_it_at_tick_zero() {
        let w = world(AssistanceConfig {
            mode: Mode::Static,
            episodes_per_tick: 10,
            ..Default::default()
        });
        assert_eq!((w.rule(), w.map()), (&expert_rule(92)[..], [0, 1, 2]));
        let s = w.stats.latest().unwrap();
        assert_eq!(
            (
                s.tick,
                s.threshold_low,
                s.threshold_high,
                s.map_0,
                s.map_2,
                s.distance
            ),
            (0, 0.5, 0.5, 0.0, 2.0, 10.0)
        );
        assert!(s.reward.is_nan() && s.delta_exact.is_nan());
    }

    #[test]
    fn iterated_best_response_from_the_expert_reaches_the_teaching_pair_in_one_round_and_stays() {
        let mut w = world(AssistanceConfig {
            episodes_per_tick: 10,
            ..Default::default()
        });
        w.step();
        assert_eq!((w.rule(), w.map()), (&equilibrium_rule(92)[..], [0, 1, 2]));
        assert_eq!(w.stats.latest().unwrap().distance, 0.0);
        w.run(5);
        assert_eq!(w.stats.latest().unwrap().stable_rounds, 5.0);
        let o = w.outcome();
        assert_eq!(o.history.len(), 2);
        assert_eq!(
            (o.optimal_map, o.teaching_thresholds),
            ([0, 1, 2], (41.0 / 92.0, 51.0 / 92.0))
        );
        assert!((o.expert_posterior_mean - 0.25).abs() < 1e-12);
    }

    #[test]
    fn respond_every_delays_the_rounds_and_a_fixed_map_holds_in_static_mode() {
        let mut w = world(AssistanceConfig {
            respond_every: 3,
            episodes_per_tick: 10,
            ..Default::default()
        });
        w.run(2);
        assert_eq!(w.rule(), &expert_rule(92)[..]);
        w.step();
        assert_eq!(w.rule(), &equilibrium_rule(92)[..]);
        let mut fixed = world(AssistanceConfig {
            mode: Mode::Static,
            robot_map: RobotMap::Fixed,
            fixed_0: 2,
            fixed_1: 2,
            fixed_2: 2,
            episodes_per_tick: 10,
            ..Default::default()
        });
        fixed.run(3);
        assert_eq!(fixed.map(), [2, 2, 2]);
        assert!(fixed.stats.latest().unwrap().regret > 0.0);
    }

    #[test]
    fn fictitious_play_starts_from_the_prior_map_and_moves_with_observations() {
        let mut w = world(AssistanceConfig {
            mode: Mode::Learning,
            human_rule: HumanRule::UniformRandom,
            episodes_per_tick: 1000,
            ..Default::default()
        });
        assert_eq!(w.map(), [1, 1, 1]);
        assert_eq!(w.rule(), &w.outcome().history[0].rule[..]);
        w.run(50);
        let s = w.stats.latest().unwrap();
        assert!(s.distance.is_finite() && s.stable_rounds >= 0.0);
        assert_ne!(
            w.map(),
            [1, 1, 1],
            "R has seen θ and moved off the prior map"
        );
    }

    #[test]
    fn live_rule_change_restarts_the_pair_and_reset_fields_are_refused() {
        let mut w = world(AssistanceConfig {
            episodes_per_tick: 10,
            ..Default::default()
        });
        w.run(3);
        let err = w
            .set_config(ModelConfig::Assistance(AssistanceConfig {
                theta_bins: 50,
                episodes_per_tick: 10,
                ..Default::default()
            }))
            .unwrap_err();
        assert_eq!(err[0].field, "theta_bins");
        assert_eq!(w.rule().len(), 92);
        w.set_config(ModelConfig::Assistance(AssistanceConfig {
            human_rule: HumanRule::Pooling,
            episodes_per_tick: 10,
            ..Default::default()
        }))
        .unwrap();
        assert_eq!(
            (w.rule(), w.map(), w.outcome().stable_rounds),
            (&vec![1u8; 92][..], [1, 1, 1], 0)
        );
        assert_eq!(w.outcome().history.last().unwrap().tick, 3);
        let other = w
            .set_config(ModelConfig::Auctions(
                crate::auctions::AuctionsConfig::default(),
            ))
            .unwrap_err();
        assert_eq!(other[0].field, "model");
    }

    #[test]
    fn the_off_switch_world_reports_exact_and_empirical_incentives_and_recomputes_live() {
        let mut w = world(AssistanceConfig {
            game: Game::OffSwitch,
            mu: 0.25,
            sigma: 1.0,
            beta: 0.0,
            episodes_per_tick: 20_000,
            ..Default::default()
        });
        w.step();
        let s = w.stats.latest().unwrap().clone();
        assert!((s.delta_exact - 0.286_4).abs() < 1e-3, "{}", s.delta_exact);
        assert!(
            (s.delta_empirical - s.delta_exact).abs() < 0.02,
            "{} vs {}",
            s.delta_empirical,
            s.delta_exact
        );
        assert_eq!((s.share_wait, s.share_act, s.share_off), (1.0, 0.0, 0.0));
        assert!((s.value - s.value_wait).abs() < 0.02);
        w.set_config(ModelConfig::Assistance(AssistanceConfig {
            game: Game::OffSwitch,
            mu: 0.25,
            sigma: 0.1,
            beta: 4.0,
            episodes_per_tick: 20_000,
            ..Default::default()
        }))
        .unwrap();
        assert!(w.incentive().delta < 0.0);
        w.step();
        let s = w.stats.latest().unwrap();
        assert_eq!((s.share_wait, s.share_act), (0.0, 1.0));
        assert!(s.reward.is_nan());
    }

    #[test]
    fn the_report_protocol_averages_per_episode_incentives() {
        let mut w = world(AssistanceConfig {
            game: Game::OffSwitch,
            protocol: Protocol::Report,
            sigma: 0.7,
            beta: 0.5,
            actions: 4,
            episodes_per_tick: 200,
            ..Default::default()
        });
        w.step();
        let s = w.stats.latest().unwrap();
        assert!(s.delta_exact.is_finite() && s.value.is_finite());
        assert!((s.share_wait + s.share_act + s.share_off - 1.0).abs() < 1e-12);
        assert!(s.share_wait > 0.0 && s.share_wait < 1.0, "{s:?}");
    }

    #[test]
    fn two_worlds_with_one_seed_match_at_every_tick_and_the_fingerprint_moves() {
        for c in [
            AssistanceConfig {
                episodes_per_tick: 50,
                ..Default::default()
            },
            AssistanceConfig {
                mode: Mode::Learning,
                episodes_per_tick: 50,
                ..Default::default()
            },
            AssistanceConfig {
                game: Game::OffSwitch,
                beta: 1.0,
                episodes_per_tick: 50,
                ..Default::default()
            },
        ] {
            let (mut a, mut b) = (
                AssistanceWorld::new(c.clone(), 7).unwrap(),
                AssistanceWorld::new(c, 7).unwrap(),
            );
            let first = a.fingerprint();
            for _ in 0..10 {
                a.step();
                b.step();
                assert_eq!(a.fingerprint(), b.fingerprint());
            }
            assert_ne!(a.fingerprint(), first);
        }
    }

    #[test]
    fn frames_have_the_declared_sizes_and_inspection_reads_each_panel() {
        let mut w = world(AssistanceConfig {
            mode: Mode::Static,
            episodes_per_tick: 10,
            ..Default::default()
        });
        w.step();
        assert_eq!(w.size(), (184, 96));
        let mut buf = vec![];
        w.render("bands", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 184 * 96 * 4);
        assert!(w.render("values", "", &mut buf).is_err());
        let human: serde_json::Value =
            serde_json::from_str(&w.inspect_json(90, 3).unwrap()).unwrap();
        assert_eq!(
            (
                human["panel"].as_str(),
                human["bin"].as_u64(),
                human["human_action"].as_u64()
            ),
            (Some("human"), Some(45), Some(0))
        );
        assert!((human["theta"].as_f64().unwrap() - 45.5 / 92.0).abs() < 1e-12);
        let robot: serde_json::Value =
            serde_json::from_str(&w.inspect_json(100, 30).unwrap()).unwrap();
        assert_eq!(
            (robot["panel"].as_str(), robot["map_cell"].as_u64()),
            (Some("robot"), Some(1))
        );
        let post: serde_json::Value =
            serde_json::from_str(&w.inspect_json(0, 90).unwrap()).unwrap();
        assert_eq!(post["panel"], "posterior");
        assert!(w.inspect_json(184, 0).is_err());
        let o = world(AssistanceConfig {
            game: Game::OffSwitch,
            mu: 0.25,
            sigma: 0.1,
            beta: 4.0,
            ..Default::default()
        });
        assert_eq!(o.size(), (240, 252));
        o.render("surface", "", &mut buf).unwrap();
        assert_eq!(buf.len(), 240 * 252 * 4);
        // The configured point (σ = 0.1, β = 4) is the top row's second cell, drawn with a gold border.
        let at = 12 * 4; // row 0, x = 12
        assert_eq!(&buf[at..at + 3], &[240, 210, 70]);
        let cell: serde_json::Value =
            serde_json::from_str(&o.inspect_json(18, 0).unwrap()).unwrap();
        assert_eq!(
            (
                cell["sigma"].as_f64(),
                cell["beta"].as_f64(),
                cell["choice"].as_str()
            ),
            (Some(0.1), Some(4.0), Some("act"))
        );
        assert!(cell["delta"].as_f64().unwrap() < 0.0);
    }

    #[test]
    fn surface_frame_handles_points_off_the_grid() {
        let o = world(AssistanceConfig {
            game: Game::OffSwitch,
            sigma: 5.0,
            beta: 50.0,
            ..Default::default()
        });
        let mut buf = vec![];
        o.render("", "", &mut buf).unwrap();
        assert!(
            !buf.chunks(4).any(|p| p[..3] == [240, 210, 70]),
            "no marker off the grid"
        );
    }

    #[test]
    fn posterior_band_under_a_fixed_map_uses_the_prior_off_path() {
        // The pooling rule never makes (0,2) or (2,0); the posterior after (1,1) is the prior.
        let mut w = world(AssistanceConfig {
            mode: Mode::Static,
            human_rule: HumanRule::Pooling,
            episodes_per_tick: 5,
            ..Default::default()
        });
        assert!(w
            .posterior()
            .iter()
            .all(|&p| (p - 1.0 / 92.0).abs() < 1e-15));
        w.step();
        assert!(w
            .posterior()
            .iter()
            .all(|&p| (p - 1.0 / 92.0).abs() < 1e-15));
        let mut expert = world(AssistanceConfig {
            mode: Mode::Static,
            episodes_per_tick: 1,
            ..Default::default()
        });
        expert.step();
        let post = expert.posterior();
        assert!(post.iter().take(46).all(|&p| p == 0.0) || post.iter().skip(46).all(|&p| p == 0.0));
    }

    #[test]
    fn csv_exports_name_every_series_and_list_bins_or_cells() {
        let mut w = world(AssistanceConfig {
            episodes_per_tick: 5,
            ..Default::default()
        });
        w.step();
        assert!(w
            .series_csv()
            .starts_with("tick,reward,regret,threshold_low,"));
        assert_eq!(w.agents_csv().lines().count(), 93);
        let o = world(AssistanceConfig {
            game: Game::OffSwitch,
            ..Default::default()
        });
        assert_eq!(o.agents_csv().lines().count(), 421);
        assert_eq!(o.series("value").unwrap().len(), 1);
        assert!(o.series("nope").is_none());
    }
}
