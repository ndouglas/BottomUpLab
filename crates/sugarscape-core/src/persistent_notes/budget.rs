use super::types::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    pub action_events: u64,
    pub canonical_data_bytes: u64,
    pub checkpoints: u64,
    pub checkpoints_per_episode: u64,
    pub config_bytes: u64,
    pub contrast_rows: u64,
    pub envelope_rows: u64,
    pub episode_rows: u64,
    pub events: u64,
    pub header_bytes: u64,
    pub history_records: u64,
    pub logical_storage_bytes: u64,
    pub map_cell_visits: u64,
    pub mean_rows: u64,
    pub metric_atom_operations: u64,
    pub peak_rss_bytes: u64,
    pub report_bytes: u64,
    pub request_contrast_rows: u64,
    pub serialization_buffer_bytes: u64,
    pub view_history_visits: u64,
    pub wall_milliseconds: u64,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Counters {
    pub episode_rows: u64,
    pub mean_rows: u64,
    pub request_contrast_rows: u64,
    pub contrast_rows: u64,
    pub envelope_rows: u64,
    pub action_events: u64,
    pub events: u64,
    pub checkpoints: u64,
    pub history_records: u64,
    pub view_history_visits: u64,
    pub map_cell_visits: u64,
    pub metric_atom_operations: u64,
    pub logical_storage_bytes: u64,
    pub serialization_buffer_bytes: u64,
    pub canonical_data_bytes: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            action_events: 46080,
            canonical_data_bytes: 467519616,
            checkpoints: 73728,
            checkpoints_per_episode: 16,
            config_bytes: 4096,
            contrast_rows: 2304,
            envelope_rows: 576,
            episode_rows: 4608,
            events: 55296,
            header_bytes: 65536,
            history_records: 46080,
            logical_storage_bytes: 140685568,
            map_cell_visits: 442368,
            mean_rows: 2304,
            metric_atom_operations: 555840,
            peak_rss_bytes: 536870912,
            report_bytes: 467585152,
            request_contrast_rows: 4608,
            serialization_buffer_bytes: 65536,
            view_history_visits: 737280,
            wall_milliseconds: 120000,
        }
    }
}
pub(crate) struct Budget {
    pub(crate) limits: Limits,
    pub(crate) counters: Counters,
}
impl Budget {
    pub(crate) fn new(limits: &Limits) -> Result<Self, Error> {
        let mut budget = Self {
            limits: limits.clone(),
            counters: Counters::default(),
        };
        budget.charge("logical_storage_bytes", 65536)?;
        budget.charge("serialization_buffer_bytes", 65536)?;
        // Each of the five canonical arrays has two delimiters.
        budget.charge("canonical_data_bytes", 10)?;
        Ok(budget)
    }
    pub(crate) fn charge(&mut self, resource: &str, amount: u64) -> Result<(), Error> {
        let (counter, limit) = match resource {
            "episode_rows" => (&mut self.counters.episode_rows, self.limits.episode_rows),
            "mean_rows" => (&mut self.counters.mean_rows, self.limits.mean_rows),
            "request_contrast_rows" => (
                &mut self.counters.request_contrast_rows,
                self.limits.request_contrast_rows,
            ),
            "contrast_rows" => (&mut self.counters.contrast_rows, self.limits.contrast_rows),
            "envelope_rows" => (&mut self.counters.envelope_rows, self.limits.envelope_rows),
            "action_events" => (&mut self.counters.action_events, self.limits.action_events),
            "events" => (&mut self.counters.events, self.limits.events),
            "checkpoints" => (&mut self.counters.checkpoints, self.limits.checkpoints),
            "history_records" => (
                &mut self.counters.history_records,
                self.limits.history_records,
            ),
            "view_history_visits" => (
                &mut self.counters.view_history_visits,
                self.limits.view_history_visits,
            ),
            "map_cell_visits" => (
                &mut self.counters.map_cell_visits,
                self.limits.map_cell_visits,
            ),
            "metric_atom_operations" => (
                &mut self.counters.metric_atom_operations,
                self.limits.metric_atom_operations,
            ),
            "logical_storage_bytes" => (
                &mut self.counters.logical_storage_bytes,
                self.limits.logical_storage_bytes,
            ),
            "serialization_buffer_bytes" => (
                &mut self.counters.serialization_buffer_bytes,
                self.limits.serialization_buffer_bytes,
            ),
            "canonical_data_bytes" => (
                &mut self.counters.canonical_data_bytes,
                self.limits.canonical_data_bytes,
            ),
            _ => return Err(Error::InvalidInput("unknown budget resource".into())),
        };
        let next = counter
            .checked_add(amount)
            .ok_or(Error::ArithmeticOverflow)?;
        if next > limit {
            return Err(Error::ResourceLimit {
                resource: resource.into(),
                limit,
            });
        }
        *counter = next;
        Ok(())
    }
    pub(crate) fn row(&mut self, resource: &str, bytes: u64) -> Result<(), Error> {
        let old = self.counters.clone();
        if let Err(error) = self
            .charge(resource, 1)
            .and_then(|()| self.charge("logical_storage_bytes", bytes))
        {
            self.counters = old;
            return Err(error);
        }
        Ok(())
    }
    pub(crate) fn capture(
        &mut self,
        checkpoint: &Checkpoint,
        action: Option<&Primitive>,
        role: Option<Role>,
        event: bool,
        count: u64,
    ) -> Result<(), Error> {
        if count >= self.limits.checkpoints_per_episode {
            return Err(Error::ResourceLimit {
                resource: "checkpoints_per_episode".into(),
                limit: self.limits.checkpoints_per_episode,
            });
        }
        let old = self.counters.clone();
        let result = (|| {
            if action.is_some() {
                self.charge("action_events", 1)?;
                self.charge("history_records", 1)?;
            }
            if event {
                self.charge("events", 1)?;
            }
            self.charge("checkpoints", 1)?;
            for view in &checkpoint.own {
                let own_action = role == Some(view.role);
                self.charge(
                    "view_history_visits",
                    view.history.len() as u64 + u64::from(own_action),
                )?;
                let mut cells = view.map.cells.len() as u64;
                if let Some(primitive) = action.filter(|_| own_action) {
                    let cell = if primitive.kind == PrimitiveKind::Move {
                        primitive.cell
                    } else {
                        view.cell
                    };
                    if primitive.kind != PrimitiveKind::ReadNote
                        && cell.is_some_and(|c| !view.map.cells.iter().any(|m| m.cell == c))
                    {
                        cells = cells.checked_add(1).ok_or(Error::ArithmeticOverflow)?;
                    }
                }
                self.charge("map_cell_visits", cells)?;
            }
            Ok(())
        })();
        if result.is_err() {
            self.counters = old;
        }
        result
    }
    pub(crate) fn encoded<T: Serialize>(&mut self, value: &T, comma: bool) -> Result<(), Error> {
        let remaining = self
            .limits
            .canonical_data_bytes
            .checked_sub(self.counters.canonical_data_bytes)
            .ok_or(Error::ArithmeticOverflow)?;
        let bytes = encoded_len(value, remaining).map_err(|error| match error {
            Error::ResourceLimit { .. } => Error::ResourceLimit {
                resource: "canonical_data_bytes".into(),
                limit: self.limits.canonical_data_bytes,
            },
            other => other,
        })?;
        self.charge(
            "canonical_data_bytes",
            bytes
                .checked_add(u64::from(comma))
                .ok_or(Error::ArithmeticOverflow)?,
        )
    }
}
/// Compact JSON length is independent of object key order. Counts without a payload buffer.
pub(crate) fn encoded_len<T: Serialize>(value: &T, limit: u64) -> Result<u64, Error> {
    struct Counter {
        bytes: u64,
        limit: u64,
        exceeded: bool,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let next = self.bytes.checked_add(bytes.len() as u64);
            if next.is_none_or(|n| n > self.limit) {
                self.exceeded = true;
                return Err(std::io::Error::other("canonical byte limit"));
            }
            self.bytes = next.unwrap();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter {
        bytes: 0,
        limit,
        exceeded: false,
    };
    if let Err(error) = serde_json::to_writer(&mut counter, value) {
        return Err(if counter.exceeded {
            Error::ResourceLimit {
                resource: "canonical_data_bytes".into(),
                limit,
            }
        } else {
            Error::InvalidReport(error.to_string())
        });
    }
    Ok(counter.bytes)
}
