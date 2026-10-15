//! Registered report language constants and immutable protocol bytes.
pub const PROTOCOL_VERSION: u64 = 1;
pub const PROTOCOL_ID: &str = "persistent-notes-v1";
pub const REPORT_VERSION: &str = "persistent-notes-diagnostic-v1";
pub const SUMMARY_VERSION: &str = "persistent-notes-summary-v1";
pub(crate) static PROTOCOL_BYTES: &[u8] =
    include_bytes!("../../../../docs/superpowers/protocols/2026-10-14-paid-persistent-notes.md");
