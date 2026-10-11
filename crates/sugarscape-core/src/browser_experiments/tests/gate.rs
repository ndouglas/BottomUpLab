use super::super::*;
use serde_json::{json, Value};

fn default() -> Value {
    json!({"study":"gate_assistance","stratum":{"holding_cost":1,"holder_share":3,"distance":0},"selector":"SuppliedHelp","ids":{"holder":"Agent-A","retriever":"Agent-B"}})
}

#[test]
fn gate_default_is_appended_after_exact_legacy_catalog() {
    let studies = catalog();
    let ids: Vec<_> = studies
        .iter()
        .map(|s| serde_json::to_value(s.id).unwrap())
        .collect();
    assert_eq!(
        ids,
        json!([
            "wink",
            "testimony",
            "testimony_game",
            "strategic_reporting",
            "strategy_inference",
            "adversarial_audit",
            "shared_surface",
            "active_surface",
            "burrow_excavation",
            "burrow_access",
            "foraging_fixed",
            "foraging_passage",
            "foraging_construction",
            "protection_recaching",
            "deception_gestures",
            "surface_discovery",
            "gate_assistance"
        ])
        .as_array()
        .unwrap()
        .clone()
    );
    assert_eq!(studies[16].default_input, default());
    assert_eq!(serde_json::to_value(studies[16].family).unwrap(), "gate");
    assert!(normalize_input(&default().to_string()).is_ok());
}

#[test]
fn gate_input_rejects_ambiguous_choices_types_and_duplicates() {
    assert!(normalize_input(&default().to_string()).is_ok());
    for (pointer, value) in [
        ("/stratum/holding_cost", json!(3)),
        ("/stratum/distance", json!(2)),
        ("/stratum/holder_share", json!(3.0)),
        ("/ids/retriever", json!("Agent-A")),
        ("/ids/holder", json!("")),
        ("/selector", json!("unknown")),
        ("/selector", Value::Null),
    ] {
        let mut bad = default();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(normalize_input(&bad.to_string()).is_err(), "{bad}");
    }
    let mut raw = default();
    raw.as_object_mut().unwrap().remove("selector");
    assert!(normalize_input(&raw.to_string()).is_err());
    raw["profile"] = json!({"holder":728,"retriever":63});
    assert!(normalize_input(&raw.to_string()).is_ok());
    for value in [json!(729), json!(-1), json!(true), json!(0.0), json!("0")] {
        raw["profile"]["holder"] = value;
        assert!(normalize_input(&raw.to_string()).is_err());
    }
    let mut both = default();
    both["profile"] = json!({"holder":0,"retriever":0});
    assert!(normalize_input(&both.to_string()).is_err());
    let text = default().to_string();
    for bad in [
        text.replace(
            "\"holding_cost\":1",
            "\"holding_cost\":1,\"holding_cost\":1",
        ),
        text.replace("\"study\":", "\"study\":\"gate_assistance\",\"study\":"),
    ] {
        assert!(normalize_input(&bad).is_err(), "{bad}");
    }
}

#[test]
fn gate_capture_is_native_and_local_maps_are_dated() {
    use crate::gate_assistance as g;
    let input = normalize_input(&default().to_string()).expect("gate default");
    let record = run(&input).unwrap();
    let profile: g::Profile = serde_json::from_value(
        record
            .input
            .get("profile")
            .cloned()
            .unwrap_or(json!({"holder":540,"retriever":60})),
    )
    .unwrap();
    let config = g::Config {
        stratum: g::Stratum::new(1, 3, 0).unwrap(),
        ids: g::Ids::default(),
    };
    let mut captures = vec![];
    let native = g::run_with_capture(&config, profile, &mut |p| {
        captures.push(p.clone());
        Ok(())
    })
    .unwrap();
    assert_eq!(
        record.payload["episode"],
        wire::lossless_value(&native).unwrap()
    );
    assert_eq!(record.checkpoints.len(), captures.len());
    for (checkpoint, capture) in record.checkpoints.iter().zip(captures) {
        for (index, id) in ["Agent-A", "Agent-B"].into_iter().enumerate() {
            let local = &checkpoint.local[id];
            assert_eq!(
                local["view"],
                wire::lossless_value(&capture.views[index]).unwrap()
            );
            assert_eq!(
                local["map"]["cells"],
                wire::lossless_value(&capture.views[index].last_observation.cells).unwrap()
            );
            assert_eq!(
                local["map"]["observed_at"],
                wire::lossless_value(&capture.views[index].last_observation.clock).unwrap()
            );
            assert!(local.get("classification").is_none());
            assert!(checkpoint.public.get("profile").is_none());
        }
        assert_eq!(
            checkpoint.researcher.as_ref().unwrap()["physical"],
            wire::lossless_value(&capture.physical).unwrap()
        );
        assert_eq!(
            checkpoint.researcher.as_ref().unwrap()["classification"],
            record.payload["classification"]
        );
    }
    let text = episode_json(&record).unwrap();
    assert_eq!(validate_episode(&text).unwrap(), record);
    for bad in [
        text.replacen("\"group_net\":", "\"group_net\":\"0\",\"group_net\":", 1),
        text.replacen("\"study\":", "\"study\":\"gate_assistance\",\"study\":", 1),
    ] {
        assert!(validate_episode(&bad).is_err());
    }
    let mut forged = record.clone();
    forged.rules_identity.push('x');
    assert_eq!(
        validate_episode(&episode_json(&forged).unwrap()).unwrap_err()[0].field,
        "rules_identity"
    );
    let mut forged = record.clone();
    forged.payload["classification"]["holder_regret"] = json!("-1");
    assert!(validate_episode(&episode_json(&forged).unwrap()).is_err());
}

#[test]
fn gate_maximum_input_names_and_raw_whitespace_are_bounded_without_name_caps() {
    let mut value = default();
    let overhead = value.to_string().len() - "Agent-A".len();
    value["ids"]["holder"] = json!("a".repeat(MAX_INPUT_BYTES - overhead));
    let text = value.to_string();
    assert_eq!(text.len(), MAX_INPUT_BYTES);
    assert!(normalize_input(&text).is_ok());
    assert!(normalize_input(&(text + " ")).is_err());
    let mut escaped = default();
    escaped["ids"]["holder"] = json!("\u{1}".repeat(MAX_INPUT_BYTES));
    assert!(normalize_input(&escaped.to_string()).is_err());
}

#[test]
fn gate_all_registered_strata_selectors_and_raw_endpoints_normalize() {
    for stratum in crate::gate_assistance::registered_strata() {
        let mut input = default();
        input["stratum"] = serde_json::to_value(stratum).unwrap();
        for selector in [
            "SoloAttempt",
            "SuppliedHelp",
            "TeamOptimal",
            "NashLex",
            "NashBestGroup",
        ] {
            input["selector"] = json!(selector);
            assert!(normalize_input(&input.to_string()).is_ok());
        }
        input.as_object_mut().unwrap().remove("selector");
        for profile in [
            json!({"holder":0,"retriever":0}),
            json!({"holder":728,"retriever":63}),
        ] {
            input["profile"] = profile;
            assert!(normalize_input(&input.to_string()).is_ok());
        }
    }
}
