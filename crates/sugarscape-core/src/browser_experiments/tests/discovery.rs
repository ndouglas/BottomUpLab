use super::super::*;
use crate::surface_discovery as d;
use serde_json::{json, Value};
fn default() -> Value {
    json!({"study":"surface_discovery","protocol":{"investigator":"A","policy":"RawAdaptive","ids":{"surface":"status-field","agents":["Agent-A","Agent-B"]}},"environment":{"InFamily":"SharedPersistent"},"sequence":0})
}
#[test]
fn discovery_default_and_complete_48_controls_are_registered() {
    let input =
        normalize_input(&default().to_string()).expect("discovery default must be runnable");
    let studies = catalog();
    assert_eq!(studies.len(), 19);
    let descriptor = studies
        .iter()
        .find(|s| s.default_input["study"] == "surface_discovery")
        .unwrap();
    assert_eq!(descriptor.default_input, default());
    assert_eq!(
        descriptor.controls["settings"].as_array().unwrap().len(),
        48
    );
    assert_eq!(serde_json::to_value(input).unwrap(), default());
}
#[test]
fn discovery_strict_input_rejects_undeclared_and_oversized_inputs() {
    assert!(normalize_input(&default().to_string()).is_ok());
    for (pointer, replacement) in [
        ("/sequence", json!(256)),
        ("/protocol/policy", json!("invented")),
        ("/protocol/ids/surface", json!("")),
        ("/protocol/ids/agents/1", json!("Agent-A")),
        ("/environment/InFamily", json!("DataFlip")),
    ] {
        let mut bad = default();
        *bad.pointer_mut(pointer).unwrap() = replacement;
        assert!(normalize_input(&bad.to_string()).is_err(), "{bad}");
    }
    for pointer in ["", "/protocol", "/protocol/ids"] {
        let mut bad = default();
        bad.pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("extra".into(), json!(true));
        assert!(normalize_input(&bad.to_string()).is_err());
    }
    for key in ["protocol", "environment", "sequence"] {
        let mut bad = default();
        bad.as_object_mut().unwrap().remove(key);
        assert!(normalize_input(&bad.to_string()).is_err());
    }
    let mut bad = default();
    bad["protocol"]["policy"] = json!("KnownRaw");
    bad["environment"] = json!("DataFlip");
    assert!(normalize_input(&bad.to_string()).is_err());
    assert!(normalize_input(&format!("{}{}", " ".repeat(MAX_INPUT_BYTES), default())).is_err());
}
#[test]
fn discovery_payload_is_the_complete_native_episode_and_reconstructs() {
    let input = normalize_input(&default().to_string()).expect("discovery input");
    let protocol = d::Protocol::new(d::Role::A, d::PolicyKind::RawAdaptive);
    let limits = d::Limits::registered().unwrap();
    let policy = d::CompiledPolicy::build(&protocol, d::OwnPrior::Uniform, &limits).unwrap();
    let replay = d::Replay::build(&protocol, &policy, &limits).unwrap();
    let native = d::run_episode(
        &protocol,
        d::Environment::InFamily(d::Mechanism::SharedPersistent),
        0,
        &policy,
        &replay,
    )
    .unwrap();
    let record = run(&input).unwrap();
    assert_eq!(record.payload, wire::lossless_value(&native).unwrap());
    assert_eq!(
        validate_episode(&episode_json(&record).unwrap()).unwrap(),
        record
    );
}

#[test]
fn discovery_private_choices_and_future_bits_never_enter_public_or_peer_projection() {
    let record = run(&normalize_input(&default().to_string()).unwrap()).unwrap();
    for checkpoint in &record.checkpoints {
        let public = checkpoint.public.to_string();
        for secret in [
            "OwnChoice",
            "private_bit",
            "belief",
            "lineage",
            "environment",
            "predictions",
        ] {
            assert!(!public.contains(secret), "{public}");
        }
        let peer = &checkpoint.local["Agent-B"];
        assert!(peer["decision"].is_null());
        assert!(!peer["prefix"]["entries"].to_string().contains("OwnChoice"));
        assert_eq!(peer["surface"]["current_state"], "unobserved");
        for (agent, local) in &checkpoint.local {
            assert!(!local.to_string().contains("read_lineage"), "{agent}");
            assert!(local.get("environment").is_none());
        }
    }
    let queued = record
        .checkpoints
        .iter()
        .find(|c| c.kind == "after_choice")
        .unwrap();
    assert!(!queued.local["Agent-A"]["decision"]["alternatives"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(queued.local["Agent-A"]["credits"], "48");
}
#[test]
fn discovery_first_failure_retains_charged_after_slot_and_null_terminal() {
    let mut input = default();
    input["protocol"]["investigator"] = json!("B");
    input["protocol"]["policy"] = json!("InspectOnly");
    input["environment"] = json!("DataFlip");
    let record = run(&normalize_input(&input.to_string()).unwrap()).unwrap();
    let last = record.checkpoints.last().unwrap();
    assert_eq!(last.kind, "after_slot");
    assert_eq!(last.public["status"], "stopped");
    assert_eq!(last.local["Agent-A"]["inference_status"], "unsupported");
    assert_eq!(
        last.local["Agent-B"]["inference_status"],
        "not_checked_after_peer_failure"
    );
    assert_eq!(last.local["Agent-A"]["credits"], "43");
    assert_eq!(last.local["Agent-B"]["credits"], "41");
    assert_eq!(record.payload["failure"]["role"], "A");
    assert!(record.payload["metrics"][0]["reward"].is_null());
    assert!(record.payload["predictions"][0]
        .as_array()
        .unwrap()
        .is_empty());
}
#[test]
fn discovery_opaque_renaming_and_strict_record_mutations() {
    let mut input = default();
    input["protocol"]["policy"] = json!("InspectOnly");
    input["protocol"]["ids"] = json!({"surface":"☃ field","agents":["owned first","owned second"]});
    let record = run(&normalize_input(&input.to_string()).unwrap()).unwrap();
    assert!(record.checkpoints[0].local.contains_key("owned first"));
    let base = serde_json::to_value(&record).unwrap();
    for (pointer, replacement) in [
        ("/rules_identity", json!("stale-source")),
        ("/payload/metrics/0/spent", json!("0")),
        ("/payload/privileged/0/event/credits_after", json!("48")),
        (
            "/payload/decisions/0/decision/alternatives/0/value/denominator",
            json!("0"),
        ),
        ("/checkpoints/0/public/status", json!("forged")),
    ] {
        let mut bad = base.clone();
        *bad.pointer_mut(pointer).unwrap() = replacement;
        assert!(validate_episode(&bad.to_string()).is_err(), "{pointer}");
    }
}

#[test]
fn discovery_complete_independent_witness_payloads_and_paid_chronology_match() {
    let reference: Value = serde_json::from_str(include_str!(
        "../../surface_discovery/tests/fixtures/evaluation-reference.json"
    ))
    .unwrap();
    for row in reference["episodes"].as_array().unwrap() {
        let text=json!({"study":"surface_discovery","protocol":row["protocol"],"environment":row["environment"],"sequence":row["sequence"]}).to_string();
        let record = run(&normalize_input(&text).unwrap()).unwrap();
        let episode: d::Episode = serde_json::from_value(row["episode"].clone()).unwrap();
        assert_eq!(
            record.payload,
            wire::lossless_value(&episode).unwrap(),
            "{text}"
        );
        let paid: Vec<_> = record
            .checkpoints
            .iter()
            .filter(|c| c.kind == "after_slot")
            .map(|c| c.researcher.as_ref().unwrap()["event"].clone())
            .collect();
        assert_eq!(
            paid,
            episode
                .privileged
                .iter()
                .map(|e| wire::lossless_value(e).unwrap())
                .collect::<Vec<_>>()
        );
        for (index, c) in record.checkpoints.iter().enumerate() {
            assert_eq!(c.index, index as u32);
        }
    }
}
#[test]
fn discovery_compiled_source_identity_is_distinct_and_complete() {
    let (sources, digest) = d::compiled_source_identity().unwrap();
    assert_eq!(sources.len(), 21);
    assert!(sources.iter().all(|s| s.bytes > 0 && s.sha256.len() == 64));
    assert!(!sources
        .iter()
        .any(|s| s.path.contains("foraging") || s.path.contains("war/")));
    let identity = discovery::rules_identity().unwrap();
    assert!(identity.contains(&format!("native-source-sha256:{digest}")));
    assert!(identity.contains(":adapter-source-sha256:"));
    assert!(identity.contains(":reviewed-reference-sha256:"));
    let inputs = [("bounded-test", b"original".as_slice())];
    assert_ne!(
        d::compiled_bytes_identity(&inputs).unwrap(),
        d::compiled_bytes_identity(&[("bounded-test", b"altered")]).unwrap()
    );
}
#[test]
fn discovery_large_valid_opaque_renaming_is_not_an_invented_input_domain_limit() {
    let mut input = default();
    input["protocol"]["policy"] = json!("KnownRaw");
    input["protocol"]["ids"]["surface"] = json!("x".repeat(8192));
    let normalized = normalize_input(&input.to_string()).expect("valid renamed bounded input");
    let record = run(&normalized).unwrap();
    assert_eq!(record.input, input);
    assert!(episode_json(&record).unwrap().len() <= MAX_EPISODE_BYTES);
}

#[test]
fn discovery_native_registered_compilation_resource_receipts() {
    let limits = d::Limits::registered().unwrap();
    let settings = d::registered_settings();
    let mut keys = Vec::new();
    for (protocol, environment) in &settings {
        let prior = match (protocol.policy, environment) {
            (d::PolicyKind::KnownRaw, d::Environment::InFamily(model)) => {
                d::OwnPrior::PointMass(*model)
            }
            _ => d::OwnPrior::Uniform,
        };
        if !keys.contains(&(protocol.clone(), prior)) {
            keys.push((protocol.clone(), prior));
        }
    }
    for (protocol, prior) in keys {
        let start = std::time::Instant::now();
        let policy = d::CompiledPolicy::build(&protocol, prior, &limits).unwrap();
        let planner_ms = start.elapsed().as_secs_f64() * 1000.0;
        let start = std::time::Instant::now();
        let replay = d::Replay::build(&protocol, &policy, &limits).unwrap();
        let replay_ms = start.elapsed().as_secs_f64() * 1000.0;
        let mut episodes = 0;
        let mut maximum_native_bytes = 0;
        for (p, environment) in &settings {
            if *p != protocol
                || (protocol.policy == d::PolicyKind::KnownRaw
                    && prior
                        != match environment {
                            d::Environment::InFamily(model) => d::OwnPrior::PointMass(*model),
                            _ => continue,
                        })
            {
                continue;
            }
            for sequence in [0, 255] {
                let episode =
                    d::run_episode(&protocol, *environment, sequence, &policy, &replay).unwrap();
                maximum_native_bytes =
                    maximum_native_bytes.max(serde_json::to_vec(&episode).unwrap().len());
                episodes += 1;
            }
        }
        println!(
            "DISCOVERY_NATIVE_RESOURCES {}",
            json!({"protocol":protocol,"prior":prior,"planner_ms":planner_ms,"replay_ms":replay_ms,"planner_stats":policy.stats(),"final_replay_stats":replay.stats(),"compiled_policy_shallow_bytes":std::mem::size_of_val(&policy),"private_cache_serialization":"not provided by native API","selected_endpoint_episodes":episodes,"maximum_native_episode_bytes":maximum_native_bytes})
        );
        assert!(replay.stats().decision_states <= limits.states);
    }
}

#[test]
fn discovery_valid_large_input_can_exhaust_actual_registered_resources_contextually() {
    let mut input = default();
    input["protocol"]["policy"] = json!("NoProbe");
    input["protocol"]["ids"]["surface"] = json!("x".repeat(8192));
    let normalized = normalize_input(&input.to_string()).expect("valid renamed input");
    let errors = run(&normalized).unwrap_err();
    assert_eq!(errors[0].field, "surface_discovery");
    assert!(errors[0]
        .message
        .contains("allocation_bytes limit 1443627008 exhausted"));
}
