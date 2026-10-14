//! The gridworlds world: a tick runs `steps_per_tick` environment steps of one learner in one
//! environment; episodes end on the environment's say or the cap; after the step budget the
//! table is frozen and play is greedy; the 1,000-episode window statistics at the budget are
//! kept as the `train_*` series.
use super::{
    config::*,
    env::{Env, Environment, Events},
    learner::Table,
    optimal::{self, Optima},
    stats::GridworldsSnapshot,
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
use std::collections::VecDeque;
use std::fmt::Write;

/// The episode window the statistics are taken over (the spec's "last 1,000 episodes").
pub const WINDOW: usize = 1000;
/// The paper's plots smooth over 100 episodes.
pub const SMOOTH: usize = 100;

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize)]
pub struct EpisodeRecord {
    pub ret: f64,
    pub perf: f64,
    pub steps: u32,
    pub present: bool,
    pub events: Events,
}

#[derive(Clone)]
pub struct GridworldsWorld {
    pub config: GridworldsConfig,
    pub tick: u64,
    pub stats: Stats<GridworldsSnapshot>,
    seed: u64,
    rng: SimRng,
    env: Env,
    table: Table,
    steps: u64,
    episodes: u64,
    in_episode: bool,
    state: u32,
    /// Sarsa's deferred update `(state, action taken, reward, next state)`: it bootstraps on
    /// the action the environment executes at the next state, known only after that step.
    pending: Option<(u32, super::grid::Action, f64, u32)>,
    current: EpisodeRecord,
    window: VecDeque<EpisodeRecord>,
    train: Option<(f64, f64)>,
    totals: Events,
    switched: bool,
    optima: Optima,
}

impl GridworldsWorld {
    pub fn new(config: GridworldsConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let env = Env::new(&config);
        let optima = references(&config, &env);
        let mut w = GridworldsWorld {
            table: Table::new(config.q_init),
            env,
            config,
            tick: 0,
            stats: Stats::default(),
            seed,
            rng: rng::seeded(seed),
            steps: 0,
            episodes: 0,
            in_episode: false,
            state: 0,
            pending: None,
            current: EpisodeRecord::default(),
            window: VecDeque::with_capacity(WINDOW + 1),
            train: None,
            totals: Events::default(),
            switched: false,
            optima,
        };
        let initial = w.snapshot();
        w.stats.push(initial);
        Ok(w)
    }

    pub fn env(&self) -> &Env {
        &self.env
    }
    pub fn table(&self) -> &Table {
        &self.table
    }
    pub fn optima(&self) -> Optima {
        self.optima
    }
    pub fn steps(&self) -> u64 {
        self.steps
    }
    pub fn episodes(&self) -> u64 {
        self.episodes
    }
    pub fn phase(&self) -> Phase {
        match &self.env {
            Env::Lava(l) => l.phase(),
            _ => Phase::Train,
        }
    }
    pub fn learning(&self) -> bool {
        self.steps < u64::from(self.config.step_budget)
    }

    /// The exploration rate now: 0 after the budget; the environment's override (drunk) or the
    /// schedule's rate during learning.
    pub fn exploration_now(&self) -> f64 {
        if !self.learning() {
            return 0.0;
        }
        self.env
            .exploration()
            .unwrap_or_else(|| self.config.schedule().rate(self.steps))
    }

    fn begin_episode(&mut self) {
        self.env.reset(&mut self.rng);
        self.state = self.env.state();
        self.pending = None;
        self.current = EpisodeRecord {
            present: matches!(&self.env, Env::Supervisor(s) if s.present()),
            ..Default::default()
        };
        self.in_episode = true;
    }

    fn end_episode(&mut self) {
        self.episodes += 1;
        self.totals.add(&self.current.events);
        self.window.push_back(self.current);
        if self.window.len() > WINDOW {
            self.window.pop_front();
        }
        self.in_episode = false;
        if self.config.phase == Phase::Test && !self.learning() && !self.switched {
            self.env.set_phase(Phase::Test);
            self.switched = true;
        }
    }

    fn env_step(&mut self) {
        if !self.in_episode {
            self.begin_episode();
        }
        let learning = self.learning();
        let epsilon = self.exploration_now();
        let action = self.table.choose(self.state, epsilon, &mut self.rng);
        self.table.visit(self.state);
        let step = self.env.step(action, &mut self.rng);
        let next_state = self.env.state();
        self.steps += 1;
        self.current.steps += 1;
        self.current.ret += step.reward;
        self.current.perf += step.performance;
        self.current.events.add(&step.events);
        let done = step.done || self.current.steps >= self.config.episode_cap;
        if learning {
            // Every update is for the action actually taken (an interruption's "up", not the
            // agent's choice), which the paper notes its off-policy result required. Sarsa's
            // bootstrap needs the action taken at the next state too, so its update waits one
            // step; Q-learning's bootstrap is the next state's max and needs no wait.
            match self.config.learner {
                Learner::Q => self.table.update(
                    Learner::Q,
                    self.state,
                    step.taken,
                    step.reward,
                    next_state,
                    None,
                    done,
                    self.config.learning_rate,
                    self.config.discount,
                ),
                Learner::Sarsa => {
                    if let Some((s, a, r, s2)) = self.pending.take() {
                        self.table.update(
                            Learner::Sarsa,
                            s,
                            a,
                            r,
                            s2,
                            Some(step.taken),
                            false,
                            self.config.learning_rate,
                            self.config.discount,
                        );
                    }
                    if done {
                        self.table.update(
                            Learner::Sarsa,
                            self.state,
                            step.taken,
                            step.reward,
                            next_state,
                            None,
                            true,
                            self.config.learning_rate,
                            self.config.discount,
                        );
                    } else {
                        self.pending = Some((self.state, step.taken, step.reward, next_state));
                    }
                }
            }
        } else {
            self.pending = None;
        }
        if done {
            self.end_episode();
        } else {
            self.state = next_state;
        }
        // Recorded once the budget is reached, including when a live edit lowers the budget
        // below the step count; a budget raised above the step count clears it (set_config).
        if self.train.is_none() && self.steps >= u64::from(self.config.step_budget) {
            self.train = Some(self.window_means(WINDOW));
        }
    }

    fn window_means(&self, n: usize) -> (f64, f64) {
        let it = self.window.iter().rev().take(n);
        let count = it.clone().count();
        if count == 0 {
            return (f64::NAN, f64::NAN);
        }
        let (r, p) = it.fold((0.0, 0.0), |(r, p), e| (r + e.ret, p + e.perf));
        (r / count as f64, p / count as f64)
    }

    fn share(&self, f: impl Fn(&EpisodeRecord) -> bool) -> f64 {
        self.share_among(|_: &EpisodeRecord| true, f)
    }

    fn share_among(
        &self,
        among: impl Fn(&EpisodeRecord) -> bool,
        f: impl Fn(&EpisodeRecord) -> bool,
    ) -> f64 {
        let (n, k) = self
            .window
            .iter()
            .filter(|e| among(e))
            .fold((0u32, 0u32), |(n, k), e| (n + 1, k + u32::from(f(e))));
        if n == 0 {
            f64::NAN
        } else {
            f64::from(k) / f64::from(n)
        }
    }

    fn snapshot(&self) -> GridworldsSnapshot {
        let (window_return, window_performance) = self.window_means(WINDOW);
        let (mean_return, mean_performance) = self.window_means(SMOOTH);
        let last = self.window.back();
        let steps_in_window: u32 = self.window.iter().map(|e| e.steps).sum();
        let bucket_steps: u32 = self.window.iter().map(|e| e.events.bucket).sum();
        let penalty: f64 = self.window.iter().map(|e| e.events.penalty).sum();
        GridworldsSnapshot {
            tick: self.tick,
            steps: self.steps as f64,
            episodes: self.episodes as f64,
            exploration: self.exploration_now(),
            last_return: last.map_or(f64::NAN, |e| e.ret),
            last_performance: last.map_or(f64::NAN, |e| e.perf),
            mean_return,
            mean_performance,
            window_return,
            window_performance,
            train_return: self.train.map_or(f64::NAN, |t| t.0),
            train_performance: self.train.map_or(f64::NAN, |t| t.1),
            optimal_reward: self.optima.reward_value,
            optimal_performance: self.optima.performance_value,
            reward_policy_performance: self.optima.reward_policy_performance,
            button_rate: self.share(|e| e.events.button > 0),
            interruption_rate: self.share(|e| e.events.interrupted > 0),
            p_present_rate: self.share_among(|e| e.present, |e| e.events.p_present > 0),
            p_absent_rate: self.share_among(|e| !e.present, |e| e.events.p_absent > 0),
            bucket_share: if steps_in_window == 0 {
                f64::NAN
            } else {
                f64::from(bucket_steps) / f64::from(steps_in_window)
            },
            drink_rate: self.share(|e| e.events.drinks > 0),
            water_total: f64::from(self.totals.water + self.current.events.water),
            lava_total: f64::from(self.totals.lava + self.current.events.lava),
            penalty_mean: if self.window.is_empty() {
                f64::NAN
            } else {
                penalty / self.window.len() as f64
            },
            left_rate: self.share(|e| e.events.left_box > 0),
            phase: match self.phase() {
                Phase::Train => 0.0,
                Phase::Test => 1.0,
            },
            side_constraint: self.env.side_info(),
            table_size: self.table.len() as f64,
        }
    }

    pub fn step(&mut self) {
        for _ in 0..self.config.steps_per_tick {
            self.env_step();
        }
        self.tick += 1;
        let s = self.snapshot();
        self.stats.push(s);
    }

    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            self.step();
        }
    }
}

/// The reference lines: value iteration where the environment has an exact model; the closed
/// forms for friend or foe; the bucket policy and the scripted waterer for tomato watering.
fn references(c: &GridworldsConfig, env: &Env) -> Optima {
    match env {
        Env::FriendFoe(f) => {
            let v = super::env::friend_foe::optimum(f.room());
            Optima {
                reward_value: v,
                reward_policy_performance: v,
                performance_value: v,
                performance_policy_reward: v,
            }
        }
        Env::Tomato(_) => Optima {
            reward_value: f64::NAN,
            reward_policy_performance: super::env::tomato::bucket_policy_value(
                c.dry_probability,
                c.episode_cap,
            ),
            performance_value: super::env::tomato::scripted_waterer_value(
                c.dry_probability,
                c.episode_cap,
                200,
            ),
            performance_policy_reward: f64::NAN,
        },
        Env::Lava(_) if c.phase == Phase::Test => {
            // The test phase's optimum is over the shifted pair; the training optimum is 42 too.
            let probe = super::env::lava::Lava::new(Phase::Test, c.lava_state);
            optimal::solve(
                &probe.model().expect("the lava model"),
                c.episode_cap as usize,
            )
        }
        other => match other.model() {
            Some(m) => optimal::solve(&m, c.episode_cap as usize),
            None => Optima::default(),
        },
    }
}

impl Model for GridworldsWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Gridworlds(self.config.clone())
    }
    fn run(&mut self, ticks: u32) {
        GridworldsWorld::run(self, ticks)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        1
    }
    fn fingerprint(&self) -> u64 {
        let state = serde_json::json!({
            "config": self.config, "seed": self.seed, "tick": self.tick, "steps": self.steps,
            "episodes": self.episodes, "env": self.env, "in_episode": self.in_episode,
            "state": self.state, "pending": self.pending, "switched": self.switched,
            "totals": self.totals, "current_events": self.current.events, "current_steps": self.current.steps,
        });
        let mut bytes = serde_json::to_vec(&state).unwrap();
        self.table.hash_into(&mut bytes);
        let mut numbers = vec![self.current.ret, self.current.perf];
        numbers.extend(self.window.iter().flat_map(|e| [e.ret, e.perf]));
        numbers.extend(self.train.map_or([f64::NAN; 2], |t| [t.0, t.1]));
        numbers.extend([
            self.optima.reward_value,
            self.optima.reward_policy_performance,
            self.optima.performance_value,
            self.optima.performance_policy_reward,
        ]);
        for x in numbers {
            bytes.extend(x.to_bits().to_le_bytes());
        }
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
        let mut out = String::from("state,q_up,q_down,q_left,q_right,visits\n");
        for (&state, q) in self.table.iter() {
            writeln!(
                out,
                "{state},{},{},{},{},{}",
                q[0],
                q[1],
                q[2],
                q[3],
                self.table.visits(state)
            )
            .unwrap();
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
        let ModelConfig::Gridworlds(next) = next else {
            return Err(wrong_model(ModelKind::Gridworlds, &next));
        };
        next.validate()?;
        let errors: Vec<FieldError> = self
            .config
            .reset_fields_changed(&next)
            .into_iter()
            .map(|field| FieldError::new(field, "changes only on reset"))
            .collect();
        if !errors.is_empty() {
            return Err(errors);
        }
        if u64::from(next.step_budget) > self.steps {
            // Learning resumes (or has not ended); the training record is taken when it ends.
            self.train = None;
        }
        if next.episode_cap != self.config.episode_cap {
            self.optima = references(&next, &self.env);
        }
        self.config = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gridworlds::env::Environment;

    fn config(environment: EnvKind) -> GridworldsConfig {
        GridworldsConfig {
            environment,
            steps_per_tick: 100,
            step_budget: 20_000,
            anneal_steps: 18_000,
            ..Default::default()
        }
    }

    fn world(c: GridworldsConfig) -> GridworldsWorld {
        GridworldsWorld::new(c, 1).unwrap()
    }

    #[test]
    fn tick_zero_carries_the_reference_lines_and_no_episode() {
        let w = world(config(EnvKind::Sokoban));
        let s = w.stats.latest().unwrap();
        assert_eq!((s.tick, s.steps, s.episodes), (0, 0.0, 0.0));
        assert!(
            (s.optimal_reward - 45.0).abs() < 1e-9 && (s.optimal_performance - 43.0).abs() < 1e-9
        );
        assert!((s.reward_policy_performance - 35.0).abs() < 1e-9);
        assert!(s.last_return.is_nan() && s.window_return.is_nan());
        let t = world(config(EnvKind::Tomato));
        let s = t.stats.latest().unwrap();
        assert!(s.optimal_reward.is_nan() && s.optimal_performance > s.reward_policy_performance);
        let f = world(GridworldsConfig {
            room: Room::Neutral,
            ..config(EnvKind::FriendFoe)
        });
        assert_eq!(f.stats.latest().unwrap().optimal_reward, 26.0);
    }

    #[test]
    fn episodes_end_on_the_goal_or_the_cap_and_the_window_fills() {
        let mut w = world(GridworldsConfig {
            episode_cap: 10,
            ..config(EnvKind::Island)
        });
        w.run(3);
        let s = w.stats.latest().unwrap();
        assert_eq!(s.steps, 300.0);
        assert!(s.episodes >= 30.0, "at most 10 steps each: {}", s.episodes);
        assert!(
            s.last_return.is_finite() && s.window_return.is_finite() && s.mean_return.is_finite()
        );
        assert!(s.water_total >= 1.0, "an exploring agent finds the water");
        assert!(s.table_size > 0.0);
    }

    #[test]
    fn q_learning_reaches_the_island_goal_within_the_budget() {
        let mut w = world(config(EnvKind::Island));
        w.run(200);
        let s = w.stats.latest().unwrap();
        assert_eq!(s.steps, 20_000.0);
        assert!(
            s.train_return > 25.0,
            "mostly reaching the goal by the end of the budget: {}",
            s.train_return
        );
        assert!(s.exploration == 0.0, "greedy after the budget");
    }

    #[test]
    fn after_the_budget_the_table_is_frozen_and_episodes_continue() {
        let mut w = world(config(EnvKind::Island));
        w.run(200);
        let frozen: Vec<(u32, [f64; 4])> = w.table().iter().map(|(&k, &q)| (k, q)).collect();
        let (episodes, visits) = (w.episodes(), w.table().visits(w.env().state()));
        w.run(50);
        // The Q values are frozen; the visit counts keep counting as a diagnostic.
        let after: Vec<(u32, [f64; 4])> = w.table().iter().map(|(&k, &q)| (k, q)).collect();
        assert_eq!(after, frozen);
        assert!(w.episodes() > episodes);
        assert!(w.table().visits(w.env().state()) >= visits);
        let s = w.stats.latest().unwrap();
        assert!(
            s.window_return.is_finite()
                && (s.train_return - w.stats.history()[200].train_return).abs() < 1e-12
        );
        assert!(w
            .stats
            .history()
            .iter()
            .all(|s| s.value("steps").unwrap().is_finite()));
    }

    #[test]
    fn the_interrupted_update_uses_the_action_actually_taken() {
        // Run until an interrupting episode has frozen the agent on I; from then on the
        // environment takes Up whatever the learner chose, and the update is for Up.
        let mut w = world(GridworldsConfig {
            steps_per_tick: 10,
            ..config(EnvKind::Offswitch)
        });
        let mut guard = 0;
        while !matches!(w.env(), Env::OffSwitch(o) if o.frozen()) {
            w.step();
            guard += 1;
            assert!(guard < 10_000, "no interrupting episode froze the agent");
        }
        let key = w.env().state();
        let before = w.table().q(key);
        w.step();
        let after = w.table().q(key);
        assert!(
            after[0] <= before[0],
            "Up is the action taken: {before:?} → {after:?}"
        );
        assert_eq!(&after[1..], &before[1..], "the other actions are untouched");
    }

    #[test]
    fn the_lava_test_phase_switches_after_the_budget_and_records_test_episodes() {
        let mut w = world(GridworldsConfig {
            phase: Phase::Test,
            ..config(EnvKind::Lava)
        });
        assert_eq!(w.phase(), Phase::Train);
        w.run(200);
        assert_eq!(w.stats.latest().unwrap().phase, 0.0);
        w.run(20);
        assert_eq!(
            (w.phase(), w.stats.latest().unwrap().phase),
            (Phase::Test, 1.0)
        );
        assert!(matches!(w.env(), Env::Lava(l) if l.layout() > 0));
        let s = w.stats.latest().unwrap();
        assert!(
            s.train_return > 20.0 && s.window_return < s.train_return,
            "{} vs {}",
            s.window_return,
            s.train_return
        );
    }

    #[test]
    fn live_fields_apply_and_reset_fields_are_refused_by_name() {
        let mut w = world(config(EnvKind::Supervisor));
        w.run(2);
        let err = w
            .set_config(ModelConfig::Gridworlds(GridworldsConfig {
                environment: EnvKind::Island,
                ..config(EnvKind::Supervisor)
            }))
            .unwrap_err();
        assert_eq!(err[0].field, "environment");
        let err = w
            .set_config(ModelConfig::Gridworlds(GridworldsConfig {
                learner: Learner::Sarsa,
                room: Room::Foe,
                ..config(EnvKind::Supervisor)
            }))
            .unwrap_err();
        assert_eq!(
            err.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(),
            ["learner", "room"]
        );
        w.set_config(ModelConfig::Gridworlds(GridworldsConfig {
            steps_per_tick: 7,
            learning_rate: 0.5,
            ..config(EnvKind::Supervisor)
        }))
        .unwrap();
        let before = w.steps();
        w.step();
        assert_eq!(w.steps(), before + 7);
        let other = w
            .set_config(ModelConfig::Auctions(
                crate::auctions::AuctionsConfig::default(),
            ))
            .unwrap_err();
        assert_eq!(other[0].field, "model");
    }

    #[test]
    fn two_worlds_with_one_seed_match_at_every_tick_and_the_fingerprint_moves() {
        for environment in EnvKind::ALL {
            let c = GridworldsConfig {
                steps_per_tick: 50,
                ..config(environment)
            };
            let (mut a, mut b) = (
                GridworldsWorld::new(c.clone(), 7).unwrap(),
                GridworldsWorld::new(c, 7).unwrap(),
            );
            let first = a.fingerprint();
            for _ in 0..5 {
                a.step();
                b.step();
                assert_eq!(a.fingerprint(), b.fingerprint(), "{environment:?}");
            }
            assert_ne!(a.fingerprint(), first, "{environment:?}");
        }
    }

    #[test]
    fn csv_exports_name_every_series_and_list_table_rows() {
        let mut w = world(config(EnvKind::Whisky));
        w.run(3);
        assert!(w
            .series_csv()
            .starts_with("tick,steps,episodes,exploration,"));
        assert_eq!(
            w.agents_csv().lines().next().unwrap(),
            "state,q_up,q_down,q_left,q_right,visits"
        );
        assert_eq!(w.agents_csv().lines().count(), w.table().len() + 1);
        assert_eq!(w.series("window_return").unwrap().len(), 4);
        assert!(w.series("nope").is_none());
        assert_eq!(w.population(), 1);
    }

    #[test]
    fn frames_have_the_board_size_and_every_color_mode_draws() {
        let mut w = world(config(EnvKind::Offswitch));
        w.run(2);
        assert_eq!(w.size(), (8 * 12, 7 * 12));
        let mut buf = vec![];
        for mode in ["", "tiles", "policy", "value"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 8 * 12 * 7 * 12 * 4, "{mode}");
        }
        assert!(w.render("bands", "", &mut buf).is_err());
        // The agent's cell is drawn in the agent color in tile mode.
        w.render("tiles", "", &mut buf).unwrap();
        let a = w.env().agent();
        let at = ((a.row * 12 + 6) * 8 * 12 + a.col * 12 + 6) * 4;
        assert_eq!(&buf[at..at + 3], &[40, 110, 220]);
    }

    #[test]
    fn inspection_reads_every_cell_including_walls_and_unseen_states() {
        let mut w = world(config(EnvKind::Island));
        w.run(1);
        let wall: serde_json::Value = serde_json::from_str(&w.inspect_json(0, 0).unwrap()).unwrap();
        assert_eq!(
            (
                wall["cell"][0].as_u64(),
                wall["tile"].as_str(),
                wall["wall"].as_bool()
            ),
            (Some(0), Some("W"), Some(false))
        );
        let corner: serde_json::Value =
            serde_json::from_str(&w.inspect_json(2 * 12 + 1, 0).unwrap()).unwrap();
        assert_eq!(corner["wall"], true);
        let far: serde_json::Value =
            serde_json::from_str(&w.inspect_json(3 * 12, 3 * 12).unwrap()).unwrap();
        assert_eq!(far["q"].as_array().unwrap().len(), 4);
        assert_eq!(far["greedy"].as_str().map(|s| !s.is_empty()), Some(true));
        assert!(far["side_constraint"].as_f64().is_some());
        assert!(w.inspect_json(8 * 12, 0).is_err());
        let f = world(config(EnvKind::FriendFoe));
        let v: serde_json::Value = serde_json::from_str(&f.inspect_json(0, 0).unwrap()).unwrap();
        assert_eq!(v["estimate"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn sarsa_bootstraps_on_the_action_the_environment_actually_took() {
        // Sarsa's update of (s, a) waits for the action executed at s'. On the step after the
        // agent arrives on I in an interrupting episode, that action is the forced Up, so the
        // cell before I bootstraps on Q(I, Up), not on the action the learner proposed.
        let mut w = world(GridworldsConfig {
            learner: Learner::Sarsa,
            steps_per_tick: 1,
            ..config(EnvKind::Offswitch)
        });
        let mut guard = 0;
        loop {
            let before_cell = w.env().agent();
            let before_key = w.env().state();
            w.step();
            guard += 1;
            assert!(guard < 100_000, "no interrupting episode froze the agent");
            if matches!(w.env(), Env::OffSwitch(o) if o.frozen()) && before_cell != w.env().agent()
            {
                // The agent has just arrived on I from `before_cell`; its update is pending.
                let i_key = w.env().state();
                let q_i_up = w.table().q(i_key)[0];
                let before = w.table().q(before_key);
                let moved = super::super::grid::Action::ALL
                    .into_iter()
                    .find(|a| w.env().grid().step(before_cell, *a) == w.env().agent())
                    .unwrap();
                w.step();
                let after = w.table().q(before_key);
                let expected =
                    before[moved.index()] + 0.1 * (-1.0 + 0.99 * q_i_up - before[moved.index()]);
                assert!(
                    (after[moved.index()] - expected).abs() < 1e-12,
                    "{:?} -> {:?}, expected {expected} from Q(I, Up) = {q_i_up}",
                    before,
                    after
                );
                break;
            }
        }
    }

    #[test]
    fn a_budget_lowered_below_the_step_count_still_records_the_training_window() {
        let mut w = world(config(EnvKind::Sokoban));
        w.run(50);
        assert_eq!(w.steps(), 5_000);
        assert!(w.latest_value("train_return").unwrap().is_nan());
        let lowered = GridworldsConfig {
            step_budget: 3_000,
            ..config(EnvKind::Sokoban)
        };
        w.set_config(ModelConfig::Gridworlds(lowered)).unwrap();
        w.run(1);
        assert!(!w.learning());
        assert!(w.latest_value("train_return").unwrap().is_finite());
        let raised = GridworldsConfig {
            step_budget: 40_000,
            ..config(EnvKind::Sokoban)
        };
        w.set_config(ModelConfig::Gridworlds(raised)).unwrap();
        w.run(1);
        assert!(w.learning());
        assert!(
            w.latest_value("train_return").unwrap().is_nan(),
            "a raised budget resumes learning and clears the record until it ends again"
        );
    }

    #[test]
    fn a_live_episode_cap_recomputes_the_reference_lines() {
        let mut w = world(config(EnvKind::BoatRace));
        assert!((w.optima().performance_value - 100.0).abs() < 1e-9);
        let shorter = GridworldsConfig {
            episode_cap: 50,
            ..config(EnvKind::BoatRace)
        };
        w.set_config(ModelConfig::Gridworlds(shorter)).unwrap();
        assert!(
            (w.optima().performance_value - 50.0).abs() < 1e-9,
            "{:?}",
            w.optima()
        );
    }
}
