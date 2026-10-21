//! The hybrid world: Booker's, Guo's or Si et al.'s population, or a coloring session, stepped
//! one paper step per tick, with the exact reference for the current cell (Booker's all-C weight
//! for these N and M, Guo's highest stable master-equation root at this y, the coloring graph's
//! solution count). `Runner` is the same dynamics without the page's history; the survey drives it.

use super::{
    booker::{self, Donation},
    coloring::{self, Session},
    config::*,
    engine::Population,
    graph::{bernoulli, erdos_renyi, lattice, place},
    guo::{self, Plane},
    si,
    stats::{HybridSnapshot, SERIES},
    view,
};
use crate::{
    config::FieldError,
    graph::{barabasi_albert, Graph},
    model::{wrong_model, Model, ModelConfig, ModelKind},
    rng::{self, SimRng},
    stats::{Series, Stats},
};
use rand::Rng;
use std::fmt::Write;

/// What a world runs: a population of humans and bots, or a coloring session.
#[derive(Clone, Debug)]
pub enum Body {
    Population(Population),
    Coloring(Session),
}

/// The dynamics without history.
#[derive(Clone, Debug)]
pub struct Runner {
    pub config: HybridConfig,
    pub body: Body,
    /// Paper steps taken.
    pub steps: u64,
    rng: SimRng,
}

pub(crate) fn booker_rules(c: &HybridConfig) -> booker::Rules {
    booker::Rules {
        game: Donation { b: c.b, c: c.c },
        kind: c.bot_kind,
        beta: c.beta,
        accumulate: c.accumulate,
    }
}

pub(crate) fn guo_rules(c: &HybridConfig) -> guo::Rules {
    guo::Rules {
        plane: Plane { t: c.t, s: c.s },
        phi: c.phi,
        beta: c.beta,
        bot_action: c.bot_action,
        accumulate: c.accumulate,
    }
}

pub(crate) fn si_rules(c: &HybridConfig) -> si::Rules {
    si::Rules {
        r: c.r,
        framework: c.framework,
        mixed_draw: c.mixed_draw,
        division: c.division,
        beta: c.beta,
        accumulate: c.accumulate,
    }
}

pub(crate) fn coloring_rules(c: &HybridConfig) -> coloring::Rules {
    coloring::Rules {
        noise: c.noise,
        errant: c.errant,
        reaction: c.reaction,
        bot_interval: c.bot_interval,
        time_limit: c.time_limit,
    }
}

/// The graph (None when well-mixed), drawn from `graph_seed`'s own stream when it is set and
/// from the run's stream otherwise.
fn graph_of(c: &HybridConfig, rng: &mut SimRng) -> Option<Vec<Vec<u32>>> {
    let mut own = c.graph_seed.map(rng::seeded);
    let g = match own.as_mut() {
        Some(r) => r,
        None => &mut *rng,
    };
    Some(match c.structure {
        Structure::WellMixed => return None,
        Structure::Lattice => lattice(c.side as usize, c.neighborhood, c.periodic),
        Structure::ErdosRenyi => erdos_renyi(c.nodes as usize, c.mean_degree, g),
        Structure::BarabasiAlbert => barabasi_albert(c.nodes as usize, c.links as usize, g),
    })
}

/// The bots: a Bernoulli draw per node, or a placed count (on degrees; all equal well-mixed).
fn bots_of(c: &HybridConfig, adj: Option<&Vec<Vec<u32>>>, rng: &mut SimRng) -> Vec<bool> {
    let n = c.size();
    match c.bot_total() {
        None => bernoulli(n, c.bot_fraction, rng),
        Some(count) => {
            let degrees: Vec<usize> = match adj {
                Some(a) => a.iter().map(Vec::len).collect(),
                None => vec![n - 1; n],
            };
            place(&degrees, count, c.placement, rng)
        }
    }
}

impl Runner {
    pub fn new(config: HybridConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        config.validate()?;
        let mut rng = rng::seeded(seed);
        let adj = graph_of(&config, &mut rng);
        let bot = bots_of(&config, adj.as_ref(), &mut rng);
        let body = if config.world == World::Coloring {
            let adj = adj.expect("the coloring game has a graph");
            assert!(
                coloring::colorings(&adj, 3) > 0,
                "a Barabási–Albert graph with m ≤ 2 is 3-colorable"
            );
            Body::Coloring(Session::new(adj, bot, coloring_rules(&config), &mut rng))
        } else {
            let mut pop = Population::new(adj.map(Graph::from_lists).unwrap_or_default(), bot);
            match config.world {
                World::Booker => booker::init(
                    &mut pop,
                    config.bot_kind,
                    config.initial_cooperation,
                    &mut rng,
                ),
                World::Guo => guo::init(&mut pop, config.phi, config.initial_cooperation, &mut rng),
                World::Si => si::init(&mut pop, config.framework, config.theta, &mut rng),
                World::Coloring => unreachable!(),
            }
            Body::Population(pop)
        };
        Ok(Runner {
            config,
            body,
            steps: 0,
            rng,
        })
    }

    /// One paper step (`StepUnit`).
    pub fn step(&mut self) {
        let c = &self.config;
        match &mut self.body {
            Body::Population(pop) => {
                let n = pop.nodes;
                let updates = if c.step == StepUnit::Sweep { n } else { 1 };
                match c.world {
                    World::Booker => {
                        let r = booker_rules(c);
                        for _ in 0..updates {
                            booker::micro(&r, pop, &mut self.rng);
                        }
                    }
                    World::Guo if pop.well_mixed() => {
                        let r = guo_rules(c);
                        for _ in 0..updates {
                            guo::micro_well_mixed(&r, pop, &mut self.rng);
                        }
                    }
                    World::Guo => {
                        let r = guo_rules(c);
                        for _ in 0..n {
                            guo::micro_network(&r, pop, &mut self.rng);
                        }
                    }
                    World::Si => si::round(&si_rules(c), pop, &mut self.rng),
                    World::Coloring => unreachable!(),
                }
            }
            Body::Coloring(s) => s.advance_to((self.steps + 1) as f64, &mut self.rng),
        }
        self.steps += 1;
    }

    /// The humans' cooperation now (NaN for coloring).
    pub fn cooperation(&self) -> f64 {
        match &self.body {
            Body::Population(p) => p.human_cooperation(),
            Body::Coloring(_) => f64::NAN,
        }
    }

    /// Runs `steps` steps and returns the mean human cooperation over the last `window`, read
    /// after each step.
    pub fn mean_over(&mut self, steps: u64, window: u64) -> f64 {
        assert!(
            window >= 1 && window <= steps,
            "the window lies inside the run"
        );
        let mut sum = 0.0;
        for t in 0..steps {
            self.step();
            if t >= steps - window {
                sum += self.cooperation();
            }
        }
        sum / window as f64
    }

    pub fn finished(&self) -> bool {
        matches!(&self.body, Body::Coloring(s) if s.finished())
    }

    pub fn rng(&self) -> &SimRng {
        &self.rng
    }
}

#[derive(Clone)]
pub struct HybridWorld {
    pub config: HybridConfig,
    pub tick: u64,
    pub stats: Stats<HybridSnapshot>,
    seed: u64,
    runner: Runner,
    reference: f64,
}

/// A number's bytes for the fingerprint, with every NaN the same NaN.
fn bits(x: f64) -> [u8; 8] {
    (if x.is_nan() { f64::NAN } else { x })
        .to_bits()
        .to_le_bytes()
}

impl HybridWorld {
    pub fn new(config: HybridConfig, seed: u64) -> Result<Self, Vec<FieldError>> {
        let runner = Runner::new(config.clone(), seed)?;
        let mut w = HybridWorld {
            config,
            tick: 0,
            stats: Stats::default(),
            seed,
            runner,
            reference: f64::NAN,
        };
        w.reference = w.compute_reference();
        let s = w.snapshot();
        w.stats.push(s);
        Ok(w)
    }

    pub fn runner(&self) -> &Runner {
        &self.runner
    }

    pub fn reference(&self) -> f64 {
        self.reference
    }

    /// Booker: the all-C weight of the chain with these humans and bots; Guo: the highest stable
    /// master-equation root at y = bots/humans; coloring: the number of proper 3-colorings; Si:
    /// none (NaN).
    fn compute_reference(&self) -> f64 {
        let c = &self.config;
        match &self.runner.body {
            Body::Population(p) => {
                let humans = p.humans.len() as u32;
                let bots = p.nodes as u32 - humans;
                match c.world {
                    World::Booker if humans >= 2 => booker::Chain {
                        n: humans,
                        m: bots,
                        game: Donation { b: c.b, c: c.c },
                        kind: c.bot_kind,
                        beta: c.beta,
                    }
                    .all_c_weight(),
                    World::Guo if humans > 0 => {
                        let y = f64::from(bots) / f64::from(humans);
                        guo::master_roots(&Plane { t: c.t, s: c.s }, y, c.phi, c.beta)
                            .iter()
                            .filter(|r| r.stable)
                            .map(|r| r.x)
                            .fold(f64::NAN, f64::max)
                    }
                    _ => f64::NAN,
                }
            }
            Body::Coloring(s) => coloring::colorings(&s.adj, 3) as f64,
        }
    }

    /// Node i's payoff from the stored actions (Inspect, the agents CSV); NaN in coloring.
    pub fn payoff_now(&self, i: usize) -> f64 {
        let c = &self.config;
        match &self.runner.body {
            Body::Population(p) => match c.world {
                World::Booker => booker::payoff(&booker_rules(c), p, i),
                World::Guo => guo::payoff_stored(&guo_rules(c), p, i),
                World::Si => si::payoff_stored(&si_rules(c), p, i),
                World::Coloring => f64::NAN,
            },
            Body::Coloring(_) => f64::NAN,
        }
    }

    pub fn step(&mut self) {
        self.runner.step();
        self.tick += 1;
        let s = self.snapshot();
        self.stats.push(s);
    }

    /// Runs `ticks` ticks, stopping once a coloring session has finished.
    pub fn run(&mut self, ticks: u32) {
        for _ in 0..ticks {
            if self.runner.finished() {
                break;
            }
            self.step();
        }
    }

    /// The bots' cooperating share; under Si's continuous framework, which never uses actions
    /// (the bots' are a sample drawn once), their mean strategy θ. NaN without bots.
    fn bot_cooperation(&self, p: &Population) -> f64 {
        if self.config.world != World::Si || self.config.framework != Framework::Continuous {
            return p.bot_cooperation();
        }
        let bots = p.nodes - p.humans.len();
        if bots == 0 {
            return f64::NAN;
        }
        let sum: f64 = (0..p.nodes)
            .filter(|&i| p.bot[i])
            .map(|i| p.strategy[i])
            .sum();
        sum / bots as f64
    }

    fn snapshot(&self) -> HybridSnapshot {
        match &self.runner.body {
            Body::Population(p) => HybridSnapshot {
                tick: self.tick,
                cooperation: p.human_cooperation(),
                bot_cooperation: self.bot_cooperation(p),
                bot_share: (p.nodes - p.humans.len()) as f64 / p.nodes as f64,
                reference: self.reference,
                ..Default::default()
            },
            Body::Coloring(s) => HybridSnapshot {
                tick: self.tick,
                bot_share: s.bot.iter().filter(|&&b| b).count() as f64 / s.bot.len() as f64,
                reference: self.reference,
                conflicts: s.conflicts as f64,
                solved_at: s.solved_at.unwrap_or(f64::NAN),
                game_time: s.time,
                ..Default::default()
            },
        }
    }
}

impl Model for HybridWorld {
    fn config(&self) -> ModelConfig {
        ModelConfig::Hybrid(self.config.clone())
    }
    fn run(&mut self, ticks: u32) {
        HybridWorld::run(self, ticks)
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn population(&self) -> usize {
        self.config.size()
    }
    fn fingerprint(&self) -> u64 {
        let state = serde_json::json!({
            "config": self.config, "seed": self.seed, "tick": self.tick, "steps": self.runner.steps,
        });
        let mut bytes = serde_json::to_vec(&state).unwrap();
        match &self.runner.body {
            Body::Population(p) => {
                for i in 0..p.nodes {
                    bytes.extend(bits(p.strategy[i]));
                    bytes.push(u8::from(p.action[i]));
                }
                bytes.extend((p.cooperating as u64).to_le_bytes());
            }
            Body::Coloring(s) => {
                bytes.extend(&s.color);
                for &t in &s.next {
                    bytes.extend(bits(t));
                }
                bytes.extend(bits(s.time));
                bytes.extend((s.conflicts as u64).to_le_bytes());
            }
        }
        bytes.extend(bits(self.reference));
        if let Some(s) = self.stats.latest() {
            for name in SERIES {
                bytes.extend(bits(s.value(name).unwrap()));
            }
        }
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for byte in bytes {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        let mut stream = self.runner.rng().clone();
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
        match &self.runner.body {
            Body::Population(p) => {
                out.push_str("node,x,y,bot,strategy,action,degree,payoff\n");
                for i in 0..p.nodes {
                    let (x, y) = view::position(self, i);
                    let action = if p.action[i] { "C" } else { "D" };
                    writeln!(
                        out,
                        "{i},{x},{y},{},{},{action},{},{}",
                        p.bot[i],
                        p.strategy[i],
                        p.degree(i),
                        self.payoff_now(i)
                    )
                    .unwrap();
                }
            }
            Body::Coloring(s) => {
                out.push_str("node,x,y,bot,color,degree\n");
                for i in 0..s.color.len() {
                    let (x, y) = view::position(self, i);
                    writeln!(
                        out,
                        "{i},{x},{y},{},{},{}",
                        s.bot[i],
                        s.color[i],
                        s.adj[i].len()
                    )
                    .unwrap();
                }
            }
        }
        out
    }
    fn inspect_json(&self, x: u32, y: u32) -> Result<String, String> {
        view::inspect(self, x, y)
    }
    fn locate(&self, id: u64) -> Option<(u32, u32)> {
        (id < self.config.size() as u64).then(|| view::position(self, id as usize))
    }
    fn set_config(&mut self, next: ModelConfig) -> Result<(), Vec<FieldError>> {
        let ModelConfig::Hybrid(next) = next else {
            return Err(wrong_model(ModelKind::Hybrid, &next));
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
        self.config = next.clone();
        self.runner.config = next;
        if let Body::Coloring(s) = &mut self.runner.body {
            s.rules = coloring_rules(&self.config);
        }
        self.reference = self.compute_reference();
        Ok(())
    }
    fn finished(&self) -> bool {
        self.runner.finished()
    }
    fn holds_when_finished(&self) -> bool {
        self.config.world == World::Coloring
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world(c: HybridConfig) -> HybridWorld {
        HybridWorld::new(c, 1).unwrap()
    }

    fn actions(w: &HybridWorld) -> Vec<bool> {
        match &w.runner().body {
            Body::Population(p) => p.action.clone(),
            Body::Coloring(_) => unreachable!(),
        }
    }

    #[test]
    fn tick_zero_carries_the_population_and_the_reference() {
        let w = world(booker_defaults());
        let s = w.stats.latest().unwrap();
        assert_eq!(s.tick, 0);
        assert!((s.cooperation - 0.5).abs() < 0.05, "{s:?}");
        assert_eq!(
            (s.bot_share, s.bot_cooperation),
            (0.1, 1.0),
            "250 Samaritans"
        );
        let want = booker::Chain {
            n: 2250,
            m: 250,
            game: Donation { b: 2.0, c: 1.0 },
            kind: BotKind::Samaritan,
            beta: 0.1,
        }
        .all_c_weight();
        assert_eq!(s.reference, want);
        assert!(s.conflicts.is_nan() && s.solved_at.is_nan() && s.game_time.is_nan());
        let g = world(HybridConfig {
            side: 20,
            bot_count: 100,
            ..guo_defaults()
        });
        let stable = guo::master_roots(&Plane { t: 1.4, s: 0.5 }, 100.0 / 300.0, 0.5, 10.0);
        assert_eq!(
            g.reference(),
            stable
                .iter()
                .filter(|r| r.stable)
                .map(|r| r.x)
                .fold(f64::NAN, f64::max)
        );
        assert!(world(HybridConfig {
            side: 10,
            ..si_defaults()
        })
        .reference()
        .is_nan());
    }

    #[test]
    fn a_tick_is_the_papers_step() {
        let mut b = world(booker_defaults());
        let before = actions(&b);
        b.run(1);
        let changed = actions(&b)
            .iter()
            .zip(&before)
            .filter(|(x, y)| x != y)
            .count();
        assert!(changed <= 1, "Booker's step is one update: {changed}");
        assert_eq!((b.tick, b.runner().steps), (1, 1));
        let mut sweep = world(HybridConfig {
            step: StepUnit::Sweep,
            ..booker_defaults()
        });
        let before = actions(&sweep);
        sweep.run(1);
        assert!(
            actions(&sweep)
                .iter()
                .zip(&before)
                .filter(|(x, y)| x != y)
                .count()
                > 1,
            "N updates"
        );
        let mut alg1 = world(HybridConfig {
            structure: Structure::WellMixed,
            nodes: 1000,
            bot_count: 487,
            ..guo_defaults()
        });
        let before = actions(&alg1);
        alg1.run(1);
        assert!(
            actions(&alg1)
                .iter()
                .zip(&before)
                .filter(|(x, y)| x != y)
                .count()
                <= 1,
            "Algorithm I: one comparison"
        );
        let mut alg2 = world(HybridConfig {
            side: 20,
            bot_count: 100,
            ..guo_defaults()
        });
        let before = actions(&alg2);
        alg2.run(1);
        assert!(
            actions(&alg2)
                .iter()
                .zip(&before)
                .filter(|(x, y)| x != y)
                .count()
                > 1,
            "Algorithm II: N updates"
        );
    }

    #[test]
    fn a_finished_coloring_world_holds() {
        let mut w = world(coloring_defaults());
        w.run(1);
        let s = w.stats.latest().unwrap();
        assert!(s.game_time <= 1.0 && s.cooperation.is_nan() && s.reference >= 6.0);
        w.run(400);
        assert!(w.finished() && w.holds_when_finished());
        let (tick, fp) = (w.tick, w.fingerprint());
        w.run(50);
        assert_eq!(
            (w.tick, w.fingerprint()),
            (tick, fp),
            "a finished session runs no more ticks"
        );
        let s = w.stats.latest().unwrap();
        assert!(
            s.game_time <= 300.0 && (s.solved_at.is_nan() || s.conflicts == 0.0),
            "{s:?}"
        );
    }

    #[test]
    fn runner_means_read_after_every_step() {
        let c = HybridConfig {
            side: 10,
            ..booker_defaults()
        };
        let mut a = Runner::new(c.clone(), 3).unwrap();
        let mean = a.mean_over(200, 50);
        let mut b = Runner::new(c, 3).unwrap();
        let mut sum = 0.0;
        for t in 0..200 {
            b.step();
            if t >= 150 {
                sum += b.cooperation();
            }
        }
        assert_eq!(mean, sum / 50.0);
    }

    #[test]
    fn the_same_seed_gives_the_same_world_and_graph_seed_fixes_the_graph() {
        let c = HybridConfig {
            side: 20,
            ..si_defaults()
        };
        let (mut a, mut b) = (world(c.clone()), world(c.clone()));
        a.run(3);
        b.run(3);
        assert_eq!(a.fingerprint(), b.fingerprint());
        let mut other = HybridWorld::new(c, 2).unwrap();
        other.run(3);
        assert_ne!(a.fingerprint(), other.fingerprint());
        let ba = HybridConfig {
            structure: Structure::BarabasiAlbert,
            nodes: 500,
            bot_count: 100,
            graph_seed: Some(7),
            ..guo_defaults()
        };
        let graph = |c: &HybridConfig, seed| match Runner::new(c.clone(), seed).unwrap().body {
            Body::Population(p) => p.graph,
            Body::Coloring(_) => unreachable!(),
        };
        assert_eq!(graph(&ba, 1), graph(&ba, 2));
        let fresh = HybridConfig {
            graph_seed: None,
            ..ba
        };
        assert_ne!(graph(&fresh, 1), graph(&fresh, 2));
    }

    #[test]
    fn a_world_without_humans_runs_and_fingerprints() {
        let c = HybridConfig {
            structure: Structure::WellMixed,
            nodes: 5,
            bot_fraction: 1.0,
            ..si_defaults()
        };
        let (mut w, mut v) = (world(c.clone()), world(c));
        w.run(5);
        v.run(5);
        assert!(w.stats.latest().unwrap().cooperation.is_nan());
        assert_eq!(w.fingerprint(), v.fingerprint());
        let mut none = world(HybridConfig {
            bots: Bots::Count,
            bot_count: 0,
            side: 10,
            ..booker_defaults()
        });
        none.run(5);
        assert!(none.stats.latest().unwrap().bot_cooperation.is_nan());
    }

    #[test]
    fn game_parameters_apply_live_and_structure_is_refused() {
        let mut w = world(booker_defaults());
        let before = w.reference();
        w.set_config(ModelConfig::Hybrid(HybridConfig {
            beta: 1.0,
            ..booker_defaults()
        }))
        .unwrap();
        assert_eq!(w.config.beta, 1.0);
        assert_ne!(w.reference(), before);
        let err = w
            .set_config(ModelConfig::Hybrid(HybridConfig {
                side: 40,
                ..booker_defaults()
            }))
            .unwrap_err();
        assert_eq!(
            (err[0].field.as_str(), err[0].message.as_str()),
            ("side", "changes only on reset")
        );
        let mut c = world(coloring_defaults());
        c.set_config(ModelConfig::Hybrid(HybridConfig {
            noise: 0.3,
            ..coloring_defaults()
        }))
        .unwrap();
        match &c.runner().body {
            Body::Coloring(s) => assert_eq!(s.rules.noise, 0.3),
            Body::Population(_) => unreachable!(),
        }
    }

    #[test]
    fn continuous_bots_report_their_strategy() {
        // Si's continuous framework never uses actions, so the bots' drawn actions are a frozen
        // sample; their cooperation is their strategy θ.
        let mut w = world(HybridConfig {
            side: 20,
            framework: Framework::Continuous,
            theta: 0.6173,
            ..si_defaults()
        });
        assert!((w.stats.latest().unwrap().bot_cooperation - 0.6173).abs() < 1e-12);
        w.run(3);
        let s = w.stats.latest().unwrap();
        assert!((s.bot_cooperation - 0.6173).abs() < 1e-12, "{s:?}");
    }
}
