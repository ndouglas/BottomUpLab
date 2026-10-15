//! Source-bound retained measurements. This module never runs training or diagnostics.
use super::{record, wire::lossless_value, FieldError, MAX_EPISODE_BYTES};
use serde_json::{json, Value};
use std::sync::OnceLock;
const MAX_ASSET_BYTES: usize = 5 * 1024 * 1024;
static STRATEGIC: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
static STRATEGY: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
static AUDIT: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
static GAME: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
static DISCOVERY: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
static KEY: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
static GATE: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
/// Embedded immutable assets are parsed once; no simulation state is cached.
pub(super) fn retained(study: &str) -> Result<&'static Value, Vec<FieldError>> {
    let (slot, text) = match study {
        "strategic_reporting" => (&STRATEGIC, include_str!("fixtures/strategic-recorded.json")),
        "strategy_inference" => (&STRATEGY, include_str!("fixtures/strategy-recorded.json")),
        "adversarial_audit" => (&AUDIT, include_str!("fixtures/audit-recorded.json")),
        "testimony_game" => (
            &GAME,
            include_str!("fixtures/testimony-search-recorded.json"),
        ),
        _ => return Err(super::error("study", "unknown retained study")),
    };
    slot.get_or_init(|| {
        if text.len() > MAX_ASSET_BYTES {
            return Err(super::error("fixture", "retained asset exceeds 5 MiB"));
        }
        let value: Value =
            serde_json::from_str(text).map_err(|e| super::error("fixture", e.to_string()))?;
        if value["transformation_version"] != "browser-retained-v1"
            || value["provenance"]["sha256"]
                .as_str()
                .is_none_or(|s| s.len() != 64)
            || value["identity"]["source_sha256"]
                .as_str()
                .is_none_or(|s| s.len() != 64)
        {
            return Err(super::error("fixture", "missing retained source identity"));
        }
        Ok(value)
    })
    .as_ref()
    .map_err(Clone::clone)
}
/// This asset is the validated first-measurement Summary, not a live episode.
fn discovery_summary() -> Result<&'static Value, Vec<FieldError>> {
    DISCOVERY
        .get_or_init(|| {
            let text = include_str!("fixtures/discovery-recorded.json");
            if text.len() > MAX_ASSET_BYTES {
                return Err(super::error("fixture", "retained asset exceeds 5 MiB"));
            }
            let summary: crate::surface_discovery::Summary = serde_json::from_str(text)
                .map_err(|error| super::error("fixture", error.to_string()))?;
            if summary.version != "surface-discovery-summary-v1"
                || [summary.report_hash.as_str(), summary.source_hash.as_str()]
                    .iter()
                    .any(|hash| {
                        hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
                    })
            {
                return Err(super::error(
                    "fixture",
                    "missing measured discovery identity",
                ));
            }
            lossless_value(&summary)
        })
        .as_ref()
        .map_err(Clone::clone)
}
/// Retained first-report summary. Loading it performs no scientific execution.
fn gate_summary() -> Result<&'static Value, Vec<FieldError>> {
    GATE.get_or_init(|| {
        let text = include_str!("fixtures/gate-recorded.json");
        if text.len() > MAX_ASSET_BYTES {
            return Err(super::error("fixture", "retained asset exceeds 5 MiB"));
        }
        let summary: crate::gate_assistance::Summary = serde_json::from_str(text)
            .map_err(|error| super::error("fixture", error.to_string()))?;
        if summary.version != "gate-assistance-summary-v1"
            || summary.settings.len() != 18
            || summary.contrasts.len() != 45
            || [summary.report_hash.as_str(), summary.source_hash.as_str()]
                .iter()
                .any(|hash| hash.len() != 64 || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()))
        {
            return Err(super::error(
                "fixture",
                "missing measured Gate identity or census",
            ));
        }
        lossless_value(&summary)
    })
    .as_ref()
    .map_err(Clone::clone)
}
/// Accepted retained first-report projection; no live game or integrity replay.
fn key_summary() -> Result<&'static Value, Vec<FieldError>> {
    KEY.get_or_init(|| {
        let text = include_str!("fixtures/key-recorded.json");
        if text.len() > MAX_ASSET_BYTES {
            return Err(super::error("fixture", "retained asset exceeds 5 MiB"));
        }
        let summary: crate::key_delegation::Summary = serde_json::from_str(text)
            .map_err(|error| super::error("fixture", error.to_string()))?;
        if summary.version != "key-delegation-summary-v1"
            || summary.settings.len() != 36
            || summary.settings.iter().any(|s| s.controls.len() != 6)
            || summary.contrasts.len() != 84
            || summary.source_receipts.len() != 18
            || summary.reference_receipts.len() != 3
            || [summary.report_hash.as_str(), summary.source_hash.as_str()]
                .iter()
                .any(|hash| hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()))
        {
            return Err(super::error(
                "fixture",
                "missing measured Key identity or census",
            ));
        }
        lossless_value(&summary)
    })
    .as_ref()
    .map_err(Clone::clone)
}
/// Complete Notes reductions; a delivery-only dictionary retains every exact row.
fn notes_summary() -> Result<&'static Value, Vec<FieldError>> {
    static NOTES: OnceLock<Result<Value, Vec<FieldError>>> = OnceLock::new();
    NOTES
        .get_or_init(|| {
            let text = include_str!("fixtures/notes-recorded.json");
            // Notes has more registered reductions; other assets retain their 5 MiB bound.
            if text.len() > MAX_EPISODE_BYTES {
                return Err(super::error(
                    "fixture",
                    "Notes summary exceeds transport bound",
                ));
            }
            let mut summary: Value =
                serde_json::from_str(text).map_err(|e| super::error("fixture", e.to_string()))?;
            let source = crate::persistent_notes::source_identity()
                .map_err(|e| super::error("fixture", e.to_string()))?;
            let reference = crate::persistent_notes::reference_identity()
                .map_err(|e| super::error("fixture", e.to_string()))?;
            if summary["version"] != "persistent-notes-summary-v1"
                || summary["protocol_version"] != 1
                || summary["source"]["sha256"] != source
                || summary["reference"]["sha256"] != reference
                || summary["report_sha256"]
                    != "376ed7f1d99a6095f958f5a7de9b0622b40b01a536ea411e51c9e75ea833a717"
            {
                return Err(super::error(
                    "fixture",
                    "invalid frozen Notes summary identity",
                ));
            }
            for (field, count) in [
                ("means", 2304),
                ("request_contrasts", 4608),
                ("contrasts", 2304),
                ("envelopes", 576),
            ] {
                if summary[field]
                    .as_array()
                    .is_none_or(|rows| rows.len() != count)
                {
                    return Err(super::error("fixture", "incomplete Notes summary"));
                }
            }
            let mut dictionary = Vec::new();
            let mut indices = std::collections::BTreeMap::new();
            for field in ["means", "request_contrasts", "contrasts"] {
                for row in summary[field].as_array_mut().expect("checked array") {
                    let metrics = row
                        .get_mut("metrics")
                        .ok_or_else(|| super::error("fixture", "missing Notes metrics"))?;
                    let key = serde_json::to_string(metrics)
                        .map_err(|e| super::error("fixture", e.to_string()))?;
                    let index = *indices.entry(key).or_insert_with(|| {
                        let index = dictionary.len();
                        dictionary.push(metrics.clone());
                        index
                    });
                    *metrics = json!(index);
                }
            }
            summary["metrics_dictionary"] = Value::Array(dictionary);
            summary["wire_encoding"] = json!("persistent-notes-metrics-dictionary-v1");
            lossless_value(&summary)
        })
        .as_ref()
        .map_err(Clone::clone)
}
/// Original search campaigns and the complete measured discovery census.
/// Each keeps its own original report/source identity.
/// Curves use the declared columns and champion dictionary without dropping points.
pub fn recorded_results_json() -> Result<String, Vec<FieldError>> {
    let mut studies = serde_json::Map::new();
    for study in ["testimony_game", "strategic_reporting"] {
        let mut value = retained(study)?.clone();
        if value["runs"].as_array().is_none_or(|r| r.len() != 40) {
            return Err(super::error(
                "recorded",
                "retained search campaign must contain all 40 runs",
            ));
        }
        // Selected-case references serve the case adapter, not the search panel.
        let object = value
            .as_object_mut()
            .ok_or_else(|| super::error("fixture", "expected retained object"))?;
        object.remove("case_references");
        object.remove("policies");
        studies.insert(study.into(), lossless_value(&value)?);
    }
    studies.insert("surface_discovery".into(), discovery_summary()?.clone());
    studies.insert("gate_assistance".into(), gate_summary()?.clone());
    studies.insert("key_delegation".into(), key_summary()?.clone());
    studies.insert("persistent_notes".into(), notes_summary()?.clone());
    record::bounded_json(
        &json!({"kind":"recorded_results","version":1,"studies":studies}),
        MAX_EPISODE_BYTES,
        "recorded",
    )
}
