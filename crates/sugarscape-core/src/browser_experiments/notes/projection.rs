//! Owned committed snapshots from native captures, with no replay observation refresh.
use super::super::{wire, Checkpoint, FieldError};
use crate::persistent_notes as n;
use serde_json::json;
pub(super) fn checkpoint(
    point: &n::Checkpoint,
    ids: &n::Ids,
    index: usize,
) -> Result<Checkpoint, Vec<FieldError>> {
    let local = [&ids.writer, &ids.reader]
        .into_iter()
        .zip(&point.own)
        .map(|(id, view)| Ok((id.clone(), wire::lossless_value(&json!({"view":view}))?)))
        .collect::<Result<_, Vec<FieldError>>>()?;
    Ok(Checkpoint {
        index: index as u32,
        clock: wire::lossless_value(&point.clock)?,
        kind: if index == 0 {
            "initial"
        } else if point.public.terminal {
            "terminal"
        } else {
            "committed"
        }
        .into(),
        public: wire::lossless_value(&point.public)?,
        local,
        researcher: Some(wire::lossless_value(
            &json!({"label":"Researcher","physical":point.physical,"event":point.event}),
        )?),
    })
}
