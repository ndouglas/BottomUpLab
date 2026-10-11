use super::{Episode, Error};
use serde::Serialize;
use serde_json::Value;

/// Compact, recursively sorted native JSON. Arrays and UTF-8 are preserved.
pub(crate) fn canonical_bytes(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    fn sorted(value: Value) -> Result<Value, Error> {
        Ok(match value {
            Value::Number(n) if !n.is_i64() && !n.is_u64() => {
                return Err(Error::InvalidReport(
                    "canonical native JSON requires integer numbers".into(),
                ))
            }
            Value::Array(a) => Value::Array(a.into_iter().map(sorted).collect::<Result<_, _>>()?),
            Value::Object(o) => {
                let ordered: std::collections::BTreeMap<_, _> = o.into_iter().collect();
                Value::Object(
                    ordered
                        .into_iter()
                        .map(|(key, value)| Ok((key, sorted(value)?)))
                        .collect::<Result<_, Error>>()?,
                )
            }
            other => other,
        })
    }
    let value = serde_json::to_value(value)
        .map_err(|e| Error::InvalidReport(format!("canonical serialization: {e}")))?;
    serde_json::to_vec(&sorted(value)?)
        .map_err(|e| Error::InvalidReport(format!("canonical bytes: {e}")))
}

pub fn trace_hash(episode: &Episode) -> Result<String, Error> {
    episode.validate()?;
    let body = canonical_bytes(episode)?;
    crate::surface_discovery::compiled_bytes_identity(&[(
        "gate-assistance-trace-v1",
        body.as_slice(),
    )])
    .map_err(|e| Error::InvalidReport(format!("trace identity: {e}")))
}
