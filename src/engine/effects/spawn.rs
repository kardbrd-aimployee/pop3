use super::types::effect_defaults;
use super::{EffectId, EffectPool, EFFECT_ATTACHED};

/// Spawn an effect at a world position with type-appropriate defaults.
pub fn spawn_at(
    pool: &mut EffectPool,
    effect_type: u8,
    x: i32,
    y: i32,
    z: i32,
    owner: u8,
) -> Option<EffectId> {
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

/// Spawn a visual effect for a spell impact at the given world position.
/// Called by the spell system (Phase 4) when a spell hits its target.
/// Maps each spell type to its corresponding visual effect type.
pub fn spawn_on_spell_impact(
    pool: &mut EffectPool,
    spell_type: u8,
    x: i32,
    y: i32,
    z: i32,
    caster_tribe: u8,
) {
    let effect_type = match spell_type {
        0x01 => 0x01, // Burn -> BurnFlame
        0x02 => 0x02, // Blast -> BlastProjectile
        0x03 => 0x03, // Lightning -> LightningBolt
        0x04 => 0x04, // Tornado -> TornadoVortex
        0x05 => 0x05, // Swamp -> SwampBubble
        0x06 => 0x06, // Flatten -> FlattenWave
        0x07 => 0x07, // Earthquake -> EarthquakeCrack
        0x08 => 0x08, // Erosion -> ErosionDrip
        0x09 => 0x09, // Volcano -> VolcanoEruption
        0x0A => 0x0A, // Firestorm -> FirestormRain
        0x0B => 0x0B, // AngelOfDeath -> AngelSwirl
        0x0C => 0x0C, // Shield -> ShieldBubble
        _ => return,  // Unknown spell, no effect
    };
    spawn_at(pool, effect_type, x, y, z, caster_tribe);
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

    // --- spawn_on_spell_impact tests ---

    #[test]
    fn spell_burn_spawns_effect() {
        let mut pool = EffectPool::new();
        spawn_on_spell_impact(&mut pool, 0x01, 100, 200, 50, 0);
        assert_eq!(pool.active_count(), 1);
    }

    #[test]
    fn spell_blast_spawns_effect() {
        let mut pool = EffectPool::new();
        spawn_on_spell_impact(&mut pool, 0x02, 100, 200, 50, 0);
        assert_eq!(pool.active_count(), 1);
    }

    #[test]
    fn spell_lightning_spawns_effect() {
        let mut pool = EffectPool::new();
        spawn_on_spell_impact(&mut pool, 0x03, 100, 200, 50, 0);
        assert_eq!(pool.active_count(), 1);
    }

    #[test]
    fn unknown_spell_spawns_nothing() {
        let mut pool = EffectPool::new();
        spawn_on_spell_impact(&mut pool, 0xFF, 100, 200, 50, 0);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn all_12_spells_have_effect_mapping() {
        for spell_type in 0x01..=0x0C {
            let mut pool = EffectPool::new();
            spawn_on_spell_impact(&mut pool, spell_type, 0, 0, 0, 0);
            assert_eq!(
                pool.active_count(),
                1,
                "Spell type 0x{:02X} should spawn an effect",
                spell_type
            );
        }
    }
}
