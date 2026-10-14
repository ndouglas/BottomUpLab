use super::r;
use crate::negotiable::{cake_payoff, cake_rows, registered_weights, Error, Policy, Ratio};

#[test]
fn paper_signal_policy_has_two_subjective_payoffs_of_27() {
    assert_eq!(cake_payoff(Policy::from_id(2).unwrap()), Ok([r(27, 1); 2]));
}

#[test]
fn always_splitting_has_two_payoffs_of_20() {
    assert_eq!(cake_payoff(Policy::from_id(4).unwrap()), Ok([r(20, 1); 2]));
}

#[test]
fn all_nine_policy_ids_preserve_action_order_and_expectations() {
    let expected = [
        [30, 0],
        [29, 18],
        [27, 27],
        [21, 2],
        [20, 20],
        [18, 29],
        [3, 3],
        [2, 21],
        [0, 30],
    ];
    for (id, payoff) in expected.into_iter().enumerate() {
        let policy = Policy::from_id(id as u8).unwrap();
        assert_eq!(policy.id(), id as u8);
        assert_eq!(policy.actions(), [id as u8 / 3, id as u8 % 3]);
        assert_eq!(cake_payoff(policy).unwrap(), payoff.map(|n| r(n, 1)));
    }
}

#[test]
fn unregistered_policy_ids_are_rejected() {
    for id in [9, 255] {
        assert_eq!(Policy::from_id(id), Err(Error::InvalidPolicy));
    }
}

#[test]
fn registered_weights_are_exact_sorted_and_unique() {
    assert_eq!(registered_weights().unwrap(), expected_weights());
}

pub(super) fn expected_weights() -> Vec<Ratio> {
    [
        (0, 1),
        (1, 20),
        (1, 19),
        (1, 10),
        (3, 20),
        (2, 11),
        (1, 5),
        (1, 4),
        (3, 10),
        (7, 20),
        (2, 5),
        (9, 20),
        (1, 2),
        (11, 20),
        (3, 5),
        (13, 20),
        (7, 10),
        (3, 4),
        (4, 5),
        (9, 11),
        (17, 20),
        (9, 10),
        (18, 19),
        (19, 20),
        (1, 1),
    ]
    .into_iter()
    .map(|(n, d)| r(n, d))
    .collect()
}

#[test]
fn all_four_boundary_ties_are_retained() {
    let rows = cake_rows().unwrap();
    for (weight, maximizers) in [
        (r(1, 19), vec![5, 8]),
        (r(2, 11), vec![2, 5]),
        (r(9, 11), vec![1, 2]),
        (r(18, 19), vec![0, 1]),
    ] {
        let matching = rows
            .iter()
            .filter(|row| row.weight == weight.wire())
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 9);
        for row in matching {
            assert_eq!(row.global_maximizers, maximizers);
            assert_eq!(row.local_maximizers, maximizers);
        }
    }
}

#[test]
fn local_and_whole_policy_maximizers_agree_for_every_weight() {
    let rows = cake_rows().unwrap();
    assert_eq!(rows.len(), 225);
    for row in rows {
        assert_eq!(row.global_maximizers, row.local_maximizers);
        assert!(!row.global_maximizers.is_empty());
    }
}

#[test]
fn endpoint_weights_and_equal_prior_select_exact_objectives() {
    let rows = cake_rows().unwrap();
    for (weight, id, objective, maximum) in [
        (r(0, 1), 2, r(27, 1), 8),
        (r(1, 1), 2, r(27, 1), 0),
        (r(1, 2), 2, r(27, 1), 2),
        (r(1, 2), 4, r(20, 1), 2),
    ] {
        let row = rows
            .iter()
            .find(|row| row.weight == weight.wire() && row.policy_id == id)
            .unwrap();
        assert_eq!(row.objective, objective.wire());
        assert_eq!(row.global_maximizers, [maximum]);
    }
}
