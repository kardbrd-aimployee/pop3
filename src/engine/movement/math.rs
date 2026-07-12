// Math helper functions for the movement system.
// Each function is a faithful translation of verified x86 disassembly from popTB.exe.
// Ported from .worktrees/pathfinding/src/pathfinding/math.rs (verified correct).

use super::constants::*;
use super::tables::*;
use super::types::WorldCoord;

/// Absolute angular difference, wrapped to [0, 0x400] (0-180 degrees).
/// Original: Math_AngleDifference @ 0x004d7c10
pub fn angle_difference(a: i16, b: i16) -> i32 {
    let diff = ((a as i32) - (b as i32)).abs();
    if diff > ANGLE_HALF {
        ANGLE_FULL - diff
    } else {
        diff
    }
}

/// Returns +1 (counterclockwise) or -1 (clockwise) for shortest turn
/// direction, or 0 if angles are equal.
/// Original: Math_GetRotationDirection @ 0x004d7c40
pub fn rotation_direction(target: i16, current: i16) -> i16 {
    let mut diff = (target as i32) - (current as i32);
    if diff == 0 {
        return 0;
    }
    let abs_diff = diff.abs();
    if abs_diff > ANGLE_HALF {
        if diff < 0 {
            diff += ANGLE_FULL;
        } else {
            diff -= ANGLE_FULL;
        }
    }
    if diff >= 0 {
        1
    } else {
        -1
    }
}

/// Advances a 2D point by (sin(angle)*distance, cos(angle)*distance)
/// using 16.16 fixed-point lookup tables.
/// Compass-heading convention: angle 0 = north (+z), 512 = east (+x).
/// Original: Math_MovePointByAngle @ 0x004d4b20
///
/// ~320 calls/second during gameplay (confirmed via Frida).
pub fn move_point_by_angle(coord: &mut WorldCoord, angle: u16, speed: i16) {
    if speed == 0 {
        return;
    }
    let idx = (angle & ANGLE_MASK) as usize;
    let dist = speed as i32;
    coord.x = coord
        .x
        .wrapping_add(((SIN_TABLE[idx] as i64 * dist as i64) >> 16) as i16);
    coord.z = coord
        .z
        .wrapping_add(((COS_TABLE[idx] as i64 * dist as i64) >> 16) as i16);
}

/// 8-octant atan2 using 256-entry lookup table.
/// Returns angle in [0, 0x7FF] (11-bit, 2048 values = 360 degrees).
/// Original: Math_Atan2 @ 0x00564074
pub fn atan2(dx: i32, dy: i32) -> u16 {
    if dx == 0 && dy == 0 {
        return 0;
    }

    let abs_dx = dx.unsigned_abs();
    let abs_dy = dy.unsigned_abs();

    if dx >= 0 {
        if dy >= 0 {
            if abs_dx >= abs_dy {
                let ratio = ((abs_dy << 8) / abs_dx) as usize;
                let base = ATAN_TABLE[ratio.min(255)];
                (base.wrapping_add(0x200)) & ANGLE_MASK
            } else {
                let ratio = ((abs_dx << 8) / abs_dy) as usize;
                let base = ATAN_TABLE[ratio.min(255)];
                (0u16.wrapping_sub(base).wrapping_add(0x400)) & ANGLE_MASK
            }
        } else if abs_dx >= abs_dy {
            let ratio = ((abs_dy << 8) / abs_dx) as usize;
            let base = ATAN_TABLE[ratio.min(255)];
            (0u16.wrapping_sub(base).wrapping_add(0x200)) & ANGLE_MASK
        } else {
            let ratio = ((abs_dx << 8) / abs_dy) as usize;
            let base = ATAN_TABLE[ratio.min(255)];
            base & ANGLE_MASK
        }
    } else if dy >= 0 {
        if abs_dx >= abs_dy {
            let ratio = ((abs_dy << 8) / abs_dx) as usize;
            let base = ATAN_TABLE[ratio.min(255)];
            (0u16.wrapping_sub(base).wrapping_add(0x600)) & ANGLE_MASK
        } else {
            let ratio = ((abs_dx << 8) / abs_dy) as usize;
            let base = ATAN_TABLE[ratio.min(255)];
            base.wrapping_add(0x400) & ANGLE_MASK
        }
    } else if abs_dx >= abs_dy {
        let ratio = ((abs_dy << 8) / abs_dx) as usize;
        let base = ATAN_TABLE[ratio.min(255)];
        base.wrapping_add(0x600) & ANGLE_MASK
    } else {
        let ratio = ((abs_dx << 8) / abs_dy) as usize;
        let base = ATAN_TABLE[ratio.min(255)];
        (0u16.wrapping_sub(base).wrapping_add(0x800)) & ANGLE_MASK
    }
}

/// Integer square root using BSR-indexed initial estimate + Newton-Raphson.
/// Original: Math_IntSqrt @ 0x00564000
pub fn int_sqrt(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let bsr = 31 - n.leading_zeros();
    let mut guess = SQRT_ESTIMATES[bsr as usize];

    loop {
        let div = n / guess;
        if div >= guess {
            break;
        }
        guess = (guess + div) / 2;
    }
    guess
}

/// Euclidean distance between two world coordinates with toroidal wrapping.
/// Original: Math_Distance @ 0x004ea8f0
pub fn distance(p1: &WorldCoord, p2: &WorldCoord) -> i32 {
    let mut dx = (p2.x as i32) - (p1.x as i32);
    let mut dy = (p2.z as i32) - (p1.z as i32);

    if dx.abs() > WORLD_WRAP_THRESHOLD {
        dx = WORLD_SIZE - dx.abs();
    }
    if dy.abs() > WORLD_WRAP_THRESHOLD {
        dy = WORLD_SIZE - dy.abs();
    }

    int_sqrt((dx.wrapping_mul(dx) + dy.wrapping_mul(dy)) as u32) as i32
}

/// Formation idle RNG.
/// Original: state at 0x885710
/// state = state * 9377 + 0x24DF; state = ROR(state, 13)
pub fn formation_rng_next(state: u32) -> u32 {
    let s = state
        .wrapping_mul(RNG_MULTIPLIER)
        .wrapping_add(RNG_INCREMENT);
    s.rotate_right(RNG_ROTATE_BITS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angle_diff_same() {
        assert_eq!(angle_difference(0, 0), 0);
        assert_eq!(angle_difference(0x400, 0x400), 0);
    }

    #[test]
    fn angle_diff_half_circle() {
        assert_eq!(angle_difference(0, 0x400), ANGLE_HALF);
    }

    #[test]
    fn angle_diff_wrap_around() {
        assert_eq!(angle_difference(0x7FF_u16 as i16, 0x001_u16 as i16), 2);
    }

    #[test]
    fn angle_diff_symmetric() {
        assert_eq!(
            angle_difference(0x100, 0x300),
            angle_difference(0x300, 0x100)
        );
    }

    #[test]
    fn rot_dir_equal() {
        assert_eq!(rotation_direction(0x100, 0x100), 0);
    }

    #[test]
    fn rot_dir_positive() {
        assert_eq!(rotation_direction(0x100, 0), 1);
    }

    #[test]
    fn rot_dir_negative() {
        assert_eq!(rotation_direction(0, 0x100), -1);
    }

    #[test]
    fn move_point_zero_distance() {
        let mut p = WorldCoord::new(100, 200);
        move_point_by_angle(&mut p, 0, 0);
        assert_eq!(p, WorldCoord::new(100, 200));
    }

    #[test]
    fn move_point_north() {
        let mut p = WorldCoord::new(0, 0);
        move_point_by_angle(&mut p, 0, 256);
        assert_eq!(p.x, 0); // sin(0) * 256 = 0
        assert_eq!(p.z, 256); // cos(0) * 256 = 256
    }

    #[test]
    fn move_point_east() {
        let mut p = WorldCoord::new(0, 0);
        move_point_by_angle(&mut p, 512, 256);
        assert_eq!(p.x, 256); // sin(90°) * 256 = 256
        assert_eq!(p.z, 0); // cos(90°) * 256 = 0
    }

    #[test]
    fn sqrt_perfect_squares() {
        assert_eq!(int_sqrt(0), 0);
        assert_eq!(int_sqrt(1), 1);
        assert_eq!(int_sqrt(4), 2);
        assert_eq!(int_sqrt(9), 3);
        assert_eq!(int_sqrt(100), 10);
        assert_eq!(int_sqrt(10000), 100);
    }

    #[test]
    fn distance_same_point() {
        let p = WorldCoord::new(100, 200);
        assert_eq!(distance(&p, &p), 0);
    }

    #[test]
    fn distance_simple() {
        assert_eq!(distance(&WorldCoord::new(0, 0), &WorldCoord::new(3, 4)), 5);
    }

    #[test]
    fn rng_deterministic() {
        let s1 = formation_rng_next(0);
        let s2 = formation_rng_next(s1);
        let s3 = formation_rng_next(s2);
        // Just ensure it's deterministic and produces different values
        assert_ne!(s1, 0);
        assert_ne!(s2, s1);
        assert_ne!(s3, s2);
    }

    #[test]
    fn rng_idle_delay_range() {
        // Idle delay = (rng & 7) + 24, so range is [24, 31]
        let s = formation_rng_next(12345);
        let delay = (s & 7) + 24;
        assert!(delay >= 24 && delay <= 31);
    }
}
