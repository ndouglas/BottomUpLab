//! Pinned compact independent projections and sealed artifact provenance.
//! Receipt binding is portable; Task 8 verifies external artifact bytes separately.
use super::*;
use serde_json::Value;
use std::collections::BTreeMap;
const EVALUATION: &[u8] = include_bytes!("../tests/fixtures/evaluation-reference.json");
const PLANNING: &[u8] = include_bytes!("../tests/fixtures/planning-reference.json");
const LIMITS: &[u8] = include_bytes!("../tests/fixtures/limits-reference.json");
pub(in crate::surface_discovery) struct Reference {
    pub value: Value,
    pub artifacts: Vec<ArtifactReceipt>,
}
impl Reference {
    pub fn read() -> Result<Self, Error> {
        let mut values = Vec::new();
        let mut artifacts = BTreeMap::new();
        for (name, bytes, expected) in [
            (
                "evaluation",
                EVALUATION,
                "64956b26b9d9ac2620c04e2db3503547295cfdc22af18030fba781f3fc10fd78",
            ),
            (
                "planning",
                PLANNING,
                "f072aceffae6021fdc3e6b767b17d6574866ae6750fb6b8fd2d77ab052ed95a6",
            ),
            (
                "limits",
                LIMITS,
                "51057b92e7260c783a10cde6dbb37c1a105115715a9caae5d4448f6e03a1cad3",
            ),
        ] {
            if hash::sha256(bytes) != expected {
                return Err(invalid(format!(
                    "independent {name} fixture SHA-256 mismatch"
                )));
            }
            let value: Value = serde_json::from_slice(bytes)
                .map_err(|e| invalid(format!("independent {name} fixture: {e}")))?;
            if value["version"] != format!("surface-discovery-{name}-reference-v1")
                || value["metadata"]["protocol_version"] != 1
                || value["metadata"]["protocol_sha256"]
                    != "11b1c8af8320fa729720d72e74a6d29fd5245bf961cafbf174827b5ba6c3de84"
            {
                return Err(invalid(format!("independent {name} provenance mismatch")));
            }
            collect(&value, &mut artifacts)?;
            let receipt=ArtifactReceipt { path:format!("crates/sugarscape-core/src/surface_discovery/tests/fixtures/{name}-reference.json"), bytes:bytes.len() as u64, sha256:expected.into() };
            artifacts.insert(receipt.path.clone(), receipt);
            values.push(value);
        }
        let value = values.remove(0);
        if value["panels"].as_array().map(Vec::len) != Some(48)
            || value["comparisons"].as_array().map(Vec::len) != Some(24)
            || value["role_expectations"].as_array().map(Vec::len) != Some(10)
            || value["role_contrasts"].as_array().map(Vec::len) != Some(6)
            || value["coverage"]["total_episodes"] != 12288
        {
            return Err(invalid("independent reference coverage mismatch"));
        }
        for (index, (protocol, environment)) in registered_settings().iter().enumerate() {
            let row = &value["panels"][index];
            if row["setting"] != index
                || row["protocol"] != json_value(protocol)?
                || row["environment"] != json_value(environment)?
                || row["prior"] != json_value(&protocol::own_prior(protocol, *environment)?)?
            {
                return Err(invalid("independent reference census mismatch"));
            }
        }
        Ok(Self {
            value,
            artifacts: artifacts.into_values().collect(),
        })
    }
    pub fn panel_matches(&self, index: usize, panel: &Panel) -> Result<bool, Error> {
        let row = &self.value["panels"][index];
        let mut projection = row["projection"].clone();
        projection
            .as_object_mut()
            .ok_or_else(|| invalid("reference projection object missing"))?
            .remove("episode_projection_receipt");
        for failure in projection["failures"]
            .as_array_mut()
            .ok_or_else(|| invalid("reference failures missing"))?
        {
            failure
                .as_object_mut()
                .ok_or_else(|| invalid("reference failure object missing"))?
                .remove("prefixes");
        }
        if row["aggregate"] != json_value(&panel.aggregate)?
            || projection != json_value(&project_panel(panel)?)?
        {
            return Ok(false);
        }
        for witness in self.value["episodes"]
            .as_array()
            .ok_or_else(|| invalid("reference episodes missing"))?
        {
            if witness["protocol"] == row["protocol"]
                && witness["environment"] == row["environment"]
            {
                let sequence = witness["sequence"]
                    .as_u64()
                    .ok_or_else(|| invalid("reference sequence missing"))?
                    as usize;
                if panel.episodes.get(sequence).map(json_value).transpose()?
                    != Some(witness["episode"].clone())
                {
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }
    pub fn summaries_match(&self, report: &DiagnosticReport) -> Result<bool, Error> {
        Ok(
            json_value(&report.comparisons)? == self.value["comparisons"]
                && json_value(&report.role_expectations)? == self.value["role_expectations"]
                && json_value(&report.role_contrasts)? == self.value["role_contrasts"],
        )
    }
}
fn collect(value: &Value, artifacts: &mut BTreeMap<String, ArtifactReceipt>) -> Result<(), Error> {
    match value {
        Value::Object(object) => {
            if object.contains_key("path")
                && object.contains_key("bytes")
                && object.contains_key("sha256")
            {
                let receipt: ArtifactReceipt = serde_json::from_value(value.clone())
                    .map_err(|e| invalid(format!("reference artifact receipt: {e}")))?;
                if receipt.bytes == 0
                    || receipt.sha256.len() != 64
                    || !receipt
                        .sha256
                        .bytes()
                        .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                {
                    return Err(invalid("reference artifact receipt invalid"));
                }
                if let Some(old) = artifacts.insert(receipt.path.clone(), receipt.clone()) {
                    if old != receipt {
                        return Err(invalid(format!(
                            "conflicting artifact receipt {}",
                            receipt.path
                        )));
                    }
                }
            } else {
                for child in object.values() {
                    collect(child, artifacts)?;
                }
            }
        }
        Value::Array(array) => {
            for child in array {
                collect(child, artifacts)?;
            }
        }
        _ => {}
    }
    Ok(())
}
