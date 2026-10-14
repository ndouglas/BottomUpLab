//! Finite exact acceptance checks, never a general theorem verdict.
use super::negotiable_records::{self, Failure, RecordKind};
use crate::claim::{Claim, Outcome, Source, Verdict};

const DESIGN: &str = "docs/superpowers/specs/2026-10-14-negotiable-assistance-design.md";
pub(super) fn outcome(result: Result<Vec<u8>, Failure>, citation: &str) -> Outcome {
    let detail = format!("Engineering acceptance for the finite exact foundation only; supplied cells do not prove a general theorem, sequential recursion, fairness or realized welfare. Design: {DESIGN}. Source: {citation}.");
    match result {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(measured) => Outcome {
                verdict: Verdict::Holds,
                measured,
                detail,
            },
            Err(error) => Outcome {
                verdict: Verdict::Error,
                measured: String::new(),
                detail: format!("{detail} UTF-8 record encoding: {error}"),
            },
        },
        Err(error) => Outcome {
            verdict: match error {
                Failure::Mathematics(_) => Verdict::Fails,
                Failure::Operational(_) => Verdict::Error,
            },
            measured: String::new(),
            detail: format!("{detail} {error}"),
        },
    }
}
fn cake(_: &[u64]) -> Outcome {
    outcome(
        negotiable_records::build_checked(RecordKind::Cake),
        "Desai, Critch and Russell (2018), §3, Lemma 3, equations 3–8",
    )
}
fn pareto(_: &[u64]) -> Outcome {
    outcome(negotiable_records::build_checked(RecordKind::Pareto),"Engineering comparison of Definition 1 paper_weak and strong conventions over the exact nine-policy payoff hull; supplement §1")
}
fn priority(_: &[u64]) -> Outcome {
    outcome(negotiable_records::build_checked(RecordKind::Priority),"Desai, Critch and Russell (2018), equation 8; engineering census of 4599 supplied history cells")
}
pub fn claims() -> Vec<Claim> {
    vec![
        Claim { id:"negotiable.cake",item:"negotiable",source:Source::Book,citation:"Desai, Critch and Russell (2018), §3 cake example; Lemma 3 and equations 3–8",text:"The specified cake has subjective payoffs (27,27) for red→Alice/green→Bob and (20,20) for always splitting; all 225 registered objective cells retain every whole/local maximizing tie.",check:cake },
        Claim { id:"negotiable.pareto",item:"negotiable",source:Source::App,citation:"Engineering check: exact stochastic hull; Desai, Critch and Russell (2018), Definition 1 and supplement §1",text:"Engineering check: compare paper_weak and strong efficiency for all nine deterministic policies against their entire exact stochastic payoff hull, with independently verified dominance witnesses.",check:pareto },
        Claim { id:"negotiable.priority",item:"negotiable",source:Source::Book,citation:"Desai, Critch and Russell (2018), equation 8; fixed-domain engineering census in the approved design",text:"Equation 8 updates normalized subjective priority masses on supported supplied observations; the finite 4599-cell engineering census preserves zero priors and the first impossible prefix.",check:priority },
    ]
}
