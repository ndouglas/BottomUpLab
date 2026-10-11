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
    gate: Option<bool>,
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
    holder_word: g::HolderWord,
    retriever_word: g::RetrieverWord,
    classification: &'a g::Classification,
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
    classification: &g::Classification,
    index: usize,
    retention: &Retention,
) -> Result<Checkpoint, g::Error> {
    let local = [&ids.holder, &ids.retriever]
        .into_iter()
        .zip(&point.views)
        .map(|(id, view)| {
            (
                id.as_str(),
                Local {
                    view,
                    map: Map {
                        cells: &view.last_observation.cells,
                        gate: view.last_observation.gate,
                        observed_at: &view.last_observation.clock,
                    },
                },
            )
        })
        .collect();
    let kind = match point.clock.phase {
        g::Phase::Initial => "initial",
        g::Phase::Reset => "reset",
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
            holder_word: g::HolderWord::from_id(profile.holder)?,
            retriever_word: g::RetrieverWord::from_id(profile.retriever)?,
            classification,
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
}
