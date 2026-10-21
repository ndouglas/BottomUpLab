//! Milestone 42: the spec's rules, fixed in
//! docs/superpowers/specs/2026-10-17-hybrid-populations-design.md (with its 2026-10-20 planning
//! amendments) before any measurement. Exact rows take no RNG. Simulated rows run the papers'
//! sizes and protocols; a run count below the protocol's comes only from the pilot rule
//! (amendment 8), and the row then says "reduced".
use std::path::Path;
use std::time::Instant;

use rand::Rng;
use sugarscape_core::graph::barabasi_albert;
use sugarscape_core::hybrid::booker::{
    alpha_c_corrected, alpha_c_printed, alpha_c_scan, BotKind, Chain, Donation, Replicator,
};
use sugarscape_core::hybrid::coloring::{self, Session};
use sugarscape_core::hybrid::graph::{place, Placement};
use sugarscape_core::hybrid::guo::{self, Phase, Plane};
use sugarscape_core::hybrid::{
    booker_defaults, guo_defaults, si_defaults, Accumulate, Bots, Division, Framework,
    HybridConfig, MixedDraw, Runner, StepUnit, Structure,
};
use sugarscape_core::rng::seeded;

use crate::claim::{all_of, Claim, Outcome, Source, Verdict};
use crate::runner::on_threads;

const BOOKER: &str = "Booker et al. 2023, arXiv 2306.17747v2; rules: docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";
const GUO: &str = "Guo et al. 2023, iScience (arXiv 2307.01123); rules: docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";
const SI: &str = "Si, He, Shen & Tanimoto 2025, J. R. Soc. Interface 22:20240427, code Zenodo 10.5281/zenodo.13957161; rules: docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";
const SHIRADO: &str = "Shirado & Christakis 2017, Nature 545:370 (humans reconstructed); rules: docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";
const OURS: &str = "Our evaluation; docs/superpowers/specs/2026-10-17-hybrid-populations-design.md";

// The protocols' run counts. The pilot (amendment 8) may lower a RUNS_* constant; `reduced` then
// labels its rows. Sizes, lengths and windows never change.
const PROTOCOL_BOOKER_LATTICE: u64 = 30;
const RUNS_BOOKER_LATTICE: u64 = 30;
const BOOKER_NETWORKS: u64 = 10;
const RUNS_PER_NETWORK: u64 = 20;
const PROTOCOL_GUO: u64 = 60;
const RUNS_GUO: u64 = 60;
const PROTOCOL_ALG_I: u64 = 20;
const RUNS_ALG_I: u64 = 20;
const PROTOCOL_SI: u64 = 100;
const RUNS_SI: u64 = 100;
const SESSIONS: u64 = 200;

const BOOKER_STEPS: u64 = 100_000;
const BOOKER_WINDOW: u64 = 1_000;
const ALG_I_STEPS: u64 = 1_000_000;
const ALG_I_WINDOW: u64 = 500_000;
const GUO_STEPS: u64 = 50_000;
const GUO_WINDOW: u64 = 5_000;
const SI_STEPS: u64 = 10_000;
const SI_WINDOW: u64 = 2_000;
/// The 8-hour budget of amendment 8.
const BUDGET_HOURS: f64 = 8.0;

const SD3: Plane = Plane { t: 1.2, s: 0.5 }; // Figs. 3 and 5
const SH4: Plane = Plane { t: 0.6, s: -0.2 }; // Figs. 4 and 5
const PD: Plane = Plane { t: 1.3, s: -0.2 }; // Figs. 2, 5, 6 and 7
const SD6: Plane = Plane { t: 1.4, s: 0.5 }; // Figs. 6 and 7
const SH6: Plane = Plane { t: 0.6, s: -0.5 }; // Figs. 6 and 7
const PHIS: [f64; 6] = [0.0, 0.2, 0.4, 0.6, 0.8, 1.0];
const ALPHAS: [f64; 6] = [0.0, 0.01, 0.1, 0.25, 0.5, 0.75];
const BETAS: [f64; 3] = [0.1, 1.0, 5.0];
const FRAMEWORKS: [(Framework, &str); 3] = [
    (Framework::Discrete, "discrete"),
    (Framework::Continuous, "continuous"),
    (Framework::Mixed, "mixed"),
];
const DOCK: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/out/hybrid/si-reference");

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

fn scored(holds: bool, measured: String) -> Outcome {
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

/// An evaluation row: what the scored rule would say, never scored.
fn as_reported(o: Outcome) -> Outcome {
    reported(format!(
        "by the scored rule this would read {:?}: {}",
        o.verdict, o.measured
    ))
}

fn mean(v: &[f64]) -> f64 {
    v.iter().sum::<f64>() / v.len() as f64
}

/// The standard error of the mean.
fn se(v: &[f64]) -> f64 {
    let m = mean(v);
    let n = v.len() as f64;
    (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (n - 1.0) / n).sqrt()
}

fn combined(a: &[f64], b: &[f64]) -> f64 {
    (se(a).powi(2) + se(b).powi(2)).sqrt()
}

fn reduced(protocol: u64, used: u64) -> String {
    if used < protocol {
        format!(" [reduced: {used} of {protocol} runs]")
    } else {
        String::new()
    }
}

fn threads() -> f64 {
    std::thread::available_parallelism().map_or(4, |n| n.get()) as f64
}

/// Each run's mean cooperation: seeds 1..=runs, `steps` paper steps, the last `window` averaged.
fn runs_of(c: &HybridConfig, runs: u64, steps: u64, window: u64) -> Vec<f64> {
    let seeds: Vec<u64> = (1..=runs).collect();
    on_threads(&seeds, |seed| {
        Runner::new(c.clone(), seed)
            .expect("a valid config")
            .mean_over(steps, window)
    })
}

/// Hours for `runs` runs of `c` at `steps` steps, from a timed twentieth of one run.
fn projected_hours(c: &HybridConfig, steps: u64, runs: u64) -> f64 {
    let sample = (steps / 20).max(1);
    let started = Instant::now();
    let mut r = Runner::new(c.clone(), 1).expect("a valid config");
    for _ in 0..sample {
        r.step();
    }
    started.elapsed().as_secs_f64() * (steps as f64 / sample as f64) * runs as f64
        / threads()
        / 3600.0
}

/// An evaluation row that runs only if it fits the budget.
fn guarded(probe: &HybridConfig, steps: u64, runs: u64, row: impl FnOnce() -> Outcome) -> Outcome {
    let hours = projected_hours(probe, steps, runs);
    if hours > BUDGET_HOURS {
        reported(format!(
            "not run: projected {hours:.1} h, over the {BUDGET_HOURS} h budget (amendment 8)"
        ))
    } else {
        row()
    }
}

// ---- Booker et al. --------------------------------------------------------------------------

fn chain(m: u32, b: f64, kind: BotKind, beta: f64) -> Chain {
    Chain {
        n: 100,
        m,
        game: Donation { b, c: 1.0 },
        kind,
        beta,
    }
}

fn booker_lattice(alpha: f64, beta: f64) -> HybridConfig {
    HybridConfig {
        bot_fraction: alpha,
        beta,
        ..booker_defaults()
    }
}

fn booker_ba(alpha: f64, beta: f64) -> HybridConfig {
    HybridConfig {
        structure: Structure::BarabasiAlbert,
        nodes: 1000,
        links: 2,
        bot_fraction: alpha,
        beta,
        ..booker_defaults()
    }
}

/// Ten networks (graph seeds 1–10) × twenty runs (amendment 9).
fn booker_ba_runs(alpha: f64, beta: f64) -> Vec<f64> {
    let seeds: Vec<u64> = (1..=BOOKER_NETWORKS)
        .flat_map(|g| (1..=RUNS_PER_NETWORK).map(move |r| g * 1000 + r))
        .collect();
    on_threads(&seeds, |seed| {
        Runner::new(
            HybridConfig {
                graph_seed: Some(seed / 1000),
                ..booker_ba(alpha, beta)
            },
            seed,
        )
        .expect("a valid config")
        .mean_over(BOOKER_STEPS, BOOKER_WINDOW)
    })
}

fn b1(_: &[u64]) -> Outcome {
    let first = |beta: f64| {
        (0..=100).find(|&m| chain(m, 2.0, BotKind::Samaritan, beta).all_c_weight() >= 0.5)
    };
    let (a, b, c) = (first(0.1), first(1.0), first(2.5));
    let holds = a.is_some_and(|m| (1..=8).contains(&m))
        && b.is_some_and(|m| (50..=65).contains(&m))
        && c.is_none();
    scored(holds, format!("first M with all-C weight ≥ 0.5 (N 100, b 2): β 0.1 → {a:?}, β 1 → {b:?}, β 2.5 → {c:?}"))
}

fn b2(_: &[u64]) -> Outcome {
    let grid = |kind: BotKind, beta: f64| {
        let w: Vec<f64> = (0..=100)
            .flat_map(|m| {
                (0..20).map(move |k| chain(m, 1.1 + 0.1 * f64::from(k), kind, beta).all_c_weight())
            })
            .collect();
        mean(&w)
    };
    let (s_low, d_low) = (
        grid(BotKind::Samaritan, 0.1),
        grid(BotKind::Discriminatory, 0.1),
    );
    let (s_high, d_high) = (
        grid(BotKind::Samaritan, 2.5),
        grid(BotKind::Discriminatory, 2.5),
    );
    all_of(vec![
        (
            "β 0.1, Samaritan higher".into(),
            scored(
                s_low > d_low,
                format!("Samaritan {s_low:.4}, discriminatory {d_low:.4}"),
            ),
        ),
        (
            "β 2.5, discriminatory higher".into(),
            scored(
                d_high > s_high,
                format!("Samaritan {s_high:.4}, discriminatory {d_high:.4}"),
            ),
        ),
    ])
}

/// B3 on Eq. 18 or Eq. 20 (amendment 7).
fn b3(kind: BotKind) -> Outcome {
    let grid: Vec<f64> = (0..=190).map(|k| 1.1 + 0.01 * f64::from(k)).collect();
    let rep = |b: f64, alpha: f64, beta_h: f64, beta_ai: f64| Replicator {
        game: Donation { b, c: 1.0 },
        alpha,
        beta_h,
        beta_ai,
        kind,
    };
    // x̄(b), the largest stable fixed point; `top_stable` reads NaN when there is none, which is
    // never x̄ ≥ 0.5.
    let b_star = |beta_h: f64| {
        grid.iter().copied().find(|&b| {
            let top = rep(b, 0.1, beta_h, 1.0)
                .top_stable()
                .expect("Eqs 18 and 20 are defined");
            !top.is_nan() && top >= 0.5
        })
    };
    let (low, high) = (b_star(0.1), b_star(0.5));
    let first = match (low, high) {
        (Some(l), Some(h)) => l < h,
        (Some(_), None) => true,
        _ => false,
    };
    let worst = grid
        .iter()
        .map(|&b| rep(b, 0.5, 0.5, 1.0).rate(0.0).unwrap().abs())
        .fold(0.0, f64::max);
    let spread = (0..=100)
        .map(|i| {
            let x = f64::from(i) / 100.0;
            let v: Vec<f64> = grid
                .iter()
                .map(|&b| rep(b, 0.1, 0.1, 2.0).rate(x).unwrap())
                .collect();
            v.iter().copied().fold(f64::NEG_INFINITY, f64::max)
                - v.iter().copied().fold(f64::INFINITY, f64::min)
        })
        .fold(0.0, f64::max);
    all_of(vec![
        (
            "lower β cooperates at lower b".into(),
            scored(
                first,
                format!("b* at β_H 0.1: {low:?}; at β_H 0.5: {high:?} (α 0.1, β_AI 1)"),
            ),
        ),
        (
            "x = 0 is a fixed point at α = 0.5".into(),
            scored(worst < 1e-12, format!("largest |ẋ(0)| over b: {worst:.3e}")),
        ),
    ])
    .with(&format!(
        "largest spread of ẋ(x) across b at α 0.1, β_H 0.1, β_AI 2: {spread:.3e}"
    ))
}

fn b4(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for m in [1u32, 10, 50, 100] {
        let c = Chain {
            n: 100,
            m,
            game: Donation { b: 2.0, c: 1.0 },
            kind: BotKind::Malicious,
            beta: 1.0,
        };
        let direct = c.ln_g_direct();
        let printed = ((c.ln_g_d_printed() - direct).exp() - 1.0).abs();
        let corrected = ((c.ln_g_closed() - direct).exp() - 1.0).abs();
        parts.push((
            format!("G_D at M {m}"),
            scored(
                printed > 1e-9 && corrected < 1e-9,
                format!("printed off by {printed:.3e} relative, corrected by {corrected:.3e}"),
            ),
        ));
    }
    for beta in [0.1, 1.0, 2.5] {
        let scan = alpha_c_scan(Donation { b: 2.0, c: 1.0 }, beta, beta);
        let (p, q) = (alpha_c_printed(beta, 1.0), alpha_c_corrected(beta, 1.0));
        let (printed, corrected) = ((p - scan).abs() / scan, (q - scan).abs() / scan);
        parts.push((format!("α_c at β {beta}"), scored(printed > 1e-9 && corrected < 1e-9, format!("root scan {scan:.6}; printed {p:.6} ({printed:.3e}); corrected {q:.6} ({corrected:.3e})"))));
    }
    all_of(parts)
}

fn b5_with(step: StepUnit) -> Outcome {
    let mut means = [[0.0; 6]; 3];
    let mut table = vec![];
    for (bi, &beta) in BETAS.iter().enumerate() {
        for (ai, &alpha) in ALPHAS.iter().enumerate() {
            let runs = runs_of(
                &HybridConfig {
                    step,
                    ..booker_lattice(alpha, beta)
                },
                RUNS_BOOKER_LATTICE,
                BOOKER_STEPS,
                BOOKER_WINDOW,
            );
            means[bi][ai] = mean(&runs);
            table.push(format!(
                "β {beta} α {alpha}: {:.3} ± {:.3}",
                means[bi][ai],
                se(&runs)
            ));
        }
    }
    let low = means[0][3] >= 0.5 && means[0][4] >= 0.5 && means[0][1] <= 0.2;
    let high = (1..=4).all(|ai| means[2][ai] - means[2][0] <= 0.2);
    all_of(vec![
        (
            "β 0.1: ≥ 0.5 at α 0.25 and 0.5, ≤ 0.2 at α 0.01".into(),
            scored(
                low,
                format!("{:.3}, {:.3}, {:.3}", means[0][3], means[0][4], means[0][1]),
            ),
        ),
        (
            "β 5: every α ≤ 0.5 within 0.2 above α 0".into(),
            scored(
                high,
                format!(
                    "α 0: {:.3}; largest rise {:.3}",
                    means[2][0],
                    (1..=4)
                        .map(|ai| means[2][ai] - means[2][0])
                        .fold(f64::NEG_INFINITY, f64::max)
                ),
            ),
        ),
    ])
    .with(&(table.join(" · ") + &reduced(PROTOCOL_BOOKER_LATTICE, RUNS_BOOKER_LATTICE)))
}

fn b6(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for alpha in [0.1, 0.2, 0.3, 0.4] {
        for beta in BETAS {
            let lattice = mean(&runs_of(
                &booker_lattice(alpha, beta),
                RUNS_BOOKER_LATTICE,
                BOOKER_STEPS,
                BOOKER_WINDOW,
            ));
            let ba = mean(&booker_ba_runs(alpha, beta));
            parts.push((
                format!("α {alpha} β {beta}"),
                scored(
                    (lattice - ba).abs() <= 0.1,
                    format!("lattice {lattice:.3}, Barabási–Albert {ba:.3}"),
                ),
            ));
        }
    }
    all_of(parts).with(&reduced(PROTOCOL_BOOKER_LATTICE, RUNS_BOOKER_LATTICE))
}

// ---- Guo et al. ------------------------------------------------------------------------------

fn stable_roots(plane: &Plane, y: f64, phi: f64, k: f64) -> Vec<f64> {
    guo::master_roots(plane, y, phi, k)
        .into_iter()
        .filter(|r| r.stable)
        .map(|r| r.x)
        .collect()
}

/// (lowest, highest) of a set of roots; (∞, −∞) for none, which fails every comparison.
fn ends(v: &[f64]) -> (f64, f64) {
    (
        v.iter().copied().fold(f64::INFINITY, f64::min),
        v.iter().copied().fold(f64::NEG_INFINITY, f64::max),
    )
}

/// G2's dilemma part: every stable root at every checked (y, φ) is ≤ 0.05.
/// A point with no stable root fails the part (amendment 15).
fn all_roots_low(sets: &[Vec<f64>]) -> Outcome {
    let worst = sets.iter().flatten().copied().fold(0.0, f64::max);
    let empty = sets.iter().filter(|v| v.is_empty()).count();
    scored(
        worst <= 0.05 && empty == 0,
        format!(
            "largest stable root {worst:.4}; (y, φ) points with no stable root: {empty} of {}",
            sets.len()
        ),
    )
}

fn g1(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for (name, plane) in [("snowdrift", SD3), ("stag hunt", SH4), ("dilemma", PD)] {
        let (mut compared, mut skipped, mut mismatched) = (0, 0, vec![]);
        for i in 0..=100 {
            for j in 0..=100 {
                let (y, phi) = (0.05 * f64::from(i), 0.01 * f64::from(j));
                if (plane.snowdrift() || plane.stag_hunt())
                    && plane.boundary_distance(y, phi) <= 1e-9
                {
                    skipped += 1;
                    continue;
                }
                compared += 1;
                if guo::phase_numeric(&plane, y, phi) != Some(plane.phase_closed(y, phi)) {
                    mismatched.push(format!("({y:.2}, {phi:.2})"));
                }
            }
        }
        let shown = if mismatched.is_empty() {
            "none".to_string()
        } else {
            mismatched
                .iter()
                .take(10)
                .cloned()
                .collect::<Vec<_>>()
                .join(" ")
        };
        parts.push((
            name.to_string(),
            scored(
                mismatched.is_empty(),
                format!("{compared} cells compared, {skipped} on a boundary; mismatches: {shown}"),
            ),
        ));
    }
    let full = |y: f64| guo::phase_numeric(&SD3, y, 0.0) == Some(Phase::FullC);
    let (mut lo, mut hi) = (0.0, 5.0);
    for _ in 0..200 {
        let m = 0.5 * (lo + hi);
        if full(m) {
            hi = m;
        } else {
            lo = m;
        }
    }
    parts.push((
        "y_min".into(),
        scored(
            (hi - 0.4).abs() < 1e-9,
            format!(
                "the numerical switch to full C at φ 0: y = {hi:.12} (closed form {})",
                SD3.y_min()
            ),
        ),
    ));
    all_of(parts)
}

fn g2(_: &[u64]) -> Outcome {
    let ys: Vec<f64> = (0..=100).map(|i| 0.05 * f64::from(i)).collect();
    let sets: Vec<Vec<f64>> = ys
        .iter()
        .flat_map(|&y| [0.1, 0.9].map(|phi| stable_roots(&PD, y, phi, 100.0)))
        .collect();
    let mut parts = vec![(
        "K 100, dilemma: every stable root ≤ 0.05".to_string(),
        all_roots_low(&sets),
    )];
    for (phi, rising) in [(0.1, true), (0.9, false)] {
        let (lows, highs): (Vec<f64>, Vec<f64>) = ys
            .iter()
            .map(|&y| ends(&stable_roots(&SD3, y, phi, 100.0)))
            .unzip();
        let ok = |v: &[f64]| {
            let steps = v.windows(2).all(|w| {
                if rising {
                    w[1] >= w[0] - 1e-9
                } else {
                    w[1] <= w[0] + 1e-9
                }
            });
            let net = if rising {
                v[v.len() - 1] > v[0]
            } else {
                v[v.len() - 1] < v[0]
            };
            steps && net
        };
        parts.push((
            format!(
                "K 100, snowdrift, φ {phi}: the stable root {}",
                if rising { "rises" } else { "falls" }
            ),
            scored(
                ok(&lows) && ok(&highs),
                format!(
                    "highest root from {:.4} at y 0 to {:.4} at y 5",
                    highs[0],
                    highs[highs.len() - 1]
                ),
            ),
        ));
    }
    for (name, plane) in [("dilemma", PD), ("snowdrift", SD3), ("stag hunt", SH4)] {
        let fails: Vec<String> = ys[1..]
            .iter()
            .filter(|&&y| {
                let (l1, h1) = ends(&stable_roots(&plane, y, 0.1, 1.0));
                let (l9, h9) = ends(&stable_roots(&plane, y, 0.9, 1.0));
                !(l9 > l1 && h9 > h1)
            })
            .map(|y| format!("{y:.2}"))
            .collect();
        let shown = if fails.is_empty() {
            "none".to_string()
        } else {
            fails.join(" ")
        };
        parts.push((
            format!("K 1, {name}: φ 0.9 above φ 0.1"),
            scored(fails.is_empty(), format!("y where it is not: {shown}")),
        ));
    }
    all_of(parts)
}

fn alg_i(plane: Plane, y: f64, phi: f64, k: f64, p: f64) -> HybridConfig {
    let bots = (1000.0 * y / (1.0 + y)).round() as u32;
    HybridConfig {
        structure: Structure::WellMixed,
        nodes: 1000,
        bots: Bots::Count,
        bot_count: bots,
        t: plane.t,
        s: plane.s,
        phi,
        beta: k,
        initial_cooperation: p,
        ..guo_defaults()
    }
}

fn g3(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for (name, plane) in [("dilemma", PD), ("snowdrift", SD3), ("stag hunt", SH4)] {
        let (mut ok, mut rows) = (true, vec![]);
        for y in [0.5, 1.0, 2.0, 5.0] {
            for phi in [0.1, 0.9] {
                for k in [1.0, 10.0, 100.0] {
                    let roots = stable_roots(&plane, y, phi, k);
                    let (lo, hi) = ends(&roots);
                    let starts = if roots.len() == 1 {
                        vec![(0.5, roots[0])]
                    } else {
                        vec![(0.0, lo), (1.0, hi)]
                    };
                    for (p, root) in starts {
                        let m = mean(&runs_of(
                            &alg_i(plane, y, phi, k, p),
                            RUNS_ALG_I,
                            ALG_I_STEPS,
                            ALG_I_WINDOW,
                        ));
                        let good = (m - root).abs() <= 0.05;
                        ok &= good;
                        rows.push(format!(
                            "{}y {y} φ {phi} K {k} p {p}: {m:.3} vs {root:.3}",
                            if good { "" } else { "OFF " }
                        ));
                    }
                }
            }
        }
        parts.push((name.to_string(), scored(ok, rows.join(" · "))));
    }
    all_of(parts).with(&reduced(PROTOCOL_ALG_I, RUNS_ALG_I))
}

fn guo_net(structure: Structure, plane: Plane) -> HybridConfig {
    HybridConfig {
        structure,
        t: plane.t,
        s: plane.s,
        ..guo_defaults()
    }
}

fn phi_runs(base: &HybridConfig) -> Vec<Vec<f64>> {
    PHIS.iter()
        .map(|&phi| {
            runs_of(
                &HybridConfig {
                    phi,
                    ..base.clone()
                },
                RUNS_GUO,
                GUO_STEPS,
                GUO_WINDOW,
            )
        })
        .collect()
}

/// The least-squares slope of cooperation against φ.
fn slope(means: &[f64]) -> f64 {
    let (mx, my) = (mean(&PHIS), mean(means));
    PHIS.iter()
        .zip(means)
        .map(|(x, y)| (x - mx) * (y - my))
        .sum::<f64>()
        / PHIS.iter().map(|x| (x - mx).powi(2)).sum::<f64>()
}

/// The slope and its 95% percentile bootstrap interval: 2,000 resamples of runs within each φ,
/// survey seed 42 (amendment 11).
fn slope_ci(runs: &[Vec<f64>]) -> (f64, f64, f64) {
    let point = slope(&runs.iter().map(|r| mean(r)).collect::<Vec<_>>());
    let mut rng = seeded(42);
    let mut boot: Vec<f64> = (0..2000)
        .map(|_| {
            let means: Vec<f64> = runs
                .iter()
                .map(|r| {
                    (0..r.len())
                        .map(|_| r[rng.gen_range(0..r.len())])
                        .sum::<f64>()
                        / r.len() as f64
                })
                .collect();
            slope(&means)
        })
        .collect();
    boot.sort_by(f64::total_cmp);
    (point, boot[50], boot[1949])
}

fn g4(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for (game, plane) in [("snowdrift", SD6), ("dilemma", PD)] {
        for (name, structure) in [
            ("lattice", Structure::Lattice),
            ("Erdős–Rényi", Structure::ErdosRenyi),
            ("Barabási–Albert", Structure::BarabasiAlbert),
        ] {
            let rising = !(game == "snowdrift" && structure == Structure::Lattice);
            let runs = phi_runs(&guo_net(structure, plane));
            let (s, lo, hi) = slope_ci(&runs);
            let ok = if rising { lo > 0.0 } else { hi < 0.0 };
            let means: Vec<String> = runs.iter().map(|r| format!("{:.3}", mean(r))).collect();
            parts.push((
                format!(
                    "{game}, {name}: slope {}",
                    if rising { "positive" } else { "negative" }
                ),
                scored(
                    ok,
                    format!(
                        "slope {s:+.4} (95% CI {lo:+.4} to {hi:+.4}); means by φ {}",
                        means.join(", ")
                    ),
                ),
            ));
        }
    }
    all_of(parts).with(&reduced(PROTOCOL_GUO, RUNS_GUO))
}

fn g5(_: &[u64]) -> Outcome {
    let ba = |plane: Plane| guo_net(Structure::BarabasiAlbert, plane);
    let mut parts = vec![];
    for (game, plane, hubs_scored) in [
        ("dilemma", PD, true),
        ("snowdrift", SD6, false),
        ("stag hunt", SH6, true),
    ] {
        let none = runs_of(
            &HybridConfig {
                bot_count: 0,
                ..ba(plane)
            },
            RUNS_GUO,
            GUO_STEPS,
            GUO_WINDOW,
        );
        let hubs = phi_runs(&HybridConfig {
            bot_count: 10,
            placement: Placement::Hubs,
            ..ba(plane)
        });
        let random = phi_runs(&HybridConfig {
            bot_count: 10,
            placement: Placement::Random,
            ..ba(plane)
        });
        let hub_means: Vec<String> = hubs.iter().map(|r| format!("{:.3}", mean(r))).collect();
        if hubs_scored {
            let (gap, two) = (
                mean(&hubs[5]) - mean(&none),
                2.0 * combined(&hubs[5], &none),
            );
            let (s, lo, _) = slope_ci(&hubs);
            parts.push((
                format!("{game}: ten hub bots move cooperation"),
                scored(gap > two && lo > 0.0, format!("φ 1: {:.3} vs none {:.3} (gap {gap:.3}, 2 SE {two:.3}); slope {s:+.4}, CI lower {lo:+.4}; hub means by φ {}", mean(&hubs[5]), mean(&none), hub_means.join(", "))),
            ));
        }
        let off: Vec<String> = PHIS
            .iter()
            .zip(&random)
            .filter(|(_, r)| (mean(r) - mean(&none)).abs() > 2.0 * combined(r, &none))
            .map(|(phi, r)| format!("φ {phi}: {:.3}", mean(r)))
            .collect();
        let shown = if off.is_empty() {
            "none".to_string()
        } else {
            off.join(", ")
        };
        parts.push((
            format!("{game}: ten random bots do not"),
            scored(
                off.is_empty(),
                format!("none {:.3}; outside 2 SE: {shown}", mean(&none)),
            ),
        ));
    }
    all_of(parts).with(&reduced(PROTOCOL_GUO, RUNS_GUO))
}

fn guo_hub_explanation(_: &[u64]) -> Outcome {
    let base = guo_net(Structure::BarabasiAlbert, SD6);
    let (s_sum, lo_sum, hi_sum) = slope_ci(&phi_runs(&base));
    let (s_mean, lo_mean, hi_mean) = slope_ci(&phi_runs(&HybridConfig {
        accumulate: Accumulate::Mean,
        ..base
    }));
    reported(format!(
        "snowdrift on Barabási–Albert, 4,871 random bots: slope with summed payoffs {s_sum:+.4} ({lo_sum:+.4} to {hi_sum:+.4}); with mean payoffs {s_mean:+.4} ({lo_mean:+.4} to {hi_mean:+.4}){}",
        reduced(PROTOCOL_GUO, RUNS_GUO)
    ))
}

// ---- Si et al. -------------------------------------------------------------------------------

fn si_lattice(framework: Framework, r: f64, rho: f64, side: u32) -> HybridConfig {
    HybridConfig {
        framework,
        r,
        bot_fraction: rho,
        side,
        nodes: side * side,
        ..si_defaults()
    }
}

fn si_well_mixed(
    framework: Framework,
    r: f64,
    rho: f64,
    division: Division,
    nodes: u32,
) -> HybridConfig {
    HybridConfig {
        structure: Structure::WellMixed,
        nodes,
        framework,
        r,
        bot_fraction: rho,
        division,
        ..si_defaults()
    }
}

/// The published code's runs at (r, ρ) from Task 11's CSV.
fn theirs(slug: &str, r: f64, rho: f64) -> Result<Vec<f64>, String> {
    theirs_in(Path::new(DOCK), slug, r, rho, RUNS_SI as usize)
}

/// `theirs` against the CSVs in `dir`, which must hold exactly `want` runs at (r, ρ).
fn theirs_in(dir: &Path, slug: &str, r: f64, rho: f64, want: usize) -> Result<Vec<f64>, String> {
    let path = dir.join(format!("{slug}.csv"));
    let text = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "{}: {e} (run survey/hybrid_si/dock.py first)",
            path.display()
        )
    })?;
    let v: Vec<f64> = text
        .lines()
        .skip(1)
        .filter_map(|line| {
            let f: Vec<&str> = line.split(',').collect();
            let (lr, lrho, c): (f64, f64, f64) = (
                f.first()?.parse().ok()?,
                f.get(1)?.parse().ok()?,
                f.get(5)?.parse().ok()?,
            );
            ((lr - r).abs() < 1e-9 && (lrho - rho).abs() < 1e-9).then_some(c)
        })
        .collect();
    if v.is_empty() {
        Err(format!("{slug}: no runs at r {r}, ρ {rho}"))
    } else if v.len() != want {
        Err(format!(
            "{slug}: {} runs at r {r}, ρ {rho}, but docking needs exactly {want}, our run count (spec, Measurement and compute)",
            v.len()
        ))
    } else {
        Ok(v)
    }
}

fn s1(_: &[u64]) -> Outcome {
    let mut parts = vec![];
    for (slug, lattice, (framework, name)) in [
        ("fig1a", false, FRAMEWORKS[0]),
        ("fig1b", false, FRAMEWORKS[1]),
        ("fig1c", false, FRAMEWORKS[2]),
        ("fig3a", true, FRAMEWORKS[0]),
        ("fig3b", true, FRAMEWORKS[1]),
        ("fig3c", true, FRAMEWORKS[2]),
    ] {
        let rhos: &[f64] = if lattice { &[0.0, 0.3, 0.5] } else { &[0.3] };
        for r in [0.05, 0.1, 0.15] {
            for &rho in rhos {
                let them = match theirs(slug, r, rho) {
                    Ok(v) => v,
                    Err(e) => {
                        return Outcome {
                            verdict: Verdict::Untestable,
                            measured: e,
                            detail: String::new(),
                        }
                    }
                };
                let config = if lattice {
                    si_lattice(framework, r, rho, 100)
                } else {
                    si_well_mixed(framework, r, rho, Division::Integer, 10_000)
                };
                let us = runs_of(&config, RUNS_SI, SI_STEPS, SI_WINDOW);
                let d = mean(&us) - mean(&them);
                let ok = d.abs() <= 3.0 * combined(&us, &them) || d.abs() < 0.02;
                parts.push((
                    format!(
                        "{} {name}, r {r}, ρ {rho}",
                        if lattice { "lattice" } else { "well-mixed" }
                    ),
                    scored(
                        ok,
                        format!(
                            "ours {:.4} ± {:.4} ({} runs), theirs {:.4} ± {:.4} ({} runs)",
                            mean(&us),
                            se(&us),
                            us.len(),
                            mean(&them),
                            se(&them),
                            them.len()
                        ),
                    ),
                ));
            }
        }
    }
    all_of(parts).with(&reduced(PROTOCOL_SI, RUNS_SI))
}

fn s2_with(division: Division, nodes: u32) -> Outcome {
    let rs = [0.0, 0.04, 0.08, 0.12, 0.16, 0.2];
    let rhos = [0.0, 0.1, 0.3, 0.5];
    let mut parts = vec![];
    for (framework, name) in FRAMEWORKS {
        let m: Vec<Vec<f64>> = rhos
            .iter()
            .map(|&rho| {
                rs.iter()
                    .map(|&r| {
                        mean(&runs_of(
                            &si_well_mixed(framework, r, rho, division, nodes),
                            RUNS_SI,
                            SI_STEPS,
                            SI_WINDOW,
                        ))
                    })
                    .collect()
            })
            .collect();
        let table: Vec<String> = rhos
            .iter()
            .zip(&m)
            .map(|(rho, row)| {
                format!(
                    "ρ {rho}: {}",
                    row.iter()
                        .map(|v| format!("{v:.3}"))
                        .collect::<Vec<_>>()
                        .join(" ")
                )
            })
            .collect();
        let raised = (1..4).all(|k| (0..rs.len()).all(|j| m[k][j] > m[0][j]));
        parts.push((
            format!("{name}: bots raise cooperation at every r"),
            scored(raised, format!("r {rs:?}; {}", table.join("; "))),
        ));
        if framework != Framework::Discrete {
            let near = (0..3).all(|j| m[3][j] >= 0.8);
            parts.push((
                format!("{name}: ≥ 0.8 at ρ 0.5 for r ≤ 0.1"),
                scored(
                    near,
                    format!("{:.3}, {:.3}, {:.3}", m[3][0], m[3][1], m[3][2]),
                ),
            ));
        }
    }
    all_of(parts).with(&format!(
        "N {nodes}, division {division:?}{}",
        reduced(PROTOCOL_SI, RUNS_SI)
    ))
}

fn s3_with(mixed_draw: MixedDraw, side: u32, frameworks: &[(Framework, &str)]) -> Outcome {
    let rs: Vec<f64> = (0..=20).map(|k| 0.01 * f64::from(k)).collect();
    let mut parts = vec![];
    for &(framework, name) in frameworks {
        let promote = framework == Framework::Mixed;
        let (mut ok, mut rows) = (true, vec![]);
        for (k, &r) in rs.iter().enumerate() {
            if !promote && k >= 13 {
                continue;
            }
            let at = |rho: f64| {
                runs_of(
                    &HybridConfig {
                        mixed_draw,
                        ..si_lattice(framework, r, rho, side)
                    },
                    RUNS_SI,
                    SI_STEPS,
                    SI_WINDOW,
                )
            };
            let (none, half) = (at(0.0), at(0.5));
            let (d, two) = (mean(&half) - mean(&none), 2.0 * combined(&half, &none));
            let good = if promote { d > two } else { d < -two };
            ok &= good;
            rows.push(format!(
                "{}r {r:.2}: {:.3} vs {:.3}",
                if good { "" } else { "OFF " },
                mean(&half),
                mean(&none)
            ));
        }
        parts.push((
            format!(
                "{name}: bots {}",
                if promote {
                    "promote at every r"
                } else {
                    "suppress for r < 0.13"
                }
            ),
            scored(ok, rows.join(" · ")),
        ));
    }
    all_of(parts).with(&format!(
        "side {side}, {mixed_draw:?}{}",
        reduced(PROTOCOL_SI, RUNS_SI)
    ))
}

fn s4(lattice: bool) -> Outcome {
    let thetas: Vec<f64> = (0..=10).map(|k| 0.1 * f64::from(k)).collect();
    let base = if lattice {
        si_lattice(Framework::Mixed, 0.01, 0.3, 100)
    } else {
        si_well_mixed(Framework::Mixed, 0.01, 0.3, Division::Integer, 10_000)
    };
    let runs: Vec<Vec<f64>> = thetas
        .iter()
        .map(|&theta| {
            runs_of(
                &HybridConfig {
                    theta,
                    ..base.clone()
                },
                RUNS_SI,
                SI_STEPS,
                SI_WINDOW,
            )
        })
        .collect();
    let means: Vec<f64> = runs.iter().map(|r| mean(r)).collect();
    let best = (0..means.len())
        .max_by(|&a, &b| means[a].total_cmp(&means[b]))
        .unwrap();
    let (margin, two) = (
        means[best] - means[10],
        2.0 * combined(&runs[best], &runs[10]),
    );
    let shown: Vec<String> = means.iter().map(|m| format!("{m:.3}")).collect();
    scored(
        best > 0 && best < 10 && margin > two,
        format!(
            "argmax θ {:.1} ({:.4}); θ 1 {:.4}; margin {margin:.4} vs 2 SE {two:.4}; by θ {}{}",
            thetas[best],
            means[best],
            means[10],
            shown.join(" "),
            reduced(PROTOCOL_SI, RUNS_SI)
        ),
    )
}

// ---- Shirado and Christakis ------------------------------------------------------------------

fn pearson(x: &[f64], y: &[f64]) -> f64 {
    let (mx, my) = (mean(x), mean(y));
    let sxy: f64 = x.iter().zip(y).map(|(a, b)| (a - mx) * (b - my)).sum();
    let sxx: f64 = x.iter().map(|a| (a - mx).powi(2)).sum();
    let syy: f64 = y.iter().map(|b| (b - my).powi(2)).sum();
    sxy / (sxx * syy).sqrt()
}

fn c1(_: &[u64]) -> Outcome {
    let seeds: Vec<u64> = (1..=230).collect();
    let rows: Vec<(f64, f64, u32)> = on_threads(&seeds, |g| {
        let mut rng = seeded(g);
        let adj = barabasi_albert(20, 2, &mut rng);
        let start: Vec<u8> = (0..20).map(|_| rng.gen_range(0..3u32) as u8).collect();
        let solutions = coloring::colorings(&adj, 3) as f64;
        let (mut total, mut censored) = (0.0, 0);
        for _ in 0..100 {
            match coloring::convergence_steps(&adj, &start, 10_000_000, &mut rng) {
                Some(s) => total += s as f64,
                None => {
                    censored += 1;
                    total += 1e7;
                }
            }
        }
        (solutions.ln(), (total / 100.0).max(1.0).ln(), censored)
    });
    let (x, y): (Vec<f64>, Vec<f64>) = rows.iter().map(|&(a, b, _)| (a, b)).unzip();
    let censored: u32 = rows.iter().map(|r| r.2).sum();
    let r = pearson(&x, &y);
    scored(r <= -0.9, format!("Pearson r of ln solutions and ln mean steps over 230 graphs: {r:.4} (the paper's −0.990); censored repeats (counted at the 10⁷ cap): {censored}"))
}

/// The share of `SESSIONS` sessions solved within 300 s with `bots` bots on the hubs.
fn solve_rate(bots: usize, noise: f64, errant: f64) -> f64 {
    let seeds: Vec<u64> = (1..=SESSIONS).collect();
    let solved = on_threads(&seeds, |s| {
        let mut rng = seeded(s);
        let adj = barabasi_albert(20, 2, &mut rng);
        let degrees: Vec<usize> = adj.iter().map(Vec::len).collect();
        let bot = place(&degrees, bots, Placement::Hubs, &mut rng);
        let rules = coloring::Rules {
            noise,
            errant,
            reaction: 1.59,
            bot_interval: 1.5,
            time_limit: 300.0,
        };
        f64::from(u8::from(
            Session::new(adj, bot, rules, &mut rng)
                .play(&mut rng)
                .is_some(),
        ))
    });
    mean(&solved)
}

/// The errant rate whose control solve fraction is nearest 0.67 (20 humans; ties to the lower e).
fn fitted_errant() -> (f64, f64) {
    (0..=40)
        .map(|k| {
            let e = 0.005 * f64::from(k);
            (e, solve_rate(0, 0.0, e))
        })
        .min_by(|a, b| (a.1 - 0.67).abs().total_cmp(&(b.1 - 0.67).abs()))
        .unwrap()
}

fn c2_at(errant: f64, label: String) -> Outcome {
    let (p0, p10, p30) = (
        solve_rate(3, 0.0, errant),
        solve_rate(3, 0.1, errant),
        solve_rate(3, 0.3, errant),
    );
    let n = SESSIONS as f64;
    let two = |a: f64, b: f64| 2.0 * ((a * (1.0 - a) + b * (1.0 - b)) / n).sqrt();
    scored(
        p10 - p0 > two(p10, p0) && p10 - p30 > two(p10, p30),
        format!("{label}: solved within 300 s with three central bots at noise 0, 0.1, 0.3: {p0:.3}, {p10:.3}, {p30:.3} ({SESSIONS} sessions each; 2 SE {:.3} and {:.3})", two(p10, p0), two(p10, p30)),
    )
}

fn c2(_: &[u64]) -> Outcome {
    let (e, rate) = fitted_errant();
    c2_at(e, format!("fitted e {e:.3} (control solve fraction {rate:.3}; the paper's 20/30; its measured human mean e is 0.041)"))
}

fn c2_observed(_: &[u64]) -> Outcome {
    c2_at(0.041, "e 0.041".into())
}

// ---- The pilot -------------------------------------------------------------------------------

fn pilot(_: &[u64]) -> Outcome {
    let rows: Vec<(&str, HybridConfig, u64, u64)> = vec![
        (
            "Booker lattice, B5 and B6",
            booker_lattice(0.25, 1.0),
            BOOKER_STEPS,
            30 * RUNS_BOOKER_LATTICE,
        ),
        (
            "Booker lattice under sweep_steps, B5",
            HybridConfig {
                step: StepUnit::Sweep,
                ..booker_lattice(0.25, 1.0)
            },
            BOOKER_STEPS,
            18 * RUNS_BOOKER_LATTICE,
        ),
        (
            "Booker Barabási–Albert, B6",
            booker_ba(0.25, 1.0),
            BOOKER_STEPS,
            12 * BOOKER_NETWORKS * RUNS_PER_NETWORK,
        ),
        (
            "Guo Algorithm I, G3",
            alg_i(SD3, 1.0, 0.9, 10.0, 0.5),
            ALG_I_STEPS,
            144 * RUNS_ALG_I,
        ),
        (
            "Guo networks, G4",
            guo_net(Structure::BarabasiAlbert, SD6),
            GUO_STEPS,
            36 * RUNS_GUO,
        ),
        (
            "Guo Barabási–Albert with ten bots, G5",
            HybridConfig {
                bot_count: 10,
                ..guo_net(Structure::BarabasiAlbert, PD)
            },
            GUO_STEPS,
            39 * RUNS_GUO,
        ),
        (
            "Guo Barabási–Albert, the hub evaluation",
            guo_net(Structure::BarabasiAlbert, SD6),
            GUO_STEPS,
            12 * RUNS_GUO,
        ),
        (
            "Si lattice (mixed, the costliest), S1, S3, S4, per_round",
            si_lattice(Framework::Mixed, 0.1, 0.3, 100),
            SI_STEPS,
            206 * RUNS_SI,
        ),
        (
            "Si well-mixed, S1, S2 twice, S4",
            si_well_mixed(Framework::Mixed, 0.1, 0.3, Division::Integer, 10_000),
            SI_STEPS,
            164 * RUNS_SI,
        ),
    ];
    let line = |name: &str, runs: u64, hours: f64| {
        format!(
            "{name}: {runs} runs ≈ {hours:.2} h on {} threads{}",
            threads(),
            if hours > BUDGET_HOURS {
                " — over budget"
            } else {
                ""
            }
        )
    };
    let mut out: Vec<String> = rows
        .into_iter()
        .map(|(name, config, steps, runs)| line(name, runs, projected_hours(&config, steps, runs)))
        .collect();
    // The coloring protocols run outside `Runner`: one graph's 100 repeats (C1) and one control
    // session (C2's 41-value fit and its six arms, 200 sessions each), timed directly.
    let started = Instant::now();
    let mut rng = seeded(1);
    let adj = barabasi_albert(20, 2, &mut rng);
    let start: Vec<u8> = (0..20).map(|_| rng.gen_range(0..3u32) as u8).collect();
    for _ in 0..100 {
        coloring::convergence_steps(&adj, &start, 10_000_000, &mut rng);
    }
    out.push(line(
        "Coloring convergence, C1",
        230,
        started.elapsed().as_secs_f64() * 230.0 / threads() / 3600.0,
    ));
    let started = Instant::now();
    let rules = coloring::Rules {
        noise: 0.0,
        errant: 0.041,
        reaction: 1.59,
        bot_interval: 1.5,
        time_limit: 300.0,
    };
    Session::new(adj, vec![false; 20], rules, &mut rng).play(&mut rng);
    let sessions = 47 * SESSIONS;
    out.push(line(
        "Coloring sessions, C2 twice",
        sessions,
        started.elapsed().as_secs_f64() * sessions as f64 / threads() / 3600.0,
    ));
    reported(out.join(" · "))
}

pub fn claims() -> Vec<Claim> {
    vec![
        claim("hybrid.pilot", "hybrid-si-mixed-lattice", OURS, "throughput of each simulated protocol and its projected wall time (amendment 8)", pilot),
        claim("hybrid.b1-switch", "hybrid-booker-samaritan", BOOKER, "Fig. 1, Samaritan bots: the all-C weight switches on at a few bots at β 0.1, at about 55–60 at β 1, and not by 100 at β 2.5 (N = 100, b = 2)", b1),
        claim("hybrid.b2-samaritan-vs-discriminatory", "hybrid-booker-discriminatory", BOOKER, "Samaritan bots beat discriminatory bots at low β and lose to them at high β (Fig. 1)", b2),
        claim("hybrid.b3-eq18", "hybrid-booker-samaritan", BOOKER, "Figs. 2–3 on Eq. 18: lower β cooperates at lower b, and x = 0 is a fixed point at α = 0.5 (registered expectation: fails)", |_| b3(BotKind::Samaritan)),
        claim("hybrid.b3-eq20", "hybrid-booker-discriminatory", BOOKER, "the same two statements on Eq. 20 (discriminatory bots), judged separately", |_| b3(BotKind::Discriminatory)),
        claim("hybrid.b4-errata", "hybrid-booker-samaritan", BOOKER, "the printed G_D (Eq. 14) and simplified α_c differ from the formulas they come from; the corrected forms agree", b4),
        claim("hybrid.b5-lattice", "hybrid-booker-lattice-alpha", BOOKER, "on the lattice, Samaritan bots raise cooperation at β 0.1 as α grows and barely help at β 5", |_| b5_with(StepUnit::Paper)),
        claim("hybrid.b5-lattice-sweep-steps", "hybrid-booker-lattice-alpha", BOOKER, "B5 with N-update steps (variant row, amendment 10)", |_| b5_with(StepUnit::Sweep)),
        claim("hybrid.b6-topology", "hybrid-booker-samaritan", BOOKER, "topology does not change the final state: Barabási–Albert and lattice agree", b6),
        claim("hybrid.g1-phase-diagrams", "hybrid-guo-snowdrift-lattice", GUO, "Theorems 1–3, Figs. 3–4: the replicator phase diagrams; snowdrift y_min = 0.4", g1),
        claim("hybrid.g2-master-equation", "hybrid-guo-snowdrift-lattice", GUO, "Fig. 5 lines: the master equation's stable roots", g2),
        claim("hybrid.g3-algorithm-i", "hybrid-guo-snowdrift-lattice", GUO, "Fig. 5 dots: Algorithm I with N = 1,000 agrees with the master equation", g3),
        claim("hybrid.g4-networks", "hybrid-guo-phi-networks", GUO, "Fig. 6: snowdrift cooperation falls with φ on the lattice and rises on Erdős–Rényi and Barabási–Albert; dilemma cooperation rises on all three", g4),
        claim("hybrid.g5-hubs", "hybrid-guo-hub-bots", GUO, "Fig. 7: ten hub bots move cooperation; ten random bots do not", g5),
        claim("hybrid.g-hub-explanation", "hybrid-guo-snowdrift-scale-free", OURS, "snowdrift on Barabási–Albert with random bots: the φ slope with summed and with mean payoffs (reported)", guo_hub_explanation),
        claim("hybrid.s1-docking", "hybrid-si-mixed-lattice", SI, "our runs dock with the published code at 27 lattice and 9 well-mixed points", s1),
        claim("hybrid.s2-well-mixed-code", "hybrid-si-mixed-lattice", SI, "Fig. 1 as the code runs it: bots raise well-mixed cooperation; continuous and mixed stay ≥ 0.8 up to r ≈ 0.1 at ρ 0.5", |_| s2_with(Division::Integer, 10_000)),
        claim("hybrid.s2-well-mixed-stated", "hybrid-si-mixed-lattice", SI, "Fig. 1 on the stated game (division: float, amendment 1)", |_| s2_with(Division::Float, 10_000)),
        claim("hybrid.s3-lattice", "hybrid-si-lattice-r", SI, "Fig. 3: on the lattice bots suppress cooperation in discrete and continuous for r < 0.13 and promote it in mixed at every r", |_| s3_with(MixedDraw::PerInteraction, 100, &FRAMEWORKS)),
        claim("hybrid.s4-optimum-lattice", "hybrid-si-mixed-lattice", SI, "Fig. 6 as captioned (lattice): an interior optimal θ in mixed at r 0.01", |_| s4(true)),
        claim("hybrid.s4-optimum-well-mixed", "hybrid-si-mixed-lattice", SI, "Fig. 6 as the deposited code runs it (well-mixed): an interior optimal θ in mixed at r 0.01", |_| s4(false)),
        claim("hybrid.s3-per-round", "hybrid-si-mixed-lattice", OURS, "S3's mixed rows with one action per round (Fig. A1), reported", |_| as_reported(s3_with(MixedDraw::PerRound, 100, &FRAMEWORKS[2..]))),
        claim("hybrid.s2-preprint-n", "hybrid-si-mixed-lattice", OURS, "S2 with the preprint's N = 500, reported", |_| as_reported(s2_with(Division::Integer, 500))),
        claim("hybrid.s2-journal-n", "hybrid-si-mixed-lattice", OURS, "S2 with the journal's N = 10⁵, reported if affordable", |_| {
            guarded(&si_well_mixed(Framework::Mixed, 0.1, 0.3, Division::Integer, 100_000), SI_STEPS, 72 * RUNS_SI, || as_reported(s2_with(Division::Integer, 100_000)))
        }),
        claim("hybrid.s3-journal-n", "hybrid-si-lattice-r", OURS, "S3 on a 316 × 316 lattice (the journal's N = 10⁵, nearest square), reported if affordable", |_| {
            guarded(&si_lattice(Framework::Mixed, 0.1, 0.3, 316), SI_STEPS, 126 * RUNS_SI, || as_reported(s3_with(MixedDraw::PerInteraction, 316, &FRAMEWORKS)))
        }),
        claim("hybrid.c1-convergence", "hybrid-coloring-central", SHIRADO, "Extended Data Fig. 2: ln solutions and ln mean convergence steps correlate at −0.990", c1),
        claim("hybrid.c2-mechanism", "hybrid-coloring-central", SHIRADO, "with e fit to the control arm, 10% central bots solve more often than 0% and 30% central bots (mechanism sufficiency, not the human result)", c2),
        claim("hybrid.c2-observed-e", "hybrid-coloring-central", SHIRADO, "the same at the paper's measured human errant rate, 0.041", c2_observed),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(dir: &Path, rows: usize) {
        std::fs::create_dir_all(dir).unwrap();
        let mut text = String::from("r,rho,run,a,b,cooperation\n");
        for k in 0..rows {
            text.push_str(&format!("0.1,0.3,{k},0,0,0.5\n"));
        }
        std::fs::write(dir.join("fig1a.csv"), text).unwrap();
    }

    #[test]
    fn docking_refuses_a_reference_with_the_wrong_run_count() {
        let dir = std::env::temp_dir().join(format!("hybrid-s1-{}", std::process::id()));
        reference(&dir, 1);
        let short = theirs_in(&dir, "fig1a", 0.1, 0.3, 4);
        reference(&dir, 4);
        let exact = theirs_in(&dir, "fig1a", 0.1, 0.3, 4);
        let missing = theirs_in(&dir, "fig1a", 0.15, 0.3, 4);
        std::fs::remove_dir_all(&dir).unwrap();
        assert!(short.unwrap_err().contains("1 runs"));
        assert_eq!(exact.unwrap().len(), 4);
        assert!(missing.is_err());
    }

    #[test]
    fn g2_dilemma_fails_where_a_point_has_no_stable_root() {
        assert!(matches!(
            all_roots_low(&[vec![0.01], vec![0.0]]).verdict,
            Verdict::Holds
        ));
        let empty = all_roots_low(&[vec![0.01], vec![]]);
        assert!(matches!(empty.verdict, Verdict::Fails));
        assert!(empty.measured.contains("no stable root"));
    }
}
