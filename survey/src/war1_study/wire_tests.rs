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
