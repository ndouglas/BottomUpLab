//! Every misaligned preset runs 2,000 ticks at seed 1 with finite series and a feasible state:
//! CI's smoke test for milestone 43 (it scores nothing; the survey scores).
use sugarscape_core::{
    model::{ModelConfig, ModelKind, ModelWorld},
    presets, sweep,
};

#[test]
fn every_preset_runs_two_thousand_ticks_with_finite_series() {
    let mut seen = 0;
    for p in presets::catalog() {
        let ModelConfig::Misaligned(_) = &p.config else {
            continue;
        };
        seen += 1;
        let mut w = ModelWorld::new(p.config.clone(), 1).unwrap();
        w.model_mut().run(2000);
        let m = w.model();
        for name in [
            "utility",
            "proxy",
            "gain",
            "a",
            "b",
            "c",
            "d",
            "optimum_gap",
            "limit_gap",
            "interactions",
            "switched_off",
        ] {
            assert!(
                m.latest_value(name).unwrap().is_finite(),
                "{}: {name}",
                p.id
            );
        }
        assert!(
            m.latest_value("optimum_gap").unwrap() >= -1e-9,
            "{}: never above the optimum",
            p.id
        );
        assert_eq!(w.kind(), ModelKind::Misaligned);
    }
    assert_eq!(seen, 8);
}

#[test]
fn the_builtin_sweeps_parse_and_name_real_paths() {
    for id in [
        "misaligned-figure2-pairs",
        "misaligned-interval",
        "misaligned-floor",
    ] {
        let s = sweep::builtin(id).unwrap_or_else(|| panic!("no builtin {id}"));
        assert!(s.ticks >= 1000, "{id}");
    }
}
