//! Si, He, Shen and Tanimoto (J. R. Soc. Interface 2025; code Zenodo 10.5281/zenodo.13957161):
//! the Prisoner's Dilemma R = 1, S = −r, T = 1 + r, P = 0 with bots of strategy θ in three
//! frameworks, on the Moore lattice and well-mixed, as the published code runs it. Defaults are
//! the code's: well-mixed discrete and mixed payoffs drop the S and P terms (`division: integer`;
//! `float` is the stated game), and mixed actions are drawn for every game on the lattice and
//! whenever a node is picked well-mixed (`mixed_draw: per_round` draws once per round, Fig. A1).

use super::engine::{accumulate, fermi, Accumulate, Population};
use crate::rng::SimRng;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    Discrete,
    Continuous,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MixedDraw {
    /// The code: on a graph both players draw for every game; well-mixed, a picked node redraws.
    PerInteraction,
    /// Fig. A1: every node draws once at the start of each round.
    PerRound,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Division {
    /// The code: `(nn - c) / nn` in integers.
    Integer,
    /// The stated game (Eq. 1).
    Float,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub r: f64,
    pub framework: Framework,
    pub mixed_draw: MixedDraw,
    pub division: Division,
    pub beta: f64,
    pub accumulate: Accumulate,
}

/// Eq. 1: R = 1, S = −r, T = 1 + r, P = 0 (true = cooperate).
pub fn pay(r: f64, me: bool, other: bool) -> f64 {
    match (me, other) {
        (true, true) => 1.0,
        (true, false) => -r,
        (false, true) => 1.0 + r,
        (false, false) => 0.0,
    }
}

/// The code's initial state, in index order. Discrete: a bot cooperates with probability θ, drawn
/// once; a human cooperates if `randi(2)` is 0. Continuous and mixed: a bot's strategy is θ, a
/// human's uniform on [0, 1); each node's action is drawn from its strategy.
pub fn init(pop: &mut Population, framework: Framework, theta: f64, rng: &mut SimRng) {
    for i in 0..pop.nodes {
        if framework == Framework::Discrete {
            let c = if pop.bot[i] {
                rng.gen::<f64>() < theta
            } else {
                rng.gen_range(0..2u32) == 0
            };
            pop.set_pure(i, c);
        } else {
            let s = if pop.bot[i] { theta } else { rng.gen::<f64>() };
            pop.set_strategy(i, s);
            let c = rng.gen::<f64>() < s;
            pop.set_action(i, c);
        }
    }
}

fn draw(pop: &Population, k: usize, rng: &mut SimRng) -> bool {
    rng.gen::<f64>() < pop.strategy[k]
}

/// Node i's payoff on a graph, as the lattice code's `cal_payoff`: summed over its neighbors
/// (averaged under `accumulate: mean`). Continuous: −r·sᵢ + (1 + r)·sⱼ per neighbor. Mixed per
/// interaction: the focal and the neighbor each draw an action for every game, and the drawn
/// actions are stored as the code stores them; otherwise the stored actions.
pub fn payoff_graph(rules: &Rules, pop: &mut Population, i: usize, rng: &mut SimRng) -> f64 {
    let fresh =
        rules.framework == Framework::Mixed && rules.mixed_draw == MixedDraw::PerInteraction;
    let degree = pop.neighbors(i).len();
    let mut total = 0.0;
    for k in 0..degree {
        let j = pop.neighbors(i)[k] as usize;
        total += match rules.framework {
            Framework::Continuous => -rules.r * pop.strategy[i] + (1.0 + rules.r) * pop.strategy[j],
            _ if fresh => {
                let a = draw(pop, i, rng);
                pop.set_action(i, a);
                let b = draw(pop, j, rng);
                pop.set_action(j, b);
                pay(rules.r, a, b)
            }
            _ => pay(rules.r, pop.action[i], pop.action[j]),
        };
    }
    accumulate(total, degree, rules.accumulate)
}

/// Node i's payoff from the stored actions and strategies (inspection, and Fig. A1): on a graph
/// as `payoff_graph` without fresh draws, well-mixed as `payoff_well_mixed`.
pub fn payoff_stored(rules: &Rules, pop: &Population, i: usize) -> f64 {
    if pop.well_mixed() {
        return payoff_well_mixed(rules, pop, i);
    }
    let total: f64 = pop
        .neighbors(i)
        .iter()
        .map(|&j| match rules.framework {
            Framework::Continuous => {
                -rules.r * pop.strategy[i] + (1.0 + rules.r) * pop.strategy[j as usize]
            }
            _ => pay(rules.r, pop.action[i], pop.action[j as usize]),
        })
        .sum();
    accumulate(total, pop.degree(i), rules.accumulate)
}

/// Node i's well-mixed payoff, from counts as the code computes it. Discrete and mixed: against
/// the N − 1 others, c of them cooperating (the focal excluded), (c/(N − 1))·M[a][C] plus
/// ((N − 1 − c)/(N − 1))·M[a][D], the second share in integers under `division: integer` (so 0
/// unless c = 0) and in floating point under `float`. Continuous: −r·sᵢ + (1 + r)·(the others'
/// mean strategy).
pub fn payoff_well_mixed(rules: &Rules, pop: &Population, i: usize) -> f64 {
    let nn = pop.nodes - 1;
    if rules.framework == Framework::Continuous {
        return -rules.r * pop.strategy[i]
            + (1.0 + rules.r) * (pop.strategy_sum - pop.strategy[i]) / nn as f64;
    }
    let a = pop.action[i];
    let c = pop.cooperating - usize::from(a);
    let defect_share = match rules.division {
        Division::Integer => ((nn - c) / nn) as f64,
        Division::Float => (nn - c) as f64 / nn as f64,
    };
    c as f64 / nn as f64 * pay(rules.r, a, true) + defect_share * pay(rules.r, a, false)
}

/// The focal adopts the model's strategy (discrete: its action).
fn adopt(rules: &Rules, pop: &mut Population, i: usize, j: usize) {
    if rules.framework == Framework::Discrete {
        let a = pop.action[j];
        pop.set_pure(i, a);
    } else {
        let s = pop.strategy[j];
        pop.set_strategy(i, s);
    }
}

/// One round: under `per_round` (mixed) every node first draws its action; then N focal picks
/// with replacement. On a graph (the lattice code's `round_game`) only ordinary players act: a
/// uniformly random neighbor is the role model, both payoffs are computed afresh, and the focal
/// adopts with the Fermi probability. Well-mixed discrete and continuous (`main_process`): only
/// ordinary players act, against a uniformly random other. Well-mixed mixed (Fig. 1C's
/// `learn_strategy`): every picked node draws a role model; an ordinary player may adopt; then
/// the picked node, bot or not, draws its action from its strategy (not under `per_round`).
pub fn round(rules: &Rules, pop: &mut Population, rng: &mut SimRng) {
    let mixed = rules.framework == Framework::Mixed;
    if mixed && rules.mixed_draw == MixedDraw::PerRound {
        for k in 0..pop.nodes {
            let a = draw(pop, k, rng);
            pop.set_action(k, a);
        }
    }
    for _ in 0..pop.nodes {
        let c = pop.random_node(rng);
        if pop.well_mixed() {
            if !mixed && pop.bot[c] {
                continue;
            }
            let j = pop.partner(c, rng).expect("two players");
            if !pop.bot[c] {
                let p = fermi(
                    rules.beta,
                    payoff_well_mixed(rules, pop, c),
                    payoff_well_mixed(rules, pop, j),
                );
                if rng.gen::<f64>() < p {
                    adopt(rules, pop, c, j);
                }
            }
            if mixed && rules.mixed_draw == MixedDraw::PerInteraction {
                let a = draw(pop, c, rng);
                pop.set_action(c, a);
            }
        } else if !pop.bot[c] {
            let Some(j) = pop.partner(c, rng) else {
                continue;
            };
            let pc = payoff_graph(rules, pop, c, rng);
            let pj = payoff_graph(rules, pop, j, rng);
            if rng.gen::<f64>() < fermi(rules.beta, pc, pj) {
                adopt(rules, pop, c, j);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::Graph,
        hybrid::graph::{lattice, Neighborhood},
        rng::seeded,
    };

    const R: f64 = 0.1;

    fn rules(framework: Framework) -> Rules {
        Rules {
            r: R,
            framework,
            mixed_draw: MixedDraw::PerInteraction,
            division: Division::Integer,
            beta: 10.0,
            accumulate: Accumulate::Sum,
        }
    }

    #[test]
    fn eq_2_is_the_expectation_of_eq_1() {
        for (si, sj) in [(0.4, 0.9), (0.0, 1.0), (0.25, 0.25)] {
            let mut e = 0.0;
            for (a, pa) in [(true, si), (false, 1.0 - si)] {
                for (b, pb) in [(true, sj), (false, 1.0 - sj)] {
                    e += pa * pb * pay(R, a, b);
                }
            }
            assert!((e - (-R * si + (1.0 + R) * sj)).abs() < 1e-15, "{si} {sj}");
        }
    }

    #[test]
    fn figure_a1_payoffs() {
        // Fig. A1: a defecting bot with three cooperating neighbors earns 3(1 + r); a defecting
        // human with two cooperating bot neighbors 2(1 + r), with four 4(1 + r).
        // Nodes: 0 the left bot (D); 1 the right human (D); 2–4 cooperators around the bot;
        // 5–8 cooperating bots around the human; 9 a defector beside the left bot.
        let lists = vec![
            vec![2, 3, 4, 9],
            vec![5, 6, 7, 8],
            vec![0],
            vec![0],
            vec![0],
            vec![1],
            vec![1],
            vec![1],
            vec![1],
            vec![0],
        ];
        let bot = vec![
            true, false, false, false, false, true, true, true, true, false,
        ];
        let mut pop = Population::new(Graph::from_lists(lists), bot);
        for (i, c) in [
            false, false, true, true, true, true, true, false, false, false,
        ]
        .into_iter()
        .enumerate()
        {
            pop.set_pure(i, c);
        }
        let per_round = Rules {
            mixed_draw: MixedDraw::PerRound,
            ..rules(Framework::Mixed)
        };
        assert!((payoff_stored(&per_round, &pop, 0) - 3.0 * (1.0 + R)).abs() < 1e-15);
        assert!(
            (payoff_stored(&per_round, &pop, 1) - 2.0 * (1.0 + R)).abs() < 1e-15,
            "panel a"
        );
        pop.set_pure(7, true);
        pop.set_pure(8, true);
        assert!(
            (payoff_stored(&per_round, &pop, 1) - 4.0 * (1.0 + R)).abs() < 1e-15,
            "panel b"
        );
    }

    #[test]
    fn a_theta_one_mixed_bot_plays_as_a_cooperating_discrete_bot() {
        let g = Graph::from_lists(lattice(5, Neighborhood::Moore, true));
        let bot: Vec<bool> = (0..25).map(|i| i % 3 == 0).collect();
        let mut mixed = Population::new(g.clone(), bot.clone());
        let mut discrete = Population::new(g, bot);
        for i in 0..25 {
            let c = mixed.bot[i] || i % 2 == 0;
            mixed.set_pure(i, c);
            discrete.set_pure(i, c);
        }
        let mut rng = seeded(1);
        for i in 0..25 {
            let m = payoff_graph(&rules(Framework::Mixed), &mut mixed, i, &mut rng);
            let d = payoff_graph(&rules(Framework::Discrete), &mut discrete, i, &mut rng);
            assert_eq!(
                m, d,
                "node {i}: strategies of 0 and 1 draw deterministically"
            );
        }
    }

    #[test]
    fn mixed_games_redraw_both_players_unless_drawn_per_round() {
        // Both nodes hold strategy 1 but a stale D action.
        let mut pop = Population::new(Graph::from_lists(vec![vec![1], vec![0]]), vec![false; 2]);
        pop.set_strategy(0, 1.0);
        pop.set_strategy(1, 1.0);
        let mut rng = seeded(4);
        let per_round = Rules {
            mixed_draw: MixedDraw::PerRound,
            ..rules(Framework::Mixed)
        };
        assert_eq!(
            payoff_graph(&per_round, &mut pop, 0, &mut rng),
            0.0,
            "per round: the stored D–D"
        );
        assert_eq!(
            payoff_graph(&rules(Framework::Mixed), &mut pop, 0, &mut rng),
            1.0,
            "drawn C–C"
        );
        assert!(
            pop.action[0] && pop.action[1] && pop.cooperating == 2,
            "the drawn actions are stored"
        );
    }

    #[test]
    fn imitation_moves_toward_the_higher_payoff() {
        // Continuous, r = 0.5, β = 100: a defector always out-earns a cooperator here, so the
        // Fermi rule carries everyone to strategy 0 and never back.
        let strong = Rules {
            r: 0.5,
            beta: 100.0,
            ..rules(Framework::Continuous)
        };
        let mut rng = seeded(5);
        let mut pair = Population::new(Graph::from_lists(vec![vec![1], vec![0]]), vec![false; 2]);
        pair.set_strategy(0, 1.0);
        for _ in 0..10 {
            round(&strong, &mut pair, &mut rng);
        }
        assert_eq!(
            pair.strategy,
            vec![0.0, 0.0],
            "graph: π_C = −r, π_D = 1 + r"
        );
        let mut wm = Population::new(Graph::default(), vec![false; 200]);
        for i in 0..100 {
            wm.set_strategy(i, 1.0);
        }
        for _ in 0..10 {
            round(&strong, &mut wm, &mut rng);
        }
        assert!(
            wm.human_cooperation() < 0.1,
            "well-mixed: {}",
            wm.human_cooperation()
        );
    }

    #[test]
    fn the_codes_well_mixed_payoff_drops_s_and_p() {
        // N = 11: ten others, four of them cooperating.
        let mut pop = Population::new(Graph::default(), vec![false; 11]);
        for i in 0..5 {
            pop.set_pure(i, true);
        }
        let int = rules(Framework::Discrete);
        let float = Rules {
            division: Division::Float,
            ..int
        };
        assert!(
            (payoff_well_mixed(&int, &pop, 0) - 0.4).abs() < 1e-15,
            "C: (4/10)·1 + 0·(−r)"
        );
        assert!(
            (payoff_well_mixed(&float, &pop, 0) - (0.4 - 0.6 * R)).abs() < 1e-15,
            "C: the stated game"
        );
        assert!(
            (payoff_well_mixed(&int, &pop, 10) - 0.5 * (1.0 + R)).abs() < 1e-15,
            "D: (5/10)(1 + r)"
        );
        // Nobody else cooperating: the integer share is 10/10 = 1.
        let mut lone = Population::new(Graph::default(), vec![false; 11]);
        lone.set_pure(0, true);
        assert!(
            (payoff_well_mixed(&int, &lone, 0) + R).abs() < 1e-15,
            "C among defectors: S = −r"
        );
        assert_eq!(
            payoff_well_mixed(&int, &lone, 1),
            0.1 * (1.0 + R),
            "D: (1/10)(1 + r)"
        );
        let mut cont = Population::new(Graph::default(), vec![false; 5]);
        for (i, s) in [0.2, 0.4, 0.6, 0.8, 1.0].into_iter().enumerate() {
            cont.set_strategy(i, s);
        }
        let want = -R * 0.2 + (1.0 + R) * (0.4 + 0.6 + 0.8 + 1.0) / 4.0;
        assert!((payoff_well_mixed(&rules(Framework::Continuous), &cont, 0) - want).abs() < 1e-15);
    }

    #[test]
    fn lattice_bots_never_change_and_well_mixed_mixed_bots_keep_redrawing() {
        let g = Graph::from_lists(lattice(10, Neighborhood::Moore, true));
        let bot: Vec<bool> = (0..100).map(|i| i % 4 == 0).collect();
        let mut pop = Population::new(g, bot.clone());
        let mut rng = seeded(2);
        init(&mut pop, Framework::Continuous, 0.9, &mut rng);
        let before: Vec<f64> = (0..100)
            .filter(|&i| bot[i])
            .map(|i| pop.strategy[i])
            .collect();
        for _ in 0..20 {
            round(&rules(Framework::Continuous), &mut pop, &mut rng);
        }
        let after: Vec<f64> = (0..100)
            .filter(|&i| bot[i])
            .map(|i| pop.strategy[i])
            .collect();
        assert_eq!(before, after);
        assert!(before.iter().all(|&s| s == 0.9));
        let n = 2000;
        let mut wm = Population::new(Graph::default(), (0..n).map(|i| i % 2 == 0).collect());
        init(&mut wm, Framework::Mixed, 0.9, &mut rng);
        // Every bot starts acting D, so only redraws can bring their cooperation to θ.
        for i in (0..n).step_by(2) {
            wm.set_action(i, false);
        }
        for _ in 0..10 {
            round(&rules(Framework::Mixed), &mut wm, &mut rng);
        }
        let mut share = 0.0;
        for _ in 0..40 {
            round(&rules(Framework::Mixed), &mut wm, &mut rng);
            share += wm.bot_cooperation();
        }
        assert!((share / 40.0 - 0.9).abs() < 0.02, "{share}");
    }

    #[test]
    fn init_follows_the_code() {
        let n = 100_000;
        let bot: Vec<bool> = (0..n).map(|i| i % 10 == 0).collect();
        let mut d = Population::new(Graph::default(), bot.clone());
        init(&mut d, Framework::Discrete, 0.9, &mut seeded(3));
        assert!(
            (d.bot_cooperation() - 0.9).abs() < 0.03 && (d.human_cooperation() - 0.5).abs() < 0.01
        );
        assert!(d.strategy.iter().all(|&s| s == 0.0 || s == 1.0));
        let mut c = Population::new(Graph::default(), bot);
        init(&mut c, Framework::Continuous, 0.9, &mut seeded(3));
        assert!(
            (c.human_cooperation() - 0.5).abs() < 0.01,
            "uniform strategies"
        );
        assert!((0..n).filter(|&i| c.bot[i]).all(|i| c.strategy[i] == 0.9));
    }
}
