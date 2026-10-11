//! Complete pure-set ranges and matched, reselected one-axis comparisons.
use super::{
    game::{row, SELECTORS},
    types::{required_option, unit_enum, wire_struct},
    *,
};
use serde::{Deserialize, Deserializer, Serialize};
wire_struct!(Representative {
    profile: Profile,
    classification: Classification
});
wire_struct!(PayoffRanges {
    #[serde(deserialize_with="required_option")] holder:Option<[i16;2]>,
    #[serde(deserialize_with="required_option")] retriever:Option<[i16;2]>,
    #[serde(deserialize_with="required_option")] group:Option<[i16;2]>
});
wire_struct!(EquilibriumSummary {pure_count:u64,delivering_pure_count:u64,ranges:PayoffRanges,
    #[serde(deserialize_with="required_option")] range_unavailable:Option<String>,
    #[serde(deserialize_with="required_option")] lex:Option<Representative>,
    #[serde(deserialize_with="required_option")] best_group:Option<Representative>
});
wire_struct!(SummaryControl {selector:Selector,
    #[serde(deserialize_with="required_option")] profile:Option<Profile>,
    #[serde(deserialize_with="required_option")] classification:Option<Classification>,
    #[serde(deserialize_with="required_option")] unavailable:Option<String>
});
wire_struct!(SummaryStratum {stratum:Stratum,stats:CompileStats,equilibrium:EquilibriumSummary,team:Representative,solo_best:i16,controls:Vec<SummaryControl>});
wire_struct!(OutcomeDelta {spent:[i16;2],reward:[i16;2],net:[i16;2],group_net:i16,
    #[serde(deserialize_with="required_option")] delivery_round:Option<i16>,
    #[serde(deserialize_with="required_option")] delivery_unavailable:Option<String>,
    holder_attempts:[i16;3],holder_blocked:[i16;3],retriever_attempts:[i16;4],retriever_blocked:[i16;4],retriever_waits:i16
});
wire_struct!(ControlContrast {selector:Selector,
    #[serde(deserialize_with="required_option")] lower:Option<Representative>,
    #[serde(deserialize_with="required_option")] upper:Option<Representative>,
    #[serde(deserialize_with="required_option")] delta:Option<OutcomeDelta>,
    #[serde(deserialize_with="required_option")] unavailable:Option<String>
});
wire_struct!(PayoffRangeDelta {
    holder: [i16; 2],
    retriever: [i16; 2],
    group: [i16; 2]
});
wire_struct!(EquilibriumContrast {lower:EquilibriumSummary,upper:EquilibriumSummary,
    #[serde(deserialize_with="required_option")] range_delta:Option<PayoffRangeDelta>,
    #[serde(deserialize_with="required_option")] range_unavailable:Option<String>
});
unit_enum!(Axis {
    HoldingCost,
    HolderShare,
    HolderDistance
});
wire_struct!(Contrast {axis:Axis,lower:Stratum,upper:Stratum,controls:Vec<ControlContrast>,equilibria:EquilibriumContrast});

pub fn registered_strata() -> Vec<Stratum> {
    let mut strata = Vec::new();
    for k in [1, 2, 4] {
        for b in [0, 3, 6] {
            for d in [0, 1] {
                strata.push(Stratum::new(k, b, d).expect("registered stratum"));
            }
        }
    }
    strata
}
fn representative(table: &GameTable, p: Profile) -> Result<Representative, Error> {
    Ok(Representative {
        profile: p,
        classification: classify(table, p)?,
    })
}
pub fn equilibrium_summary(table: &GameTable) -> Result<EquilibriumSummary, Error> {
    let mut ranges: Option<[[i16; 2]; 3]> = None;
    let mut delivering = 0u64;
    for &p in &table.analysis.pure {
        let o = &row(table, p)?.outcome;
        let values = [o.net[0], o.net[1], o.group_net];
        let current = ranges.get_or_insert(values.map(|n| [n, n]));
        for i in 0..3 {
            current[i][0] = current[i][0].min(values[i]);
            current[i][1] = current[i][1].max(values[i]);
        }
        delivering = super::budget::add(delivering, u64::from(o.delivery_round.is_some()))?;
    }
    Ok(EquilibriumSummary {
        pure_count: table
            .analysis
            .pure
            .len()
            .try_into()
            .map_err(|_| Error::ArithmeticOverflow)?,
        delivering_pure_count: delivering,
        ranges: PayoffRanges {
            holder: ranges.map(|r| r[0]),
            retriever: ranges.map(|r| r[1]),
            group: ranges.map(|r| r[2]),
        },
        range_unavailable: ranges.is_none().then(|| "no_pure_equilibrium".into()),
        lex: table
            .analysis
            .lex_pure
            .map(|p| representative(table, p))
            .transpose()?,
        best_group: table
            .analysis
            .best_group_pure
            .map(|p| representative(table, p))
            .transpose()?,
    })
}
pub fn summarize_stratum(table: &GameTable) -> Result<SummaryStratum, Error> {
    let controls = SELECTORS
        .into_iter()
        .map(|selector| {
            let profile = resolve(table, selector);
            Ok(SummaryControl {
                selector,
                profile,
                classification: profile.map(|p| classify(table, p)).transpose()?,
                unavailable: profile.is_none().then(|| "no_pure_equilibrium".into()),
            })
        })
        .collect::<Result<_, Error>>()?;
    Ok(SummaryStratum {
        stratum: table.stratum.clone(),
        stats: table.stats.clone(),
        equilibrium: equilibrium_summary(table)?,
        team: representative(table, table.analysis.team)?,
        solo_best: table.analysis.solo_best,
        controls,
    })
}
fn difference<const N: usize>(lower: [i16; N], upper: [i16; N]) -> Result<[i16; N], Error> {
    let mut delta = [0; N];
    for i in 0..N {
        delta[i] = upper[i]
            .checked_sub(lower[i])
            .ok_or(Error::ArithmeticOverflow)?;
    }
    Ok(delta)
}
fn byte_difference<const N: usize>(lower: [u8; N], upper: [u8; N]) -> Result<[i16; N], Error> {
    difference(lower.map(i16::from), upper.map(i16::from))
}
pub fn outcome_delta(lower: &Outcome, upper: &Outcome) -> Result<OutcomeDelta, Error> {
    let delivery_round = match (lower.delivery_round, upper.delivery_round) {
        (Some(l), Some(u)) => Some(
            i16::from(u)
                .checked_sub(i16::from(l))
                .ok_or(Error::ArithmeticOverflow)?,
        ),
        _ => None,
    };
    Ok(OutcomeDelta {
        spent: byte_difference(lower.spent, upper.spent)?,
        reward: byte_difference(lower.reward, upper.reward)?,
        net: difference(lower.net, upper.net)?,
        group_net: upper
            .group_net
            .checked_sub(lower.group_net)
            .ok_or(Error::ArithmeticOverflow)?,
        delivery_round,
        delivery_unavailable: delivery_round
            .is_none()
            .then(|| "delivery_round_unavailable".into()),
        holder_attempts: byte_difference(lower.holder_attempts, upper.holder_attempts)?,
        holder_blocked: byte_difference(lower.holder_blocked, upper.holder_blocked)?,
        retriever_attempts: byte_difference(lower.retriever_attempts, upper.retriever_attempts)?,
        retriever_blocked: byte_difference(lower.retriever_blocked, upper.retriever_blocked)?,
        retriever_waits: i16::from(upper.retriever_waits)
            .checked_sub(i16::from(lower.retriever_waits))
            .ok_or(Error::ArithmeticOverflow)?,
    })
}
fn absent_reason(lower: bool, upper: bool, reasons: [&str; 3]) -> Option<String> {
    match (lower, upper) {
        (false, false) => Some(reasons[2].into()),
        (false, true) => Some(reasons[0].into()),
        (true, false) => Some(reasons[1].into()),
        (true, true) => None,
    }
}
fn equilibrium_contrast(
    lower: EquilibriumSummary,
    upper: EquilibriumSummary,
) -> Result<EquilibriumContrast, Error> {
    let range_unavailable = absent_reason(
        lower.pure_count > 0,
        upper.pure_count > 0,
        [
            "lower_pure_set_empty",
            "upper_pure_set_empty",
            "both_pure_sets_empty",
        ],
    );
    let range_delta = if range_unavailable.is_none() {
        let ranges = |r: &PayoffRanges| -> Result<[[i16; 2]; 3], Error> {
            Ok([
                r.holder
                    .ok_or_else(|| Error::InvalidReport("missing Holder range".into()))?,
                r.retriever
                    .ok_or_else(|| Error::InvalidReport("missing Retriever range".into()))?,
                r.group
                    .ok_or_else(|| Error::InvalidReport("missing group range".into()))?,
            ])
        };
        let l = ranges(&lower.ranges)?;
        let u = ranges(&upper.ranges)?;
        Some(PayoffRangeDelta {
            holder: difference(l[0], u[0])?,
            retriever: difference(l[1], u[1])?,
            group: difference(l[2], u[2])?,
        })
    } else {
        None
    };
    Ok(EquilibriumContrast {
        lower,
        upper,
        range_delta,
        range_unavailable,
    })
}
fn coordinates(s: &Stratum) -> [u8; 3] {
    [s.holding_cost(), s.holder_share(), s.distance()]
}
/// Input summaries must contain all 18 registered settings, in registered order.
pub fn contrasts(settings: &[SummaryStratum]) -> Result<Vec<Contrast>, Error> {
    if settings.len() != 18
        || settings
            .iter()
            .zip(registered_strata())
            .any(|(s, r)| s.stratum != r)
    {
        return Err(Error::InvalidReport(
            "contrast settings must be the complete registered order".into(),
        ));
    }
    let mut result = Vec::new();
    for (index, axis) in [Axis::HoldingCost, Axis::HolderShare, Axis::HolderDistance]
        .into_iter()
        .enumerate()
    {
        for lower in settings {
            for upper in settings {
                let l = coordinates(&lower.stratum);
                let u = coordinates(&upper.stratum);
                if l[index] >= u[index] || (0..3).any(|i| i != index && l[i] != u[i]) {
                    continue;
                }
                let mut controls = Vec::new();
                for (i, selector) in SELECTORS.into_iter().enumerate() {
                    let endpoint =
                        |setting: &SummaryStratum| -> Result<Option<Representative>, Error> {
                            let control = setting
                                .controls
                                .get(i)
                                .ok_or_else(|| Error::InvalidReport("missing control".into()))?;
                            if control.selector != selector {
                                return Err(Error::InvalidReport("control order".into()));
                            }
                            match (&control.profile, &control.classification) {
                                (Some(p), Some(c)) => Ok(Some(Representative {
                                    profile: *p,
                                    classification: c.clone(),
                                })),
                                (None, None) => Ok(None),
                                _ => Err(Error::InvalidReport(
                                    "control availability mismatch".into(),
                                )),
                            }
                        };
                    let l = endpoint(lower)?;
                    let u = endpoint(upper)?;
                    let delta = match (&l, &u) {
                        (Some(l), Some(u)) => Some(outcome_delta(
                            &l.classification.outcome,
                            &u.classification.outcome,
                        )?),
                        _ => None,
                    };
                    let unavailable = absent_reason(
                        l.is_some(),
                        u.is_some(),
                        [
                            "lower_control_unavailable",
                            "upper_control_unavailable",
                            "both_controls_unavailable",
                        ],
                    );
                    controls.push(ControlContrast {
                        selector,
                        lower: l,
                        upper: u,
                        delta,
                        unavailable,
                    });
                }
                result.push(Contrast {
                    axis,
                    lower: lower.stratum.clone(),
                    upper: upper.stratum.clone(),
                    controls,
                    equilibria: equilibrium_contrast(
                        lower.equilibrium.clone(),
                        upper.equilibrium.clone(),
                    )?,
                });
            }
        }
    }
    Ok(result)
}

/// Exact named cost/action/payoff measures; used for complete census range audits.
pub fn outcome_metrics(
    stratum: &Stratum,
    o: &Outcome,
) -> Result<std::collections::BTreeMap<String, i16>, Error> {
    stratum.validate()?;
    o.validate()?;
    let mut values = std::collections::BTreeMap::new();
    for (name, value) in [
        ("holder_spent", i16::from(o.spent[0])),
        ("retriever_spent", i16::from(o.spent[1])),
        ("holder_unused", 32 - i16::from(o.spent[0])),
        ("retriever_unused", 32 - i16::from(o.spent[1])),
        ("holder_net", o.net[0]),
        ("retriever_net", o.net[1]),
        ("group_net", o.group_net),
        ("holder_travel_credits", i16::from(o.holder_attempts[1])),
        (
            "holder_holding_credits",
            i16::from(stratum.holding_cost())
                .checked_mul(i16::from(o.holder_attempts[2]))
                .ok_or(Error::ArithmeticOverflow)?,
        ),
        ("retriever_waits", i16::from(o.retriever_waits)),
    ] {
        values.insert(name.into(), value);
    }
    for (role, attempts, blocked) in [
        ("holder", &o.holder_attempts[..], &o.holder_blocked[..]),
        (
            "retriever",
            &o.retriever_attempts[..],
            &o.retriever_blocked[..],
        ),
    ] {
        for i in 0..attempts.len() {
            values.insert(format!("{role}_attempts_{i}"), i16::from(attempts[i]));
            values.insert(format!("{role}_blocked_{i}"), i16::from(blocked[i]));
            values.insert(
                format!("{role}_successful_{i}"),
                i16::from(
                    attempts[i]
                        .checked_sub(blocked[i])
                        .ok_or(Error::ArithmeticOverflow)?,
                ),
            );
        }
    }
    Ok(values)
}
