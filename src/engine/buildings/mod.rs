pub mod types;
pub mod state_machine;
pub mod occupants;
pub mod tick;
pub mod spawning;
pub mod training;
pub mod placement;
pub mod damage;
pub mod combat;

pub use types::{BuildingData, BuildingState, BuildingSubtype, MAX_OCCUPANTS};
pub use state_machine::{transition_building_state, on_construction_complete, on_destroy};
pub use occupants::{add_occupant, remove_occupant, eject_occupant, is_full};
pub use spawning::{tick_spawn, SpawnAction, sprog_time_for_level};
pub use training::{tick_convert, start_training, ConvertAction, training_output_subtype, training_mana_cost};
pub use placement::{validate_placement, PlacementError, GhostPreview};
pub use damage::{apply_building_damage, chain_damage_radius};
pub use combat::{tick_building_combat, set_fighters, set_building_target, BuildingCombatAction, MAX_BUILDING_FIGHTERS};
