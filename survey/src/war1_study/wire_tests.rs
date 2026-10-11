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
