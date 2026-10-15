//! Inspect raw JSON before serde's tagged-enum/Value buffering can erase keys.
//! Only a top-level Notes study enables the additional rejection rule.
use serde::de::{DeserializeSeed, MapAccess, SeqAccess, Visitor};
use std::{collections::BTreeSet, fmt};

#[derive(Default)]
struct Scan {
    key: bool,
    duplicate: Option<String>,
}
struct Node<'a> {
    scan: &'a mut Scan,
    root: bool,
}
impl<'de> DeserializeSeed<'de> for Node<'_> {
    type Value = bool;
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<bool, D::Error> {
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for Node<'_> {
    type Value = bool;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("JSON value")
    }
    fn visit_str<E: serde::de::Error>(self, s: &str) -> Result<bool, E> {
        Ok(s == "persistent_notes")
    }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<bool, E> {
        Ok(false)
    }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<bool, E> {
        Ok(false)
    }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<bool, E> {
        Ok(false)
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<bool, E> {
        Ok(false)
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<bool, E> {
        Ok(false)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<bool, A::Error> {
        while a
            .next_element_seed(Node {
                scan: self.scan,
                root: false,
            })?
            .is_some()
        {}
        Ok(false)
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<bool, A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(key) = a.next_key::<String>()? {
            if !keys.insert(key.clone()) && self.scan.duplicate.is_none() {
                self.scan.duplicate = Some(key.clone());
            }
            let is_key = a.next_value_seed(Node {
                scan: self.scan,
                root: false,
            })?;
            if self.root && key == "study" && is_key {
                self.scan.key = true;
            }
        }
        Ok(false)
    }
}
pub(crate) fn check(json: &str, field: &str) -> Result<(), Vec<super::FieldError>> {
    let mut scan = Scan::default();
    // Syntax/type/depth errors retain the original deserializer's diagnostics.
    // A valid scan never constructs a Value or loses an earlier duplicate tag.
    let mut d = serde_json::Deserializer::from_str(json);
    let _ = Node {
        scan: &mut scan,
        root: true,
    }
    .deserialize(&mut d);
    if scan.key {
        if let Some(key) = scan.duplicate {
            return Err(super::error(
                field,
                format!("duplicate Key object key {key:?}"),
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check;
    #[test]
    fn key_nested_array_objects_and_duplicate_earlier_study_are_detected() {
        for text in [
            r#"{"payload":[{"a":1,"a":2}],"study":"persistent_notes"}"#,
            r#"{"study":"persistent_notes","study":"wink"}"#,
            r#"{"study":"wink","study":"persistent_notes"}"#,
            r#"{"study":"persistent_notes","payload":{"outer":[{"x":[],"x":null}]}}"#,
        ] {
            assert!(check(text, "episode").is_err(), "{text}");
        }
    }
    #[test]
    fn legacy_records_keep_their_original_value_duplicate_behavior() {
        for text in [
            r#"{"study":"wink","payload":[{"a":1,"a":2}]}"#,
            r#"{"study":"wink","payload":{"study":"persistent_notes","x":0,"x":1}}"#,
        ] {
            assert!(check(text, "episode").is_ok(), "{text}");
        }
    }
}
