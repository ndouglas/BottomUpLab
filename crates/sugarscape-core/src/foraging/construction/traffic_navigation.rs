//! Explicit versioned step selection; request diagnostics precede paid effects.
use super::{
    draws::{checked_uniform, choose, DrawSource},
    knowledge::Knowledge,
    metrics::ComputeCounts,
    navigation::{plan_route, Navigation, RoutePlan},
    observation::Observation,
    traffic::{TrafficEffect, TrafficState},
    Checked, Pos, TrafficCounts, TrafficPolicy,
};
pub(super) fn select_route(
    policy: TrafficPolicy,
    traffic: &mut TrafficState,
    knowledge: &Knowledge,
    origin: Pos,
    goals: &[Pos],
    observation: &Observation,
    draws: &mut impl DrawSource,
) -> Checked<(Navigation, ComputeCounts, TrafficEffect)> {
    match plan_route(knowledge, origin, goals, observation)? {
        RoutePlan::AtGoal => Ok((
            Navigation::AtGoal,
            ComputeCounts::default(),
            TrafficEffect::Normal,
        )),
        RoutePlan::Unreachable { counts } => {
            Ok((Navigation::Unreachable, counts, TrafficEffect::Normal))
        }
        RoutePlan::Reachable {
            free_shortest,
            yield_candidates,
            counts,
        } => {
            if !free_shortest.is_empty() {
                let target = choose(&free_shortest, draws)?.expect("nonempty shortest set");
                return Ok((Navigation::Move(target), counts, TrafficEffect::Normal));
            }
            traffic.counts.checked_include(&TrafficCounts {
                blocked_route_opportunities: 1,
                ..Default::default()
            })?;
            if policy == TrafficPolicy::LocalYieldingV1
                && traffic.blocked_wait_streak == 2
                && !yield_candidates.is_empty()
            {
                traffic.counts.checked_include(&TrafficCounts {
                    yield_attempts: 1,
                    ..Default::default()
                })?;
                if checked_uniform(draws)? < 0.25 {
                    let target = choose(&yield_candidates, draws)?.expect("nonempty yielding set");
                    return Ok((Navigation::Move(target), counts, TrafficEffect::YieldMove));
                }
            }
            Ok((Navigation::Blocked, counts, TrafficEffect::BlockedWait))
        }
    }
}
