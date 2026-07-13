use super::handle::ObjectHandle;
use super::types::*;
use crate::data::units::ModelType;
use crate::engine::buildings::BuildingData;
use crate::engine::combat::projectile::ShotData;
use crate::engine::movement::WorldCoord;

pub const MAX_OBJECTS: usize = 1101;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolError {
    Full,
}

/// Generational-arena-style fixed-capacity object store.
/// Supports all 11 model types with stable u16 handles,
/// O(1) create/destroy, and person-specific iteration.
pub struct ObjectPool {
    slots: Box<[PoolSlot]>,
    generations: Box<[u32]>,
    free_head: Option<u16>,
    active_count: u16,
}

impl ObjectPool {
    /// Create a new pool with all slots free, linked 0->1->...->MAX_OBJECTS-1->None.
    pub fn new() -> Self {
        // Stub for RED phase — returns empty pool that won't work correctly
        let mut slots: Vec<PoolSlot> = Vec::with_capacity(MAX_OBJECTS);
        for i in 0..MAX_OBJECTS {
            let next = if i + 1 < MAX_OBJECTS {
                Some((i + 1) as u16)
            } else {
                None
            };
            slots.push(PoolSlot::Free { next_free: next });
        }
        Self {
            slots: slots.into_boxed_slice(),
            generations: vec![1; MAX_OBJECTS].into_boxed_slice(),
            free_head: Some(0),
            active_count: 0,
        }
    }

    /// Allocate a new object of the given model type. Returns None if pool is full.
    pub fn create(
        &mut self,
        model_type: ModelType,
        subtype: u8,
        tribe: u8,
        position: WorldCoord,
    ) -> Result<ObjectHandle, PoolError> {
        let slot_idx = self.free_head.ok_or(PoolError::Full)?;
        let idx = slot_idx as usize;

        // Advance free head
        match &self.slots[idx] {
            PoolSlot::Free { next_free } => {
                self.free_head = *next_free;
            }
            PoolSlot::Occupied(_) => return Err(PoolError::Full),
        }
        let handle = ObjectHandle::new(slot_idx, self.generations[idx]);

        let header = ObjectHeader {
            model_type,
            subtype,
            tribe,
            state: 0,
            state_phase: 0,
            flags1: 0,
            flags2: 0,
            flags3: 0,
            object_index: handle,
            angle: 0,
            position,
            velocity: WorldCoord::default(),
            health: 0,
            max_health: 0,
            next_in_cell: None,
            prev_in_cell: None,
        };

        let data = match model_type {
            ModelType::Person => GameObjectData::Person(PersonData::default()),
            ModelType::Building => GameObjectData::Building(BuildingData::default()),
            ModelType::Creature => GameObjectData::Creature(()),
            ModelType::Vehicle => GameObjectData::Vehicle(()),
            ModelType::Scenery => GameObjectData::Scenery(SceneryData::default()),
            ModelType::General => GameObjectData::General(()),
            ModelType::Effect => GameObjectData::Effect(()),
            ModelType::Shot => GameObjectData::Shot(ShotData::default()),
            ModelType::Shape => GameObjectData::Shape(()),
            ModelType::Internal => GameObjectData::Internal(()),
            ModelType::Spell => GameObjectData::Spell(()),
        };

        self.slots[idx] = PoolSlot::Occupied(GameObject { header, data });
        self.active_count += 1;
        Ok(handle)
    }

    /// Destroy the object at the given handle. Returns true if destroyed, false if
    /// the handle was invalid or already free.
    pub fn destroy(&mut self, handle: ObjectHandle) -> bool {
        let idx = handle.index();
        if idx >= MAX_OBJECTS {
            return false;
        }
        if self.generations[idx] != handle.generation() {
            return false;
        }
        match &self.slots[idx] {
            PoolSlot::Occupied(_) => {}
            PoolSlot::Free { .. } => return false,
        }

        // Push onto free list (LIFO)
        self.slots[idx] = PoolSlot::Free {
            next_free: self.free_head,
        };
        self.free_head = Some(handle.slot());
        self.generations[idx] = next_generation(self.generations[idx]);
        self.active_count -= 1;
        true
    }

    /// Get a reference to the game object at the given handle.
    pub fn get(&self, handle: ObjectHandle) -> Option<&GameObject> {
        let idx = handle.index();
        if idx >= MAX_OBJECTS {
            return None;
        }
        if self.generations[idx] != handle.generation() {
            return None;
        }
        match &self.slots[idx] {
            PoolSlot::Occupied(obj) => Some(obj),
            PoolSlot::Free { .. } => None,
        }
    }

    /// Get a mutable reference to the game object at the given handle.
    pub fn get_mut(&mut self, handle: ObjectHandle) -> Option<&mut GameObject> {
        let idx = handle.index();
        if idx >= MAX_OBJECTS {
            return None;
        }
        if self.generations[idx] != handle.generation() {
            return None;
        }
        match &mut self.slots[idx] {
            PoolSlot::Occupied(obj) => Some(obj),
            PoolSlot::Free { .. } => None,
        }
    }

    /// Number of currently active (occupied) objects.
    pub fn active_count(&self) -> u16 {
        self.active_count
    }

    /// Reset the pool: all slots become Free, free list is re-linked, active count = 0.
    pub fn clear(&mut self) {
        for i in 0..MAX_OBJECTS {
            let next = if i + 1 < MAX_OBJECTS {
                Some((i + 1) as u16)
            } else {
                None
            };
            self.slots[i] = PoolSlot::Free { next_free: next };
            self.generations[i] = next_generation(self.generations[i]);
        }
        self.free_head = Some(0);
        self.active_count = 0;
    }

    /// Access the raw slot storage (needed by CellGrid operations).
    pub fn slots(&self) -> &[PoolSlot] {
        &self.slots
    }

    /// Mutable access to the raw slot storage (needed by CellGrid operations).
    pub fn slots_mut(&mut self) -> &mut [PoolSlot] {
        &mut self.slots
    }

    /// Iterate over all Person objects, yielding (handle, header, person_data).
    pub fn persons(&self) -> impl Iterator<Item = (ObjectHandle, &ObjectHeader, &PersonData)> {
        self.slots.iter().enumerate().filter_map(|(i, slot)| {
            if let PoolSlot::Occupied(obj) = slot {
                if let GameObjectData::Person(ref pd) = obj.data {
                    return Some((
                        ObjectHandle::new(i as u16, self.generations[i]),
                        &obj.header,
                        pd,
                    ));
                }
            }
            None
        })
    }

    /// Iterate over all Person objects with mutable access.
    pub fn persons_mut(
        &mut self,
    ) -> impl Iterator<Item = (ObjectHandle, &mut ObjectHeader, &mut PersonData)> {
        self.slots.iter_mut().enumerate().filter_map(|(i, slot)| {
            if let PoolSlot::Occupied(obj) = slot {
                if let GameObjectData::Person(ref mut pd) = obj.data {
                    return Some((
                        ObjectHandle::new(i as u16, self.generations[i]),
                        &mut obj.header,
                        pd,
                    ));
                }
            }
            None
        })
    }

    /// Iterate over all Shot objects, yielding (handle, header, shot_data).
    pub fn shots(&self) -> impl Iterator<Item = (ObjectHandle, &ObjectHeader, &ShotData)> {
        self.slots.iter().enumerate().filter_map(|(i, slot)| {
            if let PoolSlot::Occupied(obj) = slot {
                if let GameObjectData::Shot(ref sd) = obj.data {
                    return Some((
                        ObjectHandle::new(i as u16, self.generations[i]),
                        &obj.header,
                        sd,
                    ));
                }
            }
            None
        })
    }

    /// Iterate over all Shot objects with mutable access.
    pub fn shots_mut(
        &mut self,
    ) -> impl Iterator<Item = (ObjectHandle, &mut ObjectHeader, &mut ShotData)> {
        self.slots.iter_mut().enumerate().filter_map(|(i, slot)| {
            if let PoolSlot::Occupied(obj) = slot {
                if let GameObjectData::Shot(ref mut sd) = obj.data {
                    return Some((
                        ObjectHandle::new(i as u16, self.generations[i]),
                        &mut obj.header,
                        sd,
                    ));
                }
            }
            None
        })
    }

    /// Iterate over all Building objects, yielding (handle, header, building_data).
    pub fn buildings(&self) -> impl Iterator<Item = (ObjectHandle, &ObjectHeader, &BuildingData)> {
        self.slots.iter().enumerate().filter_map(|(i, slot)| {
            if let PoolSlot::Occupied(obj) = slot {
                if let GameObjectData::Building(ref bd) = obj.data {
                    return Some((
                        ObjectHandle::new(i as u16, self.generations[i]),
                        &obj.header,
                        bd,
                    ));
                }
            }
            None
        })
    }

    /// Iterate over all Building objects with mutable access.
    pub fn buildings_mut(
        &mut self,
    ) -> impl Iterator<Item = (ObjectHandle, &mut ObjectHeader, &mut BuildingData)> {
        self.slots.iter_mut().enumerate().filter_map(|(i, slot)| {
            if let PoolSlot::Occupied(obj) = slot {
                if let GameObjectData::Building(ref mut bd) = obj.data {
                    return Some((
                        ObjectHandle::new(i as u16, self.generations[i]),
                        &mut obj.header,
                        bd,
                    ));
                }
            }
            None
        })
    }
}

fn next_generation(generation: u32) -> u32 {
    let next = generation.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_person_and_get_by_handle() {
        let mut pool = ObjectPool::new();
        let pos = WorldCoord::new(100, 200);
        let handle = pool.create(ModelType::Person, 2, 1, pos).unwrap();
        let obj = pool.get(handle).unwrap();
        assert_eq!(obj.header.model_type, ModelType::Person);
        assert_eq!(obj.header.subtype, 2);
        assert_eq!(obj.header.tribe, 1);
        assert_eq!(obj.header.position, pos);
        assert_eq!(obj.header.object_index, handle);
        assert_eq!(obj.header.health, 0);
        assert_eq!(obj.header.angle, 0);
        assert_eq!(obj.header.velocity, WorldCoord::default());
        match &obj.data {
            GameObjectData::Person(pd) => assert!(pd.alive),
            _ => panic!("Expected Person variant"),
        }
    }

    #[test]
    fn create_then_destroy_returns_none_on_get() {
        let mut pool = ObjectPool::new();
        let handle = pool
            .create(ModelType::Person, 0, 0, WorldCoord::default())
            .unwrap();
        assert!(pool.get(handle).is_some());
        assert!(pool.destroy(handle));
        assert!(pool.get(handle).is_none());
    }

    #[test]
    fn stale_handle_is_rejected_after_slot_reuse() {
        let mut pool = ObjectPool::new();
        let stale = pool
            .create(ModelType::Person, 2, 0, WorldCoord::default())
            .unwrap();
        assert!(pool.destroy(stale));
        let current = pool
            .create(ModelType::Person, 2, 0, WorldCoord::default())
            .unwrap();
        assert_eq!(stale.slot(), current.slot());
        assert_ne!(stale.generation(), current.generation());
        assert!(pool.get(stale).is_none());
        assert!(!pool.destroy(stale));
        assert!(pool.get(current).is_some());
    }

    #[test]
    fn clear_invalidates_every_existing_handle() {
        let mut pool = ObjectPool::new();
        let old = pool
            .create(ModelType::Building, 1, 0, WorldCoord::default())
            .unwrap();
        pool.clear();
        assert!(pool.get(old).is_none());
        let new = pool
            .create(ModelType::Building, 1, 0, WorldCoord::default())
            .unwrap();
        assert_eq!(old.slot(), new.slot());
        assert_ne!(old.generation(), new.generation());
    }

    #[test]
    fn destroy_reuses_slot_lifo() {
        let mut pool = ObjectPool::new();
        let h1 = pool
            .create(ModelType::Person, 0, 0, WorldCoord::default())
            .unwrap();
        let h2 = pool
            .create(ModelType::Person, 0, 0, WorldCoord::default())
            .unwrap();
        pool.destroy(h2);
        pool.destroy(h1);
        // LIFO: h1 was destroyed last, so it should be allocated first
        let h3 = pool
            .create(ModelType::Person, 0, 0, WorldCoord::default())
            .unwrap();
        assert_eq!(h3.slot(), h1.slot());
        assert_ne!(h3.generation(), h1.generation());
        let h4 = pool
            .create(ModelType::Person, 0, 0, WorldCoord::default())
            .unwrap();
        assert_eq!(h4.slot(), h2.slot());
        assert_ne!(h4.generation(), h2.generation());
    }

    #[test]
    fn capacity_max_objects() {
        let mut pool = ObjectPool::new();
        for i in 0..MAX_OBJECTS {
            assert!(
                pool.create(ModelType::Person, 0, 0, WorldCoord::default())
                    .is_ok(),
                "Failed to create object {}",
                i
            );
        }
        // 1102nd should fail
        assert_eq!(
            pool.create(ModelType::Person, 0, 0, WorldCoord::default()),
            Err(PoolError::Full)
        );
    }

    #[test]
    fn destroy_frees_capacity() {
        let mut pool = ObjectPool::new();
        let mut handles = Vec::new();
        for _ in 0..MAX_OBJECTS {
            handles.push(
                pool.create(ModelType::Person, 0, 0, WorldCoord::default())
                    .unwrap(),
            );
        }
        assert_eq!(
            pool.create(ModelType::Person, 0, 0, WorldCoord::default()),
            Err(PoolError::Full)
        );
        pool.destroy(handles[500]);
        assert!(pool
            .create(ModelType::Person, 0, 0, WorldCoord::default())
            .is_ok());
    }

    #[test]
    fn persons_iterator_filters_person_only() {
        let mut pool = ObjectPool::new();
        let _ = pool.create(ModelType::Person, 0, 0, WorldCoord::default());
        let _ = pool.create(ModelType::Building, 0, 0, WorldCoord::default());
        let _ = pool.create(ModelType::Person, 0, 1, WorldCoord::default());
        let persons: Vec<_> = pool.persons().collect();
        assert_eq!(persons.len(), 2);
        assert_eq!(persons[0].1.model_type, ModelType::Person);
        assert_eq!(persons[1].1.model_type, ModelType::Person);
    }

    #[test]
    fn persons_mut_allows_mutation() {
        let mut pool = ObjectPool::new();
        let _ = pool.create(ModelType::Person, 0, 0, WorldCoord::default());
        for (_, header, pd) in pool.persons_mut() {
            header.health = 999;
            pd.bloodlust = true;
        }
        let (_, header, pd) = pool.persons().next().unwrap();
        assert_eq!(header.health, 999);
        assert!(pd.bloodlust);
    }

    #[test]
    fn active_count_tracks_correctly() {
        let mut pool = ObjectPool::new();
        assert_eq!(pool.active_count(), 0);
        let h1 = pool
            .create(ModelType::Person, 0, 0, WorldCoord::default())
            .unwrap();
        assert_eq!(pool.active_count(), 1);
        let h2 = pool
            .create(ModelType::Building, 0, 0, WorldCoord::default())
            .unwrap();
        assert_eq!(pool.active_count(), 2);
        pool.destroy(h1);
        assert_eq!(pool.active_count(), 1);
        pool.destroy(h2);
        assert_eq!(pool.active_count(), 0);
    }

    #[test]
    fn get_invalid_handle_returns_none() {
        let pool = ObjectPool::new();
        assert!(pool.get(ObjectHandle::new(0, 1)).is_none());
        assert!(pool.get(ObjectHandle::new(MAX_OBJECTS as u16, 1)).is_none());
        assert!(pool.get(ObjectHandle::new(u16::MAX, 1)).is_none());
    }

    #[test]
    fn get_mut_invalid_handle_returns_none() {
        let mut pool = ObjectPool::new();
        assert!(pool.get_mut(ObjectHandle::new(0, 1)).is_none());
        assert!(pool
            .get_mut(ObjectHandle::new(MAX_OBJECTS as u16, 1))
            .is_none());
    }

    #[test]
    fn all_11_model_types_can_be_created() {
        let mut pool = ObjectPool::new();
        let types = [
            ModelType::Person,
            ModelType::Building,
            ModelType::Creature,
            ModelType::Vehicle,
            ModelType::Scenery,
            ModelType::General,
            ModelType::Effect,
            ModelType::Shot,
            ModelType::Shape,
            ModelType::Internal,
            ModelType::Spell,
        ];
        for mt in &types {
            let handle = pool.create(*mt, 0, 0, WorldCoord::default()).unwrap();
            let obj = pool.get(handle).unwrap();
            assert_eq!(obj.header.model_type, *mt);
        }
        assert_eq!(pool.active_count(), 11);
    }
}
