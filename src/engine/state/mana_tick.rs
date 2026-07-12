// Mana tick implementation — wires mana generation into the game loop.
// Original: Tick_UpdateMana at 0x004aeac0
//
// Each tick:
// 1. Iterate all persons, call mana_rate_for_person(subtype) per person,
//    accumulate into tribe's mana pool via add_mana
// 2. Iterate all active housing buildings, add mana based on hut level

use crate::engine::buildings::{BuildingState, BuildingSubtype};
use crate::engine::economy::mana::{add_mana, mana_rate_for_housing, mana_rate_for_person};
use crate::engine::objects::pool::ObjectPool;
use crate::engine::objects::types::GameObjectData;
use crate::engine::state::traits::ManaTick;
use crate::engine::state::tribe::TribeArray;

/// Bridge struct that holds references needed for mana generation.
/// Created inline at the game loop tick site each frame.
pub struct ManaTickBridge<'a> {
    pub pool: &'a ObjectPool,
    pub tribes: &'a mut TribeArray,
}

impl<'a> ManaTick for ManaTickBridge<'a> {
    fn tick_update_mana(&mut self) {
        // Step 1: Person-based mana generation
        // Each person generates mana based on their subtype every tick.
        for (_handle, header, _pd) in self.pool.persons() {
            let rate = mana_rate_for_person(header.subtype);
            if rate > 0 {
                let tribe_idx = header.tribe as usize;
                if tribe_idx < self.tribes.tribes.len() {
                    add_mana(&mut self.tribes.tribes[tribe_idx].mana, rate);
                }
            }
        }

        // Step 2: Housing-based mana generation
        // Active huts generate mana based on their level (subtype maps to hut level).
        for (_handle, header, bd) in self.pool.buildings() {
            if bd.state != BuildingState::Active {
                continue;
            }
            let hut_level = match bd.building_subtype {
                BuildingSubtype::SmallHut => 1,
                BuildingSubtype::MediumHut => 2,
                BuildingSubtype::LargeHut => 3,
                _ => continue, // not a housing building
            };
            let rate = mana_rate_for_housing(hut_level);
            if rate > 0 {
                let tribe_idx = header.tribe as usize;
                if tribe_idx < self.tribes.tribes.len() {
                    add_mana(&mut self.tribes.tribes[tribe_idx].mana, rate);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::units::ModelType;
    use crate::engine::movement::WorldCoord;
    use crate::engine::objects::pool::ObjectPool;
    use crate::engine::objects::types::ObjectHeader;
    use crate::engine::state::tribe::TribeArray;

    #[test]
    fn mana_tick_accumulates_person_mana() {
        let mut pool = ObjectPool::new();
        // Create a Brave (subtype 2, tribe 0) — rate = 1
        let h = pool
            .create(ModelType::Person, 2, 0, WorldCoord::default())
            .unwrap();
        if let Some(obj) = pool.get_mut(h) {
            obj.header.health = 100;
        }

        let mut tribes = TribeArray::new();
        tribes.tribes[0].active = true;

        let mut bridge = ManaTickBridge {
            pool: &pool,
            tribes: &mut tribes,
        };
        bridge.tick_update_mana();

        assert_eq!(tribes.tribes[0].mana, 1); // Brave generates 1 mana per tick
    }

    #[test]
    fn mana_tick_accumulates_preacher_mana() {
        let mut pool = ObjectPool::new();
        // Create a Preacher (subtype 4, tribe 1) — rate = 2
        pool.create(ModelType::Person, 4, 1, WorldCoord::default())
            .unwrap();

        let mut tribes = TribeArray::new();
        tribes.tribes[1].active = true;

        let mut bridge = ManaTickBridge {
            pool: &pool,
            tribes: &mut tribes,
        };
        bridge.tick_update_mana();

        assert_eq!(tribes.tribes[1].mana, 2); // Preacher generates 2 mana per tick
    }

    #[test]
    fn mana_tick_housing_generates_mana() {
        let mut pool = ObjectPool::new();
        // Create an active SmallHut (tribe 0)
        let h = pool
            .create(ModelType::Building, 1, 0, WorldCoord::default())
            .unwrap();
        if let Some(obj) = pool.get_mut(h) {
            if let GameObjectData::Building(ref mut bd) = obj.data {
                bd.state = BuildingState::Active;
                bd.building_subtype = BuildingSubtype::SmallHut;
            }
        }

        let mut tribes = TribeArray::new();
        tribes.tribes[0].active = true;

        let mut bridge = ManaTickBridge {
            pool: &pool,
            tribes: &mut tribes,
        };
        bridge.tick_update_mana();

        assert_eq!(tribes.tribes[0].mana, 1); // SmallHut (level 1) = 1 mana
    }

    #[test]
    fn mana_tick_large_hut_generates_more() {
        let mut pool = ObjectPool::new();
        let h = pool
            .create(ModelType::Building, 3, 0, WorldCoord::default())
            .unwrap();
        if let Some(obj) = pool.get_mut(h) {
            if let GameObjectData::Building(ref mut bd) = obj.data {
                bd.state = BuildingState::Active;
                bd.building_subtype = BuildingSubtype::LargeHut;
            }
        }

        let mut tribes = TribeArray::new();
        tribes.tribes[0].active = true;

        let mut bridge = ManaTickBridge {
            pool: &pool,
            tribes: &mut tribes,
        };
        bridge.tick_update_mana();

        assert_eq!(tribes.tribes[0].mana, 3); // LargeHut (level 3) = 3 mana
    }

    #[test]
    fn mana_tick_inactive_building_no_mana() {
        let mut pool = ObjectPool::new();
        // Create a SmallHut still under construction (Init state)
        let h = pool
            .create(ModelType::Building, 1, 0, WorldCoord::default())
            .unwrap();
        if let Some(obj) = pool.get_mut(h) {
            if let GameObjectData::Building(ref mut bd) = obj.data {
                bd.state = BuildingState::Init; // not active
                bd.building_subtype = BuildingSubtype::SmallHut;
            }
        }

        let mut tribes = TribeArray::new();
        tribes.tribes[0].active = true;

        let mut bridge = ManaTickBridge {
            pool: &pool,
            tribes: &mut tribes,
        };
        bridge.tick_update_mana();

        assert_eq!(tribes.tribes[0].mana, 0); // Not active = no mana
    }

    #[test]
    fn mana_tick_wild_generates_no_mana() {
        let mut pool = ObjectPool::new();
        // Wild = subtype 1, generates 0 mana
        pool.create(ModelType::Person, 1, 0, WorldCoord::default())
            .unwrap();

        let mut tribes = TribeArray::new();
        tribes.tribes[0].active = true;

        let mut bridge = ManaTickBridge {
            pool: &pool,
            tribes: &mut tribes,
        };
        bridge.tick_update_mana();

        assert_eq!(tribes.tribes[0].mana, 0);
    }

    #[test]
    fn mana_tick_multiple_tribes() {
        let mut pool = ObjectPool::new();
        // Tribe 0: 1 Brave
        pool.create(ModelType::Person, 2, 0, WorldCoord::default())
            .unwrap();
        // Tribe 1: 1 Preacher
        pool.create(ModelType::Person, 4, 1, WorldCoord::default())
            .unwrap();

        let mut tribes = TribeArray::new();
        tribes.tribes[0].active = true;
        tribes.tribes[1].active = true;

        let mut bridge = ManaTickBridge {
            pool: &pool,
            tribes: &mut tribes,
        };
        bridge.tick_update_mana();

        assert_eq!(tribes.tribes[0].mana, 1);
        assert_eq!(tribes.tribes[1].mana, 2);
    }
}
