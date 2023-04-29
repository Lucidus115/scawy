// Re-export glam math stuff
pub use glam::*;

pub fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a + f * (b - a)
}