//! Frames and inspection. Paperclips: three bands over θ-bins (H's rule with the paper's
//! thresholds ticked, R's map, R's posterior after the latest observation; in learning mode the
//! first two bands are shaded by the tables' frequencies). Off-switch: Figure 3's surface at the
//! configured μ, Δ over the fixed σ × β grid with the zero boundary and a marker at (σ, β).
use super::{
    offswitch::{self, BETAS, SIGMAS},
    paperclips::{self, H_ACTIONS, R_ACTIONS},
    world::AssistanceWorld,
    Game, Mode,
};
use crate::opinions::Canvas;
use serde_json::json;

pub const BIN_WIDTH: usize = 2;
pub const BAND: usize = 24;
pub const POSTERIOR: usize = 48;
pub const HEIGHT: usize = 2 * BAND + POSTERIOR;
pub const CELL: usize = 12;
/// (0,2) and (0,90) blue; (1,1) and (50,50) violet; (2,0) and (90,0) orange.
const ACTION_COLORS: [[u8; 3]; 3] = [[70, 130, 230], [160, 110, 200], [235, 150, 60]];
const INK: [u8; 3] = [20, 20, 20];
const MARK: [u8; 3] = [240, 210, 70];
const BAR: [u8; 3] = [220, 220, 220];

pub fn size(w: &AssistanceWorld) -> (u32, u32) {
    match w.config.game {
        Game::Paperclips => ((w.config.bins() * BIN_WIDTH) as u32, HEIGHT as u32),
        Game::OffSwitch => ((SIGMAS.len() * CELL) as u32, (BETAS.len() * CELL) as u32),
    }
}

fn shade(c: [u8; 3], f: f64) -> [u8; 3] {
    let f = 0.3 + 0.7 * f.clamp(0.0, 1.0);
    [
        (f64::from(c[0]) * f) as u8,
        (f64::from(c[1]) * f) as u8,
        (f64::from(c[2]) * f) as u8,
    ]
}

pub fn render(w: &AssistanceWorld, mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    if !["", "bands", "surface"].contains(&mode) {
        return Err(format!(
            "unknown color mode {mode:?} (expected bands or surface)"
        ));
    }
    let (wide, tall) = size(w);
    let mut canvas = Canvas { buf, wide: 0 };
    canvas.clear(wide as usize, tall as usize);
    match w.config.game {
        Game::Paperclips => bands(w, &mut canvas),
        Game::OffSwitch => surface(w, &mut canvas),
    }
    Ok(())
}

fn bands(w: &AssistanceWorld, canvas: &mut Canvas) {
    let bins = w.config.bins();
    let wide = bins * BIN_WIDTH;
    let learning = w.config.mode == Mode::Learning;
    for (k, &a) in w.rule().iter().enumerate() {
        let strength = if learning {
            let row = &w.tables().robot[k];
            row[a as usize] / row.iter().sum::<f64>()
        } else {
            1.0
        };
        let color = shade(ACTION_COLORS[a as usize], strength);
        for dx in 0..BIN_WIDTH {
            for y in 0..BAND {
                canvas.put(k * BIN_WIDTH + dx, y, color);
            }
        }
    }
    for t in [41.0 / 92.0, 0.5, 51.0 / 92.0] {
        let x = ((t * wide as f64).round() as usize).min(wide - 1);
        for y in 0..4 {
            canvas.put(x, y, INK);
        }
    }
    for (a, &d) in w.map().iter().enumerate() {
        let strength = if learning {
            let row = &w.tables().human[a];
            row[d as usize] / row.iter().sum::<f64>()
        } else {
            1.0
        };
        let color = shade(ACTION_COLORS[d as usize], strength);
        let (x0, x1) = (a * wide / 3, (a + 1) * wide / 3);
        for x in x0..x1 {
            for y in BAND..2 * BAND {
                canvas.put(x, y, if x == x0 && a > 0 { INK } else { color });
            }
        }
    }
    let post = w.posterior();
    let top = post.iter().copied().fold(0.0, f64::max).max(1e-300);
    for (k, p) in post.iter().enumerate() {
        let h = ((p / top) * POSTERIOR as f64).round() as usize;
        for dx in 0..BIN_WIDTH {
            for y in 0..h {
                canvas.put(k * BIN_WIDTH + dx, HEIGHT - 1 - y, BAR);
            }
        }
    }
}

/// Δ on a diverging scale saturating at ±0.3: red below zero, blue above.
fn delta_color(delta: f64) -> [u8; 3] {
    let f = (delta.abs() / 0.3).clamp(0.0, 1.0);
    if delta < 0.0 {
        [(60.0 + 170.0 * f) as u8, (60.0 - 20.0 * f) as u8, 60]
    } else {
        [60, (70.0 + 60.0 * f) as u8, (80.0 + 170.0 * f) as u8]
    }
}

fn surface(w: &AssistanceWorld, canvas: &mut Canvas) {
    let n = SIGMAS.len();
    let rows = BETAS.len();
    let cells = &w.surface().cells;
    let marker = offswitch::nearest_cell(w.config.sigma, w.config.beta);
    let negative = |b: usize, s: usize| cells[b * n + s].delta < 0.0;
    for b in 0..rows {
        for s in 0..n {
            let color = delta_color(cells[b * n + s].delta);
            let right = s + 1 < n && negative(b, s + 1) != negative(b, s);
            let below = b > 0 && negative(b - 1, s) != negative(b, s);
            let row = rows - 1 - b;
            for dy in 0..CELL {
                for dx in 0..CELL {
                    let edge = (right && dx + 1 == CELL) || (below && dy + 1 == CELL);
                    let mark = marker == Some((s, b))
                        && (dx == 0 || dy == 0 || dx + 1 == CELL || dy + 1 == CELL);
                    canvas.put(
                        s * CELL + dx,
                        row * CELL + dy,
                        if mark {
                            MARK
                        } else if edge {
                            INK
                        } else {
                            color
                        },
                    );
                }
            }
        }
    }
}

pub fn inspect(w: &AssistanceWorld, x: u32, y: u32) -> Result<String, String> {
    let (wide, tall) = size(w);
    if x >= wide || y >= tall {
        return Err(format!("({x}, {y}) is not in the frame"));
    }
    let (x, y) = (x as usize, y as usize);
    match w.config.game {
        Game::Paperclips => {
            let bins = w.config.bins();
            let bin = x / BIN_WIDTH;
            let theta = paperclips::bin_mid(bin, bins);
            let human = w.rule()[bin];
            let deployment = w.map()[human as usize];
            let human_rewards: Vec<f64> = H_ACTIONS
                .iter()
                .map(|&a| paperclips::reward(theta, a))
                .collect();
            let deployment_rewards: Vec<f64> = R_ACTIONS
                .iter()
                .map(|&a| paperclips::reward(theta, a))
                .collect();
            let best = deployment_rewards
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let panel = if y < BAND {
                "human"
            } else if y < 2 * BAND {
                "robot"
            } else {
                "posterior"
            };
            let map_cell = (panel == "robot").then(|| (x * 3 / wide as usize).min(2));
            Ok(json!({
                "game": "paperclips", "panel": panel, "x": x, "y": y, "bin": bin, "theta": theta,
                "human_action": human, "human_rewards": human_rewards, "map_cell": map_cell,
                "deployment": deployment, "deployment_reward": deployment_rewards[deployment as usize],
                "optimal_deployment": paperclips::deployment_for(theta),
                "regret": best - deployment_rewards[deployment as usize], "posterior": w.posterior()[bin],
                "rule": w.rule(), "map": w.map(), "tick": w.tick, "outcome": w.outcome(), "agent": null,
            })
            .to_string())
        }
        Game::OffSwitch => {
            let s = x / CELL;
            let b = BETAS.len() - 1 - y / CELL;
            let cell = &w.surface().cells[b * SIGMAS.len() + s];
            Ok(json!({
                "game": "off_switch", "panel": "surface", "x": x, "y": y, "sigma": SIGMAS[s], "beta": BETAS[b],
                "mu": w.config.mu, "delta": cell.delta, "rationality_term": SIGMAS[s] * SIGMAS[s] * cell.rationality,
                "correction_term": w.config.mu.abs() * cell.prob_correct, "prob_correct": cell.prob_correct,
                "rationality": cell.rationality, "choice": offswitch::choice(cell), "configured": w.incentive(),
                "configured_choice": offswitch::choice(w.incentive()), "tick": w.tick, "outcome": w.outcome(), "agent": null,
            })
            .to_string())
        }
    }
}
