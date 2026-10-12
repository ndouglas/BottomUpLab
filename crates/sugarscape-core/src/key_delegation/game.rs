use super::{
    budget::{self, Budget, FIXED},
    canonical::{canonical_bytes, trace_identity},
    *,
};
use serde::Serialize;
wire_object!(Row {pub outcome:Outcome,pub trace_hash:String});
wire_object!(Classification {pub giver_regret:i16,pub receiver_regret:i16,pub is_pure:bool,pub team_gap:i16,pub outcome:Outcome});
wire_object!(Analysis {pub giver_best:Vec<i16>,pub receiver_best:Vec<i16>,pub pure:Vec<Profile>,pub team:Profile,pub lex_pure:Option<Profile>,pub best_group_pure:Option<Profile>,pub solo_best:i16});
/// A table is constructed only by checked exhaustive compilation; external wire cannot forge provenance.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct GameTable {
    #[serde(skip)]
    compiled_stratum: Stratum,
    stratum: Stratum,
    rows: Vec<Row>,
    analysis: Analysis,
    stats: CompileStats,
}
impl GameTable {
    pub(super) fn into_parts(self) -> (Stratum, Vec<Row>, Analysis, CompileStats) {
        (self.stratum, self.rows, self.analysis, self.stats)
    }
    pub fn stratum(&self) -> &Stratum {
        &self.stratum
    }
    pub fn rows(&self) -> &[Row] {
        &self.rows
    }
    pub fn analysis(&self) -> &Analysis {
        &self.analysis
    }
    pub fn stats(&self) -> &CompileStats {
        &self.stats
    }
}
pub fn strata() -> Vec<Stratum> {
    let mut v = Vec::new();
    for e in [false, true] {
        for t in [1, 2, 4] {
            for b in [0, 3, 6] {
                for d in [0, 1] {
                    v.push(Stratum::new(e, t, b, d).expect("registered stratum"));
                }
            }
        }
    }
    v
}
pub fn compile(stratum: Stratum, limits: &Limits) -> Result<GameTable, Error> {
    compile_with_witness(stratum, limits).map(|(g, _)| g)
}
pub fn compile_with_witness(
    stratum: Stratum,
    limits: &Limits,
) -> Result<(GameTable, PeakWitness), Error> {
    stratum.validate()?;
    let config = Config::new(stratum.clone(), Ids::default(), InitialOwner::Giver)?;
    let mut b = Budget::new(limits)?;
    let mut rows = Vec::new();
    for g in 0..729 {
        for r in 0..64 {
            b.profile()?;
            let profile = Profile::new(g, r)?;
            let mut final_scratch = 0;
            let episode = super::kernel::run_budgeted(&config, profile, &mut |e, events| {
                let scratch = b.boundary(e.json_len()?, events)?;
                final_scratch = scratch;
                b.witness(PeakWitness {
                    phase: match e.clock.phase {
                        Phase::Initial => "initial",
                        Phase::BeforeAction => "before_action",
                        Phase::AfterAction => "after_action",
                        Phase::Finished => "finished",
                    }
                    .into(),
                    profile: Some(profile),
                    clock: Some(e.clock),
                    retained_rows: rows.len() as u64,
                    pure_profiles: 0,
                    scratch_bytes: scratch,
                    live_bytes: budget::add(budget::add(FIXED, b.retained)?, scratch)?,
                });
                Ok(())
            })?;
            // Prospective Row borrows Outcome and uses the fixed-width future digest, before hash/row allocation.
            #[derive(Serialize)]
            struct PendingRow<'a> {
                outcome: &'a Outcome,
                trace_hash: &'a str,
            }
            let n = budget::json_len(&PendingRow {
                outcome: &episode.outcome,
                trace_hash: "0000000000000000000000000000000000000000000000000000000000000000",
            })?;
            b.row(n, final_scratch)?;
            b.witness(PeakWitness {
                phase: "row_append".into(),
                profile: Some(profile),
                clock: Some(Clock {
                    round: episode
                        .events
                        .last()
                        .ok_or_else(|| Error::InvalidReport("missing terminal".into()))?
                        .clock
                        .round,
                    phase: Phase::Finished,
                    role: None,
                }),
                retained_rows: rows.len() as u64 + 1,
                pure_profiles: 0,
                scratch_bytes: final_scratch,
                live_bytes: budget::add(budget::add(FIXED, b.retained)?, final_scratch)?,
            });
            // This is a native kernel result: no caller-supplied Episode can enter this trusted path.
            let hash = trace_identity(&canonical_bytes(&episode)?)?;
            rows.push(Row {
                outcome: episode.outcome,
                trace_hash: hash,
            });
        }
    }
    let analysis = analyze(&rows, 729, 64, Some(&mut b))?;
    let stats = b.stats.clone();
    let peak = b
        .peak
        .ok_or_else(|| Error::InvalidReport("missing peak".into()))?;
    Ok((
        GameTable {
            compiled_stratum: stratum.clone(),
            stratum,
            rows,
            analysis,
            stats,
        },
        peak,
    ))
}
fn rank(row: &Row, g: usize, r: usize) -> (i16, u16, u8, usize, usize) {
    (
        -row.outcome.group_net,
        u16::from(row.outcome.spent[0]) + u16::from(row.outcome.spent[1]),
        row.outcome.delivery_round.unwrap_or(7),
        g,
        r,
    )
}
fn analyze(
    rows: &[Row],
    ng: usize,
    nr: usize,
    mut budget: Option<&mut Budget<'_>>,
) -> Result<Analysis, Error> {
    if rows.len() != ng * nr || ng == 0 || nr == 0 {
        return Err(Error::InvalidReport("payoff matrix dimensions".into()));
    }
    // Fixed response storage has already been charged by Budget::new before allocation.
    let mut giver_best = vec![i16::MIN; nr];
    let mut receiver_best = vec![i16::MIN; ng];
    for (r, best) in giver_best.iter_mut().enumerate() {
        for g in 0..ng {
            *best = (*best).max(rows[g * nr + r].outcome.net[0]);
        }
    }
    for (g, best) in receiver_best.iter_mut().enumerate() {
        for r in 0..nr {
            *best = (*best).max(rows[g * nr + r].outcome.net[1]);
        }
    }
    let team_index = (0..rows.len())
        .min_by_key(|i| rank(&rows[*i], i / nr, i % nr))
        .ok_or_else(|| Error::InvalidReport("empty matrix".into()))?;
    let mut pure = Vec::new();
    let mut best_pure = None;
    for g in 0..ng {
        for r in 0..nr {
            let row = &rows[g * nr + r];
            if row.outcome.net == [giver_best[r], receiver_best[g]] {
                if let Some(b) = budget.as_mut() {
                    b.bytes(budget::add(
                        budget::add(FIXED, b.retained)?,
                        budget::mul(pure.len() as u64 + 1, 3)?,
                    )?)?;
                }
                pure.push(Profile::new(g as u16, r as u8)?);
                if best_pure.is_none_or(|i| rank(row, g, r) < rank(&rows[i], i / nr, i % nr)) {
                    best_pure = Some(g * nr + r);
                }
            }
        }
    }
    let analysis = Analysis {
        giver_best,
        receiver_best,
        pure,
        team: Profile::new((team_index / nr) as u16, (team_index % nr) as u8)?,
        lex_pure: None,
        best_group_pure: best_pure
            .map(|i| Profile::new((i / nr) as u16, (i % nr) as u8))
            .transpose()?,
        solo_best: rows[..nr]
            .iter()
            .map(|r| r.outcome.net[1])
            .max()
            .ok_or_else(|| Error::InvalidReport("solo rows".into()))?,
    };
    let mut analysis = analysis;
    analysis.lex_pure = analysis.pure.first().cloned();
    if let Some(b) = budget.as_mut() {
        let n = budget::add(
            budget::add(
                budget::add(FIXED, b.retained)?,
                budget::mul(analysis.pure.len() as u64, 3)?,
            )?,
            budget::json_len(&analysis)?,
        )?;
        b.bytes(n)?;
        b.witness(PeakWitness {
            phase: "analysis".into(),
            profile: None,
            clock: None,
            retained_rows: rows.len() as u64,
            pure_profiles: analysis.pure.len() as u64,
            scratch_bytes: 0,
            live_bytes: n,
        });
    }
    Ok(analysis)
}
pub fn classify(table: &GameTable, profile: Profile) -> Result<Classification, Error> {
    profile.validate()?;
    table.stratum.validate()?;
    if table.stratum != table.compiled_stratum {
        return Err(Error::InvalidReport("table stratum binding".into()));
    }
    if table.rows.len() != 729 * 64
        || table.analysis.giver_best.len() != 64
        || table.analysis.receiver_best.len() != 729
    {
        return Err(Error::InvalidReport("wrong game dimensions".into()));
    }
    let index = usize::from(profile.giver()) * 64 + usize::from(profile.receiver());
    let o = &table.rows[index].outcome;
    let giver_regret = table.analysis.giver_best[usize::from(profile.receiver())]
        .checked_sub(o.net[0])
        .ok_or(Error::ArithmeticOverflow)?;
    let receiver_regret = table.analysis.receiver_best[usize::from(profile.giver())]
        .checked_sub(o.net[1])
        .ok_or(Error::ArithmeticOverflow)?;
    let team = &table.analysis.team;
    let best = table.rows[usize::from(team.giver()) * 64 + usize::from(team.receiver())]
        .outcome
        .group_net;
    let team_gap = best
        .checked_sub(o.group_net)
        .ok_or(Error::ArithmeticOverflow)?;
    if giver_regret < 0 || receiver_regret < 0 || team_gap < 0 {
        return Err(Error::InvalidReport("negative regret or team gap".into()));
    }
    Ok(Classification {
        giver_regret,
        receiver_regret,
        is_pure: giver_regret == 0 && receiver_regret == 0,
        team_gap,
        outcome: o.clone(),
    })
}
#[cfg(test)]
pub(super) struct MatrixResult {
    pub pure: Vec<(u16, u8)>,
    pub giver_best: Vec<i16>,
    pub receiver_best: Vec<i16>,
}
#[cfg(test)]
pub(super) fn test_matrix<const G: usize, const R: usize>(
    payoffs: &[[[i16; 2]; R]; G],
) -> MatrixResult {
    let base = run_episode(
        &Config::new(
            Stratum::new(false, 1, 0, 0).unwrap(),
            Ids::default(),
            InitialOwner::Giver,
        )
        .unwrap(),
        Profile::new(0, 0).unwrap(),
    )
    .unwrap()
    .outcome;
    let rows: Vec<_> = payoffs
        .iter()
        .flatten()
        .map(|p| {
            let mut o = base.clone();
            o.net = *p;
            o.group_net = p[0] + p[1];
            Row {
                outcome: o,
                trace_hash: String::new(),
            }
        })
        .collect();
    let a = analyze(&rows, G, R, None).unwrap();
    MatrixResult {
        pure: a.pure.iter().map(|p| (p.giver(), p.receiver())).collect(),
        giver_best: a.giver_best,
        receiver_best: a.receiver_best,
    }
}
#[cfg(test)]
pub(super) fn test_invalid_table(table: &GameTable) {
    let mut wrong = table.clone();
    wrong.stratum = Stratum::new(
        !table.stratum.transfer_enabled(),
        table.stratum.transfer_cost(),
        table.stratum.giver_share(),
        table.stratum.distance(),
    )
    .unwrap();
    assert!(classify(&wrong, Profile::new(0, 0).unwrap()).is_err());
    let mut short = table.clone();
    short.rows.pop();
    assert!(classify(&short, Profile::new(0, 0).unwrap()).is_err());
}

#[cfg(test)]
mod invariant_tests {
    use super::*;
    #[test]
    fn team_order_uses_group_spending_delivery_then_raw_ids() {
        let o = run_episode(
            &Config::new(
                Stratum::new(true, 1, 3, 0).unwrap(),
                Ids::default(),
                InitialOwner::Giver,
            )
            .unwrap(),
            Profile::new(0, 0).unwrap(),
        )
        .unwrap()
        .outcome;
        let mut rows = vec![
            Row {
                outcome: o.clone(),
                trace_hash: String::new()
            };
            4
        ];
        rows[0].outcome.net = [1, 0];
        rows[0].outcome.group_net = 1;
        rows[1].outcome.net = [0, 1];
        rows[1].outcome.group_net = 1;
        rows[1].outcome.spent = [1, 0];
        rows[2].outcome.net = [1, 0];
        rows[2].outcome.group_net = 1;
        rows[2].outcome.delivery_round = Some(6);
        rows[3].outcome.net = [1, 0];
        rows[3].outcome.group_net = 1;
        rows[3].outcome.delivery_round = Some(4);
        let a = analyze(&rows, 2, 2, None).unwrap();
        assert_eq!(a.team, Profile::new(1, 1).unwrap());
        assert_eq!(a.lex_pure, Some(Profile::new(1, 0).unwrap()));
        assert_eq!(
            a.pure,
            vec![Profile::new(1, 0).unwrap(), Profile::new(1, 1).unwrap()]
        );
        assert_eq!(a.best_group_pure, Some(Profile::new(1, 1).unwrap()));
    }
}
