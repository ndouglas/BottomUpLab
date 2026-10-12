//! Dated own observations and evaluator truth, taken only from native captures.
use super::g;
use crate::browser_experiments::{record, wire, Checkpoint};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

pub(super) struct Retention {
    pub bytes: usize,
    pub limit: usize,
}
impl Retention {
    pub fn reserve(&self, additional: usize) -> Result<(), g::Error> {
        if self
            .bytes
            .checked_add(additional)
            .ok_or(g::Error::ArithmeticOverflow)?
            > self.limit
        {
            return Err(g::Error::ResourceLimit {
                resource: "browser_episode_bytes".into(),
                limit: self.limit as u64,
            });
        }
        Ok(())
    }
    pub fn size<T: Serialize>(&self, value: &T) -> Result<usize, g::Error> {
        record::serialized_size(value, self.limit, "episode").map_err(|_| g::Error::ResourceLimit {
            resource: "browser_episode_bytes".into(),
            limit: self.limit as u64,
        })
    }
    pub fn value<T: Serialize>(&self, value: &T) -> Result<Value, g::Error> {
        let native = self.size(value)?;
        // One native logical serialization plus at most three times its bytes
        // for Value conversion (a one-digit integer becomes a three-byte string).
        // Reserve the coexistence peak before creating that temporary Value.
        self.reserve(native.checked_mul(4).ok_or(g::Error::ArithmeticOverflow)?)?;
        wire::lossless_value(value)
            .map_err(|e| g::Error::InvalidReport(format!("browser conversion: {e:?}")))
    }
    pub fn push(
        &mut self,
        checkpoints: &mut Vec<Checkpoint>,
        checkpoint: Checkpoint,
    ) -> Result<(), g::Error> {
        if checkpoints.len() >= super::super::MAX_CHECKPOINTS {
            return Err(g::Error::ResourceLimit {
                resource: "browser_checkpoints".into(),
                limit: super::super::MAX_CHECKPOINTS as u64,
            });
        }
        let bytes = self
            .size(&checkpoint)?
            .checked_add(usize::from(!checkpoints.is_empty()))
            .ok_or(g::Error::ArithmeticOverflow)?;
        self.reserve(bytes)?;
        self.bytes += bytes;
        checkpoints.push(checkpoint);
        Ok(())
    }
}
#[derive(Serialize)]
struct Map<'a> {
    cells: &'a [g::VisibleCell],
    observed_at: &'a g::Clock,
}
#[derive(Serialize)]
struct Local<'a> {
    view: &'a g::View,
    map: Map<'a>,
}
#[derive(Serialize)]
struct Researcher<'a> {
    label: &'static str,
    physical: &'a g::PhysicalState,
    profile: g::Profile,
    words: [g::OwnWord; 2],
    classification: Option<&'a g::Classification>,
    classification_scope: g::ClassificationScope,
}
#[derive(Serialize)]
struct Public {
    status: &'static str,
}
#[derive(Serialize)]
struct Projection<'a> {
    index: u32,
    clock: &'a g::Clock,
    kind: &'static str,
    public: Public,
    local: BTreeMap<&'a str, Local<'a>>,
    researcher: Researcher<'a>,
}
pub(super) fn checkpoint(
    point: &g::CapturePoint,
    ids: &g::Ids,
    profile: g::Profile,
    classification: Option<&g::Classification>,
    classification_scope: g::ClassificationScope,
    index: usize,
    retention: &Retention,
) -> Result<Checkpoint, g::Error> {
    let local = [ids.giver(), ids.receiver()]
        .into_iter()
        .zip(&point.views)
        .map(|(id, view)| {
            (
                id,
                Local {
                    view,
                    map: Map {
                        cells: &view.last_observation.cells,
                        observed_at: &view.last_observation.clock,
                    },
                },
            )
        })
        .collect();
    let kind = match point.clock.phase {
        g::Phase::Initial => "initial",
        g::Phase::BeforeAction => "before_action",
        g::Phase::AfterAction => "after_action",
        g::Phase::Finished => "finished",
    };
    let projection = Projection {
        index: u32::try_from(index).map_err(|_| g::Error::ArithmeticOverflow)?,
        clock: &point.clock,
        kind,
        public: Public {
            status: if point.clock.phase == g::Phase::Finished {
                "finished"
            } else {
                "running"
            },
        },
        local,
        researcher: Researcher {
            label: "Researcher",
            physical: &point.physical,
            profile,
            words: profile.words(),
            classification,
            classification_scope,
        },
    };
    let mut value = retention.value(&projection)?;
    value["index"] = serde_json::json!(projection.index);
    serde_json::from_value(value)
        .map_err(|e| g::Error::InvalidReport(format!("browser checkpoint: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serialized_checkpoint_limit_counts_comma_and_rejects_before_retention() {
        let checkpoint = Checkpoint {
            index: 0,
            clock: Value::Null,
            kind: "initial".into(),
            public: Value::Null,
            local: BTreeMap::new(),
            researcher: None,
        };
        let bytes = serde_json::to_vec(&checkpoint).unwrap().len();
        let mut budget = Retention {
            bytes: 2,
            limit: 2 + bytes,
        };
        let mut checkpoints = vec![];
        budget.push(&mut checkpoints, checkpoint.clone()).unwrap();
        assert_eq!(
            budget.bytes,
            serde_json::to_vec(&checkpoints).unwrap().len()
        );
        assert!(budget.push(&mut checkpoints, checkpoint).is_err());
        assert_eq!(checkpoints.len(), 1);
    }
    #[test]
    fn temporary_conversion_reservation_precedes_value_allocation() {
        let budget = Retention {
            bytes: 8,
            limit: 16,
        };
        assert!(budget.value(&123).is_err());
        assert_eq!(budget.bytes, 8);
    }
    #[test]
    fn checkpoint_count_and_arithmetic_reject_without_retaining_candidate() {
        let checkpoint = Checkpoint {
            index: 0,
            clock: Value::Null,
            kind: "initial".into(),
            public: Value::Null,
            local: BTreeMap::new(),
            researcher: None,
        };
        let mut records = vec![checkpoint.clone(); super::super::super::MAX_CHECKPOINTS];
        let mut budget = Retention {
            bytes: 2,
            limit: super::super::super::MAX_EPISODE_BYTES,
        };
        assert!(budget.push(&mut records, checkpoint).is_err());
        assert_eq!(records.len(), super::super::super::MAX_CHECKPOINTS);
        assert_eq!(budget.bytes, 2);
        assert!(Retention {
            bytes: usize::MAX,
            limit: usize::MAX
        }
        .reserve(1)
        .is_err());
    }
    #[test]
    fn failed_serialization_and_capture_preserve_retained_prefix_and_native_replay() {
        struct Fail;
        impl Serialize for Fail {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(serde::ser::Error::custom("injected serialization failure"))
            }
        }
        let budget = Retention {
            bytes: 8,
            limit: 1024,
        };
        assert!(budget.value(&Fail).is_err());
        assert_eq!(budget.bytes, 8);
        let config = g::Config::new(
            g::Stratum::new(true, 1, 3, 0).unwrap(),
            g::Ids::default(),
            g::InitialOwner::Giver,
        )
        .unwrap();
        let profile = g::Profile::new(486, 60).unwrap();
        let expected = g::run_episode(&config, profile).unwrap();
        let mut records = vec![];
        let mut budget = Retention {
            bytes: 2,
            limit: 16,
        };
        assert!(g::run_with_capture(&config, profile, &mut |point| {
            let candidate = checkpoint(
                point,
                config.ids(),
                profile,
                None,
                g::ClassificationScope::Main,
                records.len(),
                &budget,
            )?;
            budget.push(&mut records, candidate)
        })
        .is_err());
        assert!(records.is_empty());
        assert_eq!(budget.bytes, 2);
        assert_eq!(g::run_episode(&config, profile).unwrap(), expected);
    }
}
