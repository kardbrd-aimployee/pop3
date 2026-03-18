pub mod mana;
pub mod population;
pub mod wood;

pub use mana::{mana_rate_for_person, mana_rate_for_housing, add_mana, deduct_mana, MAX_MANA};
pub use population::{calculate_housing_capacity, hut_capacity, can_spawn, MAX_POP_VALUE};
pub use wood::{construction_wood_cost, training_wood_cost, total_wood_stored, find_nearest_tree_position, find_nearest_building_position};
