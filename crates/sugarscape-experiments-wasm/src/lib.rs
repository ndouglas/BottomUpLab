//! JavaScript-facing exports that reach the Minds provenance cluster
//! (`browser_experiments` and `burrow`). The episodes worker is the only page
//! consumer. Nothing the playground or the sweeps need belongs here: an export
//! added to this crate never loads on the default page, and an export added to
//! `sugarscape-wasm` that touches the cluster pulls the cluster back into the
//! engine bundle. Errors cross the boundary as JSON strings of
//! `[{ field, message }]`.

mod experiment_view;
pub use experiment_view::{
    experiment_catalog_json, experiment_recorded_json, experiment_run_json,
    experiment_validate_json,
};

use sugarscape_core::config::FieldError;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

fn field_errors(errors: Vec<FieldError>) -> JsValue {
    JsValue::from_str(&serde_json::to_string(&errors).expect("errors serialize"))
}

/// Checked standalone excavation replay; serializes the same core record as the native CLI.
#[wasm_bindgen]
pub fn burrow_replay_json(
    config_json: &str,
    seed: &str,
    ticks: f64,
    sample_every: f64,
) -> Result<String, JsValue> {
    let config: sugarscape_core::burrow::LabConfig = serde_json::from_str(config_json)
        .map_err(|error| field_errors(vec![FieldError::new("burrow_config", error.to_string())]))?;
    let seed = decimal_seed(seed)
        .map_err(|message| field_errors(vec![FieldError::new("seed", message)]))?;
    let ticks = checked_burrow_u32(ticks, "ticks", 0)?;
    let sample_every = checked_burrow_u32(sample_every, "sample_every", 1)?;
    let episode = sugarscape_core::burrow::run_episode(
        config,
        seed,
        sugarscape_core::burrow::RunOptions {
            ticks,
            sample_every,
        },
    )
    .map_err(field_errors)?;
    Ok(serde_json::to_string(&episode).expect("episode serializes"))
}

/// Checked resource-access replay; returns the complete shared outer record.
#[wasm_bindgen]
pub fn burrow_access_replay_json(
    config_json: &str,
    seed: &str,
    ticks: f64,
    sample_every: f64,
) -> Result<String, JsValue> {
    let config: sugarscape_core::burrow::AccessConfig =
        serde_json::from_str(config_json).map_err(|error| {
            field_errors(vec![FieldError::new(
                "burrow_access_config",
                error.to_string(),
            )])
        })?;
    let seed = decimal_seed(seed)
        .map_err(|message| field_errors(vec![FieldError::new("seed", message)]))?;
    let ticks = checked_burrow_u32(ticks, "ticks", 0)?;
    let sample_every = checked_burrow_u32(sample_every, "sample_every", 1)?;
    let episode = sugarscape_core::burrow::run_access_episode(
        config,
        seed,
        sugarscape_core::burrow::RunOptions {
            ticks,
            sample_every,
        },
    )
    .map_err(field_errors)?;
    Ok(serde_json::to_string(&episode).expect("access episode serializes"))
}

fn checked_burrow_u32(value: f64, field: &str, minimum: u32) -> Result<u32, JsValue> {
    if !value.is_finite()
        || value.fract() != 0.0
        || value < f64::from(minimum)
        || value > f64::from(u32::MAX)
    {
        return Err(field_errors(vec![FieldError::new(
            field,
            format!("expected finite integer in {minimum}..={}", u32::MAX),
        )]));
    }
    Ok(value as u32)
}

fn decimal_seed(seed: &str) -> Result<u64, &'static str> {
    if seed.is_empty() || !seed.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("expected unsigned decimal u64");
    }
    seed.parse().map_err(|_| "seed exceeds u64")
}

#[cfg(test)]
mod tests {
    #[test]
    fn decimal_seed_is_strict_and_preserves_full_u64_range() {
        assert_eq!(super::decimal_seed("18446744073709551615"), Ok(u64::MAX));
        assert!(super::decimal_seed("18446744073709551616").is_err());
        assert!(super::decimal_seed("").is_err());
        assert!(super::decimal_seed("+1").is_err());
        assert!(super::decimal_seed("1.0").is_err());
    }

    #[test]
    fn burrow_counts_accept_integers_from_the_minimum_to_u32_max() {
        assert_eq!(super::checked_burrow_u32(0.0, "ticks", 0).unwrap(), 0);
        assert_eq!(
            super::checked_burrow_u32(1.0, "sample_every", 1).unwrap(),
            1
        );
        assert_eq!(
            super::checked_burrow_u32(f64::from(u32::MAX), "ticks", 0).unwrap(),
            u32::MAX
        );
    }

    // Rejections build a JsValue, which only exists on the wasm target.
    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn burrow_counts_reject_non_integers_and_values_below_minimum() {
        assert!(super::checked_burrow_u32(1.5, "ticks", 0).is_err());
        assert!(super::checked_burrow_u32(f64::NAN, "ticks", 0).is_err());
        assert!(super::checked_burrow_u32(-1.0, "ticks", 0).is_err());
        assert!(super::checked_burrow_u32(0.0, "sample_every", 1).is_err());
        assert!(super::checked_burrow_u32(4294967296.0, "ticks", 0).is_err());
    }
}
