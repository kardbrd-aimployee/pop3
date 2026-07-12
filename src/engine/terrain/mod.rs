pub mod cascade;
pub mod modify;
#[cfg(test)]
mod tests;
pub mod water;

pub use cascade::{invalidate_segments_in_region, terrain_cascade, CascadeRegion, CascadeResult};
pub use modify::{flatten_area, modify_height, modify_height_area};
pub use water::{is_water_cell, update_water_cells, WATER_WALKABILITY_FLAG};
