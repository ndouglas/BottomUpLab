//! Worker decisions receive only private knowledge, destination labels and local sensing.
use super::{
    draws::{checked_uniform, draw_index, DrawSource},
    excavation_navigation::assess,
    excavation_state::ExcavationEffect,
    knowledge::CellKnowledge,
    metrics::ComputeCounts,
    navigation::{
        face_approaches, faces, frontiers, select_face, select_frontier, wander, Navigation,
    },
    observation::Observation,
    state::Agent,
    traffic::TrafficEffect,
    traffic_navigation::select_route,
    Checked, FoodPhase, Mode, Parameters, Pos, Setup, TrafficPolicy, WorkCounts,
};
use crate::config::FieldError;
pub(super) struct Policy {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) nest: Vec<Pos>,
    pub(super) waste: Pos,
    pub(super) parameters: Parameters,
}
impl From<&Setup> for Policy {
    fn from(setup: &Setup) -> Self {
        Self {
            width: setup.width,
            height: setup.height,
            nest: setup.nest.clone(),
            waste: setup.waste,
            parameters: setup.parameters.clone(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum WaitReason {
    Transition,
    EmptyArrival,
    NoNeighbor,
    Congestion,
    TrafficHold,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action {
    Move(Pos),
    Dig(Pos),
    PickupFood,
    DepositFood,
    DisposeSpoil,
    Wait(WaitReason),
}
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Decision {
    pub(super) agent: Agent,
    pub(super) action: Action,
    pub(super) traffic_effect: super::traffic::TrafficEffect,
    pub(super) excavation_effect: ExcavationEffect,
}
struct SelectedAction {
    action: Action,
    effect: TrafficEffect,
    excavation_effect: ExcavationEffect,
}
impl From<Action> for SelectedAction {
    fn from(action: Action) -> Self {
        Self {
            action,
            effect: TrafficEffect::Normal,
            excavation_effect: ExcavationEffect::Ordinary,
        }
    }
}
fn error(message: &str) -> Vec<FieldError> {
    vec![FieldError::new("navigation", message)]
}
fn search(agent: &mut Agent, abandoned: bool) -> Checked<SelectedAction> {
    agent.phase = FoodPhase::Searching;
    agent.site = None;
    agent.frontier = None;
    agent.face = None;
    agent.work.checked_include(&WorkCounts {
        search_entries: 1,
        abandoned_targets: u64::from(abandoned),
        ..Default::default()
    })?;
    Ok(Action::Wait(WaitReason::Transition).into())
}
fn route(
    traffic_policy: TrafficPolicy,
    agent: &mut Agent,
    goals: &[Pos],
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<(Navigation, TrafficEffect)> {
    let (navigation, counts, effect) = select_route(
        traffic_policy,
        &mut agent.traffic,
        &agent.map,
        agent.pos,
        goals,
        observation,
        draws,
    )?;
    agent.compute.checked_include(&counts)?;
    Ok((navigation, effect))
}
fn movement((navigation, effect): (Navigation, TrafficEffect)) -> Checked<SelectedAction> {
    match navigation {
        Navigation::Move(p) => Ok(SelectedAction {
            action: Action::Move(p),
            effect,
            excavation_effect: ExcavationEffect::Ordinary,
        }),
        Navigation::Blocked => Ok(SelectedAction {
            action: Action::Wait(WaitReason::Congestion),
            effect,
            excavation_effect: ExcavationEffect::Ordinary,
        }),
        _ => Err(error("expected a reachable travel destination")),
    }
}
fn frontier(
    agent: &mut Agent,
    site: Option<Pos>,
    draws: &mut impl DrawSource,
) -> Checked<Option<Pos>> {
    if let Some(target) = agent.frontier {
        let (candidates, counts) = frontiers(&agent.map, agent.pos)?;
        agent.compute.checked_include(&counts)?;
        if candidates.iter().any(|f| f.pos == target) {
            agent.face = None;
            return Ok(Some(target));
        }
    }
    let (target, counts) = select_frontier(&agent.map, agent.pos, site, draws)?;
    agent.compute.checked_include(&counts)?;
    agent.frontier = target;
    if target.is_some() {
        agent.face = None;
    }
    Ok(target)
}
fn excavation(
    traffic_policy: TrafficPolicy,
    agent: &mut Agent,
    site: Option<Pos>,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<Option<SelectedAction>> {
    if let Some(target) = agent.face {
        let (candidates, counts) = faces(&agent.map, agent.pos)?;
        agent.compute.checked_include(&counts)?;
        if !candidates.iter().any(|f| f.pos == target) {
            agent.face = None;
        }
    }
    if agent.face.is_none() {
        let (target, counts) = select_face(&agent.map, agent.pos, site, draws)?;
        agent.compute.checked_include(&counts)?;
        agent.face = target;
    }
    let Some(target) = agent.face else {
        return Ok(None);
    };
    if observation
        .cells
        .iter()
        .any(|c| c.pos == target && !c.open && c.diggable)
    {
        return Ok(Some(Action::Dig(target).into()));
    }
    let (approaches, counts) = face_approaches(&agent.map, agent.pos, target)?;
    agent.compute.checked_include(&counts)?;
    let navigation = route(traffic_policy, agent, &approaches, observation, draws)?;
    Ok(Some(movement(navigation)?))
}
fn pursue(
    traffic_policy: TrafficPolicy,
    agent: &mut Agent,
    site: Pos,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<SelectedAction> {
    if site == agent.pos {
        return search(agent, false);
    }
    match agent.map.kind(site)? {
        CellKnowledge::KnownSolid { diggable: false } => search(agent, true),
        CellKnowledge::KnownOpen => {
            // Preserve F3's retained frontier field until its normal transition;
            // fresh sensing may invalidate an excavation commitment here.
            agent.face = None;
            let navigation = route(traffic_policy, agent, &[site], observation, draws)?;
            if navigation.0 == Navigation::Unreachable {
                search(agent, true)
            } else {
                movement(navigation)
            }
        }
        CellKnowledge::Unknown | CellKnowledge::KnownSolid { diggable: true } => {
            if let Some(target) = frontier(agent, Some(site), draws)? {
                let navigation = route(traffic_policy, agent, &[target], observation, draws)?;
                movement(navigation)
            } else if let Some(action) =
                excavation(traffic_policy, agent, Some(site), observation, draws)?
            {
                Ok(action)
            } else {
                search(agent, true)
            }
        }
    }
}
fn informed_pursuit(
    traffic_policy: TrafficPolicy,
    agent: &mut Agent,
    site: Pos,
    outlet: Pos,
    opportunity_tick: u32,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<SelectedAction> {
    let plan = assess(&agent.map, agent.pos, site, outlet, opportunity_tick)?;
    let selected = if plan.tied_candidates.is_empty() {
        None
    } else {
        let length = u32::try_from(plan.tied_candidates.len())
            .map_err(|_| vec![FieldError::new("excavation.candidates", "exceeds u32")])?;
        Some(plan.tied_candidates[draw_index(draws, length)? as usize].clone())
    };
    agent
        .excavation
        .include_assessment(&plan, selected.clone())?;
    let Some(candidate) = selected else {
        return pursue(traffic_policy, agent, site, observation, draws);
    };
    agent.frontier = None;
    agent.face = Some(candidate.face);
    if agent.pos == candidate.approach {
        if !observation
            .cells
            .iter()
            .any(|c| c.pos == candidate.face && !c.open && c.diggable)
        {
            return Err(error(
                "selected dig requires a fresh adjacent eligible face",
            ));
        }
        Ok(SelectedAction {
            action: Action::Dig(candidate.face),
            effect: TrafficEffect::Normal,
            excavation_effect: ExcavationEffect::SelectedDig,
        })
    } else {
        let navigation = route(
            traffic_policy,
            agent,
            &[candidate.approach],
            observation,
            draws,
        )?;
        let mut selected = movement(navigation)?;
        selected.excavation_effect = ExcavationEffect::SelectedTravel;
        Ok(selected)
    }
}
#[cfg(test)]
pub(super) fn decide(
    policy: &Policy,
    agent: &Agent,
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<Decision> {
    decide_with_traffic(
        policy,
        agent,
        observation,
        TrafficPolicy::ShortestOnlyV1,
        draws,
    )
}
#[cfg(test)]
pub(super) fn decide_with_traffic(
    policy: &Policy,
    agent: &Agent,
    observation: &Observation,
    traffic_policy: TrafficPolicy,
    draws: &mut impl DrawSource,
) -> Checked<Decision> {
    decide_with_policies(
        policy,
        agent,
        observation,
        traffic_policy,
        super::ExcavationPolicy::ExplorationFirstV1,
        1,
        draws,
    )
}
pub(super) fn decide_with_policies(
    policy: &Policy,
    agent: &Agent,
    observation: &Observation,
    traffic_policy: TrafficPolicy,
    excavation_policy: super::ExcavationPolicy,
    opportunity_tick: u32,
    draws: &mut impl DrawSource,
) -> Checked<Decision> {
    observation.validate(policy.width, policy.height)?;
    if observation.origin != agent.pos || agent.map.dimensions() != (policy.width, policy.height) {
        return Err(vec![FieldError::new(
            "observation",
            "must be fresh for the worker and actual policy dimensions",
        )]);
    }
    let mut agent = agent.clone();
    let delta = agent.map.learn(observation)?;
    agent.compute.checked_include(&ComputeCounts {
        observations: 1,
        cells_inspected: observation.cells.len() as u64,
        cells_learned: delta.first,
        observed_revisions: delta.observed_revisions,
        dig_confirmations: delta.dig_confirmations,
        ..Default::default()
    })?;
    let terminal = match agent.mode() {
        Mode::FoodReturning | Mode::EmptyReturning => policy.nest.contains(&agent.pos),
        Mode::SpoilHauling => agent.pos == policy.waste,
        _ => false,
    };
    if !terminal && agent.traffic.hold_pending {
        if excavation_policy == super::ExcavationPolicy::InformedCostV1 {
            if let Some(face) = agent.face {
                // Fresh learning may retire a peer-opened face without reassessing on hold.
                if agent.map.kind(face)? != (CellKnowledge::KnownSolid { diggable: true }) {
                    agent.face = None;
                }
            }
        }
        return Ok(Decision {
            agent,
            action: Action::Wait(WaitReason::TrafficHold),
            traffic_effect: TrafficEffect::Hold,
            excavation_effect: ExcavationEffect::Ordinary,
        });
    }
    let selected = match agent.mode() {
        Mode::FoodReturning | Mode::EmptyReturning => {
            if policy.nest.contains(&agent.pos) {
                if agent.cargo.is_some() {
                    Action::DepositFood.into()
                } else {
                    Action::Wait(WaitReason::EmptyArrival).into()
                }
            } else {
                let navigation =
                    route(traffic_policy, &mut agent, &policy.nest, observation, draws)?;
                movement(navigation)?
            }
        }
        Mode::SpoilHauling => {
            if agent.pos == policy.waste {
                Action::DisposeSpoil.into()
            } else {
                match agent.map.kind(policy.waste)? {
                    CellKnowledge::KnownOpen => {
                        let navigation = route(
                            traffic_policy,
                            &mut agent,
                            &[policy.waste],
                            observation,
                            draws,
                        )?;
                        movement(navigation)?
                    }
                    CellKnowledge::Unknown => {
                        let target =
                            frontier(&mut agent, Some(policy.waste), draws)?.ok_or_else(|| {
                                error("unknown outlet after exhausting private open frontiers")
                            })?;
                        let navigation =
                            route(traffic_policy, &mut agent, &[target], observation, draws)?;
                        movement(navigation)?
                    }
                    CellKnowledge::KnownSolid { .. } => {
                        return Err(error("validated outlet cannot be known solid"))
                    }
                }
            }
        }
        Mode::Departing => {
            if let Some(site) = agent.site {
                if site == agent.pos {
                    search(&mut agent, false)?
                } else if agent.map.kind(site)? == (CellKnowledge::KnownSolid { diggable: false }) {
                    search(&mut agent, true)?
                } else if excavation_policy == super::ExcavationPolicy::InformedCostV1
                    && agent.cargo.is_none()
                    && agent.find.is_none()
                {
                    informed_pursuit(
                        traffic_policy,
                        &mut agent,
                        site,
                        policy.waste,
                        opportunity_tick,
                        observation,
                        draws,
                    )?
                } else {
                    pursue(traffic_policy, &mut agent, site, observation, draws)?
                }
            } else if checked_uniform(draws)? < policy.parameters.p_search
                || agent.frontier == Some(agent.pos)
            {
                search(&mut agent, false)?
            } else if let Some(target) = frontier(&mut agent, None, draws)? {
                let navigation = route(traffic_policy, &mut agent, &[target], observation, draws)?;
                movement(navigation)?
            } else {
                search(&mut agent, false)?
            }
        }
        Mode::Searching => {
            if checked_uniform(draws)? < policy.parameters.p_return {
                agent.phase = FoodPhase::Returning;
                agent.find = None;
                agent.site = None;
                agent.frontier = None;
                agent.face = None;
                Action::Wait(WaitReason::Transition).into()
            } else if observation
                .cells
                .iter()
                .any(|c| c.pos == agent.pos && c.food)
            {
                Action::PickupFood.into()
            } else if let Some(target) = frontier(&mut agent, None, draws)? {
                let navigation = route(traffic_policy, &mut agent, &[target], observation, draws)?;
                movement(navigation)?
            } else if let Some(action) =
                excavation(traffic_policy, &mut agent, None, observation, draws)?
            {
                action
            } else {
                match wander(agent.pos, observation, draws)? {
                    Navigation::Move(p) => Action::Move(p).into(),
                    Navigation::Blocked => Action::Wait(WaitReason::NoNeighbor).into(),
                    _ => return Err(error("unexpected wandering result")),
                }
            }
        }
    };
    Ok(Decision {
        agent,
        action: selected.action,
        traffic_effect: selected.effect,
        excavation_effect: selected.excavation_effect,
    })
}
