use super::super::*;
use serde::{
    de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor},
    Deserialize,
};
use std::{fmt, fs::File, io::BufReader};
struct Array<'a, T>(&'a [T]);
impl<'de, T: Deserialize<'de> + PartialEq + fmt::Debug> DeserializeSeed<'de> for Array<'_, T> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        struct Compare<'a, T>(&'a [T]);
        impl<'de, T: Deserialize<'de> + PartialEq + fmt::Debug> Visitor<'de> for Compare<'_, T> {
            type Value = ();
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("complete ordered reference array")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
                for (k, expected) in self.0.iter().enumerate() {
                    let actual = a
                        .next_element::<T>()?
                        .ok_or_else(|| A::Error::custom(format!("missing reference row {k}")))?;
                    if actual != *expected {
                        return Err(A::Error::custom(format!(
                            "reference mismatch at row {k}: {actual:?} != {expected:?}"
                        )));
                    }
                }
                if a.next_element::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(A::Error::custom("extra reference row"));
                }
                Ok(())
            }
        }
        d.deserialize_seq(Compare(self.0))
    }
}
struct Fixture<'a>(&'a Evaluation, bool);
impl<'de> DeserializeSeed<'de> for Fixture<'_> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        d.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for Fixture<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("sealed evaluation fixture")
    }
    fn visit_map<A: MapAccess<'de>>(self, mut a: A) -> Result<(), A::Error> {
        let mut seen = 0;
        while let Some(key) = a.next_key::<String>()? {
            if self.1 {
                let bit = match key.as_str() {
                    "rows" => {
                        a.next_value_seed(Array(&self.0.rows))?;
                        1
                    }
                    "means" => {
                        a.next_value_seed(Array(&self.0.means))?;
                        2
                    }
                    "request_contrasts" => {
                        a.next_value_seed(Array(&self.0.request_contrasts))?;
                        4
                    }
                    "contrasts" => {
                        a.next_value_seed(Array(&self.0.contrasts))?;
                        8
                    }
                    "envelopes" => {
                        a.next_value_seed(Array(&self.0.envelopes))?;
                        16
                    }
                    _ => return Err(A::Error::custom("unknown reference payload field")),
                };
                if seen & bit != 0 {
                    return Err(A::Error::custom("duplicate reference array"));
                }
                seen |= bit;
            } else if key == "payload" {
                a.next_value_seed(Fixture(self.0, true))?;
                seen += 1;
            } else {
                a.next_value::<serde::de::IgnoredAny>()?;
            }
        }
        if seen != if self.1 { 31 } else { 1 } {
            return Err(A::Error::custom("incomplete fixture"));
        }
        Ok(())
    }
}
#[test]
fn engineering_census_agrees_with_every_sealed_row_and_reducer() -> Result<(), Error> {
    let evaluation = evaluate(&Limits::default())?;
    assert_eq!(
        (
            evaluation.rows.len(),
            evaluation.means.len(),
            evaluation.request_contrasts.len(),
            evaluation.contrasts.len(),
            evaluation.envelopes.len()
        ),
        (4608, 2304, 4608, 2304, 576)
    );
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/persistent_notes/tests/fixtures/evaluation-reference.json"
    );
    let file = File::open(path).unwrap();
    let mut decoder = serde_json::Deserializer::from_reader(BufReader::with_capacity(65536, file));
    Fixture(&evaluation, false)
        .deserialize(&mut decoder)
        .unwrap();
    decoder.end().unwrap();
    let limits: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/limits-reference.json")).unwrap();
    let counters: Counters = serde_json::from_value(limits["payload"]["counters"].clone()).unwrap();
    assert_eq!(evaluation.counters, counters);
    let frozen: Limits = serde_json::from_value(limits["payload"]["limits"].clone()).unwrap();
    assert_eq!(Limits::default(), frozen);
    Ok(())
}
