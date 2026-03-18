use crate::data::units::ModelType;
use crate::engine::movement::{PersonMovement, WorldCoord};
use crate::engine::units::person_state::PersonState;
use crate::engine::units::animation::AnimationState;
use crate::engine::buildings::BuildingData;
use super::handle::ObjectHandle;

/// Common header fields for all game objects.
/// Matches the original binary's object layout (offsets 0x00..0x70).
#[derive(Debug, Clone)]
pub struct ObjectHeader {
    pub model_type: ModelType,
    pub subtype: u8,
    pub tribe: u8,
    pub state: u8,
    pub state_phase: u8,
    pub flags1: u32,
    pub flags2: u32,
    pub flags3: u32,
    pub object_index: ObjectHandle,
    pub angle: u16,
    pub position: WorldCoord,
    pub velocity: WorldCoord,
    pub health: u16,
    pub max_health: u16,
    pub next_in_cell: Option<u16>,
    pub prev_in_cell: Option<u16>,
}

/// Type-specific data for each object kind.
pub enum GameObjectData {
    Person(PersonData),
    Building(BuildingData),
    Creature(()),
    Vehicle(()),
    Scenery(()),
    General(()),
    Effect(()),
    Shot(()),
    Shape(()),
    Internal(()),
    Spell(()),
}

/// Person-specific data, extracted from the current Unit struct.
/// Contains all fields that are not shared across object types.
#[derive(Debug, Clone)]
pub struct PersonData {
    pub movement: PersonMovement,
    pub anim: AnimationState,
    pub state: PersonState,
    pub prev_state: PersonState,
    pub state_timer: u16,
    pub state_counter: u8,
    pub target_unit: Option<ObjectHandle>,
    pub attacker_unit: Option<ObjectHandle>,
    pub alive: bool,
    pub home_pos: WorldCoord,
    pub behavior_flags: u16,
    pub wander_duration: u8,
    pub wander_range: u8,
    pub linked_obj_id: Option<ObjectHandle>,
    pub bloodlust: bool,
    pub shielded: bool,
    pub cell_x: f32,
    pub cell_y: f32,
}

impl Default for PersonData {
    fn default() -> Self {
        Self {
            movement: PersonMovement::default(),
            anim: AnimationState::default(),
            state: PersonState::default(),
            prev_state: PersonState::default(),
            state_timer: 0,
            state_counter: 0,
            target_unit: None,
            attacker_unit: None,
            alive: true,
            home_pos: WorldCoord::default(),
            behavior_flags: 0,
            wander_duration: 0,
            wander_range: 0,
            linked_obj_id: None,
            bloodlust: false,
            shielded: false,
            cell_x: 0.0,
            cell_y: 0.0,
        }
    }
}

/// A complete game object: shared header + type-specific data.
pub struct GameObject {
    pub header: ObjectHeader,
    pub data: GameObjectData,
}

/// A slot in the object pool: either occupied by a game object, or free
/// and linked into the appropriate free list.
pub enum PoolSlot {
    Occupied(GameObject),
    Free { next_free: Option<u16> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn object_header_construction() {
        let header = ObjectHeader {
            model_type: ModelType::Person,
            subtype: 2,
            tribe: 0,
            state: 1,
            state_phase: 0,
            flags1: 0x1000,
            flags2: 0,
            flags3: 0,
            object_index: 42,
            angle: 512,
            position: WorldCoord::new(100, 200),
            velocity: WorldCoord::default(),
            health: 500,
            max_health: 1000,
            next_in_cell: Some(10),
            prev_in_cell: None,
        };
        assert_eq!(header.model_type, ModelType::Person);
        assert_eq!(header.subtype, 2);
        assert_eq!(header.tribe, 0);
        assert_eq!(header.state, 1);
        assert_eq!(header.flags1, 0x1000);
        assert_eq!(header.object_index, 42);
        assert_eq!(header.angle, 512);
        assert_eq!(header.position, WorldCoord::new(100, 200));
        assert_eq!(header.health, 500);
        assert_eq!(header.max_health, 1000);
        assert_eq!(header.next_in_cell, Some(10));
        assert_eq!(header.prev_in_cell, None);
    }

    #[test]
    fn person_data_in_game_object_data() {
        let person = PersonData::default();
        let data = GameObjectData::Person(person);
        match data {
            GameObjectData::Person(p) => {
                assert!(p.alive);
                assert_eq!(p.state_timer, 0);
                assert_eq!(p.cell_x, 0.0);
            }
            _ => panic!("Expected Person variant"),
        }
    }

    #[test]
    fn game_object_data_has_11_variants() {
        // Ensure all 11 variants can be constructed
        let variants: Vec<GameObjectData> = vec![
            GameObjectData::Person(PersonData::default()),
            GameObjectData::Building(BuildingData::default()),
            GameObjectData::Creature(()),
            GameObjectData::Vehicle(()),
            GameObjectData::Scenery(()),
            GameObjectData::General(()),
            GameObjectData::Effect(()),
            GameObjectData::Shot(()),
            GameObjectData::Shape(()),
            GameObjectData::Internal(()),
            GameObjectData::Spell(()),
        ];
        assert_eq!(variants.len(), 11);
    }

    #[test]
    fn pool_slot_free_and_occupied() {
        let free_slot = PoolSlot::Free { next_free: Some(42) };
        match free_slot {
            PoolSlot::Free { next_free } => assert_eq!(next_free, Some(42)),
            _ => panic!("Expected Free variant"),
        }

        let obj = GameObject {
            header: ObjectHeader {
                model_type: ModelType::Person,
                subtype: 2,
                tribe: 0,
                state: 0,
                state_phase: 0,
                flags1: 0,
                flags2: 0,
                flags3: 0,
                object_index: 0,
                angle: 0,
                position: WorldCoord::default(),
                velocity: WorldCoord::default(),
                health: 0,
                max_health: 0,
                next_in_cell: None,
                prev_in_cell: None,
            },
            data: GameObjectData::Person(PersonData::default()),
        };
        let occupied_slot = PoolSlot::Occupied(obj);
        match occupied_slot {
            PoolSlot::Occupied(go) => assert_eq!(go.header.model_type, ModelType::Person),
            _ => panic!("Expected Occupied variant"),
        }
    }

    #[test]
    fn pool_slot_size_reasonable() {
        let size = std::mem::size_of::<PoolSlot>();
        println!("PoolSlot size: {} bytes", size);
        assert!(size < 1024, "PoolSlot should be less than 1KB, got {} bytes", size);
    }

    #[test]
    fn object_handle_is_u16() {
        let handle: ObjectHandle = 42u16;
        assert_eq!(std::mem::size_of::<ObjectHandle>(), std::mem::size_of::<u16>());
        assert_eq!(handle, 42u16);
    }
}
