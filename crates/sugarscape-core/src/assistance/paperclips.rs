//! The paperclip game's closed forms. Pure functions of a rule table and a map; no RNG.
//! H makes two items, R deploys 50/50 or 90 of one; reward θ·p + (1−θ)·q, θ uniform on [0, 1].
//! A rule is H's action per θ-bin (bin k covers [k/bins, (k+1)/bins) and is evaluated at its
//! midpoint); 92 bins make the paper's thresholds 1/2, 41/92 and 51/92 bin edges.
use std::cmp::Ordering;

pub const H_ACTIONS: [(u32, u32); 3] = [(0, 2), (1, 1), (2, 0)];
pub const R_ACTIONS: [(u32, u32); 3] = [(0, 90), (50, 50), (90, 0)];
pub type Rule = Vec<u8>;
pub type Map = [u8; 3];
type Joint = (Map, Vec<(Ratio, Ratio, u8)>, Ratio);

/// A rational number in lowest terms with a positive denominator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ratio {
    pub num: i64,
    pub den: i64,
}

fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a.abs()
    } else {
        gcd(b, a % b)
    }
}

impl Ratio {
    pub fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "zero denominator");
        let g = gcd(num, den).max(1);
        let sign = if den < 0 { -1 } else { 1 };
        Ratio {
            num: sign * num / g,
            den: sign * den / g,
        }
    }
    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }
    fn add(self, o: Ratio) -> Ratio {
        Ratio::new(self.num * o.den + o.num * self.den, self.den * o.den)
    }
    fn sub(self, o: Ratio) -> Ratio {
        Ratio::new(self.num * o.den - o.num * self.den, self.den * o.den)
    }
    fn mul(self, o: Ratio) -> Ratio {
        Ratio::new(self.num * o.num, self.den * o.den)
    }
    fn div(self, o: Ratio) -> Ratio {
        Ratio::new(self.num * o.den, self.den * o.num)
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.cmp(o))
    }
}

impl Ord for Ratio {
    fn cmp(&self, o: &Self) -> Ordering {
        (i128::from(self.num) * i128::from(o.den)).cmp(&(i128::from(o.num) * i128::from(self.den)))
    }
}

pub fn reward(theta: f64, (p, q): (u32, u32)) -> f64 {
    theta * f64::from(p) + (1.0 - theta) * f64::from(q)
}

pub fn bin_mid(bin: usize, bins: usize) -> f64 {
    (bin as f64 + 0.5) / bins as f64
}

/// The smallest index of the largest value.
pub(super) fn argmax(values: impl IntoIterator<Item = f64>) -> u8 {
    let mut best = (0u8, f64::NEG_INFINITY);
    for (i, v) in values.into_iter().enumerate() {
        if v > best.1 {
            best = (i as u8, v);
        }
    }
    best.0
}

/// The demonstration-by-expert rule: the immediate reward's maximizer (appendix eq. 2).
pub fn expert_rule(bins: usize) -> Rule {
    (0..bins)
        .map(|k| argmax(H_ACTIONS.iter().map(|&a| reward(bin_mid(k, bins), a))))
        .collect()
}

/// (0,2) below `low`, (1,1) from `low` to `high` inclusive, (2,0) above `high`, at bin midpoints.
pub fn threshold_rule(bins: usize, low: Ratio, high: Ratio) -> Rule {
    (0..bins)
        .map(|k| {
            let mid = Ratio::new(2 * k as i64 + 1, 2 * bins as i64);
            if mid < low {
                0
            } else if mid <= high {
                1
            } else {
                2
            }
        })
        .collect()
}

/// The paper's equilibrium rule (appendix, proof of Theorem 3).
pub fn equilibrium_rule(bins: usize) -> Rule {
    threshold_rule(bins, Ratio::new(41, 92), Ratio::new(51, 92))
}

pub fn pooling_rule(bins: usize) -> Rule {
    vec![1; bins]
}

/// R's posterior over bins after seeing `action` under `rule`: uniform on the bins choosing it,
/// or the uniform prior when no bin does (an off-path action).
pub fn posterior(rule: &[u8], action: u8) -> Vec<f64> {
    let n = rule.iter().filter(|&&a| a == action).count();
    if n == 0 {
        return vec![1.0 / rule.len() as f64; rule.len()];
    }
    rule.iter()
        .map(|&a| if a == action { 1.0 / n as f64 } else { 0.0 })
        .collect()
}

pub fn posterior_mean(post: &[f64]) -> f64 {
    let bins = post.len();
    post.iter()
        .enumerate()
        .map(|(k, p)| p * bin_mid(k, bins))
        .sum()
}

/// The deployment maximizing reward at the mean θ (Theorem 2 of the 2016 paper).
pub fn deployment_for(theta_bar: f64) -> u8 {
    argmax(R_ACTIONS.iter().map(|&a| reward(theta_bar, a)))
}

pub fn robot_best_reply(rule: &[u8]) -> Map {
    let mut map = [0u8; 3];
    for (a, slot) in map.iter_mut().enumerate() {
        *slot = deployment_for(posterior_mean(&posterior(rule, a as u8)));
    }
    map
}

/// H's two-round payoff from `action` when R answers by `map`.
pub fn human_payoff(theta: f64, action: usize, map: &Map) -> f64 {
    reward(theta, H_ACTIONS[action]) + reward(theta, R_ACTIONS[map[action] as usize])
}

pub fn human_best_reply(map: &Map, bins: usize) -> Rule {
    (0..bins)
        .map(|k| argmax((0..3).map(|a| human_payoff(bin_mid(k, bins), a, map))))
        .collect()
}

/// H's payoff line for `action`: total paperclips `A` and staples `B`, worth `A·θ + B·(1 − θ)`.
fn line(action: usize, map: &Map) -> (i64, i64) {
    let (p, q) = H_ACTIONS[action];
    let (pp, qq) = R_ACTIONS[map[action] as usize];
    (i64::from(p + pp), i64::from(q + qq))
}

fn line_at(l: (i64, i64), theta: Ratio) -> Ratio {
    Ratio::new(l.0, 1)
        .mul(theta)
        .add(Ratio::new(l.1, 1).mul(Ratio::new(1, 1).sub(theta)))
}

/// H's pointwise best reply over [0, 1] as exact intervals: breakpoints are the pairwise
/// crossings of the three payoff lines inside (0, 1); each piece takes the argmax at its
/// midpoint (ties to the smaller index); adjacent pieces with one action merge.
pub fn exact_best_reply(map: &Map) -> Vec<(Ratio, Ratio, u8)> {
    let lines: Vec<(i64, i64)> = (0..3).map(|a| line(a, map)).collect();
    let mut points = vec![Ratio::new(0, 1), Ratio::new(1, 1)];
    for a in 0..3 {
        for b in a + 1..3 {
            // A_a θ + B_a (1−θ) = A_b θ + B_b (1−θ)  ⇒  θ = (B_b − B_a) / ((A_a − B_a) − (A_b − B_b)).
            let den = (lines[a].0 - lines[a].1) - (lines[b].0 - lines[b].1);
            if den != 0 {
                let theta = Ratio::new(lines[b].1 - lines[a].1, den);
                if theta > Ratio::new(0, 1) && theta < Ratio::new(1, 1) {
                    points.push(theta);
                }
            }
        }
    }
    points.sort();
    points.dedup();
    let mut out: Vec<(Ratio, Ratio, u8)> = vec![];
    for w in points.windows(2) {
        let mid = w[0].add(w[1]).div(Ratio::new(2, 1));
        let mut best = (0u8, line_at(lines[0], mid));
        for (a, l) in lines.iter().enumerate().skip(1) {
            let v = line_at(*l, mid);
            if v > best.1 {
                best = (a as u8, v);
            }
        }
        match out.last_mut() {
            Some(last) if last.2 == best.0 => last.1 = w[1],
            _ => out.push((w[0], w[1], best.0)),
        }
    }
    out
}

/// ∫₀¹ max_a payoff_a(θ) dθ, exactly: each piece is ∫ (A θ + B (1−θ)) = (A − B)(hi² − lo²)/2 + B (hi − lo).
pub fn expected_total(map: &Map) -> Ratio {
    let mut total = Ratio::new(0, 1);
    for (lo, hi, a) in exact_best_reply(map) {
        let (aa, bb) = line(a as usize, map);
        let slope = Ratio::new(aa - bb, 2).mul(hi.mul(hi).sub(lo.mul(lo)));
        let flat = Ratio::new(bb, 1).mul(hi.sub(lo));
        total = total.add(slope).add(flat);
    }
    total
}

/// The map (and H's exact best reply to it) with the highest expected total reward over R's 27
/// maps; ties keep the first in lexicographic order. The paper computed this "exhaustively".
pub fn optimal_joint() -> (Map, Vec<(Ratio, Ratio, u8)>, Ratio) {
    let mut best: Option<Joint> = None;
    for code in 0..27u8 {
        let map = [code / 9, (code / 3) % 3, code % 3];
        let value = expected_total(&map);
        if best.as_ref().is_none_or(|b| value > b.2) {
            best = Some((map, exact_best_reply(&map), value));
        }
    }
    best.unwrap()
}

/// Every map whose best reply's best reply is itself, with the rule it induces.
pub fn mutual_best_responses(bins: usize) -> Vec<(Rule, Map)> {
    (0..27u8)
        .map(|code| [code / 9, (code / 3) % 3, code % 3])
        .filter_map(|map| {
            let rule = human_best_reply(&map, bins);
            (robot_best_reply(&rule) == map).then_some((rule, map))
        })
        .collect()
}

/// A rule read as thresholds: the left edge of the first bin not making (0,2), and of the first
/// making (2,0); 1 when there is none. Equal to the exact thresholds for threshold rules.
pub fn thresholds_of(rule: &[u8]) -> (f64, f64) {
    let bins = rule.len() as f64;
    let low = rule.iter().position(|&a| a != 0).unwrap_or(rule.len()) as f64 / bins;
    let high = rule.iter().position(|&a| a == 2).unwrap_or(rule.len()) as f64 / bins;
    (low, high)
}

/// Bins and map entries differing from the paper's equilibrium pair; 0 at the pair itself.
pub fn distance(rule: &[u8], map: &Map) -> u32 {
    let target = equilibrium_rule(rule.len());
    let bins = rule.iter().zip(&target).filter(|(a, b)| a != b).count();
    let entries = map.iter().zip(&[0u8, 1, 2]).filter(|(a, b)| a != b).count();
    (bins + entries) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_expert_rule_splits_at_one_half_and_its_best_reply_is_the_papers_equation_3() {
        let rule = expert_rule(92);
        assert!(rule[..46].iter().all(|&a| a == 0) && rule[46..].iter().all(|&a| a == 2));
        assert_eq!(robot_best_reply(&rule), [0, 1, 2]);
    }

    #[test]
    fn posteriors_after_two_staples_have_the_papers_means() {
        // Under the expert rule the posterior is uniform on [0, 1/2): R acts as θ = 1/4.
        let m = posterior_mean(&posterior(&expert_rule(92), 0));
        assert!((m - 0.25).abs() < 1e-12, "{m}");
        // Under cuts at 1/3 and 2/3 (96 bins make them edges) it is uniform on [0, 1/3): θ = 1/6.
        let rule = threshold_rule(96, Ratio::new(1, 3), Ratio::new(2, 3));
        let m = posterior_mean(&posterior(&rule, 0));
        assert!((m - 1.0 / 6.0).abs() < 1e-12, "{m}");
        assert_eq!(deployment_for(0.25), 0);
        assert_eq!(deployment_for(1.0 / 6.0), 0);
    }

    #[test]
    fn off_path_actions_resolve_to_the_prior() {
        let post = posterior(&expert_rule(92), 1);
        assert!(post.iter().all(|&p| (p - 1.0 / 92.0).abs() < 1e-15));
        assert_eq!(deployment_for(posterior_mean(&post)), 1);
    }

    #[test]
    fn the_teaching_thresholds_are_41_92_and_51_92_exactly() {
        assert_eq!(
            exact_best_reply(&[0, 1, 2]),
            vec![
                (Ratio::new(0, 1), Ratio::new(41, 92), 0),
                (Ratio::new(41, 92), Ratio::new(51, 92), 1),
                (Ratio::new(51, 92), Ratio::new(1, 1), 2),
            ]
        );
    }

    #[test]
    fn theorem_3_the_best_reply_to_the_best_reply_is_not_the_expert_rule() {
        let twice = human_best_reply(&robot_best_reply(&expert_rule(92)), 92);
        assert_ne!(twice, expert_rule(92));
        assert_eq!(twice, equilibrium_rule(92));
        // θ = 0.49: 92(1 − θ) = 46.92, 51, 92θ = 45.08, so H makes one of each.
        let payoffs: Vec<f64> = (0..3).map(|a| human_payoff(0.49, a, &[0, 1, 2])).collect();
        assert!(
            (payoffs[0] - 46.92).abs() < 1e-12
                && payoffs[1] == 51.0
                && (payoffs[2] - 45.08).abs() < 1e-12
        );
        assert_eq!(twice[(0.49 * 92.0) as usize], 1);
    }

    #[test]
    fn the_optimal_joint_policy_is_the_teaching_pair() {
        let (map, rule, value) = optimal_joint();
        assert_eq!(map, [0, 1, 2]);
        assert_eq!(rule, exact_best_reply(&[0, 1, 2]));
        // ∫ max(92(1−θ), 51, 92θ) dθ = 6373/92 ≈ 69.27, against 69 for the expert pair.
        assert_eq!(value, Ratio::new(6373, 92));
        assert_eq!(expected_total(&[0, 1, 2]), Ratio::new(6373, 92));
    }

    #[test]
    fn pooling_is_not_a_mutual_best_response_but_the_teaching_pair_is() {
        assert_eq!(robot_best_reply(&pooling_rule(92)), [1, 1, 1]);
        assert_eq!(human_best_reply(&[1, 1, 1], 92), expert_rule(92));
        let set = mutual_best_responses(92);
        assert!(set.contains(&(equilibrium_rule(92), [0, 1, 2])));
        assert!(!set.contains(&(pooling_rule(92), [1, 1, 1])));
    }

    #[test]
    fn thresholds_and_distance_read_a_rule_table() {
        assert_eq!(thresholds_of(&expert_rule(92)), (0.5, 0.5));
        let (lo, hi) = thresholds_of(&equilibrium_rule(92));
        assert!((lo - 41.0 / 92.0).abs() < 1e-15 && (hi - 51.0 / 92.0).abs() < 1e-15);
        assert_eq!(distance(&equilibrium_rule(92), &[0, 1, 2]), 0);
        assert_eq!(distance(&expert_rule(92), &[0, 1, 2]), 10);
        assert_eq!(distance(&expert_rule(92), &[1, 1, 1]), 12);
    }

    #[test]
    fn ratios_normalize_and_order() {
        assert_eq!(Ratio::new(82, 184), Ratio::new(41, 92));
        assert_eq!(Ratio::new(-3, -6), Ratio::new(1, 2));
        assert!(Ratio::new(41, 92) < Ratio::new(1, 2) && Ratio::new(1, 2) < Ratio::new(51, 92));
    }
}
