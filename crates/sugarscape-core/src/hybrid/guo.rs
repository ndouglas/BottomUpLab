//! Guo et al. (iScience 2023; arXiv 2307.01123 and its supplement's source): games with R = 1,
//! P = 0 on the (T, S) plane, bots that cooperate with probability φ, and y bots per human. The
//! replicator dynamics (Eqs 3–5, Theorems 1–3) and the pairwise-comparison master equation
//! (Eqs 6–9) are exact references; Algorithms I and II follow in this file's second part.

use super::engine::{accumulate, fermi, Accumulate, Population};
use super::exact::{logistic, roots, Root};
use crate::rng::SimRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// A game with R = 1 and P = 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plane {
    pub t: f64,
    pub s: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    FullC,
    FullD,
    Coexistence,
    Bistable,
}

impl Plane {
    /// The row player's payoff (true = cooperate).
    pub fn payoff(&self, me: bool, other: bool) -> f64 {
        match (me, other) {
            (true, true) => 1.0,
            (true, false) => self.s,
            (false, true) => self.t,
            (false, false) => 0.0,
        }
    }

    /// ρ̂ = (P − S)/(R − T − S + P), the interior rest point without bots.
    pub fn rho_hat(&self) -> f64 {
        -self.s / (1.0 - self.t - self.s)
    }

    /// z = (ρ + yφ)/(1 + y), the share of all players cooperating.
    pub fn z(rho: f64, y: f64, phi: f64) -> f64 {
        (rho + y * phi) / (1.0 + y)
    }

    /// π_C − π_D = z(R − T − S + P) + S − P (Eq 4).
    pub fn gap(&self, rho: f64, y: f64, phi: f64) -> f64 {
        Self::z(rho, y, phi) * (1.0 - self.t - self.s) + self.s
    }

    /// ρ̇ under replicator dynamics (Eq 3): ρ(1 − ρ)(π_C − π_D)/(1 + y).
    pub fn replicator(&self, rho: f64, y: f64, phi: f64) -> f64 {
        rho * (1.0 - rho) * self.gap(rho, y, phi) / (1.0 + y)
    }

    /// ρ̇ = Q⁺ − Q⁻ under pairwise comparison with imitation strength K (Eqs 6–9): the reference
    /// is drawn from all 1 + y units, bots included, credited with their action's payoff.
    pub fn master(&self, rho: f64, y: f64, phi: f64, k: f64) -> f64 {
        let g = self.gap(rho, y, phi);
        let p1 = rho * (1.0 - rho + y * (1.0 - phi)) / (1.0 + y);
        let p2 = (rho + y * phi) * (1.0 - rho) / (1.0 + y);
        p2 * logistic(k * g) - p1 * logistic(-k * g)
    }

    /// T > R, S > P.
    pub fn snowdrift(&self) -> bool {
        self.t > 1.0 && self.s > 0.0
    }

    /// R > T, P > S.
    pub fn stag_hunt(&self) -> bool {
        self.t < 1.0 && self.s < 0.0
    }

    /// Theorems 1–3. z runs from yφ/(1 + y) (no human cooperates) to (1 + yφ)/(1 + y).
    pub fn phase_closed(&self, y: f64, phi: f64) -> Phase {
        let lo = y * phi / (1.0 + y);
        let hi = (1.0 + y * phi) / (1.0 + y);
        let h = self.rho_hat();
        if self.snowdrift() {
            if hi <= h {
                Phase::FullC
            } else if lo >= h {
                Phase::FullD
            } else {
                Phase::Coexistence
            }
        } else if self.stag_hunt() {
            if lo >= h {
                Phase::FullC
            } else if hi <= h {
                Phase::FullD
            } else {
                Phase::Bistable
            }
        } else if self.t > 1.0 {
            Phase::FullD
        } else {
            Phase::FullC
        }
    }

    /// The distance in z from the nearer closed-form boundary; G1 compares only where it exceeds
    /// 1e-9.
    pub fn boundary_distance(&self, y: f64, phi: f64) -> f64 {
        let h = self.rho_hat();
        (y * phi / (1.0 + y) - h)
            .abs()
            .min(((1.0 + y * phi) / (1.0 + y) - h).abs())
    }

    /// The fewest bots for full cooperation in snowdrift, at φ = 0: (T − R)/(S − P).
    pub fn y_min(&self) -> f64 {
        (self.t - 1.0) / self.s
    }
}

/// The phase read off the numerical roots of Eq 3 on [0, 1] (1,000 cells); None if the stable set
/// is none of the four.
pub fn phase_numeric(plane: &Plane, y: f64, phi: f64) -> Option<Phase> {
    let stable: Vec<f64> = roots(|r| plane.replicator(r, y, phi), 0.0, 1.0, 1000)
        .into_iter()
        .filter(|r| r.stable)
        .map(|r| r.x)
        .collect();
    match stable.as_slice() {
        [x] if *x == 1.0 => Some(Phase::FullC),
        [x] if *x == 0.0 => Some(Phase::FullD),
        [x] if *x > 0.0 && *x < 1.0 => Some(Phase::Coexistence),
        [a, b] if *a == 0.0 && *b == 1.0 => Some(Phase::Bistable),
        _ => None,
    }
}

/// The master equation's roots on [0, 1], 10,000 cells.
pub fn master_roots(plane: &Plane, y: f64, phi: f64, k: f64) -> Vec<Root> {
    roots(|r| plane.master(r, y, phi, k), 0.0, 1.0, 10_000)
}

/// How a bot's action is drawn on a graph (amendment 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BotAction {
    /// Algorithm II: a bot chosen as focal plays C with probability φ from then on.
    Redraw,
    /// The initial draw, kept.
    Frozen,
    /// A fresh draw for every game and every imitation.
    PerInteraction,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub plane: Plane,
    pub phi: f64,
    pub beta: f64,
    pub bot_action: BotAction,
    pub accumulate: Accumulate,
}

/// In index order: a bot cooperates with probability φ, a human with probability `p` (½ in
/// Algorithm II, the initial frequency p in Algorithm I).
pub fn init(pop: &mut Population, phi: f64, p: f64, rng: &mut SimRng) {
    for i in 0..pop.nodes {
        let c = rng.gen::<f64>() < if pop.bot[i] { phi } else { p };
        pop.set_pure(i, c);
    }
}

fn act(rules: &Rules, pop: &Population, k: usize, rng: &mut SimRng) -> bool {
    if pop.bot[k] && rules.bot_action == BotAction::PerInteraction {
        rng.gen::<f64>() < rules.phi
    } else {
        pop.action[k]
    }
}

/// Node i's payoff on a graph (Algorithm II): the game against every neighbor, summed or
/// averaged; under `per_interaction` every bot's action in every game is a fresh draw.
pub fn payoff(rules: &Rules, pop: &Population, i: usize, rng: &mut SimRng) -> f64 {
    let mut total = 0.0;
    for &j in pop.neighbors(i) {
        let a = act(rules, pop, i, rng);
        let b = act(rules, pop, j as usize, rng);
        total += rules.plane.payoff(a, b);
    }
    accumulate(total, pop.degree(i), rules.accumulate)
}

/// Node i's payoff from the stored actions (inspection): on a graph against its neighbors,
/// well-mixed against everyone else.
pub fn payoff_stored(rules: &Rules, pop: &Population, i: usize) -> f64 {
    if pop.well_mixed() {
        let a = pop.action[i];
        return expected(
            &rules.plane,
            a,
            pop.cooperating - usize::from(a),
            pop.nodes - 1,
        );
    }
    let total: f64 = pop
        .neighbors(i)
        .iter()
        .map(|&j| rules.plane.payoff(pop.action[i], pop.action[j as usize]))
        .sum();
    accumulate(total, pop.degree(i), rules.accumulate)
}

/// Algorithm II's inner step: a uniformly random node; a bot redraws its action under `redraw`
/// and does nothing otherwise; a human plays its neighbors, picks a uniformly random neighbor
/// and adopts its action with the Fermi probability (copying a `per_interaction` bot adopts a
/// fresh draw). An isolated human does nothing.
pub fn micro_network(rules: &Rules, pop: &mut Population, rng: &mut SimRng) {
    let i = pop.random_node(rng);
    if pop.bot[i] {
        if rules.bot_action == BotAction::Redraw {
            let c = rng.gen::<f64>() < rules.phi;
            pop.set_pure(i, c);
        }
        return;
    }
    let Some(j) = pop.partner(i, rng) else {
        return;
    };
    let p = fermi(
        rules.beta,
        payoff(rules, pop, i, rng),
        payoff(rules, pop, j, rng),
    );
    if rng.gen::<f64>() < p {
        let a = act(rules, pop, j, rng);
        pop.set_pure(i, a);
    }
}

/// The expected payoff of an action against the other `others` players, `c` of them
/// cooperating.
pub fn expected(plane: &Plane, cooperate: bool, c: usize, others: usize) -> f64 {
    let (c, o) = (c as f64, others as f64);
    (c * plane.payoff(cooperate, true) + (o - c) * plane.payoff(cooperate, false)) / o
}

/// Algorithm I's step: a uniformly random human and a uniformly random other player; if their
/// actions differ the human adopts the other's with the Fermi probability, both payoffs expected
/// against everyone else. Bots keep the action drawn at the start (amendment 5).
pub fn micro_well_mixed(rules: &Rules, pop: &mut Population, rng: &mut SimRng) {
    let Some(i) = pop.random_human(rng) else {
        return;
    };
    let j = pop
        .partner(i, rng)
        .expect("a well-mixed population has two players");
    let (a, b) = (pop.action[i], pop.action[j]);
    if a == b {
        return;
    }
    let others = pop.nodes - 1;
    let pi = expected(&rules.plane, a, pop.cooperating - usize::from(a), others);
    let pj = expected(&rules.plane, b, pop.cooperating - usize::from(b), others);
    if rng.gen::<f64>() < fermi(rules.beta, pi, pj) {
        pop.set_pure(i, b);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graph::Graph, rng::seeded};

    const SD: Plane = Plane { t: 1.2, s: 0.5 };
    const SH: Plane = Plane { t: 0.6, s: -0.2 };
    const PD: Plane = Plane { t: 1.3, s: -0.2 };

    #[test]
    fn eq_4_is_the_payoff_gap_of_eq_2() {
        for (rho, y, phi) in [(0.3, 0.95, 0.25), (0.8, 4.0, 0.9), (0.0, 0.0, 0.0)] {
            for p in [SD, SH, PD] {
                let pi_c = (rho + (1.0 - rho) * p.s) / (1.0 + y)
                    + y * (phi + (1.0 - phi) * p.s) / (1.0 + y);
                let pi_d = rho * p.t / (1.0 + y) + y * phi * p.t / (1.0 + y);
                assert!((p.gap(rho, y, phi) - (pi_c - pi_d)).abs() < 1e-15);
            }
        }
    }

    #[test]
    fn the_interior_root_is_eq_5() {
        let (y, phi) = (0.5, 0.6);
        let h = SD.rho_hat();
        let want = h + y * (h - phi);
        let r = roots(|r| SD.replicator(r, y, phi), 0.0, 1.0, 1000);
        assert!(
            r.iter().any(|r| r.stable && (r.x - want).abs() < 1e-12),
            "{r:?} vs {want}"
        );
        assert!((SD.rho_hat() - 5.0 / 7.0).abs() < 1e-15 && (SD.y_min() - 0.4).abs() < 1e-15);
        assert!((SH.rho_hat() - 1.0 / 3.0).abs() < 1e-15);
    }

    #[test]
    fn theorems_agree_with_the_numerical_roots_away_from_boundaries() {
        // The survey checks the full grid (G1); these cells sit well inside their regions.
        for (plane, y, phi, phase) in [
            (PD, 2.0, 0.9, Phase::FullD),
            (SD, 0.3, 0.0, Phase::Coexistence),
            (SD, 1.0, 0.0, Phase::FullC),
            (SD, 4.0, 0.9, Phase::FullD),
            (SH, 4.0, 0.9, Phase::FullC),
            (SH, 0.5, 0.5, Phase::Bistable),
            (SH, 4.0, 0.0, Phase::FullD),
        ] {
            assert_eq!(plane.phase_closed(y, phi), phase, "{plane:?} y {y} φ {phi}");
            assert_eq!(
                phase_numeric(&plane, y, phi),
                Some(phase),
                "{plane:?} y {y} φ {phi}"
            );
            assert!(plane.boundary_distance(y, phi) > 1e-3);
        }
    }

    #[test]
    fn the_master_equation_runs_from_the_replicator_root_to_phi() {
        let (y, phi) = (0.5, 0.6);
        let want = SD.rho_hat() + y * (SD.rho_hat() - phi);
        let strong = master_roots(&SD, y, phi, 1e4);
        assert!(
            strong.iter().any(|r| r.stable && (r.x - want).abs() < 1e-3),
            "{strong:?}"
        );
        // As K → 0 the comparison is a coin toss and humans drift to the bots' φ.
        let weak = master_roots(&SD, y, phi, 1e-9);
        assert!(
            weak.iter().any(|r| r.stable && (r.x - phi).abs() < 1e-6),
            "{weak:?}"
        );
        assert!(
            SD.master(0.0, y, phi, 10.0) > 0.0 && SD.master(1.0, y, phi, 10.0) < 0.0,
            "bots keep both ends open"
        );
    }

    #[test]
    fn a_root_in_the_cell_next_to_an_exact_endpoint_zero_is_still_found() {
        // ρ* = 0.99971 sits inside the last cell, beside the exact zero at ρ = 1.
        let phase = SD.phase_closed(0.45, 0.08);
        assert_eq!(phase, Phase::Coexistence);
        assert_eq!(phase_numeric(&SD, 0.45, 0.08), Some(phase));
    }

    fn rules(phi: f64, bot_action: BotAction) -> Rules {
        Rules {
            plane: PD,
            phi,
            beta: 10.0,
            bot_action,
            accumulate: Accumulate::Sum,
        }
    }

    #[test]
    fn a_focal_bot_redraws_under_redraw_and_keeps_its_action_when_frozen() {
        let mut pop = Population::new(Graph::from_lists(vec![vec![1], vec![0]]), vec![true, true]);
        let redraw = rules(0.3, BotAction::Redraw);
        let mut rng = seeded(1);
        let mut c = 0;
        for _ in 0..20_000 {
            micro_network(&redraw, &mut pop, &mut rng);
            c += pop.cooperating;
        }
        let share = c as f64 / 40_000.0;
        assert!((share - 0.3).abs() < 0.02, "{share}");
        let before = pop.action.clone();
        for _ in 0..100 {
            micro_network(&rules(0.3, BotAction::Frozen), &mut pop, &mut rng);
        }
        assert_eq!(pop.action, before);
    }

    #[test]
    fn per_interaction_bots_draw_a_fresh_action_for_every_game() {
        // A human with 1,000 bot neighbors, all storing D.
        let n = 1001;
        let mut lists = vec![(1..n as u32).collect::<Vec<_>>()];
        lists.extend((1..n).map(|_| vec![0]));
        let pop = Population::new(Graph::from_lists(lists), (0..n).map(|i| i > 0).collect());
        let fresh = rules(0.5, BotAction::PerInteraction);
        let pay = payoff(&fresh, &pop, 0, &mut seeded(4));
        assert!(
            (pay / 1000.0 - 0.65).abs() < 0.08,
            "a defector earns T = 1.3 from about half: {pay}"
        );
        assert_eq!(
            payoff_stored(&fresh, &pop, 0),
            0.0,
            "stored D everywhere: P"
        );
        assert_eq!(
            payoff(&rules(0.5, BotAction::Redraw), &pop, 0, &mut seeded(4)),
            0.0
        );
    }

    #[test]
    fn guo_isolated_humans_do_nothing() {
        let g = Graph::from_lists(vec![vec![1], vec![0], vec![]]);
        let mut pop = Population::new(g, vec![false, false, false]);
        pop.set_pure(2, true);
        let mean = Rules {
            accumulate: Accumulate::Mean,
            ..rules(0.5, BotAction::Redraw)
        };
        assert_eq!(payoff_stored(&mean, &pop, 2), 0.0);
        let mut rng = seeded(6);
        for _ in 0..1000 {
            micro_network(&mean, &mut pop, &mut rng);
        }
        assert!(pop.action[2], "an isolated human has no role model");
    }

    #[test]
    fn algorithm_i_uses_expected_payoffs_against_everyone_else() {
        // Ten players, four cooperating: a cooperator meets 3 C and 6 D, a defector 4 C and 5 D.
        assert!((expected(&SD, true, 3, 9) - (3.0 + 6.0 * 0.5) / 9.0).abs() < 1e-15);
        assert!((expected(&SD, false, 4, 9) - 4.0 * 1.2 / 9.0).abs() < 1e-15);
        let mut pop = Population::new(Graph::default(), vec![false; 10]);
        for i in 0..10 {
            pop.set_pure(i, i < 4);
        }
        let coin = Rules {
            plane: SD,
            phi: 0.5,
            beta: 0.0,
            bot_action: BotAction::Frozen,
            accumulate: Accumulate::Sum,
        };
        // β = 0: a comparison with a player acting otherwise flips the human half the time.
        let mut rng = seeded(5);
        let trials = 40_000;
        let mut flips = 0;
        for _ in 0..trials {
            let mut q = pop.clone();
            micro_well_mixed(&coin, &mut q, &mut rng);
            flips += usize::from(q.action != pop.action);
        }
        // P(the pair differs) = 2 · (4 · 6)/(10 · 9).
        let want = 0.5 * 2.0 * 24.0 / 90.0;
        assert!(
            (flips as f64 / f64::from(trials) - want).abs() < 0.015,
            "{flips}"
        );
    }

    #[test]
    fn init_draws_bots_at_phi_and_humans_at_p() {
        let n = 100_000;
        let mut pop = Population::new(Graph::default(), (0..n).map(|i| i % 2 == 0).collect());
        init(&mut pop, 0.9, 0.5, &mut seeded(7));
        assert!(
            (pop.bot_cooperation() - 0.9).abs() < 0.01
                && (pop.human_cooperation() - 0.5).abs() < 0.01
        );
    }

    /// A human (node 0) between two frozen bots; returns the rate at which one update turns
    /// node 0 into the other action.
    fn flip_rate(rules: &Rules, actions: [bool; 3], seed: u64, trials: u32) -> f64 {
        let g = Graph::from_lists(vec![vec![1, 2], vec![0], vec![0]]);
        let mut pop = Population::new(g, vec![false, true, true]);
        for (i, a) in actions.into_iter().enumerate() {
            pop.set_pure(i, a);
        }
        let mut rng = seeded(seed);
        let mut flips = 0;
        for _ in 0..trials {
            let mut q = pop.clone();
            micro_network(rules, &mut q, &mut rng);
            flips += usize::from(q.action[0] != pop.action[0]);
        }
        flips as f64 / f64::from(trials)
    }

    #[test]
    fn a_human_imitates_a_better_paid_model_and_rarely_a_worse_one() {
        // Focal is node 0 with probability 1/3, picks the differing model with probability 1/2,
        // then copies with probability logistic(β Δπ). Tolerance: 5 standard deviations of a
        // binomial rate at 60,000 trials.
        let r = Rules {
            beta: 1.0,
            ..rules(0.5, BotAction::Frozen)
        };
        let trials = 60_000;
        // C human between a D bot (earning T = 1.3) and a C bot: π_0 = S + R = 0.8, Δ = +0.5.
        let better = flip_rate(&r, [true, false, true], 11, trials);
        // D human between a C bot (earning S = −0.2) and a D bot: π_0 = T + P = 1.3, Δ = −1.5.
        let worse = flip_rate(&r, [false, true, false], 12, trials);
        for (got, delta) in [(better, 0.5), (worse, -1.5)] {
            let want = logistic(delta) / 6.0;
            let sd = (want * (1.0 - want) / f64::from(trials)).sqrt();
            assert!((got - want).abs() < 5.0 * sd, "{got} vs {want}");
        }
    }

    #[test]
    fn copying_a_per_interaction_bot_adopts_a_fresh_draw() {
        // The human stores D, its only neighbor is a bot storing D, φ = 1, β = 0: half of the
        // focal-human updates copy, and a copy draws C. Focal is the human half the time, so
        // 1/4 of updates make it C; 5 standard deviations of that rate at 40,000 trials is 0.011.
        let g = Graph::from_lists(vec![vec![1], vec![0]]);
        let pop = Population::new(g, vec![false, true]);
        let r = Rules {
            beta: 0.0,
            ..rules(1.0, BotAction::PerInteraction)
        };
        let mut rng = seeded(13);
        let trials = 40_000;
        let mut became_c = 0;
        for _ in 0..trials {
            let mut q = pop.clone();
            micro_network(&r, &mut q, &mut rng);
            became_c += usize::from(q.action[0]);
        }
        let got = became_c as f64 / f64::from(trials);
        assert!((got - 0.25).abs() < 0.012, "{got}");
    }

    #[test]
    fn algorithm_i_follows_the_expected_payoff_gap_and_never_moves_bots() {
        // One human among nine frozen bots of the other action on the snowdrift plane, β = 5.
        // Every partner differs, so each update flips the human with probability
        // logistic(β(π_partner − π_human)), the focal excluded from both counts.
        // Case 1, a D human among C bots: π_D = T = 1.2, π_C = (8R + S)/9.
        // Case 2, a C human among D bots: π_C = S = 0.5, π_D = T/9.
        let r = Rules {
            plane: SD,
            phi: 0.5,
            beta: 5.0,
            bot_action: BotAction::Frozen,
            accumulate: Accumulate::Sum,
        };
        let cases = [
            (false, logistic(5.0 * ((8.0 * 1.0 + 0.5) / 9.0 - 1.2))),
            (true, logistic(5.0 * (1.2 / 9.0 - 0.5))),
        ];
        for (seed, (human, want)) in (14..).zip(cases) {
            let mut pop = Population::new(Graph::default(), (0..10).map(|i| i > 0).collect());
            for i in 0..10 {
                pop.set_pure(i, if i == 0 { human } else { !human });
            }
            let mut rng = seeded(seed);
            let trials = 40_000;
            let mut flips = 0;
            for _ in 0..trials {
                let mut q = pop.clone();
                micro_well_mixed(&r, &mut q, &mut rng);
                assert!(
                    q.action[1..].iter().all(|&a| a == !human),
                    "bots never change"
                );
                flips += usize::from(q.action[0] != human);
            }
            // 5 standard deviations of a binomial rate at 40,000 trials.
            let sd = (want * (1.0 - want) / f64::from(trials)).sqrt();
            assert!(
                (flips as f64 / f64::from(trials) - want).abs() < 5.0 * sd,
                "{flips} vs {want}"
            );
        }
    }

    #[test]
    fn per_interaction_payoffs_use_the_fresh_draws_not_the_stored_actions() {
        // Human and bot both store D; φ = 1 and β = 10. Stored payoffs tie (0 vs 0, a coin
        // flip), but in play the bot cooperates against the human's D: π_human = T = 1.3,
        // π_bot = S = −0.2, so a copy has probability logistic(−15), about 3e-7. Across 20,000
        // updates the human should never become C (the stored-payoff bug gives about 1/4 of them).
        let g = Graph::from_lists(vec![vec![1], vec![0]]);
        let pop = Population::new(g, vec![false, true]);
        let r = Rules {
            beta: 10.0,
            ..rules(1.0, BotAction::PerInteraction)
        };
        let mut rng = seeded(18);
        let mut became_c = 0;
        for _ in 0..20_000 {
            let mut q = pop.clone();
            micro_network(&r, &mut q, &mut rng);
            became_c += usize::from(q.action[0]);
        }
        assert_eq!(became_c, 0);
    }

    #[test]
    fn algorithm_i_bots_keep_their_initial_actions() {
        let mut pop = Population::new(Graph::default(), (0..20).map(|i| i % 2 == 0).collect());
        init(&mut pop, 0.5, 0.5, &mut seeded(15));
        let bots: Vec<bool> = (0..20)
            .filter(|&i| pop.bot[i])
            .map(|i| pop.action[i])
            .collect();
        let r = Rules {
            plane: SD,
            phi: 0.5,
            beta: 2.0,
            bot_action: BotAction::Frozen,
            accumulate: Accumulate::Sum,
        };
        let mut rng = seeded(16);
        let mut human_moves = 0;
        for _ in 0..5000 {
            let before = pop.action.clone();
            micro_well_mixed(&r, &mut pop, &mut rng);
            human_moves += usize::from(pop.action != before);
        }
        let after: Vec<bool> = (0..20)
            .filter(|&i| pop.bot[i])
            .map(|i| pop.action[i])
            .collect();
        assert_eq!(bots, after);
        assert!(human_moves > 0, "the humans do imitate");
    }
}
