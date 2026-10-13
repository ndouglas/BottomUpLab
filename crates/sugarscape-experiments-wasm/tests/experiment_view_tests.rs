//! Node tests of actual checked WASM exports, including complete reproduction.
use serde_json::{json, Value};
use sugarscape_core::browser_experiments as core;
use sugarscape_experiments_wasm::{
    experiment_catalog_json, experiment_recorded_json, experiment_run_json,
    experiment_validate_json,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
fn errors(result: Result<String, JsValue>, field: &str) {
    let errors: Vec<Value> =
        serde_json::from_str(&result.unwrap_err().as_string().unwrap()).unwrap();
    assert!(
        errors.iter().any(|error| error["field"] == field),
        "expected {field}, actual {errors:?}"
    );
}
#[wasm_bindgen_test]
fn catalog_and_retained_measurements_are_complete_core_records() {
    assert_eq!(
        serde_json::from_str::<Value>(&experiment_catalog_json().unwrap()).unwrap(),
        serde_json::to_value(core::catalog()).unwrap()
    );
    assert_eq!(
        experiment_recorded_json().unwrap(),
        core::recorded_results_json().unwrap()
    );
}
#[wasm_bindgen_test]
fn max_seed_and_complete_reconstruction_are_lossless() {
    let input =
        r#"{"study":"wink","seed":"18446744073709551615","policy":"passive","mode":"ordinary"}"#;
    let record = experiment_run_json(input).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&record).unwrap()["input"]["seed"],
        "18446744073709551615"
    );
    assert_eq!(experiment_validate_json(&record).unwrap(), record);
}
#[wasm_bindgen_test]
fn malformed_overflow_unknown_fields_and_raw_size_limits_are_checked() {
    for text in [
        "{".into(),
        r#"{"study":"wink","seed":"18446744073709551616","policy":"passive","mode":"ordinary"}"#
            .into(),
        r#"{"study":"testimony","fixture":"transfer","extra":true}"#.into(),
        " ".repeat(core::MAX_INPUT_BYTES + 1),
    ] {
        errors(experiment_run_json(&text), "input");
    }
    errors(
        experiment_validate_json(&" ".repeat(core::MAX_EPISODE_BYTES + 1)),
        "episode",
    );
}
#[wasm_bindgen_test]
fn forged_flags_zero_denominators_versions_and_checkpoint_limits_are_checked() {
    let base: Value = serde_json::from_str(&experiment_run_json(r#"{"study":"testimony_game","environment":"training","history":31,"listener":"bayesian"}"#).unwrap()).unwrap();
    let mut forged = base.clone();
    forged["passed"] = json!(true);
    errors(experiment_validate_json(&forged.to_string()), "episode");
    let mut forged = base.clone();
    forged["version"] = json!(2);
    errors(experiment_validate_json(&forged.to_string()), "version");
    let mut forged = base.clone();
    forged["payload"]["reference_posterior"]["denominator"] = json!("0");
    errors(experiment_validate_json(&forged.to_string()), "episode");
    let mut forged = base.clone();
    forged["checkpoints"] = json!(vec![base["checkpoints"][0].clone(); 4097]);
    errors(experiment_validate_json(&forged.to_string()), "checkpoints");
}

#[path = "../../test-support/spatial_experiments.rs"]
mod spatial_test_support;
use spatial_test_support::{
    checked_snapshot_bytes, default_seed_cases, invalid_spatial_inputs, spatial_defaults,
    spatial_saved_mutations, spatial_snapshot_inputs,
};

#[wasm_bindgen_test]
fn runnable_catalog_and_five_legacy_spatial_defaults() {
    assert_eq!(core::catalog().len(), 18);
    assert_eq!(spatial_defaults().len(), 5);
    for input in default_seed_cases() {
        let text = input.to_string();
        let expected =
            serde_json::to_value(core::run(&core::normalize_input(&text).unwrap()).unwrap())
                .unwrap();
        let actual = serde_json::from_str::<Value>(&experiment_run_json(&text).unwrap()).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(
            serde_json::from_str::<Value>(&experiment_validate_json(&actual.to_string()).unwrap())
                .unwrap(),
            actual
        );
        assert!(actual["checkpoints"].as_array().unwrap().len() <= core::MAX_CHECKPOINTS);
        assert!(actual.to_string().len() <= core::MAX_EPISODE_BYTES);
        if let Some((reported, bytes)) = checked_snapshot_bytes(&text) {
            assert_eq!(reported, bytes);
            assert_eq!(
                actual["payload"]["native"]["snapshot_bytes"],
                json!(bytes.to_string())
            );
        }
    }
}

#[wasm_bindgen_test]
fn spatial_boundary_rejects_missing_fields_large_ids_and_resource_products() {
    for input in spatial_defaults() {
        for (bad, field) in invalid_spatial_inputs(&input) {
            errors(experiment_run_json(&bad.to_string()), field);
        }
    }
}

#[wasm_bindgen_test]
fn spatial_saved_records_require_complete_same_target_reconstruction() {
    for input in spatial_defaults() {
        let text = input.to_string();
        let actual = serde_json::from_str::<Value>(&experiment_run_json(&text).unwrap()).unwrap();
        for (bad, field) in spatial_saved_mutations(&input, &actual) {
            errors(experiment_validate_json(&bad.to_string()), field);
        }
    }
}

#[wasm_bindgen_test]
fn spatial_scene_and_bounded_extreme_snapshot_bytes_are_recomputed_on_this_target() {
    for input in spatial_snapshot_inputs() {
        let text = input.to_string();
        let (reported, bytes) = checked_snapshot_bytes(&text).unwrap();
        assert_eq!(reported, bytes);
        let actual = serde_json::from_str::<Value>(&experiment_run_json(&text).unwrap()).unwrap();
        assert_eq!(
            actual["payload"]["native"]["snapshot_bytes"],
            json!(bytes.to_string())
        );
    }
}

#[wasm_bindgen_test]
fn caching_defaults_presets_and_max_u64_reconstruct_complete_native_records() {
    let scenes: Value = serde_json::from_str(include_str!(
        "../../sugarscape-core/src/browser_experiments/fixtures/caching-scenes.json"
    ))
    .unwrap();
    for scene in scenes["scenes"].as_array().unwrap() {
        for seed in ["7", "18446744073709551615"] {
            let mut input = scene["input"].clone();
            input["seed"] = json!(seed);
            if input["study"] == "protection_recaching" {
                input["lab"]["exposure_span"] = json!("18446744073709551615");
            }
            let text = input.to_string();
            let expected =
                serde_json::to_value(core::run(&core::normalize_input(&text).unwrap()).unwrap())
                    .unwrap();
            let raw = experiment_run_json(&text).unwrap();
            let actual: Value = serde_json::from_str(&raw).unwrap();
            assert_eq!(actual, expected);
            assert_eq!(experiment_validate_json(&raw).unwrap(), raw);
            assert_eq!(actual["input"]["seed"], seed);
            assert!(actual["checkpoints"].as_array().unwrap().len() <= 65);
        }
    }
}

#[wasm_bindgen_test]
fn caching_imports_reject_forged_evidence_source_version_and_input() {
    for descriptor in core::catalog().into_iter().filter(|d| {
        matches!(
            d.id,
            core::StudyId::ProtectionRecaching | core::StudyId::DeceptionGestures
        )
    }) {
        let raw = experiment_run_json(&descriptor.default_input.to_string()).unwrap();
        let base: Value = serde_json::from_str(&raw).unwrap();
        for (pointer, replacement, field) in [
            ("/version", json!(99), "version"),
            ("/rules_identity", json!("foreign-source"), "rules_identity"),
            ("/input/seed", json!("18446744073709551615"), "episode"),
            ("/payload/native/seed", json!("0"), "episode"),
            (
                "/checkpoints/1/researcher/frame/fingerprint",
                json!("forged"),
                "episode",
            ),
            ("/checkpoints/1/local/1/seen", json!([]), "episode"),
        ] {
            let mut forged = base.clone();
            *forged.pointer_mut(pointer).unwrap() = replacement;
            // The empty seen mutation may match initial memory; use a clearly
            // non-native atom rather than assume evidence exists at this tick.
            if pointer.ends_with("/seen") {
                *forged.pointer_mut(pointer).unwrap() = json!(["forged"]);
            }
            errors(experiment_validate_json(&forged.to_string()), field);
        }
        let mut forged = base.clone();
        forged["checkpoints"] = json!(vec![
            json!({"index":0,"clock":null,"kind":"caching_initial","public":null,"local":{},"researcher":null});
            4097
        ]);
        errors(experiment_validate_json(&forged.to_string()), "checkpoints");
        let mut input = descriptor.default_input.clone();
        input["lab"]["invented"] = json!(true);
        errors(experiment_run_json(&input.to_string()), "input");
        for seed in ["07", "+7", "18446744073709551616"] {
            let mut input = descriptor.default_input.clone();
            input["seed"] = json!(seed);
            errors(experiment_run_json(&input.to_string()), "input");
        }
    }
}

#[wasm_bindgen_test]
fn discovery_default_payload_and_export_reconstruct_through_public_boundary() {
    let descriptor = core::catalog()
        .into_iter()
        .find(|s| s.id == core::StudyId::SurfaceDiscovery)
        .unwrap();
    let raw = experiment_run_json(&descriptor.default_input.to_string()).unwrap();
    let record: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(record["study"], "surface_discovery");
    assert_eq!(experiment_validate_json(&raw).unwrap(), raw);
    assert!(
        !record["payload"]["decisions"][0]["decision"]["alternatives"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let mut bad = descriptor.default_input.clone();
    bad["sequence"] = json!(256);
    errors(experiment_run_json(&bad.to_string()), "surface_discovery");
    let mut forged = record.clone();
    forged["rules_identity"] = json!("stale-source");
    errors(
        experiment_validate_json(&forged.to_string()),
        "rules_identity",
    );
    let mut forged = record;
    forged["payload"]["metrics"][0]["spent"] = json!("0");
    errors(experiment_validate_json(&forged.to_string()), "episode");
}

#[wasm_bindgen_test]
fn discovery_wasm_registered_compilation_resource_receipts() {
    use sugarscape_core::surface_discovery as d;
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
        let start = js_sys::Date::now();
        let policy = d::CompiledPolicy::build(&protocol, prior, &limits).unwrap();
        let planner_ms = js_sys::Date::now() - start;
        let start = js_sys::Date::now();
        let replay = d::Replay::build(&protocol, &policy, &limits).unwrap();
        let replay_ms = js_sys::Date::now() - start;
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
        console_log!(
            "DISCOVERY_WASM_RESOURCES {}",
            json!({"protocol":protocol,"prior":prior,"planner_ms":planner_ms,"replay_ms":replay_ms,"planner_stats":policy.stats(),"final_replay_stats":replay.stats(),"compiled_policy_shallow_bytes":std::mem::size_of_val(&policy),"private_cache_serialization":"not provided by native API","selected_endpoint_episodes":episodes,"maximum_native_episode_bytes":maximum_native_bytes})
        );
        assert!(replay.stats().decision_states <= limits.states);
    }
}

fn gate_input() -> Value {
    json!({"study":"gate_assistance","stratum":{"holding_cost":1,"holder_share":3,"distance":0},"selector":"SuppliedHelp","ids":{"holder":"Agent-A","retriever":"Agent-B"}})
}
#[wasm_bindgen_test]
fn gate_actual_wasm_native_capture_complete_roundtrip_and_resource_receipt() {
    let input = gate_input();
    let start = js_sys::Date::now();
    let text = experiment_run_json(&input.to_string()).unwrap();
    let run_ms = js_sys::Date::now() - start;
    let record: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        record["payload"]["episode"]["profile"],
        json!({"holder":"540","retriever":"60"})
    );
    assert_eq!(record["payload"]["episode"]["outcome"]["group_net"], "6");
    assert_eq!(
        record["checkpoints"][0]["local"]["Agent-A"]["map"]["observed_at"]["round"],
        "0"
    );
    let start = js_sys::Date::now();
    assert_eq!(experiment_validate_json(&text).unwrap(), text);
    let import_ms = js_sys::Date::now() - start;
    let memory: js_sys::WebAssembly::Memory = wasm_bindgen::memory().unchecked_into();
    console_log!(
        "GATE_WASM_RESOURCES {}",
        json!({"run_ms":run_ms,"import_ms":import_ms,"wire_bytes":text.len(),"checkpoints":record["checkpoints"].as_array().unwrap().len(),"compiler_stats":record["payload"]["compiler_stats"],"wasm_linear_memory_bytes":js_sys::Uint8Array::new(&memory.buffer()).length(),"product_timeout_ms":60000})
    );
    assert!(
        run_ms < 60000.0 && import_ms < 60000.0,
        "Gate request exceeds product worker limit"
    );
    let duplicate = text.replacen("\"group_net\":", "\"group_net\":\"0\",\"group_net\":", 1);
    errors(experiment_validate_json(&duplicate), "episode");
    let mut forged = record;
    forged["rules_identity"] = json!("stale");
    errors(
        experiment_validate_json(&forged.to_string()),
        "rules_identity",
    );
}
#[wasm_bindgen_test]
fn gate_actual_wasm_strict_raw_inputs_and_raw_duplicate_keys() {
    let input = gate_input();
    for (pointer, value) in [
        ("/stratum/holding_cost", json!(true)),
        ("/stratum/distance", json!(2)),
        ("/ids/holder", json!("")),
        ("/selector", json!(null)),
    ] {
        let mut bad = input.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        errors(experiment_run_json(&bad.to_string()), "input");
    }
    let duplicate = input.to_string().replace(
        "\"holding_cost\":1",
        "\"holding_cost\":1,\"holding_cost\":1",
    );
    errors(experiment_run_json(&duplicate), "input");
    let mut both = input.clone();
    both["profile"] = json!({"holder":0,"retriever":0});
    errors(experiment_run_json(&both.to_string()), "gate_assistance");
    errors(
        experiment_run_json(&format!("{}{}", " ".repeat(core::MAX_INPUT_BYTES), input)),
        "input",
    );
}

#[wasm_bindgen_test]
fn key_actual_wasm_capture_and_import_each_have_separate_request_budgets() {
    use sugarscape_core::key_delegation as k;
    let input = json!({"study":"key_delegation","stratum":{"transfer_enabled":true,"transfer_cost":1,"giver_share":3,"distance":0},"ids":{"giver":"@T@","receiver":"e\u{301}"},"selector":"SuppliedHandoff"});
    let start = js_sys::Date::now();
    let raw = experiment_run_json(&input.to_string()).unwrap();
    let run_ms = js_sys::Date::now() - start;
    assert!(
        run_ms < 60000.0,
        "selected key run exceeds 60-second request budget: {run_ms}"
    );
    let record: Value = serde_json::from_str(&raw).unwrap();
    let config = k::Config::new(
        k::Stratum::new(true, 1, 3, 0).unwrap(),
        k::Ids::new("@T@".into(), "e\u{301}".into()).unwrap(),
        k::InitialOwner::Giver,
    )
    .unwrap();
    let mut captures = vec![];
    let native = k::run_with_capture(&config, k::Profile::new(486, 60).unwrap(), &mut |point| {
        captures.push(point.clone());
        Ok(())
    })
    .unwrap();
    assert_eq!(
        record["payload"]["episode"],
        core::wire::lossless_value(&native).unwrap()
    );
    assert_eq!(record["payload"]["classification_scope"], "main");
    assert_eq!(
        record["checkpoints"].as_array().unwrap().len(),
        captures.len()
    );
    for (checkpoint, point) in record["checkpoints"]
        .as_array()
        .unwrap()
        .iter()
        .zip(captures)
    {
        for (id, view) in ["@T@", "e\u{301}"].into_iter().zip(point.views) {
            assert_eq!(
                checkpoint["local"][id],
                json!({"view":core::wire::lossless_value(&view).unwrap(),"map":{"cells":core::wire::lossless_value(&view.last_observation.cells).unwrap(),"observed_at":core::wire::lossless_value(&view.last_observation.clock).unwrap()}})
            );
        }
        assert!(checkpoint["local"].get("unknown Agent").is_none());
    }
    let start = js_sys::Date::now();
    let imported = experiment_validate_json(&raw).unwrap();
    let import_ms = js_sys::Date::now() - start;
    assert!(
        import_ms < 60000.0,
        "selected key import exceeds 60-second request budget: {import_ms}"
    );
    assert_eq!(imported, raw);
    let memory: js_sys::WebAssembly::Memory = wasm_bindgen::memory().unchecked_into();
    console_log!(
        "KEY_WASM_RESOURCES {}",
        json!({"run_ms":run_ms,"import_ms":import_ms,"wire_bytes":raw.len(),"checkpoints":record["checkpoints"].as_array().unwrap().len(),"compiler_stats":record["payload"]["compiler_stats"],"wasm_linear_memory_bytes":js_sys::Uint8Array::new(&memory.buffer()).length(),"per_request_timeout_ms":60000})
    );
    errors(
        experiment_validate_json(&raw.replacen(
            "\"classification_scope\":",
            "\"classification_scope\":null,\"classification_scope\":",
            1,
        )),
        "episode",
    );
}
#[wasm_bindgen_test]
fn key_actual_wasm_rejects_raw_null_duplicates_and_initial_ownership() {
    for input in [
        r#"{"study":"key_delegation","stratum":{"transfer_enabled":true,"transfer_cost":1,"giver_share":3,"distance":0},"ids":{"giver":"A","receiver":"B"},"selector":null}"#,
        r#"{"study":"key_delegation","stratum":{"transfer_enabled":true,"transfer_cost":1,"giver_share":3,"distance":0},"ids":{"giver":"A","receiver":"B"},"profile":{"giver":0,"receiver":60},"initial_owner":"Receiver"}"#,
        r#"{"study":"key_delegation","study":"wink","stratum":{"transfer_enabled":true,"transfer_cost":1,"giver_share":3,"distance":0},"ids":{"giver":"A","receiver":"B"},"selector":"SuppliedHandoff"}"#,
    ] {
        errors(experiment_run_json(input), "input");
    }
}
