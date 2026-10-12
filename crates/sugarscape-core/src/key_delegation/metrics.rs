use super::*;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Selector {
    SoloAttempt,
    SuppliedHandoff,
    TeamOptimal,
    NashLex,
    NashBestGroup,
    InitiallyEquipped,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ClassificationScope {
    #[serde(rename = "main")]
    Main,
    #[serde(rename = "changed_initial_control")]
    ChangedInitialControl,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ContrastAxis {
    TransferPermission,
    TransferCost,
    GiverShare,
    Distance,
}
wire_object!(Selection {pub initial_owner:InitialOwner,pub profile:Profile,pub classification:Option<Classification>});
wire_object!(ControlRecord {pub selector:Selector,pub classification_scope:ClassificationScope,pub selection:Option<Selection>,pub episode:Option<Episode>,pub unavailable:Option<String>});
wire_object!(Representative {pub profile:Profile,pub classification:Classification});
wire_object!(PayoffRanges {pub giver:Option<[i16;2]>,pub receiver:Option<[i16;2]>,pub group:Option<[i16;2]>});
wire_object!(EquilibriumSummary {pub pure_count:u64,pub delivering_pure_count:u64,pub ranges:PayoffRanges,pub range_unavailable:Option<String>,pub delivery_round_range:Option<[u8;2]>,pub delivery_unavailable:Option<String>,pub team_gap_range:Option<[i16;2]>,pub lex:Option<Representative>,pub best_group:Option<Representative>});
wire_object!(SummaryControl {pub selector:Selector,pub classification_scope:ClassificationScope,pub selection:Option<Selection>,pub outcome:Option<Outcome>,pub unavailable:Option<String>});
wire_object!(SummaryStratum {pub stratum:Stratum,pub stats:CompileStats,pub equilibrium:EquilibriumSummary,pub team:Representative,pub solo_best:i16,pub controls:Vec<SummaryControl>});
wire_object!(OutcomeDelta {pub spent:[i16;2],pub reward:[i16;2],pub net:[i16;2],pub group_net:i16,pub delivery_round:Option<i16>,pub delivery_unavailable:Option<String>,pub giver_attempts:[i16;3],pub giver_blocked:[i16;3],pub receiver_attempts:[i16;4],pub receiver_blocked:[i16;4],pub receiver_waits:i16,pub transfer_count:i16,pub approach_spent:i16,pub pass_spent:i16,pub unused_credits:[i16;2]});
wire_object!(ClassificationDelta {pub giver_regret:i16,pub receiver_regret:i16,pub team_gap:i16});
wire_object!(FixedWordDelta {pub profile:Profile,pub delta:OutcomeDelta});
wire_object!(FixedWordSummary {pub profiles:u64,pub delta_hash:String,pub physical_equal_count:u64,pub spending_equal_count:u64,pub group_net_equal_count:u64,pub delivery_equal_count:u64,pub share_rule_count:Option<u64>});
wire_object!(ControlContrast {pub selector:Selector,pub lower:SummaryControl,pub upper:SummaryControl,pub delta:Option<OutcomeDelta>,pub classification_delta:Option<ClassificationDelta>,pub classification_unavailable:Option<String>,pub unavailable:Option<String>});
wire_object!(RangeDelta {pub giver:[i16;2],pub receiver:[i16;2],pub group:[i16;2],pub team_gap:[i16;2]});
wire_object!(EquilibriumContrast {pub lower:EquilibriumSummary,pub upper:EquilibriumSummary,pub pure_count:i64,pub delivering_pure_count:i64,pub range_delta:Option<RangeDelta>,pub range_unavailable:Option<String>,pub delivery_round_range_delta:Option<[i16;2]>,pub delivery_unavailable:Option<String>});
wire_object!(Contrast {pub axis:ContrastAxis,pub lower:Stratum,pub upper:Stratum,pub fixed_words:FixedWordSummary,pub controls:Vec<ControlContrast>,pub equilibrium:EquilibriumContrast});
pub const SELECTORS: [Selector; 6] = [
    Selector::SoloAttempt,
    Selector::SuppliedHandoff,
    Selector::TeamOptimal,
    Selector::NashLex,
    Selector::NashBestGroup,
    Selector::InitiallyEquipped,
];
pub fn resolve(table: &GameTable, selector: Selector) -> Result<Option<Selection>, Error> {
    let profile = match selector {
        Selector::SoloAttempt => Some(Profile::new(0, 63)?),
        Selector::SuppliedHandoff => Some(if table.stratum().distance() == 0 {
            Profile::new(486, 60)?
        } else {
            Profile::new(405, 30)?
        }),
        Selector::TeamOptimal => Some(table.analysis().team),
        Selector::NashLex => table.analysis().lex_pure,
        Selector::NashBestGroup => table.analysis().best_group_pure,
        Selector::InitiallyEquipped => {
            return Ok(Some(Selection {
                initial_owner: InitialOwner::Receiver,
                profile: Profile::new(0, 60)?,
                classification: None,
            }))
        }
    };
    profile
        .map(|p| {
            Ok(Selection {
                initial_owner: InitialOwner::Giver,
                classification: Some(classify(table, p)?),
                profile: p,
            })
        })
        .transpose()
}
pub fn controls(table: &GameTable) -> Result<Vec<ControlRecord>, Error> {
    SELECTORS
        .into_iter()
        .map(|selector| {
            let selection = resolve(table, selector)?;
            let episode = selection
                .as_ref()
                .map(|s| {
                    run_episode(
                        &Config::new(table.stratum().clone(), Ids::default(), s.initial_owner)?,
                        s.profile,
                    )
                })
                .transpose()?;
            Ok(ControlRecord {
                selector,
                classification_scope: if selector == Selector::InitiallyEquipped {
                    ClassificationScope::ChangedInitialControl
                } else {
                    ClassificationScope::Main
                },
                unavailable: selection.is_none().then(|| "no_pure_equilibrium".into()),
                selection,
                episode,
            })
        })
        .collect()
}
impl ControlRecord {
    pub fn summary(&self) -> SummaryControl {
        SummaryControl {
            selector: self.selector,
            classification_scope: self.classification_scope,
            selection: self.selection.clone(),
            outcome: self.episode.as_ref().map(|e| e.outcome.clone()),
            unavailable: self.unavailable.clone(),
        }
    }
}
fn range<T: Copy + Ord>(it: impl Iterator<Item = T>) -> Option<[T; 2]> {
    let mut it = it;
    let first = it.next()?;
    Some(it.fold([first, first], |[a, b], x| [a.min(x), b.max(x)]))
}
fn representative(table: &GameTable, p: Profile) -> Result<Representative, Error> {
    Ok(Representative {
        classification: classify(table, p)?,
        profile: p,
    })
}
pub fn equilibrium(table: &GameTable) -> Result<EquilibriumSummary, Error> {
    let classes = table
        .analysis()
        .pure
        .iter()
        .map(|p| classify(table, *p))
        .collect::<Result<Vec<_>, _>>()?;
    let delivery = range(classes.iter().filter_map(|c| c.outcome.delivery_round));
    Ok(EquilibriumSummary {
        pure_count: classes.len() as u64,
        delivering_pure_count: classes
            .iter()
            .filter(|c| c.outcome.delivery_round.is_some())
            .count() as u64,
        ranges: PayoffRanges {
            giver: range(classes.iter().map(|c| c.outcome.net[0])),
            receiver: range(classes.iter().map(|c| c.outcome.net[1])),
            group: range(classes.iter().map(|c| c.outcome.group_net)),
        },
        range_unavailable: classes.is_empty().then(|| "no_pure_equilibrium".into()),
        delivery_round_range: delivery,
        delivery_unavailable: delivery
            .is_none()
            .then(|| "no_delivering_pure_equilibrium".into()),
        team_gap_range: range(classes.iter().map(|c| c.team_gap)),
        lex: table
            .analysis()
            .lex_pure
            .map(|p| representative(table, p))
            .transpose()?,
        best_group: table
            .analysis()
            .best_group_pure
            .map(|p| representative(table, p))
            .transpose()?,
    })
}
pub fn summarize(table: &GameTable) -> Result<SummaryStratum, Error> {
    Ok(SummaryStratum {
        stratum: table.stratum().clone(),
        stats: table.stats().clone(),
        equilibrium: equilibrium(table)?,
        team: representative(table, table.analysis().team)?,
        solo_best: table.analysis().solo_best,
        controls: controls(table)?
            .iter()
            .map(ControlRecord::summary)
            .collect(),
    })
}
fn sub(a: i16, b: i16) -> Result<i16, Error> {
    a.checked_sub(b).ok_or(Error::ArithmeticOverflow)
}
fn delta_u8<const N: usize>(a: [u8; N], b: [u8; N]) -> [i16; N] {
    std::array::from_fn(|i| i16::from(b[i]) - i16::from(a[i]))
}
fn delta_i16<const N: usize>(a: [i16; N], b: [i16; N]) -> Result<[i16; N], Error> {
    let mut result = [0; N];
    for i in 0..N {
        result[i] = sub(b[i], a[i])?;
    }
    Ok(result)
}
pub fn outcome_delta(lower: &Outcome, upper: &Outcome) -> Result<OutcomeDelta, Error> {
    lower.validate()?;
    upper.validate()?;
    Ok(OutcomeDelta {
        spent: delta_u8(lower.spent, upper.spent),
        reward: delta_u8(lower.reward, upper.reward),
        net: delta_i16(lower.net, upper.net)?,
        group_net: sub(upper.group_net, lower.group_net)?,
        delivery_round: lower
            .delivery_round
            .zip(upper.delivery_round)
            .map(|(a, b)| i16::from(b) - i16::from(a)),
        delivery_unavailable: (lower.delivery_round.is_none() || upper.delivery_round.is_none())
            .then(|| "delivery_round_unavailable".into()),
        giver_attempts: delta_u8(lower.giver_attempts, upper.giver_attempts),
        giver_blocked: delta_u8(lower.giver_blocked, upper.giver_blocked),
        receiver_attempts: delta_u8(lower.receiver_attempts, upper.receiver_attempts),
        receiver_blocked: delta_u8(lower.receiver_blocked, upper.receiver_blocked),
        receiver_waits: i16::from(upper.receiver_waits) - i16::from(lower.receiver_waits),
        transfer_count: i16::from(upper.transfer_count) - i16::from(lower.transfer_count),
        approach_spent: i16::from(upper.approach_spent) - i16::from(lower.approach_spent),
        pass_spent: i16::from(upper.pass_spent) - i16::from(lower.pass_spent),
        unused_credits: delta_u8(lower.unused_credits, upper.unused_credits),
    })
}
pub fn contrast_pairs() -> Vec<(ContrastAxis, Stratum, Stratum)> {
    let strata = strata();
    let mut out = Vec::new();
    for axis in [
        ContrastAxis::TransferPermission,
        ContrastAxis::TransferCost,
        ContrastAxis::GiverShare,
        ContrastAxis::Distance,
    ] {
        for a in &strata {
            for b in &strata {
                let same_e = a.transfer_enabled() == b.transfer_enabled();
                let same_t = a.transfer_cost() == b.transfer_cost();
                let same_b = a.giver_share() == b.giver_share();
                let same_d = a.distance() == b.distance();
                let matched = match axis {
                    ContrastAxis::TransferPermission => {
                        !a.transfer_enabled() && b.transfer_enabled() && same_t && same_b && same_d
                    }
                    ContrastAxis::TransferCost => {
                        same_e
                            && same_b
                            && same_d
                            && matches!((a.transfer_cost(), b.transfer_cost()), (1, 2) | (2, 4))
                    }
                    ContrastAxis::GiverShare => {
                        same_e
                            && same_t
                            && same_d
                            && matches!((a.giver_share(), b.giver_share()), (0, 3) | (3, 6))
                    }
                    ContrastAxis::Distance => {
                        same_e && same_t && same_b && a.distance() == 0 && b.distance() == 1
                    }
                };
                if matched {
                    out.push((axis, a.clone(), b.clone()));
                }
            }
        }
    }
    out
}
pub(super) fn missing(lower: bool, upper: bool, kind: &str) -> Option<String> {
    match (lower, upper) {
        (true, true) => None,
        (false, true) => Some(format!("lower_{kind}")),
        (true, false) => Some(format!("upper_{kind}")),
        (false, false) => Some(
            match kind {
                "control_unavailable" => "both_controls_unavailable",
                "pure_set_empty" => "both_pure_sets_empty",
                "delivering_pure_set_empty" => "both_delivering_pure_sets_empty",
                _ => unreachable!("registered absence kind"),
            }
            .into(),
        ),
    }
}
fn control_contrast(
    lower: SummaryControl,
    upper: SummaryControl,
) -> Result<ControlContrast, Error> {
    if lower.selector != upper.selector {
        return Err(Error::InvalidReport("contrast selector mismatch".into()));
    }
    let unavailable = missing(
        lower.outcome.is_some(),
        upper.outcome.is_some(),
        "control_unavailable",
    );
    let delta = lower
        .outcome
        .as_ref()
        .zip(upper.outcome.as_ref())
        .map(|(a, b)| outcome_delta(a, b))
        .transpose()?;
    let classifications = lower
        .selection
        .as_ref()
        .and_then(|s| s.classification.as_ref())
        .zip(
            upper
                .selection
                .as_ref()
                .and_then(|s| s.classification.as_ref()),
        );
    let classification_delta = classifications
        .map(|(a, b)| {
            Ok::<_, Error>(ClassificationDelta {
                giver_regret: sub(b.giver_regret, a.giver_regret)?,
                receiver_regret: sub(b.receiver_regret, a.receiver_regret)?,
                team_gap: sub(b.team_gap, a.team_gap)?,
            })
        })
        .transpose()?;
    let classification_unavailable =
        if lower.classification_scope == ClassificationScope::ChangedInitialControl {
            Some("changed_initial_control".into())
        } else {
            unavailable.clone()
        };
    Ok(ControlContrast {
        selector: lower.selector,
        lower,
        upper,
        delta,
        classification_delta,
        classification_unavailable,
        unavailable,
    })
}
fn equilibrium_contrast(
    lower: EquilibriumSummary,
    upper: EquilibriumSummary,
) -> Result<EquilibriumContrast, Error> {
    let range_unavailable = missing(lower.pure_count > 0, upper.pure_count > 0, "pure_set_empty");
    let get = |v: Option<[i16; 2]>| {
        v.ok_or_else(|| Error::InvalidReport("missing nonempty pure range".into()))
    };
    let range_delta = if range_unavailable.is_none() {
        Some(RangeDelta {
            giver: delta_i16(get(lower.ranges.giver)?, get(upper.ranges.giver)?)?,
            receiver: delta_i16(get(lower.ranges.receiver)?, get(upper.ranges.receiver)?)?,
            group: delta_i16(get(lower.ranges.group)?, get(upper.ranges.group)?)?,
            team_gap: delta_i16(get(lower.team_gap_range)?, get(upper.team_gap_range)?)?,
        })
    } else {
        None
    };
    let delivery_unavailable = missing(
        lower.delivering_pure_count > 0,
        upper.delivering_pure_count > 0,
        "delivering_pure_set_empty",
    );
    let delivery_round_range_delta = lower
        .delivery_round_range
        .zip(upper.delivery_round_range)
        .map(|(a, b)| delta_u8(a, b));
    Ok(EquilibriumContrast {
        pure_count: i64::try_from(upper.pure_count)
            .map_err(|_| Error::ArithmeticOverflow)?
            .checked_sub(i64::try_from(lower.pure_count).map_err(|_| Error::ArithmeticOverflow)?)
            .ok_or(Error::ArithmeticOverflow)?,
        delivering_pure_count: i64::try_from(upper.delivering_pure_count)
            .map_err(|_| Error::ArithmeticOverflow)?
            .checked_sub(
                i64::try_from(lower.delivering_pure_count)
                    .map_err(|_| Error::ArithmeticOverflow)?,
            )
            .ok_or(Error::ArithmeticOverflow)?,
        lower,
        upper,
        range_delta,
        range_unavailable,
        delivery_round_range_delta,
        delivery_unavailable,
    })
}
/// Exact registered projection; finance and supplied Config/words are excluded only here.
pub(super) fn physical_projection(e: &Episode) -> Result<serde_json::Value, Error> {
    let mut events =
        serde_json::to_value(&e.events).map_err(|e| Error::InvalidReport(e.to_string()))?;
    for event in events
        .as_array_mut()
        .ok_or_else(|| Error::InvalidReport("event array".into()))?
    {
        for side in ["before", "after"] {
            if let Some(s) = event[side].as_object_mut() {
                s.remove("spent");
                s.remove("reward");
            }
        }
        if let Some(action) = event["kind"]
            .get_mut("Action")
            .and_then(serde_json::Value::as_object_mut)
        {
            action.remove("paid");
        }
    }
    Ok(events)
}
pub fn contrast(
    lower: &GameTable,
    upper: &GameTable,
    axis: ContrastAxis,
) -> Result<Contrast, Error> {
    contrast_with_deltas(lower, upper, axis).map(|(c, _)| c)
}
pub fn contrast_with_deltas(
    lower: &GameTable,
    upper: &GameTable,
    axis: ContrastAxis,
) -> Result<(Contrast, Vec<FixedWordDelta>), Error> {
    if !contrast_pairs()
        .iter()
        .any(|(a, l, u)| *a == axis && l == lower.stratum() && u == upper.stratum())
    {
        return Err(Error::InvalidReport(
            "unregistered contrast endpoints".into(),
        ));
    }
    let mut deltas = Vec::with_capacity(46656);
    let mut physical_equal_count = 0;
    let mut spending_equal_count = 0;
    let mut group_net_equal_count = 0;
    let mut delivery_equal_count = 0;
    let mut share_count = 0;
    let lc = Config::new(lower.stratum().clone(), Ids::default(), InitialOwner::Giver)?;
    let uc = Config::new(upper.stratum().clone(), Ids::default(), InitialOwner::Giver)?;
    for (i, (l, u)) in lower.rows().iter().zip(upper.rows()).enumerate() {
        let profile = Profile::new((i / 64) as u16, (i % 64) as u8)?;
        let a = &l.outcome;
        let b = &u.outcome;
        let physical = physical_projection(&run_episode(&lc, profile)?)?
            == physical_projection(&run_episode(&uc, profile)?)?;
        physical_equal_count += u64::from(physical);
        spending_equal_count += u64::from(a.spent == b.spent);
        group_net_equal_count += u64::from(a.group_net == b.group_net);
        delivery_equal_count += u64::from(a.delivery_round == b.delivery_round);
        let delta = outcome_delta(a, b)?;
        if axis == ContrastAxis::GiverShare {
            let d =
                i16::from(upper.stratum().giver_share()) - i16::from(lower.stratum().giver_share());
            let expected = if a.delivery_round.is_some() {
                [d, -d]
            } else {
                [0, 0]
            };
            let same = physical
                && a.spent == b.spent
                && a.giver_attempts == b.giver_attempts
                && a.giver_blocked == b.giver_blocked
                && a.receiver_attempts == b.receiver_attempts
                && a.receiver_blocked == b.receiver_blocked
                && a.receiver_waits == b.receiver_waits
                && a.transfer_count == b.transfer_count
                && a.delivery_round == b.delivery_round
                && a.group_net == b.group_net
                && delta.net == expected
                && delta.reward == expected;
            share_count += u64::from(same);
        }
        deltas.push(FixedWordDelta { profile, delta });
    }
    #[derive(Serialize)]
    struct Body<'a> {
        axis: ContrastAxis,
        lower: &'a Stratum,
        upper: &'a Stratum,
        rows: &'a [FixedWordDelta],
    }
    let body = super::canonical::canonical_bytes(&Body {
        axis,
        lower: lower.stratum(),
        upper: upper.stratum(),
        rows: &deltas,
    })?;
    let delta_hash =
        super::canonical::receipt_identity("key-delegation-fixed-word-deltas-v1", &body)?;
    let controls = controls(lower)?
        .iter()
        .map(ControlRecord::summary)
        .zip(controls(upper)?.iter().map(ControlRecord::summary))
        .map(|(l, u)| control_contrast(l, u))
        .collect::<Result<_, _>>()?;
    Ok((
        Contrast {
            axis,
            lower: lower.stratum().clone(),
            upper: upper.stratum().clone(),
            fixed_words: FixedWordSummary {
                profiles: deltas.len() as u64,
                delta_hash,
                physical_equal_count,
                spending_equal_count,
                group_net_equal_count,
                delivery_equal_count,
                share_rule_count: (axis == ContrastAxis::GiverShare).then_some(share_count),
            },
            controls,
            equilibrium: equilibrium_contrast(equilibrium(lower)?, equilibrium(upper)?)?,
        },
        deltas,
    ))
}

impl<'de> Deserialize<'de> for Selector {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "SoloAttempt" => Ok(Self::SoloAttempt),
            "SuppliedHandoff" => Ok(Self::SuppliedHandoff),
            "TeamOptimal" => Ok(Self::TeamOptimal),
            "NashLex" => Ok(Self::NashLex),
            "NashBestGroup" => Ok(Self::NashBestGroup),
            "InitiallyEquipped" => Ok(Self::InitiallyEquipped),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}

impl<'de> Deserialize<'de> for ClassificationScope {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "main" => Ok(Self::Main),
            "changed_initial_control" => Ok(Self::ChangedInitialControl),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}

impl<'de> Deserialize<'de> for ContrastAxis {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match String::deserialize(d)?.as_str() {
            "TransferPermission" => Ok(Self::TransferPermission),
            "TransferCost" => Ok(Self::TransferCost),
            "GiverShare" => Ok(Self::GiverShare),
            "Distance" => Ok(Self::Distance),
            _ => Err(serde::de::Error::custom("unknown variant")),
        }
    }
}
