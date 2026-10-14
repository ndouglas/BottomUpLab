//! Frame and inspection: four bars, one per attribute, scaled to the attribute's feasible
//! maximum, with the floor, the start and the human optimum marked on each, and a strip beneath
//! that places the current utility between the start, the current proxy's limit and the optimum.
use super::{
    world::MisalignedWorld,
    worlds::{L, NAMES},
};
use crate::opinions::Canvas;
use serde_json::json;

pub const BAR: usize = 40;
pub const GAP: usize = 8;
pub const BAR_HEIGHT: usize = 120;
pub const STRIP: usize = 16;
pub const WIDE: usize = L * BAR + (L + 1) * GAP;
pub const TALL: usize = BAR_HEIGHT + GAP + STRIP + GAP;
const PROXY: [u8; 3] = [70, 130, 230];
const UNMENTIONED: [u8; 3] = [235, 150, 60];
const LOSS: [u8; 3] = [230, 70, 70];
const OFF: [u8; 3] = [150, 150, 150];
const INK: [u8; 3] = [20, 20, 20];
const MARK: [u8; 3] = [240, 210, 70];

pub fn size(_w: &MisalignedWorld) -> (u32, u32) {
    (WIDE as u32, TALL as u32)
}

fn bar_x(i: usize) -> usize {
    GAP + i * (BAR + GAP)
}

/// Which bar covers column x, if any.
fn bar_at(x: usize) -> Option<usize> {
    (0..L).find(|&i| (bar_x(i)..bar_x(i) + BAR).contains(&x))
}

struct Scale {
    lo: f64,
    hi: f64,
}

impl Scale {
    fn y(&self, v: f64) -> usize {
        let f = ((v - self.lo) / (self.hi - self.lo)).clamp(0.0, 1.0);
        BAR_HEIGHT - 1 - (f * (BAR_HEIGHT - 1) as f64).round() as usize
    }
}

fn scale(w: &MisalignedWorld) -> Scale {
    let shape = w.shape();
    let lo = shape.floor.min(0.0);
    Scale {
        lo,
        hi: shape.maximum().max(lo + 1e-9),
    }
}

pub fn render(w: &MisalignedWorld, mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    if !["", "bars"].contains(&mode) {
        return Err(format!("unknown color mode {mode:?} (expected bars)"));
    }
    let mut canvas = Canvas { buf, wide: 0 };
    canvas.clear(WIDE, TALL);
    let shape = w.shape();
    let sc = scale(w);
    let state = w.state();
    let (optimum, u_star) = shape.optimum();
    let (_, u_limit) = shape.fixed_proxy_limit(w.proxy());
    for i in 0..L {
        let x0 = bar_x(i);
        let color = if w.proxy()[i] { PROXY } else { UNMENTIONED };
        let top = sc.y(state[i]);
        for y in top..BAR_HEIGHT {
            let row = if w.frozen()[i] && y % 4 == 0 {
                INK
            } else {
                color
            };
            for x in x0..x0 + BAR {
                canvas.put(x, y, row);
            }
        }
        let floor_y = sc.y(shape.floor);
        for x in x0..x0 + BAR {
            canvas.put(x, floor_y, INK);
        }
        let s = sc.y(w.start()[i]);
        for x in x0..x0 + BAR {
            if s > 0 {
                canvas.put(x, s - 1, INK);
            }
            if s + 1 < BAR_HEIGHT {
                canvas.put(x, s + 1, INK);
            }
        }
        canvas.put(x0, s, INK);
        canvas.put(x0 + BAR - 1, s, INK);
        let o = sc.y(optimum[i]);
        for x in x0..x0 + BAR {
            canvas.put(x, o, MARK);
            if o > 0 {
                canvas.put(x, o - 1, MARK);
            }
        }
    }
    // The strip.
    let u = shape.utility(&state);
    let u0 = shape.utility(&w.start());
    let ulo = u0.min(u_limit).min(u);
    let uhi = u_star.max(u);
    let span = (uhi - ulo).max(1e-9);
    let (x0, x1) = (GAP, WIDE - GAP);
    let x_of = |v: f64| {
        (x0 as f64 + ((v - ulo) / span).clamp(0.0, 1.0) * (x1 - x0 - 1) as f64).round() as usize
    };
    let fill = if w.switched_off() {
        OFF
    } else if u < u0 {
        LOSS
    } else {
        PROXY
    };
    let top = BAR_HEIGHT + GAP;
    for y in top..top + STRIP {
        for x in x0..=x_of(u) {
            canvas.put(x, y, fill);
        }
        canvas.put(x_of(u_limit), y, INK);
        canvas.put(x_of(u_star), y, MARK);
        if y == top || y + 1 == top + STRIP {
            canvas.put(x_of(u0), y, INK);
        }
    }
    canvas.put(x_of(u0), top + STRIP / 2, INK);
    Ok(())
}

pub fn inspect(w: &MisalignedWorld, x: u32, y: u32) -> Result<String, String> {
    if x as usize >= WIDE || y as usize >= TALL {
        return Err(format!("({x}, {y}) is not in the frame"));
    }
    let (x, y) = (x as usize, y as usize);
    let shape = w.shape();
    let state = w.state();
    let sens = shape.sensitivities(&state);
    let outcome = w.outcome();
    if y < BAR_HEIGHT {
        if let Some(i) = bar_at(x) {
            let rank = 1 + (0..L)
                .filter(|&j| sens[j] > sens[i] || (sens[j] == sens[i] && j < i))
                .count();
            return Ok(json!({
                "panel": "attribute", "index": i, "name": NAMES[i], "x": x, "y": y,
                "value": state[i], "in_proxy": w.proxy()[i], "frozen": w.frozen()[i],
                "sensitivity": sens[i], "rank": rank, "start": w.start()[i],
                "limit": outcome.limit.0[i], "optimum": outcome.optimum.0[i], "floor": shape.floor,
                "tick": w.tick, "outcome": outcome, "agent": null,
            })
            .to_string());
        }
    }
    Ok(json!({
        "panel": "utility", "x": x, "y": y,
        "utility": outcome.utility, "utility_start": outcome.utility_start, "optimum": outcome.optimum.1,
        "limit": outcome.limit.1, "gain": outcome.utility - outcome.utility_start,
        "interactions": outcome.interactions, "switched_off": w.switched_off(),
        "tick": w.tick, "outcome": outcome, "agent": null,
    })
    .to_string())
}
