//! Replaced in the next task.
use super::world::GridworldsWorld;
pub fn size(_w: &GridworldsWorld) -> (u32, u32) {
    (1, 1)
}
pub fn render(_w: &GridworldsWorld, _mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    buf.clear();
    buf.extend([0, 0, 0, 255]);
    Ok(())
}
pub fn inspect(_w: &GridworldsWorld, _x: u32, _y: u32) -> Result<String, String> {
    Err("no inspection yet".into())
}
