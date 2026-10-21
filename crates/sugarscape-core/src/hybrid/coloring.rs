//! Shirado and Christakis (Nature 545:370, 2017): a network 3-coloring game with noisy bots. The
//! game, the bot, the Barabási–Albert graph, the solution count and Extended Data Fig. 2's
//! statistic are the paper's; the human players are our labeled reconstruction (greedy
//! responders with exponential decision times and an errant-move rate).

use crate::{portable::ln, rng::SimRng};
use rand::Rng;

pub const COLORS: usize = 3;

/// The number of proper colorings with `k` colors, by backtracking in index order. Exact on any
/// graph; fast on a Barabási–Albert graph grown with two links per node, where every node after
/// the triangle has exactly two earlier neighbors (so at most two branches each).
pub fn colorings(adj: &[Vec<u32>], k: u8) -> u64 {
    fn go(adj: &[Vec<u32>], k: u8, v: usize, color: &mut [u8]) -> u64 {
        if v == adj.len() {
            return 1;
        }
        let mut total = 0;
        for c in 0..k {
            if adj[v]
                .iter()
                .all(|&u| u as usize >= v || color[u as usize] != c)
            {
                color[v] = c;
                total += go(adj, k, v + 1, color);
            }
        }
        total
    }
    go(adj, k, 0, &mut vec![0; adj.len()])
}

/// How many of node v's neighbors hold each color.
pub fn counts(adj: &[Vec<u32>], color: &[u8], v: usize) -> [usize; COLORS] {
    let mut c = [0; COLORS];
    for &u in &adj[v] {
        c[color[u as usize] as usize] += 1;
    }
    c
}

/// Edges joining two equal colors.
pub fn conflicts_of(adj: &[Vec<u32>], color: &[u8]) -> usize {
    adj.iter()
        .enumerate()
        .map(|(v, ns)| {
            ns.iter()
                .filter(|&&u| (u as usize) > v && color[u as usize] == color[v])
                .count()
        })
        .sum()
}

/// The least-common rule: if the node's color is not among the least common around it, a
/// uniformly random least-common color (ties at random, ours); otherwise its own.
pub fn greedy(counts: [usize; COLORS], own: u8, rng: &mut SimRng) -> u8 {
    let min = *counts.iter().min().unwrap();
    if counts[own as usize] == min {
        return own;
    }
    let least: Vec<u8> = (0..COLORS as u8)
        .filter(|&c| counts[c as usize] == min)
        .collect();
    least[rng.gen_range(0..least.len() as u32) as usize]
}

/// An errant move (our reconstruction): a uniformly random color that would raise the node's
/// conflicts; its own color if none would.
pub fn errant(counts: [usize; COLORS], own: u8, rng: &mut SimRng) -> u8 {
    let worse: Vec<u8> = (0..COLORS as u8)
        .filter(|&c| counts[c as usize] > counts[own as usize])
        .collect();
    if worse.is_empty() {
        own
    } else {
        worse[rng.gen_range(0..worse.len() as u32) as usize]
    }
}

/// An exponential wait of the given mean, −mean·ln(1 − u) with the portable logarithm.
pub fn wait(mean: f64, rng: &mut SimRng) -> f64 {
    -mean * ln(1.0 - rng.gen::<f64>())
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rules {
    pub noise: f64,
    pub errant: f64,
    pub reaction: f64,
    pub bot_interval: f64,
    pub time_limit: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Session {
    pub adj: Vec<Vec<u32>>,
    pub bot: Vec<bool>,
    pub color: Vec<u8>,
    /// Each node's next decision time (s).
    pub next: Vec<f64>,
    pub time: f64,
    pub conflicts: usize,
    pub solved_at: Option<f64>,
    pub decisions: u64,
    pub rules: Rules,
}

impl Session {
    /// Colors uniform at random (ours, amendment 12); each human's first decision after an
    /// exponential wait, each bot's at a uniform offset in [0, bot_interval).
    pub fn new(adj: Vec<Vec<u32>>, bot: Vec<bool>, rules: Rules, rng: &mut SimRng) -> Self {
        let n = adj.len();
        let color: Vec<u8> = (0..n)
            .map(|_| rng.gen_range(0..COLORS as u32) as u8)
            .collect();
        let next = (0..n)
            .map(|i| {
                if bot[i] {
                    rng.gen::<f64>() * rules.bot_interval
                } else {
                    wait(rules.reaction, rng)
                }
            })
            .collect();
        let conflicts = conflicts_of(&adj, &color);
        Session {
            adj,
            bot,
            color,
            next,
            time: 0.0,
            conflicts,
            solved_at: (conflicts == 0).then_some(0.0),
            decisions: 0,
            rules,
        }
    }

    pub fn finished(&self) -> bool {
        self.solved_at.is_some() || self.time >= self.rules.time_limit
    }

    /// Runs the decisions due by `t` (at most the time limit) in time order, ties to the lower
    /// index, stopping at the first proper coloring.
    pub fn advance_to(&mut self, t: f64, rng: &mut SimRng) {
        if self.finished() {
            return;
        }
        let until = t.min(self.rules.time_limit);
        loop {
            let (v, at) = self.next.iter().copied().enumerate().fold(
                (0, f64::INFINITY),
                |(bv, bt), (i, x)| if x < bt { (i, x) } else { (bv, bt) },
            );
            if at > until {
                break;
            }
            self.time = at;
            self.decide(v, rng);
            if self.conflicts == 0 {
                self.solved_at = Some(at);
                return;
            }
        }
        self.time = self.time.max(until);
    }

    fn decide(&mut self, v: usize, rng: &mut SimRng) {
        let c = counts(&self.adj, &self.color, v);
        let own = self.color[v];
        let new = if self.bot[v] {
            if rng.gen::<f64>() < self.rules.noise {
                rng.gen_range(0..COLORS as u32) as u8
            } else {
                greedy(c, own, rng)
            }
        } else if rng.gen::<f64>() < self.rules.errant {
            errant(c, own, rng)
        } else {
            greedy(c, own, rng)
        };
        self.conflicts = self.conflicts + c[new as usize] - c[own as usize];
        self.color[v] = new;
        self.decisions += 1;
        self.next[v] += if self.bot[v] {
            self.rules.bot_interval
        } else {
            wait(self.rules.reaction, rng)
        };
    }

    /// Plays to a proper coloring or the time limit; the solve time, if any.
    pub fn play(&mut self, rng: &mut SimRng) -> Option<f64> {
        self.advance_to(f64::INFINITY, rng);
        self.solved_at
    }
}

/// Extended Data Fig. 2's statistic: from `start`, a uniformly random node recolors to a
/// uniformly random color other than a uniformly random neighbor's, until the coloring is
/// proper; the number of recolorings, or None past `cap` (amendment 12).
pub fn convergence_steps(
    adj: &[Vec<u32>],
    start: &[u8],
    cap: u64,
    rng: &mut SimRng,
) -> Option<u64> {
    let mut color = start.to_vec();
    let mut conflicts = conflicts_of(adj, &color);
    let mut steps = 0;
    while conflicts > 0 {
        if steps == cap {
            return None;
        }
        steps += 1;
        let v = rng.gen_range(0..adj.len() as u32) as usize;
        if adj[v].is_empty() {
            continue;
        }
        let u = adj[v][rng.gen_range(0..adj[v].len() as u32) as usize] as usize;
        let new = (color[u] + 1 + rng.gen_range(0..2u32) as u8) % COLORS as u8;
        let c = counts(adj, &color, v);
        conflicts = conflicts + c[new as usize] - c[color[v] as usize];
        color[v] = new;
    }
    Some(steps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graph::barabasi_albert, rng::seeded};

    fn path(n: usize) -> Vec<Vec<u32>> {
        (0..n)
            .map(|i| {
                [i.checked_sub(1), (i + 1 < n).then_some(i + 1)]
                    .into_iter()
                    .flatten()
                    .map(|j| j as u32)
                    .collect()
            })
            .collect()
    }
    fn cycle(n: usize) -> Vec<Vec<u32>> {
        (0..n)
            .map(|i| vec![((i + n - 1) % n) as u32, ((i + 1) % n) as u32])
            .collect()
    }
    fn complete(n: usize) -> Vec<Vec<u32>> {
        (0..n)
            .map(|i| (0..n as u32).filter(|&j| j as usize != i).collect())
            .collect()
    }
    fn wheel(n: usize) -> Vec<Vec<u32>> {
        // A hub (node n) joined to every node of an n-cycle.
        let mut g = cycle(n);
        for ns in g.iter_mut() {
            ns.push(n as u32);
        }
        g.push((0..n as u32).collect());
        g
    }
    const RULES: Rules = Rules {
        noise: 0.1,
        errant: 0.041,
        reaction: 1.59,
        bot_interval: 1.5,
        time_limit: 300.0,
    };

    #[test]
    fn colorings_match_the_deletion_contraction_closed_forms() {
        for n in 1..=8u32 {
            assert_eq!(
                colorings(&path(n as usize), 3),
                3 * 2u64.pow(n - 1),
                "path {n}"
            );
        }
        for n in 3..=8u32 {
            let want = 2u64.pow(n) as i64 + if n % 2 == 0 { 2 } else { -2 };
            assert_eq!(colorings(&cycle(n as usize), 3) as i64, want, "cycle {n}");
            assert_eq!(
                colorings(&wheel(n as usize), 3),
                if n % 2 == 0 { 6 } else { 0 },
                "wheel {n}"
            );
        }
        assert_eq!(
            (colorings(&complete(3), 3), colorings(&complete(4), 3)),
            (6, 0)
        );
    }

    #[test]
    fn barabasi_albert_graphs_from_a_triangle_are_always_three_colorable() {
        let mut rng = seeded(1);
        for _ in 0..50 {
            let n = colorings(&barabasi_albert(20, 2, &mut rng), 3);
            assert!((6..=6 * 2u64.pow(17)).contains(&n), "{n}");
        }
    }

    #[test]
    fn the_least_common_rule_keeps_a_least_common_color_and_breaks_ties_at_random() {
        let mut rng = seeded(2);
        assert_eq!(greedy([0, 1, 1], 0, &mut rng), 0);
        assert_eq!(
            greedy([2, 1, 1], 1, &mut rng),
            1,
            "tied for least common: keep"
        );
        assert_eq!(greedy([2, 0, 1], 0, &mut rng), 1);
        let mut seen = [0; 3];
        for _ in 0..10_000 {
            seen[greedy([1, 0, 0], 0, &mut rng) as usize] += 1;
        }
        assert_eq!(seen[0], 0);
        assert!((4700..5300).contains(&seen[1]), "{seen:?}");
    }

    #[test]
    fn an_errant_move_raises_conflicts_when_it_can() {
        let mut rng = seeded(3);
        for _ in 0..100 {
            assert!([1, 2].contains(&errant([0, 1, 2], 0, &mut rng)));
            assert_eq!(
                errant([0, 1, 2], 2, &mut rng),
                2,
                "nothing is worse than the worst"
            );
        }
        assert_eq!(errant([1, 1, 1], 0, &mut rng), 0);
    }

    #[test]
    fn waits_have_the_given_mean() {
        let mut rng = seeded(4);
        let mean = (0..100_000).map(|_| wait(1.59, &mut rng)).sum::<f64>() / 1e5;
        assert!((mean - 1.59).abs() < 0.02, "{mean}");
    }

    #[test]
    fn sessions_count_conflicts_and_bots_decide_on_their_clock() {
        let mut rng = seeded(5);
        let adj = barabasi_albert(20, 2, &mut rng);
        let mut bot = vec![false; 20];
        bot[0] = true;
        let mut s = Session::new(adj.clone(), bot, RULES, &mut rng);
        assert_eq!(s.conflicts, conflicts_of(&s.adj, &s.color));
        assert!(
            s.next[0] < 1.5,
            "a bot's first decision falls in its first interval"
        );
        let first = s.next[0];
        while s.time < first && !s.finished() {
            let t = s.time + 0.01;
            s.advance_to(t, &mut rng);
        }
        if !s.finished() {
            assert!(
                (s.next[0] - (first + 1.5)).abs() < 1e-12,
                "then every 1.5 s"
            );
        }
        assert_eq!(s.conflicts, conflicts_of(&s.adj, &s.color));
    }

    #[test]
    fn a_finished_session_stays_finished() {
        let mut rng = seeded(6);
        // No edges: properly colored at the start.
        let mut empty = Session::new(vec![vec![]; 3], vec![false; 3], RULES, &mut rng);
        assert_eq!(empty.solved_at, Some(0.0));
        empty.advance_to(100.0, &mut rng);
        assert_eq!((empty.time, empty.decisions), (0.0, 0));
        // K4 cannot be 3-colored: the session runs to the time limit and holds there.
        let mut k4 = Session::new(
            complete(4),
            vec![false; 4],
            Rules {
                time_limit: 30.0,
                ..RULES
            },
            &mut rng,
        );
        assert_eq!(k4.play(&mut rng), None);
        assert!(k4.finished() && k4.time == 30.0);
        let (decisions, color) = (k4.decisions, k4.color.clone());
        k4.advance_to(1e9, &mut rng);
        assert_eq!(
            (k4.decisions, k4.color.clone(), k4.time),
            (decisions, color, 30.0)
        );
        // A solvable session stops at its solve time.
        let mut p = Session::new(path(6), vec![false; 6], RULES, &mut rng);
        let at = p.play(&mut rng).expect("a path solves within 300 s");
        assert!(p.conflicts == 0 && p.time == at && p.finished());
        p.advance_to(1e9, &mut rng);
        assert_eq!(p.time, at);
    }

    #[test]
    fn the_extended_data_fig_2_recolorer_converges_or_is_censored() {
        let mut rng = seeded(7);
        let steps = convergence_steps(&path(5), &[0, 0, 0, 0, 0], 1_000_000, &mut rng);
        assert!(steps.is_some_and(|s| s > 0));
        assert_eq!(
            convergence_steps(&path(3), &[0, 1, 0], 10, &mut rng),
            Some(0),
            "already proper"
        );
        assert_eq!(
            convergence_steps(&complete(4), &[0, 1, 2, 0], 1000, &mut rng),
            None
        );
    }

    // The tests below were added beyond the brief: each targets a bug the brief's tests let by.

    #[test]
    fn conflicts_of_counts_each_equal_edge_once() {
        assert_eq!(conflicts_of(&path(3), &[0, 0, 1]), 1);
        assert_eq!(conflicts_of(&path(3), &[1, 1, 1]), 2);
        assert_eq!(conflicts_of(&cycle(4), &[0, 0, 0, 0]), 4);
        assert_eq!(conflicts_of(&complete(3), &[0, 0, 1]), 1);
        assert_eq!(conflicts_of(&complete(4), &[0, 1, 2, 0]), 1);
        assert_eq!(counts(&complete(4), &[0, 1, 2, 0], 1), [2, 0, 1]);
    }

    #[test]
    fn the_running_conflict_count_never_drifts_from_the_recount() {
        for seed in 0..5 {
            let mut rng = seeded(100 + seed);
            let adj = barabasi_albert(20, 2, &mut rng);
            let bot = (0..20).map(|i| i % 3 == 0).collect();
            let mut s = Session::new(
                adj,
                bot,
                Rules {
                    noise: 0.5,
                    errant: 0.5,
                    ..RULES
                },
                &mut rng,
            );
            for k in 1..=200 {
                s.advance_to(0.25 * k as f64, &mut rng);
                assert_eq!(
                    s.conflicts,
                    conflicts_of(&s.adj, &s.color),
                    "seed {seed} step {k}"
                );
                assert_eq!(s.solved_at.is_some(), s.conflicts == 0);
            }
        }
    }

    #[test]
    fn noise_belongs_to_bots_and_errant_moves_to_humans() {
        // Two nodes on one edge, node 0 colored 0 beside a neighbor colored 1: staying (0) is
        // best, 1 is the only worse color.
        let run = |bot: bool, noise: f64, errant: f64| {
            let mut rng = seeded(8);
            let rules = Rules {
                noise,
                errant,
                ..RULES
            };
            let mut s = Session::new(path(2), vec![bot; 2], rules, &mut rng);
            let mut moved = [0usize; 3];
            for _ in 0..3000 {
                s.color = vec![0, 1];
                s.conflicts = 0;
                s.decide(0, &mut rng);
                moved[s.color[0] as usize] += 1;
            }
            moved
        };
        // A bot with no noise is greedy; errant is ignored by bots.
        assert_eq!(run(true, 0.0, 1.0), [3000, 0, 0]);
        // A bot with full noise is uniform over all three colors.
        assert!(run(true, 1.0, 0.0)
            .iter()
            .all(|&m| (900..1100).contains(&m)));
        // A human ignores noise ...
        assert_eq!(run(false, 1.0, 0.0), [3000, 0, 0]);
        // ... and, when always errant, always takes the worse color.
        assert_eq!(run(false, 0.0, 1.0), [0, 3000, 0]);
    }

    #[test]
    fn bots_keep_their_interval_and_humans_wait_exponentially() {
        let mut rng = seeded(9);
        // K4 is never solved, so the clock runs the whole time.
        let mut s = Session::new(
            complete(4),
            vec![true, false, false, false],
            RULES,
            &mut rng,
        );
        let first = s.next[0];
        let before = s.next[1];
        s.advance_to(10.0, &mut rng);
        let k = (s.next[0] - first) / 1.5;
        assert!((k - k.round()).abs() < 1e-9 && k >= 6.0, "{k}");
        assert!(s.next[0] > 10.0 && s.next[0] - 1.5 <= 10.0, "{}", s.next[0]);
        assert_eq!(s.time, 10.0);
        assert!(s.next[1] > before, "a human who decided has waited again");
    }

    #[test]
    fn simultaneous_decisions_go_to_the_lower_index() {
        let mut rng = seeded(10);
        let mut s = Session::new(
            path(2),
            vec![true; 2],
            Rules {
                noise: 0.0,
                ..RULES
            },
            &mut rng,
        );
        s.color = vec![0, 0];
        s.conflicts = 1;
        s.solved_at = None;
        s.next = vec![1.0, 1.0];
        s.advance_to(1.0, &mut rng);
        assert_eq!((s.decisions, s.color[1], s.solved_at), (1, 0, Some(1.0)));
        assert_ne!(s.color[0], 0);
    }

    #[test]
    fn a_recoloring_never_repeats_the_neighbors_color() {
        // On one edge the recolored node always has the other as its neighbor, so one step solves.
        for seed in 0..50 {
            let mut rng = seeded(200 + seed);
            assert_eq!(convergence_steps(&path(2), &[1, 1], 100, &mut rng), Some(1));
        }
    }
}
