use super::{EffectPool, EffectId, EFFECT_ATTACHED};
use super::types::effect_defaults;

/// Spawn an effect at a world position with type-appropriate defaults.
pub fn spawn_at(pool: &mut EffectPool, effect_type: u8, x: i32, y: i32, z: i32, owner: u8) -> Option<EffectId> {
    let id = pool.spawn(effect_type, x, y, z, owner)?;
    let (max_frame, flags, scale, alpha) = effect_defaults(effect_type);
    if let Some(effect) = pool.get_mut(id) {
        effect.max_frame = max_frame;
        effect.flags = flags;
        effect.scale = scale;
        effect.alpha = alpha;
    }
    Some(id)
}

/// Attach an existing effect to an entity so it tracks the entity's position.
pub fn attach_to_entity(pool: &mut EffectPool, effect_id: EffectId, entity_id: u32) {
    if let Some(effect) = pool.get_mut(effect_id) {
        effect.target = Some(entity_id);
        effect.flags |= EFFECT_ATTACHED;
    }
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
pub fn update_attached_positions(pool: &mut EffectPool, entities: &[EntityPosition]) {
    for i in 0..super::MAX_EFFECTS {
        if let Some(effect) = pool.get_mut(i as u16) {
            if effect.flags & EFFECT_ATTACHED == 0 {
                continue;
            }
            if let Some(target_id) = effect.target {
                if let Some(ent) = entities.iter().find(|e| e.entity_id == target_id) {
                    if ent.alive {
                        effect.x = ent.x;
                        effect.y = ent.y;
                        effect.z = ent.z;
                    } else {
                        // Entity dead -- detach
                        effect.target = None;
                        effect.flags &= !EFFECT_ATTACHED;
                    }
                } else {
                    // Entity not found -- detach
                    effect.target = None;
                    effect.flags &= !EFFECT_ATTACHED;
                }
            }
        }
    }
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
