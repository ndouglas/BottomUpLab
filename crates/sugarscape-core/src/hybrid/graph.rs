//! The structures the hybrid worlds use, on top of `crate::graph`: square lattices (von Neumann
//! or Moore, periodic or open), Erdős–Rényi with a target mean degree, and bot placement (random,
//! hubs or leaves, or one Bernoulli draw per node as in Si et al.'s code).

use crate::{graph::gnp_sparse, rng::SimRng};
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Neighborhood {
    VonNeumann,
    Moore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Placement {
    Random,
    Hubs,
    Leaves,
}

/// A side × side lattice, node i at (i mod side, i div side). Neighbors in Si et al.'s order:
/// up, down, left, right, then (Moore) up-left, up-right, down-left, down-right. Open boundaries
/// drop links that would leave the square; a side of at least 3 keeps periodic neighbors distinct.
pub fn lattice(side: usize, hood: Neighborhood, periodic: bool) -> Vec<Vec<u32>> {
    let offsets: &[(i64, i64)] = match hood {
        Neighborhood::VonNeumann => &[(0, -1), (0, 1), (-1, 0), (1, 0)],
        Neighborhood::Moore => &[
            (0, -1),
            (0, 1),
            (-1, 0),
            (1, 0),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ],
    };
    let s = side as i64;
    (0..side * side)
        .map(|i| {
            let (x, y) = ((i % side) as i64, (i / side) as i64);
            offsets
                .iter()
                .filter_map(|&(dx, dy)| {
                    let (nx, ny) = (x + dx, y + dy);
                    if periodic {
                        Some((nx.rem_euclid(s) + ny.rem_euclid(s) * s) as u32)
                    } else if (0..s).contains(&nx) && (0..s).contains(&ny) {
                        Some((nx + ny * s) as u32)
                    } else {
                        None
                    }
                })
                .collect()
        })
        .collect()
}

/// Erdős–Rényi G(n, p) with p = mean_degree / (n − 1), drawn in time proportional to its links.
pub fn erdos_renyi(n: usize, mean_degree: f64, rng: &mut SimRng) -> Vec<Vec<u32>> {
    gnp_sparse(n, (mean_degree / (n - 1) as f64).min(1.0), rng)
}

/// `count` bots as a flag per node. Hubs: highest degree first; leaves: lowest first; ties to the
/// lower index. Random: a uniform sample without replacement (a partial Fisher–Yates shuffle).
pub fn place(degrees: &[usize], count: usize, placement: Placement, rng: &mut SimRng) -> Vec<bool> {
    let n = degrees.len();
    let count = count.min(n);
    let mut order: Vec<usize> = (0..n).collect();
    match placement {
        Placement::Random => {
            for k in 0..count {
                let j = k + rng.gen_range(0..(n - k) as u32) as usize;
                order.swap(k, j);
            }
        }
        Placement::Hubs => order.sort_by(|&a, &b| degrees[b].cmp(&degrees[a]).then(a.cmp(&b))),
        Placement::Leaves => order.sort_by(|&a, &b| degrees[a].cmp(&degrees[b]).then(a.cmp(&b))),
    }
    let mut bot = vec![false; n];
    for &i in &order[..count] {
        bot[i] = true;
    }
    bot
}

/// Each node a bot with probability p: one uniform draw per node in index order (Si et al.'s
/// `randf() < rho`).
pub fn bernoulli(n: usize, p: f64, rng: &mut SimRng) -> Vec<bool> {
    (0..n).map(|_| rng.gen::<f64>() < p).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{graph::barabasi_albert, rng::seeded};

    #[test]
    fn periodic_lattices_have_full_degree_and_wrap() {
        let vn = lattice(5, Neighborhood::VonNeumann, true);
        assert!(vn.iter().all(|n| n.len() == 4));
        assert_eq!(
            vn[0],
            vec![20, 5, 4, 1],
            "up wraps to the bottom row, left to the last column"
        );
        let moore = lattice(5, Neighborhood::Moore, true);
        assert!(moore.iter().all(|n| n.len() == 8));
        assert_eq!(&moore[0][4..], &[24, 21, 9, 6]);
        for (i, ns) in moore.iter().enumerate() {
            for &j in ns {
                assert!(
                    moore[j as usize].contains(&(i as u32)),
                    "symmetric at {i}–{j}"
                );
            }
        }
    }

    #[test]
    fn open_lattices_lose_the_edge_links() {
        let g = lattice(50, Neighborhood::VonNeumann, false);
        let degrees: Vec<usize> = g.iter().map(Vec::len).collect();
        assert_eq!((degrees[0], degrees[1], degrees[51]), (2, 3, 4));
        assert_eq!(degrees.iter().sum::<usize>(), 2 * 2 * 50 * 49);
    }

    #[test]
    fn random_graphs_have_the_target_mean_degree() {
        let mut rng = seeded(1);
        let er = erdos_renyi(10_000, 4.0, &mut rng);
        let mean = er.iter().map(Vec::len).sum::<usize>() as f64 / 10_000.0;
        assert!((mean - 4.0).abs() < 0.1, "{mean}");
        let ba = barabasi_albert(1000, 2, &mut rng);
        assert_eq!(
            ba.iter().map(Vec::len).sum::<usize>() / 2,
            3 + 2 * 997,
            "a triangle, then two links per node"
        );
    }

    #[test]
    fn placement_orders_by_degree_with_ties_by_index() {
        let degrees = [2, 5, 5, 1, 3, 1];
        let mut rng = seeded(1);
        assert_eq!(
            place(&degrees, 3, Placement::Hubs, &mut rng),
            vec![false, true, true, false, true, false]
        );
        assert_eq!(
            place(&degrees, 3, Placement::Leaves, &mut rng),
            vec![true, false, false, true, false, true]
        );
        assert_eq!(
            place(&degrees, 4, Placement::Random, &mut rng)
                .iter()
                .filter(|&&b| b)
                .count(),
            4
        );
        assert_eq!(
            place(&degrees, 9, Placement::Random, &mut rng),
            vec![true; 6],
            "at most every node"
        );
        let b = bernoulli(100_000, 0.3, &mut seeded(2));
        let share = b.iter().filter(|&&x| x).count() as f64 / 1e5;
        assert!((share - 0.3).abs() < 0.01, "{share}");
    }
}
