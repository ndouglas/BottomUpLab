use super::r;
use crate::negotiable::{classify, Classification, Error, ParetoKind, Point, Ratio};
use std::cmp::Ordering;

pub(super) fn verify_classification(
    points: &[Point],
    candidate: usize,
    kind: ParetoKind,
    result: &Classification,
    max_parts: usize,
) {
    if result.efficient {
        assert!(result.witness.is_none());
        return;
    }
    let witness = result
        .witness
        .as_ref()
        .expect("dominated point needs a witness");
    assert!(!witness.parts.is_empty() && witness.parts.len() <= max_parts);
    let mut total = r(0, 1);
    let mut payoff = [r(0, 1); 2];
    let mut previous = None;
    for part in &witness.parts {
        let id = part.policy_id as usize;
        assert!(id < points.len());
        if let Some(previous) = previous {
            assert!(previous < id);
        }
        previous = Some(id);
        let weight = Ratio::from_wire(part.weight.clone()).unwrap();
        assert!(weight.num() >= 0);
        total = total.checked_add(weight).unwrap();
        for (coordinate, value) in payoff.iter_mut().enumerate() {
            *value = value
                .checked_add(weight.checked_mul(points[id][coordinate]).unwrap())
                .unwrap();
        }
    }
    assert_eq!(total, r(1, 1));
    assert_eq!(
        witness.payoff.clone().map(|x| Ratio::from_wire(x).unwrap()),
        payoff
    );
    let order = [0, 1].map(|k| payoff[k].checked_cmp(points[candidate][k]).unwrap());
    match kind {
        ParetoKind::PaperWeak => assert_eq!(order, [Ordering::Greater; 2]),
        ParetoKind::Strong => {
            assert!(!order.contains(&Ordering::Less));
            assert!(order.contains(&Ordering::Greater));
        }
    }
}

#[test]
fn a_mixture_can_dominate_without_a_dominating_vertex() {
    let points = [[r(1, 1), r(1, 1)], [r(3, 1), r(0, 1)], [r(0, 1), r(3, 1)]];
    let result = classify(&points, 0, ParetoKind::PaperWeak).unwrap();
    assert!(!result.efficient);
    verify_classification(&points, 0, ParetoKind::PaperWeak, &result, 2);
    assert_eq!(result.witness.unwrap().parts.len(), 2);
}

#[test]
fn coordinate_only_improvement_distinguishes_weak_and_strong() {
    let points = [[r(0, 1); 2], [r(0, 1), r(1, 1)]];
    assert!(
        classify(&points, 0, ParetoKind::PaperWeak)
            .unwrap()
            .efficient
    );
    let result = classify(&points, 0, ParetoKind::Strong).unwrap();
    assert!(!result.efficient);
    verify_classification(&points, 0, ParetoKind::Strong, &result, 2);
}

#[test]
fn strong_dominance_accepts_a_coordinate_crossing_witness() {
    let points = [[r(1, 1); 2], [r(3, 1), r(0, 1)], [r(0, 1), r(3, 1)]];
    let result = classify(&points, 0, ParetoKind::Strong).unwrap();
    assert!(!result.efficient);
    verify_classification(&points, 0, ParetoKind::Strong, &result, 2);
    assert_eq!(
        result.witness.unwrap().payoff,
        [r(2, 1).wire(), r(1, 1).wire()]
    );
}

#[test]
fn a_narrow_open_dominance_interval_is_found_exactly() {
    let points = [
        [r(1, 1); 2],
        [r(2, 1), r(0, 1)],
        [r(0, 1), r(2000001, 1000000)],
    ];
    let result = classify(&points, 0, ParetoKind::PaperWeak).unwrap();
    assert!(!result.efficient);
    verify_classification(&points, 0, ParetoKind::PaperWeak, &result, 2);
}

#[test]
fn interior_three_vertex_mixtures_are_covered_by_boundary_segments() {
    // Equal mixing of the last three points gives (2,2), dominating (1,1).
    // The upper hull edge between indices 2 and 3 contains another witness.
    let points = [
        [r(1, 1); 2],
        [r(0, 1); 2],
        [r(6, 1), r(0, 1)],
        [r(0, 1), r(6, 1)],
    ];
    let result = classify(&points, 0, ParetoKind::PaperWeak).unwrap();
    assert!(!result.efficient);
    verify_classification(&points, 0, ParetoKind::PaperWeak, &result, 2);
}

#[test]
fn singleton_and_identical_points_are_efficient() {
    for points in [vec![[r(2, 1); 2]], vec![[r(2, 1); 2]; 3]] {
        for candidate in 0..points.len() {
            for kind in [ParetoKind::PaperWeak, ParetoKind::Strong] {
                let result = classify(&points, candidate, kind).unwrap();
                assert!(result.efficient);
                verify_classification(&points, candidate, kind, &result, 2);
            }
        }
    }
}

#[test]
fn duplicate_points_keep_original_indices_in_witnesses() {
    let points = [[r(1, 1); 2], [r(1, 1); 2], [r(0, 1); 2]];
    for id in [0, 1] {
        assert!(classify(&points, id, ParetoKind::Strong).unwrap().efficient);
    }
    let result = classify(&points, 2, ParetoKind::Strong).unwrap();
    verify_classification(&points, 2, ParetoKind::Strong, &result, 2);
    assert_eq!(result.witness.unwrap().parts[0].policy_id, 0);
}

#[test]
fn collinear_tradeoffs_remain_efficient() {
    let points = [[r(0, 1), r(4, 1)], [r(2, 1); 2], [r(4, 1), r(0, 1)]];
    for candidate in 0..3 {
        for kind in [ParetoKind::PaperWeak, ParetoKind::Strong] {
            assert!(classify(&points, candidate, kind).unwrap().efficient);
        }
    }
}

#[test]
fn bounded_input_and_candidate_are_validated() {
    for (points, candidate) in [
        (vec![], 0),
        (vec![[r(0, 1); 2]; 10], 0),
        (vec![[r(0, 1); 2]], 1),
    ] {
        assert!(classify(&points, candidate, ParetoKind::Strong).is_err());
    }
}

#[test]
fn segment_arithmetic_overflow_is_propagated() {
    let points = [[r(i128::MIN, 1); 2], [r(i128::MAX, 1); 2]];
    assert_eq!(
        classify(&points, 0, ParetoKind::PaperWeak),
        Err(Error::Overflow)
    );
}
