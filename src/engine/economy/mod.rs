pub mod mana;
pub mod population;
pub mod wood;

pub use mana::{add_mana, deduct_mana, mana_rate_for_housing, mana_rate_for_person, MAX_MANA};
pub use population::{calculate_housing_capacity, can_spawn, hut_capacity, MAX_POP_VALUE};
pub use wood::{
    construction_wood_cost, find_nearest_building_position, find_nearest_tree_position,
    total_wood_stored, training_wood_cost,
};
