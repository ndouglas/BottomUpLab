use super::super::*;
#[test]
fn selected_notes_are_bounded_and_reconstructed() {
    let input = r#"{"study":"persistent_notes","stratum":{"acquisition_cost":1,"write_cost":2,"note_read_cost":1,"manifest_inspection_cost":4,"writer_share":3},"arm":"Retained","routine":"ReadAdaptive","request":"Left","ids":{"writer":"Writer","reader":"Reader"}}"#;
    let selected = normalize_input(input).unwrap();
    let record = run(&selected).unwrap();
    assert!(record.checkpoints.len() <= 16);
    assert_eq!(
        validate_episode(&episode_json(&record).unwrap()).unwrap(),
        record
    );
    // Saved input is still a raw Value: reject aliases before fresh native allocation.
    for (field, value) in [
        ("stratum", serde_json::json!([1, 2, 1, 4, 3])),
        ("ids", serde_json::json!(["Writer", "Reader"])),
        ("arm", serde_json::json!({"Retained": null})),
        ("routine", serde_json::json!({"ReadAdaptive": null})),
        ("request", serde_json::json!({"Left": null})),
    ] {
        let mut saved = record.clone();
        saved.input[field] = value;
        let errors = validate_episode(&serde_json::to_string(&saved).unwrap()).unwrap_err();
        assert!(errors.iter().any(|error| error.field == "input"));
    }
    for field in ["known_request", "cost", "clock", "success"] {
        let bad = input.replacen("{", &format!("{{\"{field}\":null,"), 1);
        assert!(normalize_input(&bad).is_err());
    }
}
#[test]
fn notes_rejects_duplicate_keys_input_bytes_checkpoint_caps_and_saved_changes() {
    use crate::persistent_notes as n;
    let config = n::Config::example(
        n::Arm::Retained,
        n::ReaderRoutine::ReadAdaptive,
        n::Good::Left,
    );
    let mut input = serde_json::to_value(&config).unwrap();
    input["study"] = serde_json::json!("persistent_notes");
    let text = serde_json::to_string(&input).unwrap();
    assert!(normalize_input(&text.replacen("{", "{\"request\":\"Right\",", 1)).is_err());
    assert!(normalize_input(&format!("{text}{}", " ".repeat(MAX_INPUT_BYTES))).is_err());
    let mut record = run(&normalize_input(&text).unwrap()).unwrap();
    let initial = record.checkpoints[0].clone();
    record.checkpoints.resize(17, initial.clone());
    assert!(episode_json(&record).is_err());
    record.checkpoints.resize(MAX_CHECKPOINTS + 1, initial);
    assert!(episode_json(&record).is_err());
    record.checkpoints.truncate(1);
    assert!(validate_episode(&serde_json::to_string(&record).unwrap()).is_err());
}
#[test]
fn selected_notes_rejects_positional_structs_and_unit_variant_objects() {
    let input = r#"{"study":"persistent_notes","stratum":{"acquisition_cost":1,"write_cost":2,"note_read_cost":1,"manifest_inspection_cost":4,"writer_share":3},"arm":"Retained","routine":"ReadAdaptive","request":"Left","ids":{"writer":"Writer","reader":"Reader"}}"#;
    for (field, value) in [
        ("stratum", serde_json::json!([1, 2, 1, 4, 3])),
        ("ids", serde_json::json!(["Writer", "Reader"])),
        ("arm", serde_json::json!({"Retained":null})),
        ("routine", serde_json::json!({"ReadAdaptive":null})),
        ("request", serde_json::json!({"Left":null})),
    ] {
        let mut wire: serde_json::Value = serde_json::from_str(input).unwrap();
        wire[field] = value;
        assert!(
            normalize_input(&wire.to_string()).is_err(),
            "accepted noncanonical {field}"
        );
    }
}
#[test]
fn selected_notes_rejects_missing_null_unknown_and_noninteger_fields() {
    let text = r#"{"study":"persistent_notes","stratum":{"acquisition_cost":1,"write_cost":2,"note_read_cost":1,"manifest_inspection_cost":4,"writer_share":3},"arm":"Retained","routine":"ReadAdaptive","request":"Left","ids":{"writer":"Writer","reader":"Reader"}}"#;
    let valid: serde_json::Value = serde_json::from_str(text).unwrap();
    for field in ["study", "stratum", "arm", "routine", "request", "ids"] {
        let mut missing = valid.clone();
        missing.as_object_mut().unwrap().remove(field);
        assert!(normalize_input(&missing.to_string()).is_err());
        let mut null = valid.clone();
        null[field] = serde_json::Value::Null;
        assert!(normalize_input(&null.to_string()).is_err());
    }
    for atom in ["true", "false", "2.0", "2e0", "-0", "18446744073709551616"] {
        let bad = text.replace("\"write_cost\":2", &format!("\"write_cost\":{atom}"));
        assert!(normalize_input(&bad).is_err(), "accepted {atom}");
    }
    for field in [
        "known_request",
        "clock",
        "costs",
        "result",
        "writer_program",
    ] {
        let bad = text.replacen("{", &format!("{{\"{field}\":null,"), 1);
        assert!(normalize_input(&bad).is_err());
    }
    assert!(normalize_input(&text.replace(
        "\"request\":\"Left\"",
        "\"request\":\"Right\",\"request\":\"Left\""
    ))
    .is_err());
}
