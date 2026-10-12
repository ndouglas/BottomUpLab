use super::wire;
#[test]
fn selected_semantic_validation_refuses_wrong_schema_key_sampler_and_roster() {
    let original = wire::decode_manifest(wire::SELECTED).unwrap();
    for kind in 0..5 {
        let mut m = original.clone();
        match kind {
            0 => m.schema = "unknown".into(),
            1 => m.cases[1].ordinal = 0,
            2 => m.sampler.bytes_drawn = 8,
            3 => m.arms[0].config.blue = 2,
            _ => m.sampler.command = vec!["x".repeat(1025)],
        };
        assert!(wire::validate_fields(&m, wire::SELECTED).is_err());
    }
}
#[test]
fn bounded_vector_parser_refuses_excess_elements_before_retaining_them() {
    #[derive(serde::Deserialize)]
    struct Values {
        #[serde(deserialize_with = "wire::bounded_vec::<_,_,2>")]
        values: Vec<u32>,
    }
    let valid: Values = serde_json::from_str("{\"values\":[1,2]}").unwrap();
    assert_eq!(valid.values, vec![1, 2]);
    assert!(serde_json::from_str::<Values>("{\"values\":[1,2,3]}").is_err());
}
#[test]
fn nullable_binding_and_observation_fields_require_explicit_keys() {
    use super::{
        gate::CampaignBinding,
        observer::{Observation, ReferenceWire},
    };
    let binding = serde_json::json!({"manifest_sha256":"m","compiled_inputs_sha256":"c","executable_sha256":"e","declaration_sha256":null,"acceptance_sha256":null,"execution_approval_sha256":null});
    assert!(serde_json::from_value::<CampaignBinding>(binding.clone()).is_ok());
    for key in [
        "declaration_sha256",
        "acceptance_sha256",
        "execution_approval_sha256",
    ] {
        let mut bad = binding.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<CampaignBinding>(bad).is_err(),
            "{key}"
        );
    }
    let observation = serde_json::json!({"step":64,"counts":[1,1],"origin":"native","reason":null,"reference":null,"reference_error":null});
    assert!(serde_json::from_value::<Observation>(observation.clone()).is_ok());
    for key in ["reason", "reference", "reference_error"] {
        let mut bad = observation.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(serde_json::from_value::<Observation>(bad).is_err(), "{key}");
    }
    let reference = serde_json::json!({"requested_time":1.0,"evaluated_time":1.0,"forces":[1.0,1.0],"regime":"asymptotic","extinction_time":null,"at_boundary":false,"survivor":null});
    assert!(serde_json::from_value::<ReferenceWire>(reference.clone()).is_ok());
    for key in ["extinction_time", "survivor"] {
        let mut bad = reference.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<ReferenceWire>(bad).is_err(),
            "{key}"
        );
    }
}

#[test]
fn numeric_guard_preserves_escaped_quote_and_backslash_parity() {
    // Odd runs escape the quote; even runs leave it as the string terminator.
    for (text, rejects) in [
        (r#""\"1e-999""#, false),
        (r#""\\" 1e-999"#, true),
        (r#""\\\"1e-999""#, false),
        (r#""\\\\" 1e-999"#, true),
        (r#""\\1e-999""#, false),
        (r#""\\\\1e-999""#, false),
        (r#"["\"1e-999",1e-999]"#, true),
        (r#"["\\\"1e-999",1e-999]"#, true),
    ] {
        assert_eq!(
            wire::reject_numeric_underflow(text.as_bytes()).is_err(),
            rejects,
            "{text:?}"
        );
    }
}

#[test]
fn numeric_guard_ignores_unicode_quoted_numbers_but_rejects_following_underflow() {
    for (text, rejects) in [
        (r#"["雪🦀 café 1e-999 -2e-999",0e-999,1e-3]"#, false),
        (r#"["雪🦀 café 1e-999",1e-999]"#, true),
        (r#"["雪\\🦀\" café 1e-999",-2e-999]"#, true),
        (r#"["1e-999","2e-999"]"#, false),
    ] {
        assert_eq!(
            wire::reject_numeric_underflow(text.as_bytes()).is_err(),
            rejects,
            "{text:?}"
        );
    }
}

#[test]
fn unterminated_quoted_text_is_left_to_typed_payload_readers() {
    use super::observer;
    for text in [
        "\"",
        "\"雪1e-999",
        "\"1e-999\\",
        "\"雪\\\"1e-999",
        "\"1e-999\\\\",
    ] {
        assert!(
            wire::reject_numeric_underflow(text.as_bytes()).is_ok(),
            "{text:?}"
        );
        assert!(observer::read_frame(text).is_err(), "{text:?}");
        assert!(observer::read_checkpoint(text).is_err(), "{text:?}");
    }
}

#[test]
fn numeric_guard_rejects_invalid_utf8_even_inside_unterminated_quotes() {
    for bytes in [
        b"\"\xff1e-999".as_slice(),
        b"\"ok\" \xff".as_slice(),
        b"\"\xc3".as_slice(),
        b"\"\\\xff".as_slice(),
    ] {
        let error = wire::reject_numeric_underflow(bytes).unwrap_err();
        assert!(error.contains("utf-8"), "{error}");
    }
}

#[test]
fn saved_frame_rejects_underflow_after_quoted_field_name() {
    use super::observer;
    use sugarscape_core::war::Engagement;
    let manifest = wire::decode_manifest(wire::FIXTURE).unwrap();
    let mut engine = Engagement::new(manifest.arms[0].config.clone(), 7).unwrap();
    let frame = engine.step().unwrap().unwrap();
    let text = serde_json::to_string(&frame).unwrap();
    assert!(observer::read_frame(&text).is_ok());
    let literal = serde_json::to_string(&frame.calendar_time).unwrap();
    let bad = text.replacen(
        &format!("\"calendar_time\":{literal}"),
        "\"calendar_time\":1e-999",
        1,
    );
    assert!(observer::read_frame(&bad)
        .unwrap_err()
        .contains("underflows binary64"));
}
