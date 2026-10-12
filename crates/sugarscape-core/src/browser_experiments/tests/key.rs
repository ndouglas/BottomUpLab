use super::super::*;
use serde_json::{json, Value};
fn default() -> Value {
    json!({"study":"key_delegation","stratum":{"transfer_enabled":true,"transfer_cost":1,"giver_share":3,"distance":0},"selector":"SuppliedHandoff","ids":{"giver":"Agent-A","receiver":"Agent-B"}})
}
#[test]
fn key_default_normalizes_and_catalog_appends_exactly_one_study() {
    assert!(normalize_input(&default().to_string()).is_ok());
    let studies = catalog();
    assert_eq!(studies.len(), 18);
    assert_eq!(studies[17].default_input, default());
    assert_eq!(serde_json::to_value(studies[17].family).unwrap(), "key");
}
#[test]
fn key_raw_input_cannot_claim_initial_receiver_ownership() {
    let input = r#"{"study":"key_delegation","stratum":{"transfer_enabled":true,"transfer_cost":1,"giver_share":3,"distance":0},"ids":{"giver":"Agent-A","receiver":"Agent-B"},"profile":{"giver":0,"receiver":60},"initial_owner":"Receiver"}"#;
    assert!(normalize_input(input).is_err());
}
#[test]
fn key_strict_choices_and_raw_quotas() {
    assert!(normalize_input(&default().to_string()).is_ok());
    for (pointer, value) in [
        ("/selector", Value::Null),
        ("/stratum/distance", json!(2)),
        ("/stratum/transfer_cost", json!(true)),
        ("/ids/receiver", json!("Agent-A")),
        ("/ids/giver", json!("")),
    ] {
        let mut bad = default();
        *bad.pointer_mut(pointer).unwrap() = value;
        assert!(normalize_input(&bad.to_string()).is_err(), "{bad}");
    }
    let mut raw = default();
    raw.as_object_mut().unwrap().remove("selector");
    assert!(normalize_input(&raw.to_string()).is_err());
    raw["profile"] = json!({"giver":728,"receiver":63});
    assert!(normalize_input(&raw.to_string()).is_ok());
    for atom in [
        Value::Null,
        json!(729),
        json!(-1),
        json!(0.0),
        json!("0"),
        json!(true),
    ] {
        let mut bad = raw.clone();
        bad["profile"]["giver"] = atom;
        assert!(normalize_input(&bad.to_string()).is_err());
    }
    raw["selector"] = json!("InitiallyEquipped");
    assert!(normalize_input(&raw.to_string()).is_err());
    for text in [
        default().to_string().replace(
            "\"transfer_cost\":1",
            "\"transfer_cost\":1,\"transfer_cost\":1",
        ),
        default()
            .to_string()
            .replace("\"study\":", "\"study\":\"key_delegation\",\"study\":"),
        default().to_string().replace(
            "\"giver\":\"Agent-A\"",
            "\"giver\":\"Agent-A\",\"giver\":\"Agent-A\"",
        ),
    ] {
        assert!(normalize_input(&text).is_err());
    }
    let mut large = default();
    let overhead = large.to_string().len() - 7;
    large["ids"]["giver"] = json!("a".repeat(MAX_INPUT_BYTES - overhead));
    let text = large.to_string();
    assert_eq!(text.len(), MAX_INPUT_BYTES);
    assert!(normalize_input(&text).is_ok());
    assert!(normalize_input(&(text + " ")).is_err());
}
#[test]
fn key_default_native_capture_and_passive_receipt_chronology() {
    let record = run(&normalize_input(&default().to_string()).unwrap()).unwrap();
    assert_eq!(record.payload["episode"]["outcome"]["group_net"], "7");
    assert_eq!(record.payload["classification_scope"], "main");
    assert!(record.payload["classification"].is_object());
    assert_eq!(
        validate_episode(&episode_json(&record).unwrap()).unwrap(),
        record
    );
}
#[test]
fn key_selected_initially_equipped_and_raw_deadline_are_distinct() {
    for (enabled, distance) in [(false, 0), (false, 1), (true, 0), (true, 1)] {
        let mut input = default();
        input["stratum"]["transfer_enabled"] = json!(enabled);
        input["stratum"]["distance"] = json!(distance);
        input["selector"] = json!("InitiallyEquipped");
        let record = run(&normalize_input(&input.to_string()).unwrap()).unwrap();
        assert_eq!(record.payload["classification"], Value::Null);
        assert_eq!(
            record.payload["classification_scope"],
            "changed_initial_control"
        );
        assert_eq!(
            record.payload["episode"]["config"]["initial_owner"],
            "Receiver"
        );
        assert_eq!(record.payload["episode"]["outcome"]["group_net"], "8");
        if enabled && distance == 0 {
            let mut forged = record.clone();
            forged.payload["classification"] = json!({
                "giver_regret":"0", "receiver_regret":"0", "is_pure":true,
                "team_gap":"0", "outcome":record.payload["episode"]["outcome"]
            });
            assert!(validate_episode(&episode_json(&forged).unwrap()).is_err());
        }
        let mut forged = record.clone();
        forged.payload["classification_scope"] = json!("main");
        assert!(validate_episode(&episode_json(&forged).unwrap()).is_err());
        let mut forged = record.clone();
        forged.payload["episode"]["config"]["initial_owner"] = json!("Giver");
        assert!(validate_episode(&episode_json(&forged).unwrap()).is_err());
    }
    let mut input = default();
    input.as_object_mut().unwrap().remove("selector");
    input["profile"] = json!({"giver":0,"receiver":0});
    let record = run(&normalize_input(&input.to_string()).unwrap()).unwrap();
    assert_eq!(
        record.payload["episode"]["config"]["initial_owner"],
        "Giver"
    );
    assert_eq!(
        record.payload["episode"]["outcome"]["delivery_round"],
        Value::Null
    );
    assert_eq!(record.payload["episode"]["outcome"]["group_net"], "0");
}
#[test]
fn key_near_far_disabled_and_opaque_ids_preserve_exact_native_views() {
    use crate::key_delegation as k;
    for (enabled, distance, giver, receiver) in [
        (true, 0, "@T@", "@B@"),
        (true, 1, "007", "42"),
        (false, 0, " A\"\\\n", "e\u{301}"),
        (false, 1, "é", " Agent-B "),
    ] {
        let mut input = default();
        input["stratum"]["transfer_enabled"] = json!(enabled);
        input["stratum"]["distance"] = json!(distance);
        input["ids"] = json!({"giver":giver,"receiver":receiver});
        let record = run(&normalize_input(&input.to_string()).unwrap()).unwrap();
        let config = k::Config::new(
            k::Stratum::new(enabled, 1, 3, distance).unwrap(),
            k::Ids::new(giver.into(), receiver.into()).unwrap(),
            k::InitialOwner::Giver,
        )
        .unwrap();
        let profile = k::Profile::new(
            if distance == 0 { 486 } else { 405 },
            if distance == 0 { 60 } else { 30 },
        )
        .unwrap();
        let mut captures = vec![];
        let episode = k::run_with_capture(&config, profile, &mut |p| {
            captures.push(p.clone());
            Ok(())
        })
        .unwrap();
        assert_eq!(
            record.payload["episode"],
            wire::lossless_value(&episode).unwrap()
        );
        assert_eq!(record.checkpoints.len(), captures.len());
        for (checkpoint, capture) in record.checkpoints.iter().zip(captures) {
            assert_eq!(checkpoint.local.len(), 2);
            for (id, view) in [giver, receiver].into_iter().zip(&capture.views) {
                assert_eq!(
                    checkpoint.local[id],
                    json!({"view":wire::lossless_value(view).unwrap(),"map":{"cells":wire::lossless_value(&view.last_observation.cells).unwrap(),"observed_at":wire::lossless_value(&view.last_observation.clock).unwrap()}})
                );
            }
            assert!(!checkpoint.local.contains_key("unknown Agent"));
            assert_eq!(
                checkpoint.public,
                json!({"status":if capture.clock.phase==k::Phase::Finished {"finished"} else {"running"}})
            );
            assert_eq!(
                checkpoint.researcher.as_ref().unwrap()["physical"],
                wire::lossless_value(&capture.physical).unwrap()
            );
            if enabled
                && capture.clock.phase == k::Phase::AfterAction
                && capture.clock.role == Some(k::Role::Giver)
                && capture.physical.key_owner == k::Role::Receiver
                && capture.clock.round == if distance == 0 { 1 } else { 2 }
            {
                assert!(capture.views[1].own_key);
                assert_ne!(capture.views[1].last_observation.clock, capture.clock);
                assert!(capture.views[1]
                    .history
                    .iter()
                    .any(|e| matches!(e, k::LocalEntry::Ownership(_))));
            }
        }
        assert_eq!(record.input["ids"], input["ids"]);
    }
}
