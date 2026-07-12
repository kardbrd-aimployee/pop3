pub mod catalog;
pub mod combat;
pub mod damage;
pub mod occupants;
pub mod placement;
pub mod spawning;
pub mod state_machine;
pub mod tick;
pub mod training;
pub mod types;

pub use catalog::BuildingCatalog;
pub use combat::{
    set_building_target, set_fighters, tick_building_combat, BuildingCombatAction,
    MAX_BUILDING_FIGHTERS,
};
pub use damage::{apply_building_damage, chain_damage_radius};
pub use occupants::{add_occupant, eject_occupant, is_full, remove_occupant};
pub use placement::{validate_placement, GhostPreview, PlacementError};
pub use spawning::{sprog_time_for_level, tick_spawn, SpawnAction};
pub use state_machine::{on_construction_complete, on_destroy, transition_building_state};
pub use training::{
    start_training, tick_convert, training_mana_cost, training_output_subtype, ConvertAction,
};
pub use types::{BuildingData, BuildingState, BuildingSubtype, MAX_OCCUPANTS};
