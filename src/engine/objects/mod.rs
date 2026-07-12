pub mod cell_grid;
pub mod handle;
pub mod pool;
pub mod types;

pub use crate::engine::buildings::BuildingData;
pub use cell_grid::{CellGrid, CELL_GRID_SIZE};
pub use handle::ObjectHandle;
pub use pool::{ObjectPool, MAX_OBJECTS};
pub use types::{GameObject, GameObjectData, ObjectHeader, PersonData, PoolSlot};
