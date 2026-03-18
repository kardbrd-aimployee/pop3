pub mod handle;
pub mod types;
pub mod pool;

pub use handle::ObjectHandle;
pub use types::{ObjectHeader, GameObjectData, PersonData, GameObject, PoolSlot};
pub use pool::ObjectPool;
