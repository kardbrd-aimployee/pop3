use crate::engine::objects::handle::ObjectHandle;
use crate::engine::objects::types::ObjectHeader;
use crate::engine::movement::WorldCoord;

/// Shot type constants matching original binary.
pub const SHOT_STANDARD: u8 = 1;
pub const SHOT_TRAIL: u8 = 2;
pub const SHOT_FIREBALL: u8 = 4;

/// Shot-specific data stored in ObjectPool.
#[derive(Debug, Clone)]
pub struct ShotData {
    pub shot_type: u8,
    pub target_handle: Option<ObjectHandle>,
    pub target_pos: WorldCoord,     // fallback if target destroyed
    pub damage: u16,
    pub aoe_radius: u16,
    pub knockback_force: u16,
    pub lifetime: u16,
    pub speed: u16,                 // world units per tick
    pub source_handle: Option<ObjectHandle>, // who fired this
}

impl Default for ShotData {
    fn default() -> Self {
        Self {
            shot_type: SHOT_STANDARD,
            target_handle: None,
            target_pos: WorldCoord::default(),
            damage: 100,
            aoe_radius: 0,
            knockback_force: 0,
            lifetime: 120,
            speed: 64,
            source_handle: None,
        }
    }
}

pub enum ProjectileResult {
    Continue,
    Impact { position: WorldCoord, damage: u16, aoe_radius: u16, knockback_force: u16 },
    Expired,
}

/// Update projectile position. Move toward target_pos by speed.
/// Returns Impact when close enough, Expired when lifetime runs out.
pub fn tick_projectile(shot: &mut ShotData, header: &mut ObjectHeader) -> ProjectileResult {
    if shot.lifetime == 0 {
        return ProjectileResult::Expired;
    }
    shot.lifetime -= 1;

    // Move toward target
    let dx = (shot.target_pos.x - header.position.x) as i32;
    let dz = (shot.target_pos.z - header.position.z) as i32;
    let dist_sq = (dx as i64 * dx as i64 + dz as i64 * dz as i64) as u64;
    let speed = shot.speed as i64;

    if dist_sq <= (speed * speed) as u64 {
        // Arrived at target
        header.position = shot.target_pos;
        return ProjectileResult::Impact {
            position: shot.target_pos,
            damage: shot.damage,
            aoe_radius: shot.aoe_radius,
            knockback_force: shot.knockback_force,
        };
    }

    // Move toward target by speed
    let dist = (dist_sq as f64).sqrt() as i32;
    if dist > 0 {
        let move_x = ((dx as i64 * speed) / dist as i64) as i16;
        let move_z = ((dz as i64 * speed) / dist as i64) as i16;
        header.position.x += move_x;
        header.position.z += move_z;
    }

    ProjectileResult::Continue
}

/// Create ShotData for a drum tower shot.
pub fn drum_tower_shot(target_pos: WorldCoord, target_handle: Option<ObjectHandle>) -> ShotData {
    ShotData {
        shot_type: SHOT_STANDARD,
        target_handle,
        target_pos,
        damage: 150,
        aoe_radius: 2,
        knockback_force: 64,
        lifetime: 180,
        speed: 48,
        source_handle: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::units::ModelType;

    fn make_header(pos: WorldCoord) -> ObjectHeader {
        ObjectHeader {
            model_type: ModelType::Shot,
            subtype: 0,
            tribe: 0,
            state: 0,
            state_phase: 0,
            flags1: 0,
            flags2: 0,
            flags3: 0,
            object_index: 0,
            angle: 0,
            position: pos,
            velocity: WorldCoord::default(),
            health: 0,
            max_health: 0,
            next_in_cell: None,
            prev_in_cell: None,
        }
    }

    #[test]
    fn projectile_moves_toward_target() {
        let mut shot = ShotData {
            target_pos: WorldCoord::new(1000, 0),
            speed: 50,
            lifetime: 100,
            ..Default::default()
        };
        let mut header = make_header(WorldCoord::new(0, 0));
        let result = tick_projectile(&mut shot, &mut header);
        assert!(matches!(result, ProjectileResult::Continue));
        // Should have moved closer to target
        assert!(header.position.x > 0, "Should move in +x direction");
    }

    #[test]
    fn projectile_impacts_at_threshold() {
        let mut shot = ShotData {
            target_pos: WorldCoord::new(30, 0),
            speed: 64,
            lifetime: 100,
            damage: 200,
            aoe_radius: 3,
            knockback_force: 50,
            ..Default::default()
        };
        let mut header = make_header(WorldCoord::new(0, 0));
        // Distance 30 < speed 64, so should impact immediately
        let result = tick_projectile(&mut shot, &mut header);
        match result {
            ProjectileResult::Impact { damage, aoe_radius, knockback_force, .. } => {
                assert_eq!(damage, 200);
                assert_eq!(aoe_radius, 3);
                assert_eq!(knockback_force, 50);
            }
            _ => panic!("Expected Impact"),
        }
        assert_eq!(header.position, WorldCoord::new(30, 0));
    }

    #[test]
    fn projectile_expires_at_zero_lifetime() {
        let mut shot = ShotData {
            lifetime: 0,
            ..Default::default()
        };
        let mut header = make_header(WorldCoord::default());
        let result = tick_projectile(&mut shot, &mut header);
        assert!(matches!(result, ProjectileResult::Expired));
    }

    #[test]
    fn projectile_lifetime_decrements() {
        let mut shot = ShotData {
            target_pos: WorldCoord::new(5000, 5000),
            speed: 10,
            lifetime: 5,
            ..Default::default()
        };
        let mut header = make_header(WorldCoord::new(0, 0));
        tick_projectile(&mut shot, &mut header);
        assert_eq!(shot.lifetime, 4);
    }

    #[test]
    fn drum_tower_shot_has_correct_defaults() {
        let shot = drum_tower_shot(WorldCoord::new(100, 200), Some(42));
        assert_eq!(shot.shot_type, SHOT_STANDARD);
        assert_eq!(shot.damage, 150);
        assert_eq!(shot.aoe_radius, 2);
        assert_eq!(shot.knockback_force, 64);
        assert_eq!(shot.lifetime, 180);
        assert_eq!(shot.speed, 48);
        assert_eq!(shot.target_handle, Some(42));
        assert_eq!(shot.target_pos, WorldCoord::new(100, 200));
    }

    #[test]
    fn shot_data_default() {
        let shot = ShotData::default();
        assert_eq!(shot.shot_type, SHOT_STANDARD);
        assert_eq!(shot.damage, 100);
        assert_eq!(shot.lifetime, 120);
        assert_eq!(shot.speed, 64);
        assert!(shot.target_handle.is_none());
        assert!(shot.source_handle.is_none());
    }
}
