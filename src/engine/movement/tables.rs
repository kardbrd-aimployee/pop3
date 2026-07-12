// Lookup tables for the movement math system.
// These replicate the tables embedded in popTB.exe.
// Ported from .worktrees/pathfinding/src/pathfinding/tables.rs (verified correct).

use std::sync::LazyLock;

/// Cosine table: 2048 entries, i32, 16.16 fixed-point.
/// Binary location: 0x5ACEA0
/// cos[i] = round(cos(i * 2*PI / 2048) * 65536)
pub static COS_TABLE: LazyLock<[i32; 2048]> = LazyLock::new(|| {
    let mut table = [0i32; 2048];
    for (i, entry) in table.iter_mut().enumerate() {
        let angle = (i as f64) * std::f64::consts::TAU / 2048.0;
        *entry = (angle.cos() * 65536.0).round() as i32;
    }
    table
});

/// Sine table: 2048 entries, i32, 16.16 fixed-point.
/// Binary location: 0x5AC6A0
/// sin[i] = round(sin(i * 2*PI / 2048) * 65536)
pub static SIN_TABLE: LazyLock<[i32; 2048]> = LazyLock::new(|| {
    let mut table = [0i32; 2048];
    for (i, entry) in table.iter_mut().enumerate() {
        let angle = (i as f64) * std::f64::consts::TAU / 2048.0;
        *entry = (angle.sin() * 65536.0).round() as i32;
    }
    table
});

/// Atan lookup table: 256 entries, u16 angles.
/// Binary location: 0x5641B4
/// Used by the 8-octant atan2 for ratio → angle mapping.
pub static ATAN_TABLE: LazyLock<[u16; 256]> = LazyLock::new(|| {
    let mut table = [0u16; 256];
    for (i, entry) in table.iter_mut().enumerate() {
        let ratio = (i as f64) / 256.0;
        let angle = ratio.atan();
        *entry = (angle * 2048.0 / std::f64::consts::TAU).round() as u16;
    }
    table
});

/// Integer square root initial estimates, indexed by BSR (bit scan reverse) result.
/// Binary location: 0x564034
pub const SQRT_ESTIMATES: [u32; 32] = [
    0x0001, 0x0002, 0x0002, 0x0004, 0x0005, 0x0008, 0x000B, 0x0010, 0x0016, 0x0020, 0x002D, 0x0040,
    0x005A, 0x0080, 0x00B5, 0x0100, 0x016A, 0x0200, 0x02D4, 0x0400, 0x05A8, 0x0800, 0x0B50, 0x1000,
    0x16A0, 0x2000, 0x2D41, 0x4000, 0x5A82, 0x8000, 0xB504, 0xFFFF,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cos_table_cardinal_points() {
        assert_eq!(COS_TABLE[0], 0x10000); // cos(0°) = 1.0
        assert_eq!(COS_TABLE[512], 0); // cos(90°) = 0
        assert_eq!(COS_TABLE[1024], -0x10000); // cos(180°) = -1.0
        assert_eq!(COS_TABLE[1536], 0); // cos(270°) = 0
    }

    #[test]
    fn sin_table_cardinal_points() {
        assert_eq!(SIN_TABLE[0], 0); // sin(0°) = 0
        assert_eq!(SIN_TABLE[512], 0x10000); // sin(90°) = 1.0
        assert_eq!(SIN_TABLE[1024], 0); // sin(180°) = 0
        assert_eq!(SIN_TABLE[1536], -0x10000); // sin(270°) = -1.0
    }

    #[test]
    fn cos_sin_quadrature() {
        for i in [0, 100, 256, 512, 700, 1024, 1500] {
            let c = COS_TABLE[i] as i64;
            let s = SIN_TABLE[i] as i64;
            let sum = c * c + s * s;
            let expected = 0x1_0000_0000i64;
            let error = (sum - expected).abs();
            assert!(error < 0x20000, "angle {i}: sum={sum:#x}, error={error}");
        }
    }
}
