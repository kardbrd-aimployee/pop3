pub mod handle;
pub mod types;
pub mod pool;
pub mod cell_grid;

pub use handle::ObjectHandle;
pub use types::{ObjectHeader, GameObjectData, PersonData, GameObject, PoolSlot};
pub use pool::{ObjectPool, MAX_OBJECTS};
pub use cell_grid::{CellGrid, CELL_GRID_SIZE};
