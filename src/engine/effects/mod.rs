pub mod types;
pub mod spawn;

pub type EffectId = u16;
pub const MAX_EFFECTS: usize = 512;
pub const EFFECT_INACTIVE: u8 = 0xFF;

// Flags
pub const EFFECT_GRAVITY: u8 = 0x01;
pub const EFFECT_LOOP: u8 = 0x02;
pub const EFFECT_ATTACHED: u8 = 0x04;

pub const GRAVITY_ACCEL: i32 = 32; // fixed-point gravity per tick

#[derive(Clone, Debug)]
pub struct Effect {
    pub effect_type: u8,     // 0x00-0x5C (93 types)
    pub state: u8,           // current state (0xFF = inactive sentinel)
    pub flags: u8,           // GRAVITY, LOOP, ATTACHED
    pub owner: u8,           // tribe index 0-3
    pub x: i32,              // world position
    pub y: i32,
    pub z: i32,              // height
    pub velocity_x: i32,     // fixed-point velocity (>>8 per tick)
    pub velocity_y: i32,
    pub velocity_z: i32,
    pub frame: i16,          // animation frame
    pub max_frame: i16,      // total frames
    pub scale: i16,          // render scale (0x100 = 100%)
    pub alpha: i16,          // transparency (0x100 = fully opaque)
    pub target: Option<u32>, // attached entity ID
    pub damage: i32,
    pub radius: i32,
    pub duration: i32,       // ticks remaining
    pub color: u32,          // RGBA packed
}

impl Default for Effect {
    fn default() -> Self {
        Self {
            effect_type: 0,
            state: EFFECT_INACTIVE,
            flags: 0,
            owner: 0,
            x: 0,
            y: 0,
            z: 0,
            velocity_x: 0,
            velocity_y: 0,
            velocity_z: 0,
            frame: 0,
            max_frame: 1,
            scale: 0x100,
            alpha: 0x100,
            target: None,
            damage: 0,
            radius: 0,
            duration: 0,
            color: 0xFFFFFFFF,
        }
    }
}

pub struct EffectPool {
    slots: Vec<Effect>,
    free_list: Vec<u16>,     // LIFO free indices
    active_count: u32,
}

impl EffectPool {
    /// Create a new pool with all 512 slots free.
    pub fn new() -> Self {
        // Stub - will be implemented in GREEN phase
        Self {
            slots: Vec::new(),
            free_list: Vec::new(),
            active_count: 0,
        }
    }

    /// Allocate a new effect. Returns None if pool is full.
    pub fn spawn(&mut self, _effect_type: u8, _x: i32, _y: i32, _z: i32, _owner: u8) -> Option<EffectId> {
        None // Stub
    }

    /// Free an effect slot back to the pool.
    pub fn destroy(&mut self, _id: EffectId) {
        // Stub
    }

    /// Get a reference to an active effect.
    pub fn get(&self, _id: EffectId) -> Option<&Effect> {
        None // Stub
    }

    /// Get a mutable reference to an active effect.
    pub fn get_mut(&mut self, _id: EffectId) -> Option<&mut Effect> {
        None // Stub
    }

    /// Number of currently active effects.
    pub fn active_count(&self) -> u32 {
        self.active_count
    }

    /// Update all active effects: apply velocity, gravity, advance frames, handle lifetime.
    pub fn update_all(&mut self) {
        // Stub
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_pool_has_512_free_slots_0_active() {
        let pool = EffectPool::new();
        assert_eq!(pool.active_count(), 0);
        // Free list should have 512 entries
        assert_eq!(pool.free_list.len(), MAX_EFFECTS);
    }

    #[test]
    fn spawn_returns_id_and_increments_active() {
        let mut pool = EffectPool::new();
        let id = pool.spawn(0x01, 100, 200, 50, 0);
        assert!(id.is_some());
        assert_eq!(pool.active_count(), 1);
    }

    #[test]
    fn spawn_512_fills_pool_513th_returns_none() {
        let mut pool = EffectPool::new();
        for i in 0..MAX_EFFECTS {
            assert!(
                pool.spawn(0x01, 0, 0, 0, 0).is_some(),
                "Failed to spawn effect {}",
                i
            );
        }
        assert!(pool.spawn(0x01, 0, 0, 0, 0).is_none());
    }

    #[test]
    fn destroy_frees_slot_lifo_reuse() {
        let mut pool = EffectPool::new();
        let id1 = pool.spawn(0x01, 0, 0, 0, 0).unwrap();
        let id2 = pool.spawn(0x01, 0, 0, 0, 0).unwrap();
        pool.destroy(id2);
        pool.destroy(id1);
        // LIFO: id1 destroyed last, should be allocated first
        let id3 = pool.spawn(0x01, 0, 0, 0, 0).unwrap();
        assert_eq!(id3, id1);
        let id4 = pool.spawn(0x01, 0, 0, 0, 0).unwrap();
        assert_eq!(id4, id2);
    }

    #[test]
    fn get_returns_spawned_effect_with_correct_position() {
        let mut pool = EffectPool::new();
        let id = pool.spawn(0x01, 100, 200, 50, 2).unwrap();
        let effect = pool.get(id).unwrap();
        assert_eq!(effect.x, 100);
        assert_eq!(effect.y, 200);
        assert_eq!(effect.z, 50);
        assert_eq!(effect.owner, 2);
        assert_eq!(effect.effect_type, 0x01);
    }

    #[test]
    fn get_after_destroy_returns_none() {
        let mut pool = EffectPool::new();
        let id = pool.spawn(0x01, 0, 0, 0, 0).unwrap();
        pool.destroy(id);
        assert!(pool.get(id).is_none());
    }

    #[test]
    fn update_all_advances_frame_counter() {
        let mut pool = EffectPool::new();
        let id = pool.spawn(0x01, 0, 0, 0, 0).unwrap();
        if let Some(e) = pool.get_mut(id) {
            e.max_frame = 10;
        }
        pool.update_all();
        let effect = pool.get(id).unwrap();
        assert_eq!(effect.frame, 1);
    }

    #[test]
    fn update_all_applies_gravity() {
        let mut pool = EffectPool::new();
        let id = pool.spawn(0x01, 0, 0, 1000, 0).unwrap();
        if let Some(e) = pool.get_mut(id) {
            e.flags = EFFECT_GRAVITY;
            e.max_frame = 100;
        }
        pool.update_all();
        let effect = pool.get(id).unwrap();
        // velocity_z should have decreased by GRAVITY_ACCEL
        assert_eq!(effect.velocity_z, -GRAVITY_ACCEL);
    }

    #[test]
    fn effect_reaching_max_frame_without_loop_becomes_inactive() {
        let mut pool = EffectPool::new();
        let id = pool.spawn(0x01, 0, 0, 0, 0).unwrap();
        if let Some(e) = pool.get_mut(id) {
            e.max_frame = 2;
            e.frame = 1; // one update will reach max_frame
            e.flags = 0; // no LOOP
        }
        pool.update_all();
        // Effect should be inactive now
        assert!(pool.get(id).is_none());
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn effect_with_loop_wraps_frame() {
        let mut pool = EffectPool::new();
        let id = pool.spawn(0x01, 0, 0, 0, 0).unwrap();
        if let Some(e) = pool.get_mut(id) {
            e.max_frame = 4;
            e.frame = 3; // one update will reach max_frame
            e.flags = EFFECT_LOOP;
        }
        pool.update_all();
        let effect = pool.get(id).unwrap();
        assert_eq!(effect.frame, 0); // wrapped back to 0
    }
}
