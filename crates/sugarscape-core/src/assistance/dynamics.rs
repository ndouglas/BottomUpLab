//! Sampled episodes of both games and the two paperclip dynamics (ours): iterated best response
//! uses `paperclips`' exact replies; fictitious play keeps smoothed frequency tables. Every draw
//! comes from the shared stream as an `f64` or a `u32` range.
use super::offswitch::{allow_probability, choice, incentive, Belief, Choice, Incentive};
use super::paperclips::{argmax, bin_mid, deployment_for, reward, Map, Rule, H_ACTIONS, R_ACTIONS};
use crate::{portable, rng::SimRng};
use rand::Rng;
use serde::Serialize;

/// A standard normal by the Box–Muller transform in polar form: two uniforms on (−1, 1),
/// accepted inside the unit disc, then `u·sqrt(−2 ln s / s)`; `sqrt` and `portable::ln` only.
pub fn standard_normal(rng: &mut SimRng) -> f64 {
    loop {
        let u = 2.0 * rng.gen::<f64>() - 1.0;
        let v = 2.0 * rng.gen::<f64>() - 1.0;
        let s = u * u + v * v;
        if s > 0.0 && s < 1.0 {
            return u * (-2.0 * portable::ln(s) / s).sqrt();
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaperclipEpisode {
    pub theta: f64,
    pub bin: usize,
    pub human: u8,
    pub robot: u8,
    pub reward: f64,
    pub regret: f64,
}

/// One game: θ uniform, H by its rule's bin, R by its map; both rounds' reward and R's regret
/// against the θ-optimal deployment.
pub fn paperclip_episode(rng: &mut SimRng, rule: &[u8], map: &Map) -> PaperclipEpisode {
    let bins = rule.len();
    let theta = rng.gen::<f64>();
    let bin = ((theta * bins as f64) as usize).min(bins - 1);
    let human = rule[bin];
    let robot = map[human as usize];
    let deployed = reward(theta, R_ACTIONS[robot as usize]);
    let best = R_ACTIONS
        .iter()
        .map(|&a| reward(theta, a))
        .fold(f64::NEG_INFINITY, f64::max);
    PaperclipEpisode {
        theta,
        bin,
        human,
        robot,
        reward: reward(theta, H_ACTIONS[human as usize]) + deployed,
        regret: best - deployed,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OffSwitchEpisode {
    pub u: f64,
    pub allowed: bool,
    pub choice: Choice,
    pub value: f64,
    pub incentive: Incentive,
}

fn resolve(u: f64, allowed: bool, incentive: Incentive) -> OffSwitchEpisode {
    let choice = choice(&incentive);
    let value = match choice {
        Choice::Act => u,
        Choice::Wait if allowed => u,
        _ => 0.0,
    };
    OffSwitchEpisode {
        u,
        allowed,
        choice,
        value,
        incentive,
    }
}

/// The paper's game: U_a from the belief (which is the prior), H's coin against π^H(U_a; β);
/// the coin is drawn for the rational H too, so the draw budget does not depend on β.
pub fn prior_episode(
    rng: &mut SimRng,
    belief: Belief,
    beta: f64,
    incentive: &Incentive,
) -> OffSwitchEpisode {
    let u = belief.mu + belief.sigma * standard_normal(rng);
    let coin = rng.gen::<f64>();
    resolve(u, coin < allow_probability(u, beta), *incentive)
}

pub fn posterior_mean(report: f64, noise: f64) -> f64 {
    report / (1.0 + noise * noise)
}

pub fn posterior_sd(noise: f64) -> f64 {
    noise / (1.0 + noise * noise).sqrt()
}

/// Figure 4's reconstruction (ours): `actions` candidates with U ~ N(0, 1) and reports
/// U + N(0, noise²); R takes the candidate with the highest posterior mean (ties to the first),
/// believes N(mean, sigma²) and plays the game on it.
pub fn report_episode(
    rng: &mut SimRng,
    sigma: f64,
    beta: f64,
    noise: f64,
    actions: u32,
    density: u32,
) -> OffSwitchEpisode {
    let mut best = (f64::NEG_INFINITY, 0.0);
    for _ in 0..actions {
        let u = standard_normal(rng);
        let report = u + noise * standard_normal(rng);
        let mean = posterior_mean(report, noise);
        if mean > best.0 {
            best = (mean, u);
        }
    }
    let (mean, u) = best;
    let incentive = incentive(Belief { mu: mean, sigma }, beta, density);
    let coin = rng.gen::<f64>();
    resolve(u, coin < allow_probability(u, beta), incentive)
}

/// Fictitious play's frequency tables: H's over R's deployments after each of H's actions;
/// R's over H's actions in each θ-bin (θ is revealed to R after deployment, a modeling choice).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Tables {
    pub human: [[f64; 3]; 3],
    pub robot: Vec<[f64; 3]>,
}

impl Tables {
    pub fn uniform(bins: usize) -> Self {
        Tables {
            human: [[1.0; 3]; 3],
            robot: vec![[1.0; 3]; bins],
        }
    }
    pub fn observe(&mut self, bin: usize, human: u8, robot: u8, weight: f64) {
        self.human[human as usize][robot as usize] += weight;
        self.robot[bin][human as usize] += weight;
    }
}

/// H's best reply to its table: per bin, immediate reward plus the frequency-weighted deployment.
pub fn human_reply_to_tables(t: &Tables) -> Rule {
    let bins = t.robot.len();
    (0..bins)
        .map(|k| {
            let theta = bin_mid(k, bins);
            argmax((0..3).map(|a| {
                let total: f64 = t.human[a].iter().sum();
                let expected: f64 = (0..3)
                    .map(|r| t.human[a][r] / total * reward(theta, R_ACTIONS[r]))
                    .sum();
                reward(theta, H_ACTIONS[a]) + expected
            }))
        })
        .collect()
}

/// R's posterior over bins after `action`: the uniform prior times each bin's frequency of it.
pub fn robot_posterior_from_tables(t: &Tables, action: u8) -> Vec<f64> {
    let weights: Vec<f64> = t
        .robot
        .iter()
        .map(|row| row[action as usize] / row.iter().sum::<f64>())
        .collect();
    let total: f64 = weights.iter().sum();
    weights.iter().map(|w| w / total).collect()
}

pub fn robot_reply_to_tables(t: &Tables) -> Map {
    let mut map = [0u8; 3];
    for (a, slot) in map.iter_mut().enumerate() {
        let post = robot_posterior_from_tables(t, a as u8);
        let bins = post.len();
        let mean: f64 = post
            .iter()
            .enumerate()
            .map(|(k, p)| p * bin_mid(k, bins))
            .sum();
        *slot = deployment_for(mean);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assistance::paperclips::{equilibrium_rule, expert_rule};
    use crate::rng;

    #[test]
    fn standard_normals_have_unit_moments_and_use_only_uniform_pairs() {
        let mut r = rng::seeded(3);
        let xs: Vec<f64> = (0..200_000).map(|_| standard_normal(&mut r)).collect();
        let mean = xs.iter().sum::<f64>() / xs.len() as f64;
        let var = xs.iter().map(|x| x * x).sum::<f64>() / xs.len() as f64;
        assert!(
            mean.abs() < 0.01 && (var - 1.0).abs() < 0.02,
            "{mean} {var}"
        );
        // Deterministic: the same seed gives the same draws.
        let (mut a, mut b) = (rng::seeded(9), rng::seeded(9));
        assert_eq!(standard_normal(&mut a), standard_normal(&mut b));
    }

    #[test]
    fn paperclip_episodes_pay_both_rounds_and_measure_regret_against_the_best_deployment() {
        let mut r = rng::seeded(1);
        let e = paperclip_episode(&mut r, &expert_rule(92), &[0, 1, 2]);
        let expected_h = if e.theta < 0.5 {
            2.0 * (1.0 - e.theta)
        } else {
            2.0 * e.theta
        };
        let expected_r = if e.theta < 0.5 {
            90.0 * (1.0 - e.theta)
        } else {
            90.0 * e.theta
        };
        assert!((e.reward - expected_h - expected_r).abs() < 1e-12);
        assert_eq!(e.bin, (e.theta * 92.0) as usize);
        assert_eq!(e.regret, 0.0, "the expert pair deploys the θ-optimal item");
        let e = paperclip_episode(&mut r, &equilibrium_rule(92), &[0, 1, 2]);
        if e.human == 1 {
            let best = (90.0 * e.theta).max(90.0 * (1.0 - e.theta));
            assert!((e.regret - (best - 50.0)).abs() < 1e-12);
        }
    }

    #[test]
    fn draw_budget_is_independent_of_beta() {
        let belief = Belief {
            mu: 0.25,
            sigma: 1.0,
        };
        let rational = incentive(belief, 0.0, 8);
        let noisy = incentive(belief, 2.0, 8);
        let (mut a, mut b) = (rng::seeded(5), rng::seeded(5));
        for _ in 0..50 {
            prior_episode(&mut a, belief, 0.0, &rational);
            prior_episode(&mut b, belief, 2.0, &noisy);
        }
        assert_eq!(a.gen::<f64>(), b.gen::<f64>());
    }

    #[test]
    fn the_rational_human_allows_exactly_the_positive_utilities() {
        let belief = Belief {
            mu: 0.0,
            sigma: 1.0,
        };
        let i = incentive(belief, 0.0, 8);
        let mut r = rng::seeded(2);
        for _ in 0..1000 {
            let e = prior_episode(&mut r, belief, 0.0, &i);
            assert_eq!(e.allowed, e.u >= 0.0);
            assert_eq!(e.choice, Choice::Wait);
            assert_eq!(e.value, if e.allowed { e.u } else { 0.0 });
        }
    }

    #[test]
    fn report_episodes_shrink_the_report_and_pick_the_best_of_k_candidates() {
        assert_eq!(posterior_mean(2.0, 1.0), 1.0);
        assert!((posterior_sd(1.0) - 1.0 / 2f64.sqrt()).abs() < 1e-15);
        let mut r = rng::seeded(4);
        let one: Vec<f64> = (0..2000)
            .map(|_| report_episode(&mut r, 0.7, 0.5, 1.0, 1, 8).u)
            .collect();
        let eight: Vec<f64> = (0..2000)
            .map(|_| report_episode(&mut r, 0.7, 0.5, 1.0, 8, 8).u)
            .collect();
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        assert!(mean(&one).abs() < 0.1, "{}", mean(&one));
        assert!(mean(&eight) > 0.8, "{}", mean(&eight));
    }

    #[test]
    fn uniform_tables_reproduce_the_prior_map_and_observations_move_the_replies() {
        let mut t = Tables::uniform(92);
        assert_eq!(robot_reply_to_tables(&t), [1, 1, 1]);
        assert_eq!(human_reply_to_tables(&t), expert_rule(92));
        // Enough observations of the expert pair and both replies become the paper's.
        for bin in 0..92 {
            let h = expert_rule(92)[bin];
            for _ in 0..100 {
                t.observe(bin, h, [0, 1, 2][h as usize], 1.0);
            }
        }
        assert_eq!(robot_reply_to_tables(&t), [0, 1, 2]);
        assert_eq!(human_reply_to_tables(&t), equilibrium_rule(92));
        let post = robot_posterior_from_tables(&t, 1);
        assert!(
            (post.iter().sum::<f64>() - 1.0).abs() < 1e-12,
            "an unseen action still gets a posterior"
        );
    }
}
