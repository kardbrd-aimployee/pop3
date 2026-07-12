// Unit control system — selection, movement orders, and per-tick updates.
//
// Provides click-to-select, right-click-to-move, and drag-box multi-select
// for person units, wired to the movement system's pathfinding.

pub mod animation;
pub mod coordinator;
pub mod coords;
pub mod person_state;
pub mod selection;
pub mod unit;

pub use coordinator::UnitCoordinator;
pub use coords::{cell_to_world, gpu_to_cell, world_to_cell};
pub use selection::{find_unit_at_cell, DragState, SelectionState};
pub use unit::{Unit, UnitId};
