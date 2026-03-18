use crate::engine::movement::WorldCoord;

/// Apply knockback velocity to a target from an impact point.
/// Original: Combat_ApplyKnockback at 0x4d7490
/// Computes angle from impact to target, applies force in that direction.
pub fn apply_knockback(
    target_pos: &WorldCoord,
    target_velocity: &mut WorldCoord,
    impact_pos: &WorldCoord,
    force: u16,
) {
    let dx = (target_pos.x - impact_pos.x) as i32;
    let dz = (target_pos.z - impact_pos.z) as i32;
    let dist_sq = (dx * dx + dz * dz) as f64;

    if dist_sq < 1.0 { return; } // no knockback if on top

    let dist = dist_sq.sqrt() as i32;
    if dist == 0 { return; }

    // Velocity = force * direction
    let vx = ((dx * force as i32) / dist) as i16;
    let vz = ((dz * force as i32) / dist) as i16;

    target_velocity.x += vx;
    target_velocity.z += vz;
}

/// Decay knockback velocity by friction (applied each tick).
pub fn decay_knockback(velocity: &mut WorldCoord) {
    // Halve velocity each tick (friction)
    velocity.x /= 2;
    velocity.z /= 2;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn knockback_direction_away_from_impact() {
        let target = WorldCoord::new(100, 0);
        let mut velocity = WorldCoord::default();
        let impact = WorldCoord::new(0, 0);

        apply_knockback(&target, &mut velocity, &impact, 50);
        // Target is to the right of impact, so knockback should push right (+x)
        assert!(velocity.x > 0, "Knockback x should be positive, got {}", velocity.x);
        assert_eq!(velocity.z, 0, "No z component expected for pure x offset");
    }

    #[test]
    fn knockback_diagonal() {
        let target = WorldCoord::new(100, 100);
        let mut velocity = WorldCoord::default();
        let impact = WorldCoord::new(0, 0);

        apply_knockback(&target, &mut velocity, &impact, 100);
        // Both should be positive and roughly equal
        assert!(velocity.x > 0);
        assert!(velocity.z > 0);
        // For a 45-degree angle, x and z should be similar
        assert!((velocity.x - velocity.z).abs() <= 1,
            "x={} z={} should be roughly equal", velocity.x, velocity.z);
    }

    #[test]
    fn knockback_zero_distance_no_effect() {
        let target = WorldCoord::new(50, 50);
        let mut velocity = WorldCoord::default();
        let impact = WorldCoord::new(50, 50); // same position

        apply_knockback(&target, &mut velocity, &impact, 100);
        assert_eq!(velocity.x, 0);
        assert_eq!(velocity.z, 0);
    }

    #[test]
    fn knockback_adds_to_existing_velocity() {
        let target = WorldCoord::new(100, 0);
        let mut velocity = WorldCoord::new(10, 5);
        let impact = WorldCoord::new(0, 0);

        apply_knockback(&target, &mut velocity, &impact, 50);
        assert!(velocity.x > 10, "Should add to existing velocity");
    }

    #[test]
    fn decay_knockback_halves_velocity() {
        let mut velocity = WorldCoord::new(100, -80);
        decay_knockback(&mut velocity);
        assert_eq!(velocity.x, 50);
        assert_eq!(velocity.z, -40);
    }

    #[test]
    fn decay_knockback_converges_to_zero() {
        let mut velocity = WorldCoord::new(8, -8);
        for _ in 0..10 {
            decay_knockback(&mut velocity);
        }
        assert_eq!(velocity.x, 0);
        assert_eq!(velocity.z, 0);
    }
}
