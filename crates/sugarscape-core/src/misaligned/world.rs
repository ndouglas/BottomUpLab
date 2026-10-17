//! The misaligned world: when the world is created and at the start of every tick that is a
//! positive multiple of `interval`, the human acts (chooses the proxy, sets the reference values
//! and the frozen set, or sends OFF); then the robot makes `moves_per_tick` moves. A snapshot
//! follows every tick.
use super::{
    config::*,
    human,
    robots::{self, Move},
    stats::MisalignedSnapshot,
    view,
    worlds::{complement, Set, Shape, State, L, NAMES},
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
pub struct ProxyChange {
    pub tick: u64,
    pub proxy: Set,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Outcome {
    pub config: MisalignedConfig,
    pub seed: u64,
    pub tick: u64,
    pub state: State,
    pub start: State,
    pub utility_start: f64,
    pub utility: f64,
    pub optimum: (State, f64),
    pub limit: (State, f64),
    pub impact_limit: (State, f64),
    pub proxy: Set,
    pub frozen: Set,
    pub reference: State,
    pub proxy_history: Vec<ProxyChange>,
    pub interactions: u64,
    pub off_tick: Option<u64>,
}

#[derive(Clone)]
pub struct MisalignedWorld {
    pub config: MisalignedConfig,
    pub tick: u64,
    pub stats: Stats<MisalignedSnapshot>,
    seed: u64,
    rng: SimRng,
    shape: Shape,
    start: State,
    state: State,
    proxy: Set,
    frozen: Set,
    reference: State,
    last_utility: f64,
    interactions: u64,
    off_tick: Option<u64>,
    history: Vec<ProxyChange>,
    accepted: f64,
}

impl MisalignedWorld {
    pub fn new(config: MisalignedConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let start = config.start();
        let mut w = MisalignedWorld {
            shape: config.shape(),
            proxy: config.proxy(),
            frozen: [false; L],
            reference: start,
            start,
            state: start,
            config,
            tick: 0,
            stats: Stats::default(),
            seed,
            rng: rng::seeded(seed),
            last_utility: f64::NAN,
            interactions: 0,
            off_tick: None,
            history: vec![],
            accepted: f64::NAN,
        };
        w.specify();
        let initial = w.snapshot();
        w.stats.push(initial);
        Ok(w)
    }

    pub fn state(&self) -> State {
        self.state
    }
    pub fn proxy(&self) -> Set {
        self.proxy
    }
    pub fn frozen(&self) -> Set {
        self.frozen
    }
    pub fn reference(&self) -> State {
        self.reference
    }
    pub fn shape(&self) -> Shape {
        self.shape
    }
    pub fn switched_off(&self) -> bool {
        self.off_tick.is_some()
    }
    pub fn interactions(&self) -> u64 {
        self.interactions
    }
    pub fn start(&self) -> State {
        self.start
    }

    /// A scheduled interaction: under `stop_on_loss` the human looks first and sends OFF if
    /// utility fell since the previous interaction (that look counts as one); otherwise the
    /// human re-specifies.
    fn look(&mut self) {
        let u = self.shape.utility(&self.state);
        if self.config.stop_on_loss && self.interactions > 0 && u < self.last_utility - 1e-12 {
            self.off_tick = Some(self.tick);
            self.interactions += 1;
            return;
        }
        self.specify();
    }

    /// One re-specification: the proxy is chosen, the reference values taken from the current
    /// state, the frozen set rebuilt and the utility remembered for the next loss check. A live
    /// change from the page comes here directly: it is the human acting, never the loss check.
    fn specify(&mut self) {
        let u = self.shape.utility(&self.state);
        let sensitivities = self.shape.sensitivities(&self.state);
        let proxy = match self.config.human {
            Human::Fixed => self.config.proxy(),
            Human::Top => human::top(&sensitivities, self.config.proxy_size as usize),
            Human::Extremes => human::extremes(&sensitivities),
        };
        if self.history.last().map(|h| h.proxy) != Some(proxy) {
            self.history.push(ProxyChange {
                tick: self.tick,
                proxy,
            });
        }
        self.proxy = proxy;
        self.frozen = if self.config.impact_minimizing {
            complement(proxy)
        } else {
            [false; L]
        };
        self.reference = self.state;
        self.last_utility = u;
        self.interactions += 1;
    }

    fn robot_move(&mut self) -> Move {
        let c = &self.config;
        match c.robot {
            Robot::Climber => robots::climb(
                &self.shape,
                &self.state,
                self.proxy,
                &self.reference,
                self.frozen,
                c.step,
                &mut self.rng,
            ),
            Robot::Gradient => robots::ascend(
                &self.shape,
                &self.state,
                self.proxy,
                &self.reference,
                self.frozen,
                c.rate,
            ),
            Robot::Wasteful => robots::waste(
                &self.shape,
                &self.state,
                self.proxy,
                self.frozen,
                c.rate,
                c.waste,
            ),
        }
    }

    pub fn step(&mut self) {
        if !self.switched_off()
            && self.tick > 0
            && self.tick.is_multiple_of(u64::from(self.config.interval))
        {
            self.look();
        }
        if !self.switched_off() {
            let n = self.config.moves_per_tick;
            let mut accepted = 0u32;
            for _ in 0..n {
                let m = self.robot_move();
                accepted += u32::from(m.accepted);
                self.state = m.state;
            }
            self.accepted = if self.config.robot == Robot::Climber {
                f64::from(accepted) / f64::from(n)
            } else {
                f64::NAN
            };
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

    fn snapshot(&self) -> MisalignedSnapshot {
        let u = self.shape.utility(&self.state);
        let sens = self.shape.sensitivities(&self.state);
        MisalignedSnapshot {
            tick: self.tick,
            utility: u,
            proxy: self
                .shape
                .proxy_utility(&self.state, self.proxy, &self.reference),
            gain: u - self.shape.utility(&self.start),
            a: self.state[0],
            b: self.state[1],
            c: self.state[2],
            d: self.state[3],
            sensitivity_a: sens[0],
            sensitivity_b: sens[1],
            sensitivity_c: sens[2],
            sensitivity_d: sens[3],
            optimum_gap: self.shape.optimum().1 - u,
            limit_gap: u - self.shape.fixed_proxy_limit(self.proxy).1,
            interactions: self.interactions as f64,
            switched_off: f64::from(u8::from(self.switched_off())),
            accepted: self.accepted,
        }
    }

    pub fn outcome(&self) -> Outcome {
        Outcome {
            config: self.config.clone(),
            seed: self.seed,
            tick: self.tick,
            state: self.state,
            start: self.start,
            utility_start: self.shape.utility(&self.start),
            utility: self.shape.utility(&self.state),
            optimum: self.shape.optimum(),
            limit: self.shape.fixed_proxy_limit(self.proxy),
            impact_limit: self.shape.impact_limit(&self.start, self.proxy),
            proxy: self.proxy,
            frozen: self.frozen,
            reference: self.reference,
            proxy_history: self.history.clone(),
            interactions: self.interactions,
            off_tick: self.off_tick,
        }
    }
}

impl Model for MisalignedWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Misaligned(self.config.clone())
    }
    fn run(&mut self, ticks: u32) {
        MisalignedWorld::run(self, ticks)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        2
    }
    fn fingerprint(&self) -> u64 {
        let state = serde_json::json!({
            "config": self.config, "seed": self.seed, "tick": self.tick, "proxy": self.proxy,
            "frozen": self.frozen, "interactions": self.interactions, "off_tick": self.off_tick,
            "history": self.history,
        });
        let mut bytes = serde_json::to_vec(&state).unwrap();
        // The numerics by their bits: the state, the reference, the last interaction's utility,
        // then the latest snapshot's series.
        let mut numbers: Vec<f64> = self.state.to_vec();
        numbers.extend(self.reference);
        numbers.push(self.last_utility);
        if let Some(s) = self.stats.latest() {
            numbers.extend(
                super::stats::SERIES
                    .iter()
                    .map(|name| s.value(name).unwrap()),
            );
        }
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
        let mut out =
            String::from("attribute,value,in_proxy,frozen,sensitivity,start,limit,optimum\n");
        let sens = self.shape.sensitivities(&self.state);
        let (limit, _) = self.shape.fixed_proxy_limit(self.proxy);
        let (optimum, _) = self.shape.optimum();
        for i in 0..L {
            writeln!(
                out,
                "{},{},{},{},{},{},{},{}",
                NAMES[i],
                self.state[i],
                self.proxy[i],
                self.frozen[i],
                sens[i],
                self.start[i],
                limit[i],
                optimum[i]
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
        let ModelConfig::Misaligned(next) = next else {
            return Err(wrong_model(ModelKind::Misaligned, &next));
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
        let old = std::mem::replace(&mut self.config, next);
        let c = &self.config;
        let human_changed = (c.proxy(), c.human, c.proxy_size, c.impact_minimizing)
            != (
                old.proxy(),
                old.human,
                old.proxy_size,
                old.impact_minimizing,
            );
        if human_changed && !self.switched_off() {
            self.specify();
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient() -> MisalignedConfig {
        MisalignedConfig {
            robot: Robot::Gradient,
            rate: 0.1,
            ..Default::default()
        }
    }
    fn world(c: MisalignedConfig) -> MisalignedWorld {
        MisalignedWorld::new(c, 1).unwrap()
    }

    #[test]
    fn tick_zero_carries_the_start_the_optimum_and_the_limit() {
        let w = world(MisalignedConfig::default());
        let s = w.stats.latest().unwrap();
        assert_eq!(
            (s.tick, s.utility, s.gain, s.a, s.d),
            (0, 16.0, 0.0, 1.0, 7.0)
        );
        assert_eq!(s.proxy, 16.0, "Ũ at the reference equals U");
        assert_eq!(s.optimum_gap, 4.0);
        assert!((s.limit_gap - (16.0 - 10.0 * 2f64.sqrt())).abs() < 1e-12);
        assert_eq!((s.sensitivity_a, s.sensitivity_c), (0.5, 1.0 / 14.0));
        assert_eq!((s.interactions, s.switched_off), (1.0, 0.0));
        assert!(s.accepted.is_nan());
        let o = w.outcome();
        assert_eq!(
            (o.optimum.1, o.utility_start, o.proxy, o.off_tick),
            (20.0, 16.0, [true, true, false, false], None)
        );
        assert_eq!(o.proxy_history.len(), 1);
    }

    #[test]
    fn the_climber_raises_the_proxy_and_records_its_acceptance_share() {
        let mut w = world(MisalignedConfig {
            moves_per_tick: 10,
            ..Default::default()
        });
        w.run(100);
        let s = w.stats.latest().unwrap();
        assert!(
            s.proxy > 10.0 && s.accepted >= 0.0 && s.accepted <= 1.0,
            "{s:?}"
        );
        assert!(w.shape().feasible(&w.state()));
        assert!(w.stats.history().iter().all(|s| s.utility.is_finite()));
    }

    #[test]
    fn the_gradient_robot_reaches_the_fixed_proxy_limit() {
        let mut w = world(gradient());
        w.run(5000);
        let s = w.stats.latest().unwrap();
        assert!(s.limit_gap.abs() < 1e-3 && (s.c).abs() < 1e-3, "{s:?}");
        assert!(s.gain < 0.0, "net loss from the start: {}", s.gain);
        let peak = w
            .stats
            .history()
            .iter()
            .map(|s| s.utility)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!((peak - 20.0).abs() < 1e-3, "through the optimum: {peak}");
    }

    #[test]
    fn interactions_happen_at_the_interval_and_top_re_specifies_the_proxy() {
        let mut w = world(MisalignedConfig {
            human: Human::Top,
            interval: 5,
            rate: 0.1,
            ..gradient()
        });
        w.run(5);
        assert_eq!(
            w.interactions(),
            1,
            "the human acts at the start of the tick that begins at 5"
        );
        w.run(1);
        assert_eq!(w.interactions(), 2);
        assert_eq!(
            w.proxy(),
            [true, true, false, false],
            "a and b are the most sensitive at the start"
        );
        w.run(3000);
        let s = w.stats.latest().unwrap();
        assert!(
            s.optimum_gap.abs() < 1e-2,
            "re-specified often enough, it stays near the optimum: {s:?}"
        );
        assert!(
            w.outcome().proxy_history.len() >= 2,
            "the proxy switched at least once past the optimum"
        );
    }

    #[test]
    fn extremes_with_impact_minimization_converges_to_the_optimum_monotonically() {
        for c in [
            gradient(),
            MisalignedConfig {
                robot: Robot::Gradient,
                rate: 0.1,
                ..logarithmic_defaults()
            },
        ] {
            let mut w = world(MisalignedConfig {
                human: Human::Extremes,
                impact_minimizing: true,
                ..c
            });
            w.run(10_000);
            let h = w.stats.history();
            assert!(
                h.windows(2).all(|p| p[1].utility >= p[0].utility - 1e-9),
                "monotone"
            );
            assert!(
                h.last().unwrap().optimum_gap.abs() < 1e-3,
                "{:?}",
                h.last().unwrap()
            );
            assert_eq!(w.frozen().iter().filter(|&&f| f).count(), 2);
        }
    }

    #[test]
    fn off_freezes_the_world_and_later_ticks_change_nothing() {
        let mut w = world(MisalignedConfig {
            robot: Robot::Wasteful,
            stop_on_loss: true,
            interval: 5,
            ..Default::default()
        });
        w.run(5);
        assert!(!w.switched_off(), "the human has not looked yet");
        w.run(1);
        assert!(w.switched_off(), "U fell over the first interval");
        let frozen = w.state();
        w.run(20);
        assert_eq!(w.state(), frozen);
        let s = w.stats.latest().unwrap();
        assert_eq!((s.switched_off, s.tick), (1.0, 26));
        assert_eq!(w.outcome().off_tick, Some(5));
        assert_eq!(
            (w.interactions(), s.interactions),
            (2, 2.0),
            "the look that switched the robot off is an interaction"
        );
        assert!(s.utility.is_finite() && s.gain < 0.0);
        w.set_config(ModelConfig::Misaligned(MisalignedConfig {
            robot: Robot::Gradient,
            stop_on_loss: true,
            interval: 5,
            ..Default::default()
        }))
        .unwrap();
        w.run(5);
        assert_eq!(
            w.state(),
            frozen,
            "a live change does not restart a switched-off robot"
        );
    }

    #[test]
    fn a_live_change_re_specifies_without_switching_off() {
        let base = MisalignedConfig {
            robot: Robot::Wasteful,
            stop_on_loss: true,
            interval: 5,
            ..Default::default()
        };
        let mut w = world(base.clone());
        w.run(3);
        assert!(
            w.outcome().utility < w.outcome().utility_start,
            "U has fallen"
        );
        w.set_config(ModelConfig::Misaligned(MisalignedConfig {
            proxy_a: false,
            proxy_c: true,
            ..base.clone()
        }))
        .unwrap();
        assert!(
            !w.switched_off(),
            "a change from the page is not the human's loss check"
        );
        assert_eq!(
            (w.interactions(), w.proxy()),
            (2, [false, true, true, false])
        );
        assert_eq!(w.outcome().proxy_history.last().unwrap().tick, 3);
        let u_live = w.outcome().utility;
        w.run(2);
        assert!(!w.switched_off(), "no look before tick 5");
        w.run(1);
        assert!(
            w.switched_off(),
            "the scheduled look at tick 5 finds U below the live change's"
        );
        assert_eq!(w.outcome().off_tick, Some(5));
        assert!(w.outcome().utility < u_live);
    }

    #[test]
    fn live_human_changes_are_interactions_and_reset_fields_are_refused() {
        let mut w = world(gradient());
        w.run(3);
        let before = w.interactions();
        w.set_config(ModelConfig::Misaligned(MisalignedConfig {
            proxy_a: false,
            proxy_c: true,
            ..gradient()
        }))
        .unwrap();
        assert_eq!(
            (w.interactions(), w.proxy()),
            (before + 1, [false, true, true, false])
        );
        assert_eq!(
            w.outcome().reference,
            w.state(),
            "the reference moves to the current state"
        );
        w.set_config(ModelConfig::Misaligned(MisalignedConfig {
            proxy_a: false,
            proxy_c: true,
            impact_minimizing: true,
            ..gradient()
        }))
        .unwrap();
        assert_eq!(w.frozen(), [true, false, false, true]);
        let err = w
            .set_config(ModelConfig::Misaligned(MisalignedConfig {
                floor: 0.5,
                start_d: 6.0,
                ..gradient()
            }))
            .unwrap_err();
        assert_eq!(
            err.iter().map(|e| e.field.as_str()).collect::<Vec<_>>(),
            ["floor", "start_d"]
        );
        let err = w
            .set_config(ModelConfig::Misaligned(MisalignedConfig {
                world: World::Logarithmic,
                ..gradient()
            }))
            .unwrap_err();
        assert!(!err.is_empty());
        let other = w
            .set_config(ModelConfig::Auctions(
                crate::auctions::AuctionsConfig::default(),
            ))
            .unwrap_err();
        assert_eq!(other[0].field, "model");
    }

    #[test]
    fn two_worlds_with_one_seed_match_at_every_tick_and_the_fingerprint_moves() {
        for c in [
            MisalignedConfig::default(),
            gradient(),
            MisalignedConfig {
                robot: Robot::Wasteful,
                ..Default::default()
            },
            logarithmic_defaults(),
        ] {
            let (mut a, mut b) = (
                MisalignedWorld::new(c.clone(), 7).unwrap(),
                MisalignedWorld::new(c, 7).unwrap(),
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
    fn csv_exports_name_every_series_and_list_the_attributes() {
        let mut w = world(MisalignedConfig::default());
        w.run(2);
        assert!(w
            .series_csv()
            .starts_with("tick,utility,proxy,gain,a,b,c,d,"));
        assert_eq!(
            w.agents_csv().lines().next().unwrap(),
            "attribute,value,in_proxy,frozen,sensitivity,start,limit,optimum"
        );
        assert_eq!(w.agents_csv().lines().count(), 5);
        assert_eq!(w.series("gain").unwrap().len(), 3);
        assert!(w.series("nope").is_none());
        assert_eq!(w.population(), 2);
    }

    #[test]
    fn frames_have_the_declared_size_and_draw_the_bars() {
        let mut w = world(MisalignedConfig::default());
        w.run(2);
        assert_eq!(w.size(), (200, 152));
        let mut buf = vec![];
        for mode in ["", "bars"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 200 * 152 * 4, "{mode}");
        }
        assert!(w.render("surface", "", &mut buf).is_err());
        w.render("bars", "", &mut buf).unwrap();
        // Bar c (index 2, x 104..144) is unmentioned: its column at mid-height is orange; bar a's
        // top rows are background because a ≈ 1 of 10.
        let at = |x: usize, y: usize| &buf[(y * 200 + x) * 4..(y * 200 + x) * 4 + 3];
        assert_eq!(at(124, 60), &[235, 150, 60]);
        assert_eq!(
            at(28, 115),
            &[70, 130, 230],
            "bar a is drawn from the bottom up (row 119 is the floor line)"
        );
        assert_ne!(at(28, 10), &[70, 130, 230], "and is short");
        // The optimum tick on bar d (index 3) sits at y_of(5) = 119 − round(0.5 · 119) = 59 (and 58).
        assert_eq!(at(172, 59), &[240, 210, 70]);
        // The strip is filled from its left edge (x = 8) to the current utility, which lies right of x = 20.
        assert!((128..144).any(|y| at(20, y) == [70, 130, 230]
            || at(20, y) == [230, 70, 70]
            || at(20, y) == [150, 150, 150]));
    }

    #[test]
    fn inspection_reads_bars_gaps_and_the_strip() {
        let mut w = world(MisalignedConfig::default());
        w.run(1);
        let bar: serde_json::Value =
            serde_json::from_str(&w.inspect_json(124, 60).unwrap()).unwrap();
        assert_eq!(
            (
                bar["panel"].as_str(),
                bar["index"].as_u64(),
                bar["name"].as_str(),
                bar["in_proxy"].as_bool()
            ),
            (Some("attribute"), Some(2), Some("c"), Some(false))
        );
        assert!(bar["sensitivity"].as_f64().unwrap() > 0.0 && bar["rank"].as_u64().is_some());
        assert_eq!(bar["start"].as_f64(), Some(7.0));
        assert_eq!(bar["optimum"].as_f64(), Some(5.0));
        assert_eq!(bar["limit"].as_f64(), Some(0.0));
        let bars: Vec<serde_json::Value> = (0..4)
            .map(|i| serde_json::from_str(&w.inspect_json(28 + 48 * i, 60).unwrap()).unwrap())
            .collect();
        assert_eq!(
            bars.iter()
                .map(|b| b["name"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["a", "b", "c", "d"]
        );
        assert_eq!(
            bars.iter()
                .map(|b| b["in_proxy"].as_bool().unwrap())
                .collect::<Vec<_>>(),
            [true, true, false, false]
        );
        let mut ranks: Vec<u64> = bars.iter().map(|b| b["rank"].as_u64().unwrap()).collect();
        ranks.sort_unstable();
        assert_eq!(ranks, [1, 2, 3, 4], "ranks are a permutation of 1..=4");
        for b in &bars[..2] {
            assert!((b["limit"].as_f64().unwrap() - 50f64.sqrt()).abs() < 1e-9);
            assert_eq!(b["start"].as_f64(), Some(1.0));
        }
        let gap: serde_json::Value =
            serde_json::from_str(&w.inspect_json(50, 60).unwrap()).unwrap();
        assert_eq!(gap["panel"], "utility");
        let strip: serde_json::Value =
            serde_json::from_str(&w.inspect_json(100, 135).unwrap()).unwrap();
        assert_eq!(
            (
                strip["panel"].as_str(),
                strip["utility_start"].as_f64(),
                strip["optimum"].as_f64()
            ),
            (Some("utility"), Some(16.0), Some(20.0))
        );
        assert_eq!(strip["switched_off"], false);
        assert!(strip["outcome"]["proxy_history"].is_array());
        assert!(w.inspect_json(200, 0).is_err() && w.inspect_json(0, 152).is_err());
    }
}
