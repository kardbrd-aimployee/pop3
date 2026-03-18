pub mod types;
pub mod state_machine;
pub mod occupants;
pub mod tick;

pub use types::{BuildingData, BuildingState, BuildingSubtype, MAX_OCCUPANTS};
pub use state_machine::{transition_building_state, on_construction_complete, on_destroy};
pub use occupants::{add_occupant, remove_occupant, eject_occupant, is_full};
