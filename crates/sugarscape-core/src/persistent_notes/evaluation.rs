use super::{
    budget::{Budget, Counters, Limits},
    controller::run_budgeted,
    metrics::*,
    types::*,
};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    pub rows: Vec<Episode>,
    pub means: Vec<MeanRow>,
    pub request_contrasts: Vec<RequestContrast>,
    pub contrasts: Vec<MeanContrast>,
    pub envelopes: Vec<Envelope>,
    pub counters: Counters,
}
const ARMS: [Arm; 4] = [
    Arm::Retained,
    Arm::Erased,
    Arm::NoNote,
    Arm::NoAcquireNoNote,
];
const ROUTINES: [ReaderRoutine; 4] = [
    ReaderRoutine::BlindGuess,
    ReaderRoutine::InspectFirst,
    ReaderRoutine::ReadAdaptive,
    ReaderRoutine::Oracle,
];
const CONTRASTS: [(usize, usize, Contrast); 4] = [
    (0, 1, Contrast::RetainedErased),
    (0, 2, Contrast::RetainedNoNote),
    (1, 2, Contrast::ErasedNoNote),
    (0, 3, Contrast::RetainedNoAcquireNoNote),
];
pub fn registered_strata() -> Vec<Stratum> {
    let mut strata = Vec::with_capacity(144);
    for s in [1, 4] {
        for w in [1, 2, 4, 6] {
            for r in [1, 2] {
                for i in [2, 4, 8] {
                    for b in [0, 3, 6] {
                        strata.push(Stratum {
                            acquisition_cost: s,
                            write_cost: w,
                            note_read_cost: r,
                            manifest_inspection_cost: i,
                            writer_share: b,
                        });
                    }
                }
            }
        }
    }
    strata
}
/// Exact engineering/collection evaluator. Calling this does not create a diagnostic report.
pub fn evaluate(limits: &Limits) -> Result<Evaluation, Error> {
    let mut budget = Budget::new(limits)?;
    let mut rows = Vec::new();
    let mut means: Vec<MeanRow> = Vec::new();
    let mut request_contrasts = Vec::new();
    let mut contrasts = Vec::new();
    let mut envelopes = Vec::new();
    for stratum in registered_strata() {
        let (s, w, r, i, b) = (
            stratum.acquisition_cost,
            stratum.write_cost,
            stratum.note_read_cost,
            stratum.manifest_inspection_cost,
            stratum.writer_share,
        );
        let row_start = rows.len();
        let mean_start = means.len();
        for arm in ARMS {
            for routine in ROUTINES {
                for request in [Good::Left, Good::Right] {
                    let config = Config {
                        stratum: stratum.clone(),
                        arm,
                        routine,
                        request,
                        ids: Ids {
                            writer: "Agent-A".into(),
                            reader: "Agent-B".into(),
                        },
                    };
                    let episode = run_budgeted(&config, &mut |_| Ok(()), &mut budget)?;
                    budget.encoded(&episode, !rows.is_empty())?;
                    rows.push(episode);
                }
                budget.row("mean_rows", 522)?;
                budget.charge("metric_atom_operations", 60)?;
                let n = rows.len();
                let mean = MeanRow {
                    key: (s, w, r, i, b, arm, routine),
                    metrics: combine(&rows[n - 2].outcome, &rows[n - 1].outcome, true)?,
                };
                budget.encoded(&mean, !means.is_empty())?;
                means.push(mean);
            }
        }
        for (routine_index, routine) in ROUTINES.into_iter().enumerate() {
            for (request_index, request) in [Good::Left, Good::Right].into_iter().enumerate() {
                for (left, right, label) in CONTRASTS {
                    budget.row("request_contrast_rows", 523)?;
                    budget.charge("metric_atom_operations", 60)?;
                    let outcome = |arm: usize| {
                        &rows[row_start + arm * 8 + routine_index * 2 + request_index].outcome
                    };
                    let contrast = RequestContrast {
                        key: (s, w, r, i, b, routine, request, label),
                        metrics: combine(outcome(left), outcome(right), false)?,
                    };
                    budget.encoded(&contrast, !request_contrasts.is_empty())?;
                    request_contrasts.push(contrast);
                }
            }
            for (left, right, label) in CONTRASTS {
                budget.row("contrast_rows", 522)?;
                budget.charge("metric_atom_operations", 60)?;
                let metrics = |arm: usize| &means[mean_start + arm * 4 + routine_index].metrics;
                let contrast = MeanContrast {
                    key: (s, w, r, i, b, routine, label),
                    metrics: difference(metrics(left), metrics(right))?,
                };
                budget.encoded(&contrast, !contrasts.is_empty())?;
                contrasts.push(contrast);
            }
        }
        for (arm_index, arm) in ARMS.into_iter().enumerate() {
            budget.row("envelope_rows", 68)?;
            budget.charge("metric_atom_operations", 5)?;
            let candidates = &means[mean_start + arm_index * 4..mean_start + arm_index * 4 + 3];
            let mut best = &candidates[0].metrics.net[1];
            let mut ties = vec![ROUTINES[0]];
            for k in 1..3 {
                let candidate = &candidates[k].metrics.net[1];
                match compare(candidate, best)? {
                    std::cmp::Ordering::Greater => {
                        best = candidate;
                        ties.clear();
                        ties.push(ROUTINES[k]);
                    }
                    std::cmp::Ordering::Equal => ties.push(ROUTINES[k]),
                    std::cmp::Ordering::Less => {}
                }
            }
            let envelope = Envelope {
                key: (s, w, r, i, b, arm),
                reader_net: best.clone(),
                ties,
            };
            budget.encoded(&envelope, !envelopes.is_empty())?;
            envelopes.push(envelope);
        }
    }
    Ok(Evaluation {
        rows,
        means,
        request_contrasts,
        contrasts,
        envelopes,
        counters: budget.counters,
    })
}
