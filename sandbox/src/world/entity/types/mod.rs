pub mod player;

/// Convert from pixels to blocks. For example, `pixels(8)` is 0.5 (half a block).
fn pixels(n: i32) -> f32 {
    n as f32 / 16.0
}
