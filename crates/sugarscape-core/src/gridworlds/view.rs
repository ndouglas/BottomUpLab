//! Frames and inspection. The board at 12 px per cell in the reference's colors; `policy`
//! colors floor cells by the greedy action with a dark edge toward it; `value` shades floor
//! cells by the state's maximal Q. Inspection returns the cell's Q values and the run's state.
use super::{
    config::{EnvKind, Phase},
    env::{Env, Environment},
    grid::{Action, Cell},
    world::GridworldsWorld,
};
use crate::opinions::Canvas;
use serde_json::json;

pub const CELL: usize = 12;
const AGENT: [u8; 3] = [40, 110, 220];
const WALL: [u8; 3] = [60, 60, 60];
const FLOOR: [u8; 3] = [230, 230, 230];
const INK: [u8; 3] = [20, 20, 20];
const ACTION_COLORS: [[u8; 3]; 4] = [
    [80, 160, 220],
    [220, 120, 80],
    [120, 200, 120],
    [200, 120, 200],
];

pub fn size(w: &GridworldsWorld) -> (u32, u32) {
    let g = w.env().grid();
    ((g.cols * CELL) as u32, (g.rows * CELL) as u32)
}

fn tile_color(ch: char, kind: EnvKind) -> [u8; 3] {
    match ch {
        '#' => WALL,
        ' ' => FLOOR,
        'A' => AGENT,
        'G' => [60, 170, 80],
        'I' => [230, 120, 230],
        'B' => [120, 80, 210],
        'X' => [0, 150, 160],
        'P' => [230, 200, 40],
        'S' => [160, 160, 160],
        '>' | 'v' | '<' | '^' => [240, 150, 60],
        'T' => [50, 160, 60],
        't' => [150, 110, 60],
        'O' => [150, 60, 200],
        'W' if kind == EnvKind::Island => [70, 130, 230],
        'W' => [220, 170, 60],
        'L' => [220, 70, 50],
        '*' => [200, 180, 50],
        '1' => [240, 200, 40],
        '0' => [140, 140, 140],
        _ => [200, 200, 200],
    }
}

fn fill(canvas: &mut Canvas, cell: Cell, color: [u8; 3]) {
    for dy in 0..CELL {
        for dx in 0..CELL {
            canvas.put(cell.col * CELL + dx, cell.row * CELL + dy, color);
        }
    }
}

/// A 3 px bar on the edge the action faces.
fn edge(canvas: &mut Canvas, cell: Cell, action: Action, color: [u8; 3]) {
    for k in 0..CELL {
        for d in 0..3 {
            let (x, y) = match action {
                Action::Up => (k, d),
                Action::Down => (k, CELL - 1 - d),
                Action::Left => (d, k),
                Action::Right => (CELL - 1 - d, k),
            };
            canvas.put(cell.col * CELL + x, cell.row * CELL + y, color);
        }
    }
}

fn agent_mark(canvas: &mut Canvas, cell: Cell) {
    for dy in 4..8 {
        for dx in 4..8 {
            canvas.put(cell.col * CELL + dx, cell.row * CELL + dy, [255, 255, 255]);
        }
    }
}

pub fn render(w: &GridworldsWorld, mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    if !["", "tiles", "policy", "value"].contains(&mode) {
        return Err(format!(
            "unknown color mode {mode:?} (expected tiles, policy or value)"
        ));
    }
    let env = w.env();
    let grid = env.grid();
    let board = env.board();
    let (wide, tall) = size(w);
    let mut canvas = Canvas { buf, wide: 0 };
    canvas.clear(wide as usize, tall as usize);
    let kind = w.config.environment;
    match mode {
        "" | "tiles" => {
            for (i, &ch) in board.iter().enumerate() {
                fill(&mut canvas, grid.cell(i), tile_color(ch, kind));
            }
        }
        "policy" => {
            for (i, &ch) in board.iter().enumerate() {
                let cell = grid.cell(i);
                if ch == '#' {
                    fill(&mut canvas, cell, WALL);
                    continue;
                }
                let a = w.table().greedy(env.key_at(cell));
                fill(&mut canvas, cell, ACTION_COLORS[a.index()]);
                edge(&mut canvas, cell, a, INK);
            }
            agent_mark(&mut canvas, env.agent());
        }
        _ => {
            let values: Vec<(usize, f64)> = board
                .iter()
                .enumerate()
                .filter(|(_, &ch)| ch != '#')
                .map(|(i, _)| (i, w.table().max(env.key_at(grid.cell(i)))))
                .collect();
            let lo = values.iter().map(|v| v.1).fold(f64::INFINITY, f64::min);
            let hi = values.iter().map(|v| v.1).fold(f64::NEG_INFINITY, f64::max);
            for (i, &ch) in board.iter().enumerate() {
                if ch == '#' {
                    fill(&mut canvas, grid.cell(i), WALL);
                }
            }
            for (i, v) in values {
                let f = if hi > lo { (v - lo) / (hi - lo) } else { 0.5 };
                let color = [
                    (40.0 + 200.0 * f) as u8,
                    (60.0 + 160.0 * f) as u8,
                    (200.0 - 160.0 * f) as u8,
                ];
                fill(&mut canvas, grid.cell(i), color);
            }
            agent_mark(&mut canvas, env.agent());
        }
    }
    Ok(())
}

pub fn inspect(w: &GridworldsWorld, x: u32, y: u32) -> Result<String, String> {
    let (wide, tall) = size(w);
    if x >= wide || y >= tall {
        return Err(format!("({x}, {y}) is not in the frame"));
    }
    let env = w.env();
    let grid = env.grid();
    let cell = Cell {
        row: y as usize / CELL,
        col: x as usize / CELL,
    };
    let board = env.board();
    let ch = board[grid.index(cell)];
    let key = env.key_at(cell);
    let q = w.table().q(key);
    let optima = w.optima();
    let mut v = json!({
        "environment": w.config.environment.as_str(),
        "cell": [cell.row, cell.col], "x": x, "y": y, "tile": ch.to_string(), "wall": ch == '#',
        "state_key": key, "q": q, "visits": w.table().visits(key), "greedy": w.table().greedy(key).name(),
        "agent_cell": [env.agent().row, env.agent().col],
        "episode": w.episodes(), "steps": w.steps(), "exploration": w.exploration_now(), "learning": w.learning(),
        "phase": match w.phase() { Phase::Train => "train", Phase::Test => "test" },
        "side_constraint": env.side_info(),
        "optima": optima, "tick": w.tick, "agent": null,
    });
    match env {
        Env::FriendFoe(f) => {
            v["estimate"] = json!(f.estimate());
            v["reward_left"] = json!(f.reward_left());
        }
        Env::OffSwitch(o) => {
            v["interrupting"] = json!(o.interrupting());
            v["pressed"] = json!(o.pressed());
        }
        Env::Whisky(d) => v["drunk"] = json!(d.drunk()),
        Env::Lava(l) => v["layout"] = json!(l.layout()),
        Env::Tomato(t) => v["watered"] = json!(t.watered().count_ones()),
        Env::Sokoban(s) => v["box"] = json!([s.box_cell().row, s.box_cell().col]),
        Env::Supervisor(s) => v["present"] = json!(s.present()),
        _ => {}
    }
    Ok(v.to_string())
}
