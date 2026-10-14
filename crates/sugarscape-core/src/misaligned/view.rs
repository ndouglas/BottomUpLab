//! Replaced in the next task.
use super::world::MisalignedWorld;
pub fn size(_w: &MisalignedWorld) -> (u32, u32) {
    (1, 1)
}
pub fn render(_w: &MisalignedWorld, _mode: &str, buf: &mut Vec<u8>) -> Result<(), String> {
    buf.clear();
    buf.extend([0, 0, 0, 255]);
    Ok(())
}
pub fn inspect(_w: &MisalignedWorld, _x: u32, _y: u32) -> Result<String, String> {
    Err("no inspection yet".into())
}
