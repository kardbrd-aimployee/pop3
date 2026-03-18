use super::{EffectPool, EffectId, EFFECT_ATTACHED};
use super::types::effect_defaults;

/// Spawn an effect at a world position with type-appropriate defaults.
pub fn spawn_at(_pool: &mut EffectPool, _effect_type: u8, _x: i32, _y: i32, _z: i32, _owner: u8) -> Option<EffectId> {
    None // Stub
}

/// Attach an existing effect to an entity so it tracks the entity's position.
pub fn attach_to_entity(_pool: &mut EffectPool, _effect_id: EffectId, _entity_id: u32) {
    // Stub
}

/// Position data for an entity that effects can track.
pub struct EntityPosition {
    pub entity_id: u32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub alive: bool,
}

/// Update positions of all attached effects from entity positions.
/// Call this each tick with current entity positions.
/// Two-phase pattern: collect entity positions first (immutable), then update effects (mutable).
pub fn update_attached_positions(_pool: &mut EffectPool, _entities: &[EntityPosition]) {
    // Stub
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::effects::*;

    #[test]
    fn spawn_at_creates_effect_with_defaults() {
        let mut pool = EffectPool::new();
        let id = spawn_at(&mut pool, 0x01, 100, 200, 50, 1).unwrap();
        let effect = pool.get(id).unwrap();
        assert_eq!(effect.effect_type, 0x01);
        assert_eq!(effect.x, 100);
        assert_eq!(effect.y, 200);
        assert_eq!(effect.z, 50);
        assert_eq!(effect.owner, 1);
        // BurnFlame defaults: max_frame=16, flags=LOOP, scale=0x100, alpha=0xC0
        let (max_frame, flags, scale, alpha) = effect_defaults(0x01);
        assert_eq!(effect.max_frame, max_frame);
        assert_eq!(effect.flags, flags);
        assert_eq!(effect.scale, scale);
        assert_eq!(effect.alpha, alpha);
    }

    #[test]
    fn spawn_at_on_full_pool_returns_none() {
        let mut pool = EffectPool::new();
        for _ in 0..MAX_EFFECTS {
            pool.spawn(0x01, 0, 0, 0, 0);
        }
        assert!(spawn_at(&mut pool, 0x01, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn attach_to_entity_sets_target_and_flag() {
        let mut pool = EffectPool::new();
        let id = spawn_at(&mut pool, 0x33, 0, 0, 0, 0).unwrap(); // HitSpark (no flags by default)
        attach_to_entity(&mut pool, id, 42);
        let effect = pool.get(id).unwrap();
        assert_eq!(effect.target, Some(42));
        assert_ne!(effect.flags & EFFECT_ATTACHED, 0);
    }

    #[test]
    fn update_attached_positions_syncs_position() {
        let mut pool = EffectPool::new();
        let id = spawn_at(&mut pool, 0x33, 0, 0, 0, 0).unwrap();
        attach_to_entity(&mut pool, id, 10);

        let entities = vec![EntityPosition {
            entity_id: 10,
            x: 500,
            y: 600,
            z: 100,
            alive: true,
        }];
        update_attached_positions(&mut pool, &entities);

        let effect = pool.get(id).unwrap();
        assert_eq!(effect.x, 500);
        assert_eq!(effect.y, 600);
        assert_eq!(effect.z, 100);
    }

    #[test]
    fn update_attached_positions_detaches_on_dead_entity() {
        let mut pool = EffectPool::new();
        let id = spawn_at(&mut pool, 0x33, 0, 0, 0, 0).unwrap();
        attach_to_entity(&mut pool, id, 10);

        let entities = vec![EntityPosition {
            entity_id: 10,
            x: 500,
            y: 600,
            z: 100,
            alive: false, // dead
        }];
        update_attached_positions(&mut pool, &entities);

        let effect = pool.get(id).unwrap();
        assert_eq!(effect.target, None);
        assert_eq!(effect.flags & EFFECT_ATTACHED, 0);
    }
}
