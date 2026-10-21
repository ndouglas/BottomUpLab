//! Frame and inspection: one cell per node, on a lattice in place and otherwise in index order on
//! the smallest square-ish frame that holds them. Humans run from defecting red to cooperating
//! blue by strategy; bots have their own shades (green acting C, orange acting D, purple for
//! Booker's discriminatory bots); coloring shows the game's three colors, bots darker.
use super::{
    booker::{Donation, Replicator},
    coloring::counts,
    config::{BotKind, Structure, World},
    guo::{Phase, Plane},
    world::{Body, HybridWorld},
};
use crate::opinions::Canvas;
use serde_json::json;

pub const HUMAN_C: [u8; 3] = [70, 130, 230];
pub const HUMAN_D: [u8; 3] = [230, 70, 70];
pub const BOT_C: [u8; 3] = [90, 200, 120];
pub const BOT_D: [u8; 3] = [245, 165, 50];
pub const BOT_MIRROR: [u8; 3] = [160, 100, 220];
pub const GAME: [[u8; 3]; 3] = [[80, 180, 80], [240, 150, 40], [150, 90, 200]];

/// The frame's width and height in cells.
pub fn layout(w: &HybridWorld) -> (usize, usize) {
    if w.config.structure == Structure::Lattice {
        let s = w.config.side as usize;
        return (s, s);
    }
    let n = w.config.size();
    let wide = (n as f64).sqrt().ceil() as usize;
    (wide, n.div_ceil(wide))
}

pub fn size(w: &HybridWorld) -> (u32, u32) {
    let (wide, tall) = layout(w);
    (wide as u32, tall as u32)
}

/// Where node i is drawn.
pub fn position(w: &HybridWorld, i: usize) -> (u32, u32) {
    let (wide, _) = layout(w);
    ((i % wide) as u32, (i / wide) as u32)
}

/// A human's color: red at strategy 0, blue at 1, the blend between.
pub fn blend(s: f64) -> [u8; 3] {
    let mix = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * s).round() as u8;
    [
        mix(HUMAN_D[0], HUMAN_C[0]),
        mix(HUMAN_D[1], HUMAN_C[1]),
        mix(HUMAN_D[2], HUMAN_C[2]),
    ]
}

pub fn darker(c: [u8; 3]) -> [u8; 3] {
    c.map(|v| (u16::from(v) * 3 / 5) as u8)
}

pub fn render(w: &HybridWorld, mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    if !["", "strategy"].contains(&mode) {
        return Err(format!("unknown color mode {mode:?} (expected strategy)"));
    }
    let (wide, tall) = layout(w);
    let mut canvas = Canvas { buf, wide: 0 };
    canvas.clear(wide, tall);
    match &w.runner().body {
        Body::Population(p) => {
            let mirror =
                w.config.world == World::Booker && w.config.bot_kind == BotKind::Discriminatory;
            for i in 0..p.nodes {
                let color = match (p.bot[i], mirror, p.action[i]) {
                    (true, true, _) => BOT_MIRROR,
                    (true, false, true) => BOT_C,
                    (true, false, false) => BOT_D,
                    (false, ..) => blend(p.strategy[i]),
                };
                canvas.put(i % wide, i / wide, color);
            }
        }
        Body::Coloring(s) => {
            for i in 0..s.color.len() {
                let c = GAME[s.color[i] as usize];
                canvas.put(i % wide, i / wide, if s.bot[i] { darker(c) } else { c });
            }
        }
    }
    Ok(())
}

/// The reference panel's second line: Booker's largest stable replicator rest point (Eq. 18 or
/// 20) at α = bots/N, and Guo's replicator phase (Theorems 1–3) at y = bots/humans.
fn reference_detail(w: &HybridWorld) -> (Option<f64>, Option<Phase>) {
    let c = &w.config;
    let Body::Population(p) = &w.runner().body else {
        return (None, None);
    };
    let bots = (p.nodes - p.humans.len()) as f64;
    match c.world {
        World::Booker => {
            let r = Replicator {
                game: Donation { b: c.b, c: c.c },
                alpha: bots / p.nodes as f64,
                beta_h: c.beta,
                beta_ai: c.beta,
                kind: c.bot_kind,
            };
            (r.top_stable(), None)
        }
        World::Guo if !p.humans.is_empty() => (
            None,
            Some(Plane { t: c.t, s: c.s }.phase_closed(bots / p.humans.len() as f64, c.phi)),
        ),
        _ => (None, None),
    }
}

pub fn inspect(w: &HybridWorld, x: u32, y: u32) -> Result<String, String> {
    let (wide, tall) = layout(w);
    if x as usize >= wide || y as usize >= tall {
        return Err(format!("({x}, {y}) is not in the frame"));
    }
    let i = y as usize * wide + x as usize;
    let mut v = match &w.runner().body {
        _ if i >= w.config.size() => json!({ "panel": "empty" }),
        Body::Population(p) => {
            let (replicator, phase) = reference_detail(w);
            json!({
                "panel": "node", "node": i, "bot": p.bot[i], "strategy": p.strategy[i],
                "action": if p.action[i] { "C" } else { "D" }, "degree": p.degree(i),
                "payoff": w.payoff_now(i), "cooperation": p.human_cooperation(),
                "replicator": replicator, "phase": phase,
            })
        }
        Body::Coloring(s) => json!({
            "panel": "node", "node": i, "bot": s.bot[i], "color": s.color[i], "degree": s.adj[i].len(),
            "conflicts_here": counts(&s.adj, &s.color, i)[s.color[i] as usize],
            "solutions": w.reference(), "conflicts": s.conflicts, "time": s.time, "solved_at": s.solved_at,
        }),
    };
    let map = v.as_object_mut().expect("an object");
    // serde_json writes a NaN as null.
    for (k, val) in [
        ("world", json!(w.config.world)),
        ("tick", json!(w.tick)),
        ("x", json!(x)),
        ("y", json!(y)),
        ("reference", json!(w.reference())),
        ("agent", serde_json::Value::Null),
    ] {
        map.insert(k.into(), val);
    }
    Ok(v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{hybrid::config::*, model::Model};

    #[test]
    fn frames_have_the_layout_size_and_draw_every_node() {
        let w = HybridWorld::new(booker_defaults(), 1).unwrap();
        assert_eq!(w.size(), (50, 50));
        let mut buf = vec![];
        for mode in ["", "strategy"] {
            w.render(mode, "", &mut buf).unwrap();
            assert_eq!(buf.len(), 50 * 50 * 4, "{mode}");
        }
        assert!(w.render("payoff", "", &mut buf).is_err());
        w.render("strategy", "", &mut buf).unwrap();
        let Body::Population(p) = &w.runner().body else {
            unreachable!()
        };
        for i in 0..2500 {
            let want = if p.bot[i] {
                BOT_C
            } else if p.action[i] {
                HUMAN_C
            } else {
                HUMAN_D
            };
            assert_eq!(&buf[i * 4..i * 4 + 3], &want, "node {i}");
        }
        let c = HybridWorld::new(coloring_defaults(), 1).unwrap();
        assert_eq!(c.size(), (5, 4), "20 nodes, 5 wide");
        let g = HybridWorld::new(
            HybridConfig {
                structure: Structure::BarabasiAlbert,
                nodes: 10,
                bot_count: 2,
                ..guo_defaults()
            },
            1,
        )
        .unwrap();
        assert_eq!((g.size(), position(&g, 9)), ((4, 3), (1, 2)));
        assert_eq!(g.locate(9), Some((1, 2)));
        assert_eq!(g.locate(10), None);
    }

    #[test]
    fn continuous_strategies_blend_and_coloring_bots_are_darker() {
        let si = HybridWorld::new(
            HybridConfig {
                side: 10,
                framework: Framework::Continuous,
                ..si_defaults()
            },
            1,
        )
        .unwrap();
        let mut buf = vec![];
        si.render("", "", &mut buf).unwrap();
        let Body::Population(p) = &si.runner().body else {
            unreachable!()
        };
        let i = (0..100).find(|&i| !p.bot[i]).unwrap();
        assert_eq!(&buf[i * 4..i * 4 + 3], &blend(p.strategy[i]));
        let c = HybridWorld::new(coloring_defaults(), 1).unwrap();
        c.render("", "", &mut buf).unwrap();
        let Body::Coloring(s) = &c.runner().body else {
            unreachable!()
        };
        for i in 0..20 {
            let base = GAME[s.color[i] as usize];
            assert_eq!(
                &buf[i * 4..i * 4 + 3],
                &if s.bot[i] { darker(base) } else { base }
            );
        }
    }

    #[test]
    fn inspection_reads_nodes_padding_and_refuses_outside() {
        let g = HybridWorld::new(
            HybridConfig {
                structure: Structure::BarabasiAlbert,
                nodes: 10,
                bot_count: 2,
                placement: Placement::Hubs,
                ..guo_defaults()
            },
            1,
        )
        .unwrap();
        let node: serde_json::Value = serde_json::from_str(&g.inspect_json(1, 2).unwrap()).unwrap();
        assert_eq!(
            (node["panel"].as_str(), node["node"].as_u64()),
            (Some("node"), Some(9))
        );
        assert!(
            node["payoff"].is_number()
                && node["degree"].as_u64().unwrap() >= 2
                && node["bot"].is_boolean()
        );
        assert!(node["reference"].is_number() && node["agent"].is_null());
        assert!(
            node["phase"].is_string() && node["replicator"].is_null(),
            "Guo: the replicator phase"
        );
        let b = HybridWorld::new(
            HybridConfig {
                side: 10,
                ..booker_defaults()
            },
            1,
        )
        .unwrap();
        let bv: serde_json::Value = serde_json::from_str(&b.inspect_json(0, 0).unwrap()).unwrap();
        assert!(
            bv["replicator"].is_number() && bv["phase"].is_null(),
            "Booker: the replicator rest point"
        );
        let pad: serde_json::Value = serde_json::from_str(&g.inspect_json(3, 2).unwrap()).unwrap();
        assert_eq!(pad["panel"], "empty");
        assert!(g.inspect_json(4, 0).is_err() && g.inspect_json(0, 3).is_err());
        let si = HybridWorld::new(
            HybridConfig {
                side: 10,
                ..si_defaults()
            },
            1,
        )
        .unwrap();
        let v: serde_json::Value = serde_json::from_str(&si.inspect_json(0, 0).unwrap()).unwrap();
        assert!(
            v["reference"].is_null(),
            "no reference in Si's world: null, not NaN"
        );
        let c = HybridWorld::new(coloring_defaults(), 1).unwrap();
        let v: serde_json::Value = serde_json::from_str(&c.inspect_json(0, 0).unwrap()).unwrap();
        assert!(v["color"].as_u64().unwrap() < 3 && v["solutions"].as_f64().unwrap() >= 6.0);
        assert!(v["conflicts_here"].is_number() && v["time"].is_number());
    }
}
