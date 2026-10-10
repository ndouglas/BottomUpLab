use super::super::excavation_navigation::{assess, estimated_cost};
use super::*;

// Private-map unit fixtures: fresh complete observations of real Terrain,
// without claiming these are paid World trajectories.
pub(super) fn private_map(
    width: u32,
    height: u32,
    open: Vec<Pos>,
    diggable: Vec<Pos>,
    learned: &[Pos],
) -> Knowledge {
    let mut fixture = setup();
    fixture.width = width;
    fixture.height = height;
    fixture.nest = vec![open[0], open[1]];
    fixture.waste = *open
        .iter()
        .find(|p| {
            !fixture.nest.contains(p)
                && super::super::setup::neighbors(**p, width, height)
                    .iter()
                    .any(|next| fixture.nest.contains(next))
        })
        .expect("fixture requires separate connected waste cell");
    fixture.workers = vec![open[0]];
    fixture.food.clear();
    fixture.open = open;
    fixture.diggable = diggable;
    let terrain = Terrain::new(&fixture).unwrap();
    let mut knowledge = Knowledge::new(width, height).unwrap();
    for &p in learned {
        knowledge
            .learn(&observe(&terrain, p, &BTreeMap::new(), &BTreeSet::new()).unwrap())
            .unwrap();
    }
    knowledge
}
pub(super) fn corridor(depth: u32) -> Knowledge {
    let mut open = vec![pos(2, 1), pos(1, 1)];
    open.extend((2..=depth).map(|y| pos(2, y)));
    open.push(pos(3, depth));
    open.extend((1..=depth).map(|y| pos(4, y)));
    private_map(7, 7, open.clone(), vec![pos(3, 1)], &open)
}
#[test]
fn cost_charges_dig_disposal_entry_and_both_spoil_legs() {
    assert_eq!(estimated_cost(0, 3, 4).unwrap(), 13);
}
#[test]
fn cost_overflow_is_an_error() {
    for components in [(0, u64::MAX, 0), (u64::MAX, 0, 0), (0, 0, u64::MAX)] {
        assert!(estimated_cost(components.0, components.1, components.2).is_err());
    }
}
#[test]
fn u_corridor_shortcut_has_exact_private_cost_and_no_map_mutation() {
    let knowledge = corridor(5);
    let before = knowledge.clone();
    let plan = assess(&knowledge, pos(2, 1), pos(4, 1), pos(1, 1), 1).unwrap();
    assert_eq!(plan.assessment.reference_cost, Some(10));
    assert_eq!(
        plan.assessment.best_candidate,
        Some(ExcavationCandidate {
            face: pos(3, 1),
            approach: pos(2, 1),
            approach_moves: 0,
            spoil_leg_moves: 1,
            continuation_moves: 1,
            pre_opening_distance: Some(10),
            estimated_cost: 6,
        })
    );
    assert_eq!(
        plan.tied_candidates,
        vec![plan.assessment.best_candidate.clone().unwrap()]
    );
    assert_eq!(plan.assessment.selected_candidate, None);
    assert_eq!(plan.assessment.candidate_pairs, 2);
    assert_eq!(plan.counts.assessments, 1);
    assert_eq!(plan.counts.distance_calls, 3);
    assert_eq!(plan.counts.selected_opportunities, 0);
    assert_eq!(plan.counts.initiated_digs, 0);
    assert_eq!(knowledge, before);
}
#[test]
fn equality_and_higher_full_cost_retain_diagnostic_best_but_reject_selection() {
    for outlet in [pos(1, 1), pos(2, 2)] {
        let plan = assess(&corridor(3), pos(2, 1), pos(4, 1), outlet, 1).unwrap();
        assert_eq!(plan.assessment.reference_cost, Some(6));
        assert_eq!(plan.assessment.best_candidate.unwrap().estimated_cost, 6);
        assert!(plan.tied_candidates.is_empty());
    }
    let plan = assess(&corridor(3), pos(2, 1), pos(4, 1), pos(2, 3), 1).unwrap();
    assert_eq!(plan.assessment.best_candidate.unwrap().estimated_cost, 8);
    assert!(plan.tied_candidates.is_empty());
}
#[test]
fn already_available_unknown_continuation_earns_no_opening_credit() {
    let open = vec![pos(2, 1), pos(1, 1), pos(2, 2)];
    let knowledge = private_map(7, 7, open, vec![pos(3, 1)], &[pos(2, 1)]);
    assert_eq!(knowledge.kind(pos(3, 2)).unwrap(), CellKnowledge::Unknown);
    let plan = assess(&knowledge, pos(2, 1), pos(4, 2), pos(1, 1), 1).unwrap();
    assert_eq!(plan.assessment.reference_cost, Some(3));
    assert_eq!(plan.counts.candidate_pairs, 1);
    assert_eq!(plan.assessment.best_candidate, None);
}
#[test]
fn diggable_site_has_zero_continuation_and_unavailable_reference() {
    let knowledge = corridor(5);
    let plan = assess(&knowledge, pos(2, 1), pos(3, 1), pos(1, 1), 1).unwrap();
    let best = plan.assessment.best_candidate.unwrap();
    assert_eq!(best.continuation_moves, 0);
    assert_eq!(best.pre_opening_distance, None);
    assert_eq!(best.estimated_cost, 5);
    assert_eq!(plan.assessment.reference_cost, None);
    assert_eq!(plan.counts.distance_calls, 2);
    assert_eq!(plan.tied_candidates.len(), 1);
}
#[test]
fn unknown_outlet_cannot_supply_spoil_route_and_solid_outlet_errors() {
    let knowledge = corridor(5);
    let before = knowledge.clone();
    let plan = assess(&knowledge, pos(2, 1), pos(4, 1), pos(0, 6), 1).unwrap();
    assert_eq!(plan.assessment.best_candidate, None);
    assert_eq!(plan.counts.candidate_pairs, 2);
    assert_eq!(plan.counts.distance_calls, 2);
    assert!(assess(&knowledge, pos(2, 1), pos(4, 1), pos(2, 0), 1).is_err());
    assert_eq!(knowledge, before);
}
#[test]
fn protected_barriers_and_disconnected_faces_cannot_create_candidates() {
    let open = vec![pos(1, 1), pos(1, 2), pos(1, 3), pos(5, 5)];
    let knowledge = private_map(7, 7, open.clone(), vec![pos(5, 4)], &open);
    let plan = assess(&knowledge, pos(1, 1), pos(5, 5), pos(1, 2), 1).unwrap();
    assert_eq!(plan.assessment.reference_cost, None);
    assert_eq!(plan.assessment.best_candidate, None);
    assert_eq!(plan.counts.candidate_pairs, 0);
    assert_eq!(plan.counts.faces_scanned, 12);
}
#[test]
fn remembered_solids_other_than_the_candidate_still_block_continuation() {
    let open = vec![pos(1, 1), pos(1, 2), pos(1, 0), pos(3, 1)];
    let knowledge = private_map(5, 3, open.clone(), vec![pos(2, 1), pos(4, 1)], &open);
    let plan = assess(&knowledge, pos(1, 1), pos(4, 1), pos(1, 2), 1).unwrap();
    assert_eq!(plan.assessment.best_candidate, None);
    assert_eq!(plan.counts.candidate_pairs, 1);
}
#[test]
fn invalid_coordinates_unknown_origin_and_zero_tick_are_errors() {
    let knowledge = corridor(5);
    for (origin, site, outlet, tick) in [
        (pos(7, 1), pos(4, 1), pos(1, 1), 1),
        (pos(2, 1), pos(7, 1), pos(1, 1), 1),
        (pos(2, 1), pos(4, 1), pos(7, 1), 1),
        (pos(0, 6), pos(4, 1), pos(1, 1), 1),
        (pos(2, 0), pos(4, 1), pos(1, 1), 1),
        (pos(2, 1), pos(4, 1), pos(1, 1), 0),
    ] {
        assert!(assess(&knowledge, origin, site, outlet, tick).is_err());
    }
}
#[test]
fn largest_grid_computation_is_bounded_and_counts_actual_dequeues() {
    let diggable: Vec<_> = (0..125)
        .flat_map(|x| (0..125).filter_map(move |y| (x % 2 == 1 && y % 2 == 1).then_some(pos(x, y))))
        .collect();
    let open: Vec<_> = (0..125)
        .flat_map(|x| (0..125).filter_map(move |y| (x % 2 == 0 || y % 2 == 0).then_some(pos(x, y))))
        .collect();
    let knowledge = private_map(125, 125, open.clone(), diggable, &open);
    let plan = assess(&knowledge, pos(0, 0), pos(124, 124), pos(0, 1), 1).unwrap();
    assert_eq!(plan.counts.distance_calls, 3);
    assert_eq!(plan.counts.distance_visits, 3 * 11_781);
    assert_eq!(plan.counts.faces_scanned, 3_844);
    assert_eq!(plan.counts.candidate_pairs, 15_376);
    assert!(plan.counts.distance_visits <= 3 * 15_625);
    assert!(plan.counts.peak_queue <= 15_625);
    assert!(plan.counts.faces_scanned <= 15_625);
    assert!(plan.counts.candidate_pairs <= 4 * 15_625);
}
#[test]
fn accepted_minimum_ties_are_ordered_by_face_then_approach() {
    let mut open = vec![pos(2, 5), pos(1, 5), pos(3, 0), pos(3, 10)];
    open.extend(
        (0..=10)
            .flat_map(|y| [pos(2, y), pos(4, y)])
            .filter(|&p| p != pos(2, 5)),
    );
    let knowledge = private_map(7, 11, open.clone(), vec![pos(3, 6), pos(3, 4)], &open);
    let plan = assess(&knowledge, pos(2, 5), pos(4, 5), pos(1, 5), 1).unwrap();
    assert_eq!(plan.assessment.reference_cost, Some(12));
    assert_eq!(
        plan.tied_candidates
            .iter()
            .map(|c| (c.face, c.approach, c.estimated_cost))
            .collect::<Vec<_>>(),
        vec![(pos(3, 4), pos(2, 4), 10), (pos(3, 6), pos(2, 6), 10)]
    );
    assert_eq!(
        plan.assessment.best_candidate,
        Some(plan.tied_candidates[0].clone())
    );
}
