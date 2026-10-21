//! Replaced in the next task.
use super::world::HybridWorld;
pub fn size(_w: &HybridWorld) -> (u32, u32) {
    (1, 1)
}
pub fn position(_w: &HybridWorld, _i: usize) -> (u32, u32) {
    (0, 0)
}
pub fn render(_w: &HybridWorld, _mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    buf.clear();
    buf.extend([0, 0, 0, 255]);
    Ok(())
}
pub fn inspect(_w: &HybridWorld, _x: u32, _y: u32) -> Result<String, String> {
    Err("no inspection yet".into())
}
