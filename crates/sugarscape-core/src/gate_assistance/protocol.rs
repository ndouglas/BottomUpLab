//! Strict native diagnostic envelopes. Nullable fields remain required.
use super::{
    types::{required_option, wire_struct},
    *,
};
wire_struct!(Receipt {
    path: String,
    bytes: u64,
    sha256: String
});
wire_struct!(Check {
    name: String,
    passed: bool,
    detail: String
});
wire_struct!(ControlRecord {selector:Selector,
    #[serde(deserialize_with="required_option")] profile:Option<Profile>,
    #[serde(deserialize_with="required_option")] episode:Option<Episode>,
    #[serde(deserialize_with="required_option")] classification:Option<Classification>
});
wire_struct!(Panel {stratum:Stratum,rows:Vec<Row>,analysis:Analysis,controls:Vec<ControlRecord>,stats:CompileStats});
wire_struct!(Report {version:String,source_hash:String,source_receipts:Vec<Receipt>,reference_receipts:Vec<Receipt>,limits:Limits,panels:Vec<Panel>,contrasts:Vec<Contrast>,checks:Vec<Check>,passed:bool});
wire_struct!(Summary {version:String,report_hash:String,source_hash:String,settings:Vec<SummaryStratum>,contrasts:Vec<Contrast>,limits:Limits,source_receipts:Vec<Receipt>,reference_receipts:Vec<Receipt>});
pub(super) const CHECK_NAMES: [&str; 10] = [
    "registered_census",
    "reference_outcomes",
    "reference_trace_hashes",
    "reference_controls",
    "best_responses",
    "pure_membership",
    "representative_ties",
    "reward_share_invariance",
    "resource_limits",
    "source_and_reference_identity",
];
