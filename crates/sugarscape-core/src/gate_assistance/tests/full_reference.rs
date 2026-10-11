//! Ignored engineering comparison only. Never writes a scientific Report.
use super::super::*;
use super::super::{canonical::canonical_bytes, game::SELECTORS};
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

fn read(path: &Path) -> Value {
    serde_json::from_reader(File::open(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
        .unwrap()
}
fn next(reader: &mut impl BufRead) -> Value {
    let mut line = String::new();
    assert!(
        reader.read_line(&mut line).unwrap() > 0,
        "reference ended early"
    );
    serde_json::from_str(&line).unwrap()
}
fn equal<T: serde::Serialize>(actual: &T, expected: &Value, context: &str) {
    assert_eq!(
        serde_json::to_value(actual).unwrap(),
        *expected,
        "{context}"
    );
}
fn ranking_stages(table: &GameTable, mut candidates: Vec<Profile>) -> Value {
    let fields = |p: Profile| {
        let o = &table.rows[usize::from(p.holder) * 64 + usize::from(p.retriever)].outcome;
        [
            -i32::from(o.group_net),
            i32::from(o.spent[0]) + i32::from(o.spent[1]),
            i32::from(o.delivery_round.unwrap_or(7)),
            i32::from(p.holder),
            i32::from(p.retriever),
        ]
    };
    let mut stages = vec![];
    for field in 0..5 {
        let Some(value) = candidates.iter().map(|p| fields(*p)[field]).min() else {
            break;
        };
        candidates.retain(|p| fields(*p)[field] == value);
        stages.push(json!({"field":field,"value":value,"profiles":candidates}));
    }
    Value::Array(stages)
}
#[test]
#[ignore = "full sealed independent census engineering comparison; requires ignored reference artifacts"]
fn full_reference_engineering_equality() -> Result<(), Error> {
    eprintln!(
        "compiled_gate_source_identity={}",
        crate::surface_discovery::compiled_bytes_identity(SOURCE_INPUTS).unwrap()
    );
    let base = std::env::var_os("GATE_REFERENCE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../.superpowers/sdd/2026-10-11-gate-assistance/numerical-author")
        });
    let game: Value = serde_json::from_str(include_str!("fixtures/game-reference.json")).unwrap();
    let kernel: Value =
        serde_json::from_str(include_str!("fixtures/kernel-reference.json")).unwrap();
    let limits: Value =
        serde_json::from_str(include_str!("fixtures/limits-reference.json")).unwrap();
    let full_analysis = read(&base.join("full-analysis.json"));
    let identities = read(&base.join("portable-row-identities.json"));
    let mut raw = BufReader::new(File::open(base.join("full-reference.jsonl")).unwrap());
    let mut classification =
        BufReader::new(File::open(base.join("classification-reference.jsonl")).unwrap());
    equal(
        &Limits::registered()?,
        &limits["provisional_limits"],
        "registered caps",
    );
    let mut summaries = vec![];
    let mut baselines: BTreeMap<(u8, u8), Vec<Outcome>> = BTreeMap::new();
    let mut total = 0usize;
    let mut transitions = 0u64;
    for (i, s) in registered_strata().into_iter().enumerate() {
        let started = std::time::Instant::now();
        let expected_stats: CompileStats =
            serde_json::from_value(limits["panels"][i]["stats"].clone()).unwrap();
        // Exact independent bounds, not just the doubled default, must succeed.
        let table = compile(
            &s,
            &Limits {
                profiles: expected_stats.profiles,
                transitions: expected_stats.transitions,
                allocation_bytes: expected_stats.peak_bytes,
            },
        )?;
        assert_eq!(table.rows.len(), 46656);
        equal(&table.stats, &limits["panels"][i]["stats"], "logical stats");
        equal(&table.stats, &full_analysis[i]["stats"], "full stats");
        equal(
            &table.analysis,
            &game["panels"][i]["analysis"],
            "all maxima/pure/representatives/solo",
        );
        equal(
            &table.analysis,
            &full_analysis[i]["analysis"],
            "full analysis",
        );
        let mut metric_ranges: BTreeMap<String, [i16; 2]> = BTreeMap::new();
        let mut delivery_counts: BTreeMap<String, u64> = [
            ("4".into(), 0),
            ("5".into(), 0),
            ("6".into(), 0),
            ("null".into(), 0),
        ]
        .into();
        for (j, row) in table.rows.iter().enumerate() {
            let p = Profile::new((j / 64) as u16, (j % 64) as u8)?;
            let expected = next(&mut raw);
            assert_eq!(
                expected["stratum"],
                json!([s.holding_cost(), s.holder_share(), s.distance()])
            );
            assert_eq!(expected["profile"], json!([p.holder, p.retriever]));
            equal(&row.outcome, &expected["outcome"], "raw outcome");
            equal(
                &row.trace_hash,
                &expected["trace_hash"],
                "trace fingerprint",
            );
            let c = classify(&table, p)?;
            let expected = next(&mut classification);
            assert_eq!(
                expected["stratum"],
                json!([s.holding_cost(), s.holder_share(), s.distance()])
            );
            assert_eq!(expected["profile"], json!([p.holder, p.retriever]));
            equal(
                &row.trace_hash,
                &expected["trace_hash"],
                "classification trace link",
            );
            equal(
                &c.holder_regret,
                &expected["holder_regret"],
                "Holder regret",
            );
            equal(
                &c.retriever_regret,
                &expected["retriever_regret"],
                "Retriever regret",
            );
            equal(&c.is_pure, &expected["is_pure"], "membership");
            equal(&c.team_gap, &expected["team_gap"], "team gap");
            for (key, value) in outcome_metrics(&s, &row.outcome)? {
                let range = metric_ranges.entry(key).or_insert([value, value]);
                range[0] = range[0].min(value);
                range[1] = range[1].max(value);
            }
            *delivery_counts
                .get_mut(
                    &row.outcome
                        .delivery_round
                        .map_or("null".into(), |q| q.to_string()),
                )
                .unwrap() += 1;
            if s.holder_share() > 0 {
                let original = &baselines[&(s.holding_cost(), s.distance())][j];
                assert_eq!(row.outcome.spent, original.spent);
                assert_eq!(row.outcome.group_net, original.group_net);
                assert_eq!(row.outcome.delivery_round, original.delivery_round);
                let shift = if original.delivery_round.is_some() {
                    i16::from(s.holder_share())
                } else {
                    0
                };
                assert_eq!(
                    row.outcome.net,
                    [original.net[0] + shift, original.net[1] - shift]
                );
            }
        }
        equal(
            &metric_ranges,
            &game["panels"][i]["metrics"],
            "all named metric ranges",
        );
        equal(
            &delivery_counts,
            &game["panels"][i]["delivery_counts"],
            "delivery counts",
        );
        let profiles = (0..46656)
            .map(|j| Profile::new((j / 64) as u16, (j % 64) as u8).unwrap())
            .collect();
        assert_eq!(
            ranking_stages(&table, profiles),
            game["panels"][i]["ties"]["team"]
        );
        assert_eq!(
            ranking_stages(&table, table.analysis.pure.clone()),
            game["panels"][i]["ties"]["best_group_pure"]
        );
        let summary = summarize_stratum(&table)?;
        equal(
            &summary.equilibrium,
            &game["panels"][i]["equilibrium"],
            "complete equilibrium ranges",
        );
        let controls = read(
            &base.join(
                kernel["complete_controls_receipts"][i]["path"]
                    .as_str()
                    .unwrap(),
            ),
        );
        for (n, selector) in SELECTORS.into_iter().enumerate() {
            let p = resolve(&table, selector);
            equal(&p, &controls[n]["profile"], "control profile");
            equal(&selector, &controls[n]["selector"], "control selector");
            if let Some(p) = p {
                // These 90 additional episodes (and public hash replays) are
                // explicit test verification work outside each compile budget.
                let episode = run_episode(
                    &Config {
                        stratum: s.clone(),
                        ids: Ids::default(),
                    },
                    p,
                )?;
                equal(&episode, &controls[n]["episode"], "full control Episode");
                equal(
                    &classify(&table, p)?,
                    &controls[n]["classification"],
                    "full control classification",
                );
                equal(
                    &trace_hash(&episode)?,
                    &game["panels"][i]["controls"][n]["trace_hash"],
                    "control public trace hash",
                );
            } else {
                assert!(controls[n]["episode"].is_null());
                assert!(controls[n]["classification"].is_null());
            }
        }
        let bytes = canonical_bytes(&table.rows)?;
        let identity = &identities["ordered_row_identities"][i];
        equal(&s, &identity["stratum"], "identity stratum");
        assert_eq!(bytes.len() as u64, identity["bytes"].as_u64().unwrap());
        let label = identity["path"].as_str().unwrap();
        assert_eq!(
            label,
            game["panels"][i]["row_receipt"]["path"].as_str().unwrap()
        );
        let hash = crate::surface_discovery::compiled_bytes_identity(&[(label, bytes.as_slice())])
            .unwrap();
        equal(
            &hash,
            &identity["receipt_identity"],
            "portable wrapped Row array identity",
        );
        total += table.rows.len();
        transitions += table.stats.transitions;
        eprintln!("engineering stratum={i} rows={} stats={} elapsed_seconds={:.6} row_path={label} row_bytes={} wrapped_identity={hash}",table.rows.len(),serde_json::to_string(&table.stats).unwrap(),started.elapsed().as_secs_f64(),bytes.len());
        if s.holder_share() == 0 {
            baselines.insert(
                (s.holding_cost(), s.distance()),
                table.rows.into_iter().map(|r| r.outcome).collect(),
            );
        }
        summaries.push(summary);
    }
    let mut remainder = String::new();
    assert_eq!(raw.read_line(&mut remainder).unwrap(), 0);
    assert_eq!(classification.read_line(&mut remainder).unwrap(), 0);
    let contrasts = contrasts(&summaries)?;
    equal(
        &contrasts,
        &read(&base.join("contrasts-reference.json")),
        "all 45 full contrasts",
    );
    assert_eq!(total, 839808);
    eprintln!("engineering totals rows={total} classified={total} transitions={transitions} controls=90 maxima_arrays=36 analyses=18 contrasts={} no_scientific_report=true",contrasts.len());
    Ok(())
}

#[test]
#[ignore = "full final-row rejection engineering check"]
fn final_raw_row_limits_do_not_silently_prune() -> Result<(), Error> {
    let s = Stratum::new(1, 0, 0)?;
    let mut limits = Limits::registered()?;
    limits.profiles = 46655;
    assert!(
        matches!(compile(&s,&limits),Err(Error::ResourceLimit{resource,limit:46655}) if resource=="profiles")
    );
    Ok(())
}
