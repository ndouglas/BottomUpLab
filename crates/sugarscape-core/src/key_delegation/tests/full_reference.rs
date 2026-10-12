use super::*;
use std::{path::PathBuf, time::Instant};
#[test]
#[ignore = "explicit engineering comparison against retained independent author arrays"]
fn every_native_row_and_analysis_matches_independent_sealed_arrays() {
    let root = PathBuf::from(std::env::var("KEY_REFERENCE_ROOT").expect("KEY_REFERENCE_ROOT"));
    let evidence = root.join(".superpowers/sdd/2026-10-12-key-delegation/task-3-evidence");
    let game: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/game-reference.json")).unwrap();
    let limits: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/limits-reference.json")).unwrap();
    let mut tables = Vec::new();
    let mut rows_checked = 0;
    for (i, stratum) in strata().into_iter().enumerate() {
        let start = Instant::now();
        let (table, peak) = compile_with_witness(stratum, &Limits::registered().unwrap()).unwrap();
        let panel = &game["panels"][i];
        let receipt = &panel["row_receipt"];
        let bytes = std::fs::read(root.join(receipt["path"].as_str().unwrap())).unwrap();
        assert_eq!(bytes.len() as u64, receipt["bytes"].as_u64().unwrap());
        assert_eq!(
            super::super::sha256::sha256(&bytes),
            receipt["sha256"].as_str().unwrap()
        );
        assert_eq!(
            super::super::canonical::receipt_identity(receipt["path"].as_str().unwrap(), &bytes)
                .unwrap(),
            panel["row_identity"].as_str().unwrap()
        );
        let reference: Vec<Row> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(reference.len(), 46656);
        for (j, (actual, expected)) in table.rows().iter().zip(&reference).enumerate() {
            assert_eq!(actual, expected, "panel {i} row {j}");
            rows_checked += 1;
        }
        assert_eq!(
            *table.analysis(),
            serde_json::from_value::<Analysis>(panel["analysis"].clone()).unwrap(),
            "analysis panel {i}"
        );
        assert_eq!(
            *table.stats(),
            serde_json::from_value::<CompileStats>(panel["stats"].clone()).unwrap(),
            "stats panel {i}"
        );
        assert_eq!(
            peak,
            serde_json::from_value::<PeakWitness>(limits["panels"][i]["peak"].clone()).unwrap(),
            "peak panel {i}"
        );
        assert_eq!(
            controls(&table).unwrap(),
            serde_json::from_value::<Vec<ControlRecord>>(panel["controls"].clone()).unwrap(),
            "controls panel {i}"
        );
        let native = super::super::canonical::canonical_bytes(&table.rows()).unwrap();
        assert_eq!(native, bytes, "canonical row array {i}");
        std::fs::write(evidence.join(format!("native-rows-{i:02}.json")), &native).unwrap();
        std::fs::write(evidence.join(format!("native-panel-{i:02}.json")),serde_json::to_vec(&serde_json::json!({"stratum":table.stratum(),"analysis":table.analysis(),"stats":table.stats(),"peak":peak,"rows_checked":reference.len(),"row_identity":panel["row_identity"],"elapsed_seconds":start.elapsed().as_secs_f64()})).unwrap()).unwrap();
        println!(
            "matched panel {i:02}: {} rows, {:?}, {:.3}s",
            reference.len(),
            table.stats(),
            start.elapsed().as_secs_f64()
        );
        tables.push(table);
    }
    assert_eq!(rows_checked, 1679616);
    for (i, (axis, lower, upper)) in contrast_pairs().into_iter().enumerate() {
        let start = Instant::now();
        let l = tables.iter().find(|t| t.stratum() == &lower).unwrap();
        let u = tables.iter().find(|t| t.stratum() == &upper).unwrap();
        let (actual, deltas) = contrast_with_deltas(l, u, axis).unwrap();
        assert_eq!(
            actual,
            serde_json::from_value::<Contrast>(game["contrasts"][i].clone()).unwrap(),
            "contrast {i}"
        );
        let file = root.join(format!(
            ".superpowers/sdd/2026-10-12-key-delegation/numerical-author/deltas-{i:02}.json"
        ));
        let bytes = std::fs::read(file).unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["axis"], serde_json::to_value(axis).unwrap());
        assert_eq!(body["lower"], serde_json::to_value(&lower).unwrap());
        assert_eq!(body["upper"], serde_json::to_value(&upper).unwrap());
        let label = format!(
            ".superpowers/sdd/2026-10-12-key-delegation/numerical-author/deltas-{i:02}.json"
        );
        let receipt = game["independent_artifacts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["path"].as_str() == Some(label.as_str()))
            .unwrap();
        assert_eq!(bytes.len() as u64, receipt["bytes"].as_u64().unwrap());
        assert_eq!(
            super::super::sha256::sha256(&bytes),
            receipt["sha256"].as_str().unwrap()
        );
        let expected: Vec<FixedWordDelta> = serde_json::from_value(body["rows"].clone()).unwrap();
        assert_eq!(deltas, expected, "full delta array {i}");
        std::fs::write(
            evidence.join(format!("native-contrast-{i:02}.json")),
            serde_json::to_vec(&actual).unwrap(),
        )
        .unwrap();
        println!(
            "matched contrast {i:02}: {} typed deltas, {:.3}s",
            deltas.len(),
            start.elapsed().as_secs_f64()
        );
    }
    println!("COMPLETE: 1679616 typed rows and trace hashes, 36 analyses/pure sets/stats/peak witnesses, 216 controls, 84 contrasts and 3919104 full typed deltas");
}
