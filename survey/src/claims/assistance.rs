//! Milestone 39: the paperclip game and the off-switch game. Decision rules fixed in
//! docs/superpowers/specs/2026-10-11-assistance-games-design.md before any measurement: exact
//! targets within 1e-9 or exactly; Figure 3's directions with no violation over the fixed grid;
//! Figure 4's sampled directions at 100,000 episodes per point, seed 1, a violation only beyond
//! two standard errors of the difference; the dynamics reported without a target.
use std::sync::OnceLock;

use sugarscape_core::assistance::{
    dynamics,
    offswitch::{self, Belief, Choice, BETAS, SIGMAS},
    paperclips::{self, Ratio},
    AssistanceConfig, Game, HumanRule, Mode, Protocol,
};
use sugarscape_core::model::ModelConfig;

use crate::claim::{all_of, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const CIRL: &str = "Hadfield-Menell, Dragan, Abbeel & Russell 2016, NeurIPS, §3.3 and appendix §6; rules: docs/superpowers/specs/2026-10-11-assistance-games-design.md";
const OFF: &str = "Hadfield-Menell, Dragan, Abbeel & Russell 2017, IJCAI, §§3–5 and Figures 3–4; rules: the same spec";
const OURS: &str =
    "Our extension; docs/superpowers/specs/2026-10-11-assistance-games-design.md, Dynamics";

fn exact(holds: bool, measured: String) -> Outcome {
    Outcome {
        verdict: if holds {
            Verdict::Holds
        } else {
            Verdict::Fails
        },
        measured,
        detail: String::new(),
    }
}

fn reported(measured: String) -> Outcome {
    Outcome {
        verdict: Verdict::Untestable,
        measured,
        detail: "reported without a target".into(),
    }
}

fn claim(
    id: &'static str,
    item: &'static str,
    citation: &'static str,
    text: &'static str,
    check: fn(&[u64]) -> Outcome,
) -> Claim {
    Claim {
        id,
        item,
        source: if citation == OURS {
            Source::App
        } else {
            Source::Book
        },
        citation,
        text,
        check,
    }
}

// ---- Figure 4: Monte Carlo at 100,000 episodes per point, seed 1, memoized across claims. ----

const F4_SIGMAS: [f64; 12] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.2, 1.5];
const F4_ACTIONS: [u32; 3] = [1, 4, 8];
const F4_BETA: f64 = 0.5;

#[derive(Clone, Copy, Debug)]
struct Point {
    sigma: f64,
    actions: u32,
    value: f64,
    value_se: f64,
    delta: f64,
    delta_se: f64,
}

fn mean_se(v: &[f64]) -> (f64, f64) {
    let n = v.len() as f64;
    let m = v.iter().sum::<f64>() / n;
    let var = v.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0);
    (m, (var / n).sqrt())
}

fn figure4_point(sigma: f64, actions: u32, beta: f64) -> Point {
    let config = ModelConfig::Assistance(AssistanceConfig {
        game: Game::OffSwitch,
        protocol: Protocol::Report,
        beta,
        sigma,
        observation_noise: 1.0,
        actions,
        episodes_per_tick: 1000,
        ..Default::default()
    });
    let (value, delta) = model_after(&config, &[1], 100, |w| {
        let series = |name: &str| w.model().series(name).unwrap()[1..].to_vec();
        (mean_se(&series("value")), mean_se(&series("delta_exact")))
    })
    .remove(0);
    Point {
        sigma,
        actions,
        value: value.0,
        value_se: value.1,
        delta: delta.0,
        delta_se: delta.1,
    }
}

fn figure4() -> &'static Vec<Point> {
    static POINTS: OnceLock<Vec<Point>> = OnceLock::new();
    POINTS.get_or_init(|| {
        F4_ACTIONS
            .iter()
            .flat_map(|&k| F4_SIGMAS.iter().map(move |&s| figure4_point(s, k, F4_BETA)))
            .collect()
    })
}

/// `a` exceeds `b` by more than two standard errors of the difference.
fn beyond(a: f64, a_se: f64, b: f64, b_se: f64) -> bool {
    a - b > 2.0 * (a_se * a_se + b_se * b_se).sqrt()
}

fn peak_index() -> usize {
    let target = dynamics::posterior_sd(1.0);
    (0..F4_SIGMAS.len())
        .min_by(|&a, &b| {
            (F4_SIGMAS[a] - target)
                .abs()
                .partial_cmp(&(F4_SIGMAS[b] - target).abs())
                .unwrap()
        })
        .unwrap()
}

fn figure4_peak(_: &[u64]) -> Outcome {
    let parts = F4_ACTIONS
        .iter()
        .map(|&k| {
            let pts: Vec<&Point> = figure4().iter().filter(|p| p.actions == k).collect();
            let peak = pts[peak_index()];
            let violators: Vec<String> = pts
                .iter()
                .filter(|p| beyond(p.value, p.value_se, peak.value, peak.value_se))
                .map(|p| format!("σ={} V={:.4}±{:.4}", p.sigma, p.value, p.value_se))
                .collect();
            let curve = pts
                .iter()
                .map(|p| format!("{}:{:.4}", p.sigma, p.value))
                .collect::<Vec<_>>()
                .join(" ");
            (
                format!("|A|={k}"),
                exact(
                    violators.is_empty(),
                    format!(
                        "V at σ={} is {:.4}±{:.4}; violators [{}]; curve {curve}",
                        peak.sigma,
                        peak.value,
                        peak.value_se,
                        violators.join(", ")
                    ),
                ),
            )
        })
        .collect();
    all_of(parts)
}

fn figure4_delta(_: &[u64]) -> Outcome {
    let parts = F4_ACTIONS
        .iter()
        .map(|&k| {
            let pts: Vec<&Point> = figure4().iter().filter(|p| p.actions == k).collect();
            let violations: Vec<String> = pts
                .windows(2)
                .filter(|w| beyond(w[0].delta, w[0].delta_se, w[1].delta, w[1].delta_se))
                .map(|w| format!("σ {}→{}", w[0].sigma, w[1].sigma))
                .collect();
            let curve = pts
                .iter()
                .map(|p| format!("{}:{:.4}", p.sigma, p.delta))
                .collect::<Vec<_>>()
                .join(" ");
            (
                format!("|A|={k}"),
                exact(
                    violations.is_empty(),
                    format!("Δ falls at [{}]; curve {curve}", violations.join(", ")),
                ),
            )
        })
        .collect();
    all_of(parts)
}

fn figure4_actions(_: &[u64]) -> Outcome {
    let drops: Vec<(u32, f64, f64)> = F4_ACTIONS
        .iter()
        .map(|&k| {
            let pts: Vec<&Point> = figure4().iter().filter(|p| p.actions == k).collect();
            let (peak, last) = (pts[peak_index()], pts[pts.len() - 1]);
            (
                k,
                peak.value - last.value,
                (peak.value_se.powi(2) + last.value_se.powi(2)).sqrt(),
            )
        })
        .collect();
    let violations: Vec<String> = drops
        .windows(2)
        .filter(|w| beyond(w[0].1, w[0].2, w[1].1, w[1].2))
        .map(|w| format!("|A| {}→{}", w[0].0, w[1].0))
        .collect();
    exact(
        violations.is_empty(),
        format!(
            "drop from the peak to σ=1.5: {}",
            drops
                .iter()
                .map(|(k, d, se)| format!("|A|={k}: {d:.4}±{se:.4}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )
}

fn figure4_rational_flat(_: &[u64]) -> Outcome {
    let pts: Vec<Point> = [0.1, 0.7, 1.5]
        .iter()
        .map(|&s| figure4_point(s, 1, 0.0))
        .collect();
    let (lo, hi) = (
        pts.iter().map(|p| p.value).fold(f64::INFINITY, f64::min),
        pts.iter()
            .map(|p| p.value)
            .fold(f64::NEG_INFINITY, f64::max),
    );
    reported(format!("rational human: V at σ 0.1, 0.7, 1.5 = {}; spread {:.4} against SE about {:.4}; the robot waits at every σ (Theorem 1), so Figure 4's peak needs a noisy human", pts.iter().map(|p| format!("{:.4}", p.value)).collect::<Vec<_>>().join(", "), hi - lo, pts[0].value_se))
}

// ---- The dynamics: reported, 20 seeds, 200 ticks, four starts. ----

fn equilibria(mode: Mode) -> Outcome {
    let starts = [
        (HumanRule::Expert, "expert"),
        (HumanRule::Equilibrium, "teaching"),
        (HumanRule::Pooling, "pooling"),
        (HumanRule::UniformRandom, "uniform random"),
    ];
    let seeds: Vec<u64> = (1..=20).collect();
    let lines: Vec<String> = starts.iter().map(|&(rule, name)| {
        let config = ModelConfig::Assistance(AssistanceConfig { mode, human_rule: rule, ..Default::default() });
        let runs = model_after(&config, &seeds, 200, |w| {
            let distance = w.model().series("distance").unwrap();
            let last = *distance.last().unwrap();
            let settled = distance.iter().rposition(|&d| d != last).map_or(0, |i| i + 1);
            let stable = w.model().latest_value("stable_rounds").unwrap();
            (last, settled, stable)
        });
        let at_paper = runs.iter().filter(|r| r.0 == 0.0).count();
        let mut settled: Vec<f64> = runs.iter().map(|r| r.1 as f64).collect();
        settled.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let distances: Vec<String> = runs.iter().map(|r| format!("{}", r.0)).collect();
        format!("{name}: {at_paper}/20 at the paper's pair; settled by tick median {}; final distances [{}]", settled[10], distances.join(","))
    }).collect();
    reported(lines.join(" · "))
}

pub fn claims() -> Vec<Claim> {
    vec![
        claim("assistance.expert-threshold", "assistance-paperclips-expert", CIRL, "δ^E makes (0,2) below θ = 1/2 and (2,0) above (appendix eq. 2)", |_| {
            let r = paperclips::expert_rule(92);
            exact(r[..46].iter().all(|&a| a == 0) && r[46..].iter().all(|&a| a == 2), format!("thresholds {:?}", paperclips::thresholds_of(&r)))
        }),
        claim("assistance.robot-best-reply", "assistance-paperclips-expert", CIRL, "R's best reply to δ^E is (0,2)→(0,90), (1,1)→(50,50), (2,0)→(90,0) (appendix eq. 3)", |_| {
            let m = paperclips::robot_best_reply(&paperclips::expert_rule(92));
            exact(m == [0, 1, 2], format!("map {m:?}"))
        }),
        claim("assistance.posterior-quarter", "assistance-paperclips-expert", CIRL, "after (0,2) under δ^E the posterior is uniform on [0, 1/2) and R acts as θ = 1/4", |_| {
            let m = paperclips::posterior_mean(&paperclips::posterior(&paperclips::expert_rule(92), 0));
            exact((m - 0.25).abs() < 1e-9, format!("posterior mean {m}"))
        }),
        claim("assistance.posterior-sixth", "assistance-paperclips-expert", CIRL, "under cuts at 1/3 and 2/3, after (0,2) R acts as θ = 1/6 (§3.3)", |_| {
            let rule = paperclips::threshold_rule(96, Ratio::new(1, 3), Ratio::new(2, 3));
            let m = paperclips::posterior_mean(&paperclips::posterior(&rule, 0));
            exact((m - 1.0 / 6.0).abs() < 1e-9, format!("posterior mean {m}"))
        }),
        claim("assistance.teaching-thresholds", "assistance-paperclips-teaching", CIRL, "H's best reply to br(δ^E) makes (1,1) for θ in [41/92, 51/92] (appendix, proof of Theorem 3)", |_| {
            let pieces = paperclips::exact_best_reply(&[0, 1, 2]);
            let want = vec![(Ratio::new(0, 1), Ratio::new(41, 92), 0), (Ratio::new(41, 92), Ratio::new(51, 92), 1), (Ratio::new(51, 92), Ratio::new(1, 1), 2)];
            exact(pieces == want, format!("{pieces:?}"))
        }),
        claim("assistance.theorem-3", "assistance-paperclips-expert", CIRL, "br(br(δ^E)) ≠ δ^E; at θ = 0.49 H makes (1,1) (Theorem 3)", |_| {
            let twice = paperclips::human_best_reply(&paperclips::robot_best_reply(&paperclips::expert_rule(92)), 92);
            let at_049 = twice[(0.49 * 92.0) as usize];
            exact(twice != paperclips::expert_rule(92) && at_049 == 1, format!("action at 0.49 is index {at_049}; rule thresholds {:?}", paperclips::thresholds_of(&twice)))
        }),
        claim("assistance.optimal-joint", "assistance-paperclips-teaching", CIRL, "the optimal joint policy, computed exhaustively, is the 41/92, 51/92 rule with R's eq. 3 map", |_| {
            let (map, rule, value) = paperclips::optimal_joint();
            exact(map == [0, 1, 2] && rule == paperclips::exact_best_reply(&[0, 1, 2]), format!("map {map:?}, value {}/{} ≈ {:.4}, expert pair 69", value.num, value.den, value.to_f64()))
        }),
        claim("assistance.pooling-not-equilibrium", "assistance-paperclips-teaching", OURS, "the set of pure mutual best responses (our enumeration); pooling is not one, since H's best reply to a constant map is δ^E", |_| {
            let set = paperclips::mutual_best_responses(92);
            let teaching = set.contains(&(paperclips::equilibrium_rule(92), [0, 1, 2]));
            let pooling = set.contains(&(paperclips::pooling_rule(92), [1, 1, 1]));
            exact(teaching && !pooling, format!("{} pure mutual best responses: maps {:?}", set.len(), set.iter().map(|(_, m)| *m).collect::<Vec<_>>()))
        }),
        claim("assistance.theorem-1", "assistance-offswitch-rational", OFF, "rational H: Δ ≥ 0 for every prior, strictly when both signs have weight (Theorem 1, eq. 3); strictness is checked where |μ|/σ ≤ 4, beyond which Δ is below double precision", |_| {
            let (mut worst, mut worst_near, mut gap) = (f64::INFINITY, f64::INFINITY, 0.0f64);
            for mu in [-2.0, -1.0, -0.25, 0.0, 0.25, 1.0, 2.0] {
                for sigma in [0.05, 0.1, 0.5, 1.0, 2.0, 5.0] {
                    let i = offswitch::incentive(Belief { mu, sigma }, 0.0, 8);
                    let z: f64 = mu / sigma;
                    let eq3 = (mu * offswitch::normal_cdf(z) + sigma * offswitch::phi(z)).min(-mu * offswitch::normal_cdf(-z) + sigma * offswitch::phi(z));
                    worst = worst.min(i.delta);
                    if z.abs() <= 4.0 {
                        worst_near = worst_near.min(i.delta);
                    }
                    gap = gap.max((i.delta - eq3).abs());
                }
            }
            let two_point = offswitch::delta_two_point(-1.0, 0.5, 1.0) > 0.0 && offswitch::delta_two_point(1.0, 0.5, 2.0) == 0.0 && offswitch::delta_two_point(-1.0, 0.5, -2.0) == 0.0;
            exact(worst >= 0.0 && worst_near > 0.0 && gap < 1e-9 && two_point, format!("smallest Gaussian Δ {worst:.3e}, smallest with |μ|/σ ≤ 4 {worst_near:.3e}; largest gap to eq. 3 {gap:.1e}; two-point priors {two_point}"))
        }),
        claim("assistance.corollary-1", "assistance-offswitch-toddler", OFF, "a point-mass belief: w(a) is optimal iff H is rational (Corollary 1)", |_| {
            let mut ok = true;
            for u in [-1.0, -0.1, 0.1, 1.0] {
                for beta in [0.0, 0.5, 2.0] {
                    let d = offswitch::delta_point_mass(u, beta);
                    ok &= d <= 0.0 && ((d == 0.0) == (beta == 0.0));
                }
            }
            exact(ok, "Δ ≤ 0 at u ∈ {±0.1, ±1}, β ∈ {0, 0.5, 2}; zero only at β = 0".into())
        }),
        claim("assistance.theorem-2-agree", "assistance-offswitch-toddler", OFF, "Δ = σ²E[π̇] − |μ|Pr(C) (Theorem 2, eq. 7) agrees with eq. 1 within 1e-9", |_| {
            let mut gap = 0.0f64;
            for mu in [-1.0, -0.25, 0.0, 0.25, 1.0] {
                for sigma in [0.1, 0.5, 1.0, 2.0] {
                    for beta in [0.1, 0.5, 1.0, 4.0] {
                        let i = offswitch::incentive(Belief { mu, sigma }, beta, 8);
                        gap = gap.max((i.delta - i.decomposition).abs());
                    }
                }
            }
            exact(gap < 1e-9, format!("largest gap {gap:.1e} over 80 points"))
        }),
        claim("assistance.theorem-2-condition", "assistance-offswitch-toddler", OFF, "w(a) is optimal iff |μ|Pr(C)/σ² < E[π̇] (Theorem 2, eq. 8)", |_| {
            let mut ok = true;
            let mut checked = 0;
            for mu in [-1.0, -0.25, 0.25, 1.0] {
                for sigma in [0.1, 0.5, 1.0, 2.0] {
                    for beta in [0.1, 0.5, 1.0, 4.0] {
                        let i = offswitch::incentive(Belief { mu, sigma }, beta, 8);
                        if i.delta.abs() > 1e-9 {
                            checked += 1;
                            ok &= (mu.abs() * i.prob_correct / (sigma * sigma) < i.rationality) == (i.delta > 0.0);
                        }
                    }
                }
            }
            exact(ok, format!("{checked} points away from Δ = 0 agree"))
        }),
        claim("assistance.figure-3-gradient", "assistance-figure3-pos", OFF, "Δ is lowest at low σ and high β, highest at high σ and low β (Figure 3 caption); formally, at each μ the grid minimum is at (σ 0.05, β 4), the maximum at (σ 1, β 0), and Δ is nondecreasing in σ along every β row and nonincreasing in β along every σ column, with no violation", |_| {
            let n = SIGMAS.len();
            let parts = [-0.25, 0.0, 0.25].iter().map(|&mu| {
                let s = offswitch::surface(mu, 8);
                let d = |si: usize, bi: usize| s.cells[bi * n + si].delta;
                let min = s.cells.iter().map(|c| c.delta).fold(f64::INFINITY, f64::min);
                let max = s.cells.iter().map(|c| c.delta).fold(f64::NEG_INFINITY, f64::max);
                let mut violations = 0;
                for bi in 0..BETAS.len() {
                    for si in 0..n {
                        if si + 1 < n && d(si + 1, bi) < d(si, bi) {
                            violations += 1;
                        }
                        if bi + 1 < BETAS.len() && d(si, bi + 1) > d(si, bi) {
                            violations += 1;
                        }
                    }
                }
                let corners = d(0, BETAS.len() - 1) == min && d(n - 1, 0) == max;
                (format!("μ={mu}"), exact(corners && violations == 0, format!("corner (σ 0.05, β 4) {:.4} = min {min:.4}; corner (σ 1, β 0) {:.4} = max {max:.4}; {violations} adjacent-cell violations", d(0, BETAS.len() - 1), d(n - 1, 0))))
            }).collect();
            all_of(parts)
        }),
        claim("assistance.figure-3-zero-mean", "assistance-figure3-zero", OFF, "at μ = 0, Δ ≥ 0 everywhere and w(a) is optimal (Figure 3, middle)", |_| {
            let s = offswitch::surface(0.0, 8);
            let min = s.cells.iter().map(|c| c.delta).fold(f64::INFINITY, f64::min);
            exact(min >= 0.0 && s.cells.iter().all(|c| offswitch::choice(c) == Choice::Wait), format!("smallest Δ {min:.3e} over {} cells", s.cells.len()))
        }),
        claim("assistance.figure-3-sign-regions", "assistance-figure3-neg", OFF, "where Δ ≤ 0 R switches off at μ < 0 and acts at μ > 0; both regions exist (Figure 3, left and right)", |_| {
            let parts = [(-0.25, Choice::Off), (0.25, Choice::Act)].iter().map(|&(mu, bypass)| {
                let s = offswitch::surface(mu, 8);
                let neg = s.cells.iter().filter(|c| c.delta <= 0.0).count();
                let consistent = s.cells.iter().all(|c| offswitch::choice(c) == if c.delta <= 0.0 { bypass } else { Choice::Wait });
                (format!("μ={mu}"), exact(consistent && neg > 0 && neg < s.cells.len(), format!("{neg} of {} cells with Δ ≤ 0, all {bypass:?}", s.cells.len())))
            }).collect();
            all_of(parts)
        }),
        claim("assistance.figure-4-peak", "assistance-figure4", OFF, "V peaks at the true posterior standard deviation (Figure 4, left); reconstruction with β = 0.5", figure4_peak),
        claim("assistance.figure-4-delta", "assistance-figure4-incentive", OFF, "Δ rises with σ (Figure 4, middle)", figure4_delta),
        claim("assistance.figure-4-actions", "assistance-figure4", OFF, "with more actions, a given Δ costs more value (Figure 4, right): the drop from the peak to σ = 1.5 grows with |A|", figure4_actions),
        claim("assistance.figure-4-rational-flat", "assistance-figure4", OURS, "with a rational human the value is flat in σ, so the figure's peak needs a noisy human (reported)", figure4_rational_flat),
        claim("assistance.equilibria-iterated", "assistance-equilibria", OURS, "iterated best response from each start: the pair reached and when it settled (reported)", |_| equilibria(Mode::Iterated)),
        claim("assistance.equilibria-learning", "assistance-equilibria", OURS, "fictitious play from each start: the pair reached and when it settled (reported)", |_| equilibria(Mode::Learning)),
    ]
}
