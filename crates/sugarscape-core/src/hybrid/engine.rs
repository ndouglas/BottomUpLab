//! The population every hybrid world but coloring shares: a graph (empty when well-mixed), which
//! nodes are bots, each node's strategy (its probability of cooperating; 0 or 1 in a pure
//! framework) and current action, with the counts the well-mixed payoffs read; the Fermi rule;
//! role-model draws.

use super::exact::logistic;
use crate::{graph::Graph, rng::SimRng};
use rand::Rng;
use serde::{Deserialize, Serialize};

/// How a node's games add up: summed over its neighbors (all three papers) or averaged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Accumulate {
    Sum,
    Mean,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Population {
    /// Neighbor lists; empty (`is_complete`) when well-mixed.
    pub graph: Graph,
    pub nodes: usize,
    pub bot: Vec<bool>,
    /// The human nodes, in index order.
    pub humans: Vec<u32>,
    /// Each node's probability of cooperating (0 or 1 in a pure framework).
    pub strategy: Vec<f64>,
    /// Each node's current action, true = cooperate.
    pub action: Vec<bool>,
    /// Nodes acting C now.
    pub cooperating: usize,
    /// Σ strategy over all nodes, and over the humans.
    pub strategy_sum: f64,
    pub human_strategy_sum: f64,
}

impl Population {
    /// Every node defecting with strategy 0.
    pub fn new(graph: Graph, bot: Vec<bool>) -> Self {
        let nodes = bot.len();
        let humans = (0..nodes as u32).filter(|&i| !bot[i as usize]).collect();
        Population {
            graph,
            nodes,
            bot,
            humans,
            strategy: vec![0.0; nodes],
            action: vec![false; nodes],
            cooperating: 0,
            strategy_sum: 0.0,
            human_strategy_sum: 0.0,
        }
    }

    pub fn well_mixed(&self) -> bool {
        self.graph.is_complete()
    }

    /// Node i's neighbors (empty when well-mixed).
    pub fn neighbors(&self, i: usize) -> &[u32] {
        if self.well_mixed() {
            &[]
        } else {
            self.graph.of(i)
        }
    }

    /// How many games node i plays: its neighbors, or every other node when well-mixed.
    pub fn degree(&self, i: usize) -> usize {
        if self.well_mixed() {
            self.nodes - 1
        } else {
            self.graph.of(i).len()
        }
    }

    pub fn set_strategy(&mut self, i: usize, s: f64) {
        let d = s - self.strategy[i];
        self.strategy_sum += d;
        if !self.bot[i] {
            self.human_strategy_sum += d;
        }
        self.strategy[i] = s;
    }

    pub fn set_action(&mut self, i: usize, cooperate: bool) {
        if self.action[i] != cooperate {
            if cooperate {
                self.cooperating += 1;
            } else {
                self.cooperating -= 1;
            }
            self.action[i] = cooperate;
        }
    }

    /// A pure strategy and its action together.
    pub fn set_pure(&mut self, i: usize, cooperate: bool) {
        self.set_strategy(i, if cooperate { 1.0 } else { 0.0 });
        self.set_action(i, cooperate);
    }

    /// The humans' mean strategy, their cooperation (NaN without humans).
    pub fn human_cooperation(&self) -> f64 {
        if self.humans.is_empty() {
            f64::NAN
        } else {
            self.human_strategy_sum / self.humans.len() as f64
        }
    }

    /// The share of bots acting C now (NaN without bots).
    pub fn bot_cooperation(&self) -> f64 {
        let bots = self.nodes - self.humans.len();
        if bots == 0 {
            return f64::NAN;
        }
        let c = (0..self.nodes)
            .filter(|&i| self.bot[i] && self.action[i])
            .count();
        c as f64 / bots as f64
    }

    pub fn random_node(&self, rng: &mut SimRng) -> usize {
        rng.gen_range(0..self.nodes as u32) as usize
    }

    /// A uniformly random human (None without humans).
    pub fn random_human(&self, rng: &mut SimRng) -> Option<usize> {
        if self.humans.is_empty() {
            return None;
        }
        Some(self.humans[rng.gen_range(0..self.humans.len() as u32) as usize] as usize)
    }

    /// The role model for node i: a uniformly random neighbor (None when isolated), or, when
    /// well-mixed, a uniformly random node redrawn until it is not i (Si et al.'s code).
    pub fn partner(&self, i: usize, rng: &mut SimRng) -> Option<usize> {
        if self.well_mixed() {
            assert!(self.nodes >= 2, "a well-mixed population needs two nodes");
            loop {
                let j = self.random_node(rng);
                if j != i {
                    return Some(j);
                }
            }
        }
        let ns = self.graph.of(i);
        if ns.is_empty() {
            return None;
        }
        Some(ns[rng.gen_range(0..ns.len() as u32) as usize] as usize)
    }
}

/// The probability that a focal earning `focal` adopts a model earning `model`:
/// 1 / (1 + e^{−β(π_model − π_focal)}).
pub fn fermi(beta: f64, focal: f64, model: f64) -> f64 {
    logistic(beta * (model - focal))
}

/// A summed payoff, or its mean over `degree` games (0 for an isolated node).
pub fn accumulate(total: f64, degree: usize, how: Accumulate) -> f64 {
    match how {
        Accumulate::Sum => total,
        Accumulate::Mean if degree == 0 => 0.0,
        Accumulate::Mean => total / degree as f64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::seeded;

    fn ring4() -> Graph {
        Graph::from_lists(vec![vec![1, 3], vec![0, 2], vec![1, 3], vec![2, 0]])
    }

    #[test]
    fn counts_follow_every_change() {
        let mut p = Population::new(ring4(), vec![true, false, false, false]);
        assert_eq!(p.humans, vec![1, 2, 3]);
        p.set_pure(0, true);
        p.set_pure(1, true);
        p.set_strategy(2, 0.5);
        assert_eq!(
            (p.cooperating, p.strategy_sum, p.human_strategy_sum),
            (2, 2.5, 1.5)
        );
        assert_eq!((p.human_cooperation(), p.bot_cooperation()), (0.5, 1.0));
        p.set_pure(1, false);
        assert_eq!((p.cooperating, p.human_strategy_sum), (1, 0.5));
        p.set_action(1, false);
        assert_eq!(
            p.cooperating, 1,
            "setting an action to itself changes nothing"
        );
    }

    #[test]
    fn isolated_nodes_have_no_partner_and_mean_payoffs_of_zero() {
        let p = Population::new(
            Graph::from_lists(vec![vec![1], vec![0], vec![]]),
            vec![false; 3],
        );
        let mut rng = seeded(1);
        assert_eq!(p.partner(2, &mut rng), None);
        assert_eq!(p.partner(0, &mut rng), Some(1));
        assert_eq!(accumulate(0.0, 0, Accumulate::Mean), 0.0);
        assert_eq!(accumulate(6.0, 3, Accumulate::Mean), 2.0);
        assert_eq!(accumulate(6.0, 3, Accumulate::Sum), 6.0);
    }

    #[test]
    fn well_mixed_partners_are_any_other_node() {
        let p = Population::new(Graph::default(), vec![false; 5]);
        assert!(p.well_mixed() && p.degree(0) == 4 && p.neighbors(0).is_empty());
        let mut rng = seeded(3);
        let mut seen = [0; 5];
        for _ in 0..10_000 {
            seen[p.partner(2, &mut rng).unwrap()] += 1;
        }
        assert_eq!(seen[2], 0);
        assert!(
            seen.iter()
                .enumerate()
                .all(|(i, &n)| i == 2 || (2300..2700).contains(&n)),
            "{seen:?}"
        );
    }

    #[test]
    fn a_population_without_humans_reports_nan() {
        let p = Population::new(ring4(), vec![true; 4]);
        assert!(p.human_cooperation().is_nan());
        assert_eq!(p.random_human(&mut seeded(1)), None);
        let q = Population::new(ring4(), vec![false; 4]);
        assert!(q.bot_cooperation().is_nan());
    }

    #[test]
    fn fermi_is_one_half_at_equal_payoffs_and_saturates() {
        assert_eq!(fermi(10.0, 1.0, 1.0), 0.5);
        assert!(fermi(10.0, 0.0, 1.0) > 0.9999 && fermi(10.0, 1.0, 0.0) < 1e-4);
        assert_eq!(fermi(0.0, 5.0, -5.0), 0.5);
    }
}
