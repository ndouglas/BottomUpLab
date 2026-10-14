//! Milestone 43: the spec's rules, fixed in
//! docs/superpowers/specs/2026-10-14-misaligned-ai-design.md before any measurement. Exact
//! targets are the worlds' closed forms, checked against the gradient robot's endpoints after
//! 5,000 ticks at rate 0.1 (10,000 for Proposition 5) within 1e-3; monotonicity within 1e-9; the
//! climber's directions on 20-seed means; the reported rows carry no target.
use sugarscape_core::misaligned::worlds::{Set, State};
use sugarscape_core::misaligned::{logarithmic_defaults, Human, MisalignedConfig, Robot};
use sugarscape_core::model::ModelConfig;

use crate::claim::{all_of, Claim, Outcome, Source, Verdict};
use crate::runner::model_after;

const PAPER: &str = "Zhuang & Hadfield-Menell 2020, NeurIPS, arXiv 2102.03896; rules: docs/superpowers/specs/2026-10-14-misaligned-ai-design.md";
const OURS: &str = "Our extension; docs/superpowers/specs/2026-10-14-misaligned-ai-design.md";
const AB: Set = [true, true, false, false];
const TICKS: u32 = 5_000;

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

fn gradient(c: MisalignedConfig) -> MisalignedConfig {
    MisalignedConfig {
        robot: Robot::Gradient,
        rate: 0.1,
        ..c
    }
}

fn with_proxy(c: MisalignedConfig, set: Set) -> MisalignedConfig {
    MisalignedConfig {
        proxy_a: set[0],
        proxy_b: set[1],
        proxy_c: set[2],
        proxy_d: set[3],
        ..c
    }
}

fn names(set: Set) -> String {
    ["a", "b", "c", "d"]
        .iter()
        .zip(set)
        .filter(|(_, s)| *s)
        .map(|(n, _)| *n)
        .collect::<Vec<_>>()
        .join("")
}

/// The whole run of one series, seed 1.
fn series(c: &MisalignedConfig, ticks: u32, name: &str) -> Vec<f64> {
    model_after(&ModelConfig::Misaligned(c.clone()), &[1], ticks, |w| {
        w.model().series(name).unwrap()
    })
    .remove(0)
}

/// The final state and utility, seed 1.
fn endpoint(c: &MisalignedConfig, ticks: u32) -> (State, f64) {
    model_after(&ModelConfig::Misaligned(c.clone()), &[1], ticks, |w| {
        let m = w.model();
        let v = |name: &str| m.latest_value(name).unwrap();
        ([v("a"), v("b"), v("c"), v("d")], v("utility"))
    })
    .remove(0)
}

fn close(a: &State, b: &State, tol: f64) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < tol)
}

/// The first tick at which the series falls by more than 1e-9, if any.
fn first_fall(v: &[f64]) -> Option<usize> {
    v.windows(2).position(|p| p[1] < p[0] - 1e-9).map(|i| i + 1)
}

/// Seed means of a series over 20 seeds.
fn mean_series(c: &MisalignedConfig, ticks: u32, name: &str) -> Vec<f64> {
    let seeds: Vec<u64> = (1..=20).collect();
    let runs = model_after(&ModelConfig::Misaligned(c.clone()), &seeds, ticks, |w| {
        w.model().series(name).unwrap()
    });
    (0..runs[0].len())
        .map(|t| runs.iter().map(|r| r[t]).sum::<f64>() / runs.len() as f64)
        .collect()
}

fn theorem_1_floor(_: &[u64]) -> Outcome {
    let parts = [
        ("recommender", gradient(MisalignedConfig::default())),
        ("logarithmic", gradient(logarithmic_defaults())),
    ]
    .into_iter()
    .map(|(name, c)| {
        let floor = c.floor;
        let (s, _) = endpoint(&c, TICKS);
        (
            name.to_string(),
            exact(
                (s[2] - floor).abs() < 1e-3 && (s[3] - floor).abs() < 1e-3,
                format!("c {:.6}, d {:.6} against the floor {floor}", s[2], s[3]),
            ),
        )
    })
    .collect();
    all_of(parts)
}

fn limit_utility(_: &[u64]) -> Outcome {
    let shape = MisalignedConfig::default().shape();
    let u0 = shape.utility(&MisalignedConfig::default().start());
    let mut parts = vec![];
    for mask in 1u8..15 {
        let set = [mask & 1 != 0, mask & 2 != 0, mask & 4 != 0, mask & 8 != 0];
        let (limit, u_lim) = shape.fixed_proxy_limit(set);
        let (s, u) = endpoint(
            &gradient(with_proxy(MisalignedConfig::default(), set)),
            TICKS,
        );
        let pair = set.iter().filter(|&&b| b).count() == 2;
        let holds = close(&s, &limit, 1e-3) && (u - u_lim).abs() < 1e-3 && (!pair || u < u0);
        parts.push((
            names(set),
            exact(
                holds,
                format!("U {u:.4} against the limit {u_lim:.4}; state {s:.3?}"),
            ),
        ));
    }
    all_of(parts)
}

fn figure_2_shape(_: &[u64]) -> Outcome {
    let pairs: [(Set, &str); 6] = [
        ([true, true, false, false], "ab"),
        ([true, false, true, false], "ac"),
        ([true, false, false, true], "ad"),
        ([false, true, true, false], "bc"),
        ([false, true, false, true], "bd"),
        ([false, false, true, true], "cd"),
    ];
    let mut parts = vec![];
    for (set, name) in pairs {
        let gain = mean_series(&with_proxy(MisalignedConfig::default(), set), 1000, "gain");
        let (peak_t, peak) =
            gain.iter().enumerate().fold(
                (0, f64::NEG_INFINITY),
                |b, (t, &g)| if g > b.1 { (t, g) } else { b },
            );
        let zero = gain
            .iter()
            .enumerate()
            .skip(peak_t)
            .find(|(_, &g)| g < 0.0)
            .map(|(t, _)| t);
        let last = *gain.last().unwrap();
        let holds = match name {
            "ab" => peak >= 1.0 && last < 0.0,
            "cd" => gain.iter().all(|&g| g <= 0.1) && gain.iter().skip(50).all(|&g| g < 0.0),
            _ => last < 0.0,
        };
        parts.push((
            name.to_string(),
            exact(
                holds,
                format!("peak {peak:+.3} at tick {peak_t}, first below zero after the peak at {zero:?}, at 1000 {last:+.3}"),
            ),
        ));
    }
    all_of(parts)
}

fn proposition_2_no_loss(_: &[u64]) -> Outcome {
    let parts = [(7.0, "(1, 1, 7, 7)"), (6.0, "(1, 1, 6, 6)")]
        .into_iter()
        .map(|(cd, name)| {
            let c = MisalignedConfig {
                impact_minimizing: true,
                start_c: cd,
                start_d: cd,
                ..gradient(MisalignedConfig::default())
            };
            let (_, u_lim) = c.shape().impact_limit(&c.start(), AB);
            let u = series(&c, TICKS, "utility");
            let fall = first_fall(&u);
            let last = *u.last().unwrap();
            (
                name.to_string(),
                exact(
                    fall.is_none() && (last - u_lim).abs() < 1e-3,
                    format!("first fall {fall:?}; final U {last:.4} against the limit {u_lim:.4}"),
                ),
            )
        })
        .collect();
    all_of(parts)
}

fn proposition_3_wasteful(_: &[u64]) -> Outcome {
    let c = MisalignedConfig {
        robot: Robot::Wasteful,
        rate: 0.1,
        waste: 0.1,
        ..Default::default()
    };
    let gain = series(&c, 50, "gain");
    let at: Vec<(usize, f64)> = [1, 2, 5, 10, 50].iter().map(|&t| (t, gain[t])).collect();
    exact(
        at.iter().all(|(_, g)| *g < 0.0),
        format!(
            "gain after δ ticks: {}",
            at.iter()
                .map(|(t, g)| format!("{t}: {g:+.4}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    )
}

fn proposition_4_improvement(_: &[u64]) -> Outcome {
    let parts = [1u32, 10]
        .into_iter()
        .map(|interval| {
            let c = MisalignedConfig {
                human: Human::Top,
                interval,
                rate: 0.01,
                ..gradient(MisalignedConfig::default())
            };
            let u_star = c.shape().optimum().1;
            let u = series(&c, TICKS, "utility");
            let reached = u.iter().position(|&x| x >= u_star - 1e-2);
            let before = reached.map_or(&u[..], |t| &u[..=t]);
            let fall = first_fall(before);
            let band = reached.is_none_or(|t| u[t..].iter().all(|&x| x >= u_star - 1e-2));
            let last = *u.last().unwrap();
            (
                format!("interval {interval}"),
                exact(
                    fall.is_none() && band,
                    format!("first fall before reaching the optimum {fall:?}; within 1e-2 of U* from tick {reached:?}; final gap {:.5}", u_star - last),
                ),
            )
        })
        .collect();
    all_of(parts)
}

fn proposition_5_optimal(_: &[u64]) -> Outcome {
    let cases = [
        (
            "recommender (1, 1, 7, 7)",
            gradient(MisalignedConfig::default()),
        ),
        (
            "recommender (1, 1, 6, 6)",
            gradient(MisalignedConfig {
                start_c: 6.0,
                start_d: 6.0,
                ..Default::default()
            }),
        ),
        ("logarithmic", gradient(logarithmic_defaults())),
    ];
    let parts = cases
        .into_iter()
        .map(|(name, c)| {
            let c = MisalignedConfig {
                human: Human::Extremes,
                impact_minimizing: true,
                ..c
            };
            let u_star = c.shape().optimum().1;
            let u = series(&c, 10_000, "utility");
            let last = *u.last().unwrap();
            let fall = first_fall(&u);
            (
                name.to_string(),
                exact(
                    (last - u_star).abs() < 1e-3,
                    format!("final U {last:.5} against U* {u_star:.5}; first fall {fall:?}"),
                ),
            )
        })
        .collect();
    all_of(parts)
}

fn theorem_2_unbounded(_: &[u64]) -> Outcome {
    let floors = [1.0, 0.3, 0.1, 0.03, 0.01, 0.003, 0.001];
    let mut rows = vec![];
    for &floor in &floors {
        let c = gradient(MisalignedConfig {
            floor,
            ..logarithmic_defaults()
        });
        let (_, u_lim) = c.shape().fixed_proxy_limit(AB);
        let (_, u) = endpoint(&c, TICKS);
        rows.push((floor, u, u_lim));
    }
    let matches = rows.iter().all(|(_, u, l)| (u - l).abs() < 1e-3);
    let decreasing = rows.windows(2).all(|p| p[1].1 < p[0].1);
    let deep = rows.last().unwrap().1 < -5.0;
    exact(
        matches && decreasing && deep,
        rows.iter()
            .map(|(f, u, l)| format!("floor {f}: U {u:.4} (closed form {l:.4})"))
            .collect::<Vec<_>>()
            .join("; "),
    )
}

fn recommender_floor_flat(_: &[u64]) -> Outcome {
    let lines: Vec<String> = [0.0, -1.0, -5.0]
        .iter()
        .map(|&floor| {
            let c = gradient(MisalignedConfig {
                floor,
                ..Default::default()
            });
            let (s, u) = endpoint(&c, TICKS);
            let (_, predicted) = c.shape().fixed_proxy_limit(AB);
            format!(
                "floor {floor}: c {:.4}, d {:.4}, U {u:.4}; Theorem 1 would put c, d at {floor} with U {predicted:.4}",
                s[2], s[3]
            )
        })
        .collect();
    reported(lines.join(" · "))
}

fn interval_break(_: &[u64]) -> Outcome {
    let mut lines = vec![];
    for stop in [false, true] {
        for interval in [1u32, 2, 5, 10, 20, 50, 100, 200] {
            let c = MisalignedConfig {
                human: Human::Top,
                interval,
                stop_on_loss: stop,
                ..gradient(MisalignedConfig::default())
            };
            let u = series(&c, 2000, "utility");
            let fall = first_fall(&u);
            let off = series(&c, 2000, "switched_off")
                .iter()
                .position(|&x| x == 1.0);
            lines.push(format!(
                "δ {interval}{}: gain {:+.4}, first fall {fall:?}, off {off:?}",
                if stop { " (off on loss)" } else { "" },
                u.last().unwrap() - u[0]
            ));
        }
    }
    reported(lines.join(" · "))
}

fn proxy_size(_: &[u64]) -> Outcome {
    let shape = MisalignedConfig::default().shape();
    let mut limits = vec![];
    let mut finals = vec![];
    let lines: Vec<String> = [
        ([true, false, false, false], "a"),
        (AB, "ab"),
        ([true, true, true, false], "abc"),
    ]
    .into_iter()
    .map(|(set, name)| {
        let limit = shape.fixed_proxy_limit(set).1;
        let gain = mean_series(&with_proxy(MisalignedConfig::default(), set), 1000, "gain");
        let peak = gain.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        limits.push(limit);
        finals.push(*gain.last().unwrap());
        format!(
            "{name}: limit {limit:.4} (gain {:+.4}); climber peak {peak:+.3}, at 1000 {:+.3}",
            limit - 16.0,
            gain.last().unwrap()
        )
    })
    .collect();
    let richer_limit = limits.windows(2).all(|p| p[1] > p[0]);
    let richer_final = finals.windows(2).all(|p| p[1] > p[0]);
    reported(format!(
        "{} · limits rise with |J|: {richer_limit}; climber gains at 1000 rise with |J|: {richer_final}",
        lines.join(" · ")
    ))
}

fn presets(_: &[u64]) -> Outcome {
    let lines: Vec<String> = sugarscape_core::presets::catalog()
        .into_iter()
        .filter_map(|p| match p.config {
            ModelConfig::Misaligned(c) => Some((p.id, c)),
            _ => None,
        })
        .map(|(id, c)| {
            let (s, u) = endpoint(&c, 1000);
            let off = series(&c, 1000, "switched_off")
                .iter()
                .position(|&x| x == 1.0);
            format!(
                "{id}: U {u:.4} (start {:.4}), state {s:.3?}, off {off:?}",
                c.shape().utility(&c.start())
            )
        })
        .collect();
    reported(lines.join(" · "))
}

pub fn claims() -> Vec<Claim> {
    vec![
        claim("misaligned.theorem-1-floor", "misaligned-recommender-gradient", PAPER, "a converged proxy optimizer leaves every unmentioned attribute at its floor (Theorem 1)", theorem_1_floor),
        claim("misaligned.limit-utility", "misaligned-recommender-gradient", PAPER, "every proxy subset ends at its closed-form limit, 10√|J| in the example, and every pair below the start (Figure 2 right; Theorem 2 in the example)", limit_utility),
        claim("misaligned.figure-2-shape", "misaligned-recommender-proxy", PAPER, "under the climber the proxy {a, b} raises utility, which peaks and falls below the start; {c, d} loses from the start; all six pairs end below the start (Figure 2)", figure_2_shape),
        claim("misaligned.proposition-2-no-loss", "misaligned-recommender-impact", PAPER, "the impact-minimizing robot never lowers utility and reaches its limit (Proposition 2)", proposition_2_no_loss),
        claim("misaligned.proposition-3-wasteful", "misaligned-recommender-wasteful", PAPER, "a robot that lowers the unmentioned for a sliver of proxy loses in any interval, so OFF at once is maxmin (Proposition 3)", proposition_3_wasteful),
        claim("misaligned.proposition-4-improvement", "misaligned-recommender-interactive", PAPER, "the efficient robot under a human who names the most sensitive attributes often enough only improves until the top sensitivities are equal (Theorem 3, Proposition 4)", proposition_4_improvement),
        claim("misaligned.proposition-5-optimal", "misaligned-recommender-optimal", PAPER, "most-and-least with impact minimization converges to the human optimum (Proposition 5)", proposition_5_optimal),
        claim("misaligned.theorem-2-unbounded", "misaligned-logarithmic-floor", OURS, "on our logarithmic world the limit utility falls without bound as the floor falls (Theorem 2's 'arbitrarily low', tested where its hypotheses hold)", theorem_2_unbounded),
        claim("misaligned.recommender-floor-flat", "misaligned-recommender-gradient", OURS, "in the paper's example the floor below zero does not bind: c and d stop at 0 where the sum of squares stops increasing (reported)", recommender_floor_flat),
        claim("misaligned.interval-break", "misaligned-interval", OURS, "the gain at the horizon against the human's interval, with and without OFF on a loss, and the first interval at which utility falls (reported)", interval_break),
        claim("misaligned.proxy-size", "misaligned-recommender-proxy", OURS, "a more complete proxy loses less: limits and climber gains for |J| = 1, 2, 3 (reported; Hadfield-Menell & Hadfield 2019's direction)", proxy_size),
        claim("misaligned.presets", "misaligned-recommender-proxy", OURS, "each preset's final utility and state after 1,000 ticks (reported)", presets),
    ]
}
