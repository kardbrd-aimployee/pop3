pub mod modify;
pub mod cascade;
pub mod water;
#[cfg(test)]
mod tests;

pub use modify::{modify_height, modify_height_area, flatten_area};
pub use cascade::{terrain_cascade, CascadeRegion, CascadeResult, invalidate_segments_in_region};
pub use water::{update_water_cells, is_water_cell, WATER_WALKABILITY_FLAG};
