use super::{
    budget::Budget,
    canonical::{json_len, trusted_trace_hash},
    types::{required_option, unit_enum, wire_struct},
    *,
};
use serde::{Deserialize, Deserializer, Serialize};
use std::cmp::Reverse;

wire_struct!(Row {
    outcome: Outcome,
    trace_hash: String
});
wire_struct!(Classification {
    holder_regret: i16,
    retriever_regret: i16,
    is_pure: bool,
    team_gap: i16,
    outcome: Outcome
});
wire_struct!(Analysis {
    holder_best:Vec<i16>,retriever_best:Vec<i16>,pure:Vec<Profile>,team:Profile,
    #[serde(deserialize_with="required_option")] lex_pure:Option<Profile>,
    #[serde(deserialize_with="required_option")] best_group_pure:Option<Profile>,solo_best:i16
});
wire_struct!(GameTable {stratum:Stratum,rows:Vec<Row>,analysis:Analysis,stats:CompileStats});
unit_enum!(Selector {
    SoloAttempt,
    SuppliedHelp,
    TeamOptimal,
    NashLex,
    NashBestGroup
});
pub const SELECTORS: [Selector; 5] = [
    Selector::SoloAttempt,
    Selector::SuppliedHelp,
    Selector::TeamOptimal,
    Selector::NashLex,
    Selector::NashBestGroup,
];

pub(crate) struct ResponseSets {
    pub holder_best: Vec<i16>,
    pub retriever_best: Vec<i16>,
    pub pure: Vec<Profile>,
}
// Algebraic matrix entry point shares both scans with the streaming compiler.
#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn response_sets(
    h: usize,
    r: usize,
    payoffs: &[[i16; 2]],
) -> Result<ResponseSets, Error> {
    responses(
        h,
        r,
        payoffs.iter().copied(),
        payoffs.len(),
        &mut |_| Ok(()),
    )
}
fn responses(
    h: usize,
    r: usize,
    payoffs: impl Iterator<Item = [i16; 2]> + Clone,
    len: usize,
    retain: &mut impl FnMut(u64) -> Result<(), Error>,
) -> Result<ResponseSets, Error> {
    if h == 0 || h > 729 || r == 0 || r > 64 || h.checked_mul(r) != Some(len) {
        return Err(Error::InvalidInput(
            "response dimensions or payoff length".into(),
        ));
    }
    let mut holder_best = vec![i16::MIN; r];
    let mut retriever_best = vec![i16::MIN; h];
    for (i, net) in payoffs.clone().enumerate() {
        holder_best[i % r] = holder_best[i % r].max(net[0]);
        retriever_best[i / r] = retriever_best[i / r].max(net[1]);
    }
    let mut pure = Vec::new();
    for (i, net) in payoffs.enumerate() {
        if net[0] == holder_best[i % r] && net[1] == retriever_best[i / r] {
            retain(3)?;
            pure.try_reserve(1)
                .map_err(|e| Error::InvalidReport(format!("pure allocation: {e}")))?;
            pure.push(Profile::new((i / r) as u16, (i % r) as u8)?);
        }
    }
    Ok(ResponseSets {
        holder_best,
        retriever_best,
        pure,
    })
}
type Rank = (Reverse<i16>, u16, u8, u16, u8);
pub(crate) fn rank(o: &Outcome, p: Profile) -> Result<Rank, Error> {
    Ok((
        Reverse(o.group_net),
        u16::from(o.spent[0])
            .checked_add(u16::from(o.spent[1]))
            .ok_or(Error::ArithmeticOverflow)?,
        o.delivery_round.unwrap_or(7),
        p.holder,
        p.retriever,
    ))
}
fn profile(index: usize) -> Result<Profile, Error> {
    Profile::new((index / 64) as u16, (index % 64) as u8)
}
pub(super) fn row(table: &GameTable, p: Profile) -> Result<&Row, Error> {
    p.validate()?;
    table
        .rows
        .get(usize::from(p.holder) * 64 + usize::from(p.retriever))
        .ok_or_else(|| Error::InvalidReport("missing raw row".into()))
}
fn analyze(rows: &[Row], budget: &mut Budget) -> Result<Analysis, Error> {
    let responses = responses(
        729,
        64,
        rows.iter().map(|row| row.outcome.net),
        rows.len(),
        &mut |n| budget.retain(n),
    )?;
    let mut team = Profile::new(0, 0)?;
    let mut team_rank = rank(&rows[0].outcome, team)?;
    for (i, row) in rows.iter().enumerate().skip(1) {
        let p = profile(i)?;
        let candidate = rank(&row.outcome, p)?;
        if candidate < team_rank {
            team = p;
            team_rank = candidate;
        }
    }
    let mut best_group_pure = None;
    let mut best_rank = None;
    for &p in &responses.pure {
        let candidate = rank(
            &rows[usize::from(p.holder) * 64 + usize::from(p.retriever)].outcome,
            p,
        )?;
        if best_rank.is_none_or(|best| candidate < best) {
            best_rank = Some(candidate);
            best_group_pure = Some(p);
        }
    }
    let analysis = Analysis {
        holder_best: responses.holder_best,
        retriever_best: responses.retriever_best,
        lex_pure: responses.pure.first().copied(),
        pure: responses.pure,
        team,
        best_group_pure,
        solo_best: rows[..64]
            .iter()
            .map(|row| row.outcome.net[1])
            .max()
            .ok_or(Error::ArithmeticOverflow)?,
    };
    budget.retain(json_len(&analysis)?)?;
    Ok(analysis)
}
pub fn compile(s: &Stratum, limits: &Limits) -> Result<GameTable, Error> {
    s.validate()?;
    let mut budget = Budget::new(limits)?;
    let config = Config {
        stratum: s.clone(),
        ids: Ids::default(),
    };
    let mut rows = Vec::new();
    for h in 0..729 {
        for r in 0..64 {
            budget.begin_profile()?;
            let episode = super::kernel::run_budgeted(&config, Profile::new(h, r)?, &mut budget)?;
            let trace_hash = trusted_trace_hash(&episode)?;
            let row = Row {
                trace_hash,
                outcome: episode.outcome,
            };
            budget.retain(super::budget::add(json_len(&row)?, 4)?)?;
            rows.try_reserve(1)
                .map_err(|e| Error::InvalidReport(format!("row allocation: {e}")))?;
            rows.push(row);
            budget.release_episode();
        }
    }
    let analysis = analyze(&rows, &mut budget)?;
    Ok(GameTable {
        stratum: s.clone(),
        rows,
        analysis,
        stats: budget.stats,
    })
}
pub fn resolve(table: &GameTable, selector: Selector) -> Option<Profile> {
    match selector {
        Selector::SoloAttempt => Some(Profile {
            holder: 0,
            retriever: 63,
        }),
        Selector::SuppliedHelp => Some(if table.stratum.distance() == 0 {
            Profile {
                holder: 540,
                retriever: 60,
            }
        } else {
            Profile {
                holder: 423,
                retriever: 30,
            }
        }),
        Selector::TeamOptimal => Some(table.analysis.team),
        Selector::NashLex => table.analysis.lex_pure,
        Selector::NashBestGroup => table.analysis.best_group_pure,
    }
}
pub fn classify(table: &GameTable, p: Profile) -> Result<Classification, Error> {
    let outcome = row(table, p)?.outcome.clone();
    let holder_best = *table
        .analysis
        .holder_best
        .get(usize::from(p.retriever))
        .ok_or_else(|| Error::InvalidReport("missing Holder maximum".into()))?;
    let retriever_best = *table
        .analysis
        .retriever_best
        .get(usize::from(p.holder))
        .ok_or_else(|| Error::InvalidReport("missing Retriever maximum".into()))?;
    let holder_regret = holder_best
        .checked_sub(outcome.net[0])
        .ok_or(Error::ArithmeticOverflow)?;
    let retriever_regret = retriever_best
        .checked_sub(outcome.net[1])
        .ok_or(Error::ArithmeticOverflow)?;
    let team_gap = row(table, table.analysis.team)?
        .outcome
        .group_net
        .checked_sub(outcome.group_net)
        .ok_or(Error::ArithmeticOverflow)?;
    if holder_regret < 0 || retriever_regret < 0 || team_gap < 0 {
        return Err(Error::InvalidReport("negative regret or team gap".into()));
    }
    Ok(Classification {
        holder_regret,
        retriever_regret,
        is_pure: holder_regret == 0 && retriever_regret == 0,
        team_gap,
        outcome,
    })
}
