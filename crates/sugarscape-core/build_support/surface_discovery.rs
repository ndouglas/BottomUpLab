//! Host-only access to the unchanged portable SHA implementation.
#[path = "../src/surface_discovery/diagnostics/hash.rs"]
mod hash;
pub fn sha256(bytes: &[u8]) -> String {
    hash::sha256(bytes)
}
