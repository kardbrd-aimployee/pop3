pub mod types;
pub mod state_machine;
pub mod occupants;
pub mod tick;
pub mod spawning;
pub mod training;

pub use types::{BuildingData, BuildingState, BuildingSubtype, MAX_OCCUPANTS};
pub use state_machine::{transition_building_state, on_construction_complete, on_destroy};
pub use occupants::{add_occupant, remove_occupant, eject_occupant, is_full};
pub use spawning::{tick_spawn, SpawnAction, sprog_time_for_level};
pub use training::{tick_convert, start_training, ConvertAction, training_output_subtype, training_mana_cost};
