use super::{Episode, Error};
use serde::Serialize;
use serde_json::Value;
/// Sorted-key compact native JSON; exact integers and unchanged Unicode strings.
pub(super) fn canonical_bytes(value: &impl Serialize) -> Result<Vec<u8>, Error> {
    fn sorted(v: Value) -> Result<Value, Error> {
        Ok(match v {
            Value::Number(ref n) if !n.is_i64() && !n.is_u64() => {
                return Err(Error::InvalidReport("noninteger canonical atom".into()))
            }
            Value::Array(a) => Value::Array(a.into_iter().map(sorted).collect::<Result<_, _>>()?),
            Value::Object(o) => {
                let ordered: std::collections::BTreeMap<_, _> = o.into_iter().collect();
                Value::Object(
                    ordered
                        .into_iter()
                        .map(|(k, v)| Ok((k, sorted(v)?)))
                        .collect::<Result<_, Error>>()?,
                )
            }
            other => other,
        })
    }
    let value = serde_json::to_value(value).map_err(|e| Error::InvalidReport(e.to_string()))?;
    serde_json::to_vec(&sorted(value)?).map_err(|e| Error::InvalidReport(e.to_string()))
}
pub fn trace_hash(episode: &Episode) -> Result<String, Error> {
    episode.validate()?;
    let body = canonical_bytes(episode)?;
    trace_identity(&body)
}
pub(super) fn trace_identity(body: &[u8]) -> Result<String, Error> {
    receipt_identity("key-delegation-trace-v1", body)
}
pub(super) fn receipt_identity(path: &str, body: &[u8]) -> Result<String, Error> {
    #[derive(Serialize)]
    struct Receipt<'a> {
        path: &'a str,
        bytes: u64,
        sha256: String,
    }
    let receipt = Receipt {
        path,
        bytes: u64::try_from(body.len()).map_err(|_| Error::ArithmeticOverflow)?,
        sha256: super::sha256::sha256(body),
    };
    let wire = serde_json::to_vec(&[receipt]).map_err(|e| Error::InvalidReport(e.to_string()))?;
    Ok(super::sha256::sha256(&wire))
}
