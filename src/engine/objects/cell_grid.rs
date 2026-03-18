use crate::engine::movement::{WorldCoord, TileCoord};
use crate::engine::movement::constants::REGION_GRID_SIZE;
use super::types::PoolSlot;

/// Size of the cell grid in each dimension (128x128).
pub const CELL_GRID_SIZE: usize = REGION_GRID_SIZE;

/// Total number of cells in the grid.
pub const CELL_GRID_TOTAL: usize = CELL_GRID_SIZE * CELL_GRID_SIZE;

/// A 128x128 spatial grid tracking which objects occupy each cell.
///
/// Each cell stores the handle of its first (head) object. Objects within
/// a cell form a doubly-linked list through their `next_in_cell` and
/// `prev_in_cell` fields in the ObjectHeader.
pub struct CellGrid {
    heads: Box<[Option<u16>; CELL_GRID_TOTAL]>,
}

impl CellGrid {
    /// Create a new empty cell grid.
    pub fn new() -> Self {
        Self {
            heads: Box::new([None; CELL_GRID_TOTAL]),
        }
    }

    /// Insert an object at the head of a cell's linked list.
    pub fn insert_object(&mut self, handle: u16, cell_idx: usize, slots: &mut [PoolSlot]) {
        let old_head = self.heads[cell_idx];

        // Set new object's links
        if let PoolSlot::Occupied(ref mut obj) = slots[handle as usize] {
            obj.header.next_in_cell = old_head;
            obj.header.prev_in_cell = None;
        }

        // Update old head's prev to point to new object
        if let Some(old_head_handle) = old_head {
            if let PoolSlot::Occupied(ref mut old_obj) = slots[old_head_handle as usize] {
                old_obj.header.prev_in_cell = Some(handle);
            }
        }

        // Set cell head to new object
        self.heads[cell_idx] = Some(handle);
    }

    /// Remove an object from a cell's linked list.
    pub fn remove_object(&mut self, handle: u16, cell_idx: usize, slots: &mut [PoolSlot]) {
        let (prev, next) = if let PoolSlot::Occupied(ref obj) = slots[handle as usize] {
            (obj.header.prev_in_cell, obj.header.next_in_cell)
        } else {
            return;
        };

        // Update prev's next (or cell head if removing head)
        if let Some(prev_handle) = prev {
            if let PoolSlot::Occupied(ref mut prev_obj) = slots[prev_handle as usize] {
                prev_obj.header.next_in_cell = next;
            }
        } else {
            self.heads[cell_idx] = next;
        }

        // Update next's prev
        if let Some(next_handle) = next {
            if let PoolSlot::Occupied(ref mut next_obj) = slots[next_handle as usize] {
                next_obj.header.prev_in_cell = prev;
            }
        }

        // Clear removed object's links
        if let PoolSlot::Occupied(ref mut obj) = slots[handle as usize] {
            obj.header.next_in_cell = None;
            obj.header.prev_in_cell = None;
        }
    }

    /// Update an object's cell when its position changes.
    /// No-op if the object stays in the same cell.
    pub fn set_position(
        &mut self,
        handle: u16,
        old_pos: &WorldCoord,
        new_pos: &WorldCoord,
        slots: &mut [PoolSlot],
    ) {
        let old_cell = old_pos.to_tile().cell_index();
        let new_cell = new_pos.to_tile().cell_index();
        if old_cell == new_cell {
            return;
        }
        self.remove_object(handle, old_cell, slots);
        self.insert_object(handle, new_cell, slots);
    }

    /// Reset all cells to empty.
    pub fn clear(&mut self) {
        for head in self.heads.iter_mut() {
            *head = None;
        }
    }

    /// Get the head object handle for a cell.
    pub fn cell_head(&self, cell_idx: usize) -> Option<u16> {
        self.heads[cell_idx]
    }

    /// Convenience: compute cell index from world coordinates.
    pub fn cell_index_from_world(pos: &WorldCoord) -> usize {
        pos.to_tile().cell_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::units::ModelType;
    use crate::engine::objects::types::{
        GameObject, GameObjectData, ObjectHeader, PersonData, PoolSlot,
    };

    /// Create a minimal occupied PoolSlot for testing.
    fn make_slot(handle: u16) -> PoolSlot {
        PoolSlot::Occupied(GameObject {
            header: ObjectHeader {
                model_type: ModelType::Person,
                subtype: 0,
                tribe: 0,
                state: 0,
                state_phase: 0,
                flags1: 0,
                flags2: 0,
                flags3: 0,
                object_index: handle,
                angle: 0,
                position: WorldCoord::default(),
                velocity: WorldCoord::default(),
                health: 100,
                max_health: 100,
                next_in_cell: None,
                prev_in_cell: None,
            },
            data: GameObjectData::Person(PersonData::default()),
        })
    }

    fn get_next(slots: &[PoolSlot], handle: u16) -> Option<u16> {
        match &slots[handle as usize] {
            PoolSlot::Occupied(obj) => obj.header.next_in_cell,
            _ => panic!("slot {} not occupied", handle),
        }
    }

    fn get_prev(slots: &[PoolSlot], handle: u16) -> Option<u16> {
        match &slots[handle as usize] {
            PoolSlot::Occupied(obj) => obj.header.prev_in_cell,
            _ => panic!("slot {} not occupied", handle),
        }
    }

    #[test]
    fn insert_one_object_sets_cell_head() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0)];
        let cell = 42;

        grid.insert_object(0, cell, &mut slots);

        assert_eq!(grid.cell_head(cell), Some(0));
        assert_eq!(get_next(&slots, 0), None);
        assert_eq!(get_prev(&slots, 0), None);
    }

    #[test]
    fn insert_two_objects_both_reachable() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0), make_slot(1)];
        let cell = 10;

        grid.insert_object(0, cell, &mut slots);
        grid.insert_object(1, cell, &mut slots);

        // Head should be 1 (most recently inserted)
        assert_eq!(grid.cell_head(cell), Some(1));
        // 1 -> 0 -> None
        assert_eq!(get_next(&slots, 1), Some(0));
        assert_eq!(get_next(&slots, 0), None);
        // None <- 1 <- 0
        assert_eq!(get_prev(&slots, 1), None);
        assert_eq!(get_prev(&slots, 0), Some(1));
    }

    #[test]
    fn insert_three_objects_doubly_linked_integrity() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0), make_slot(1), make_slot(2)];
        let cell = 5;

        grid.insert_object(0, cell, &mut slots);
        grid.insert_object(1, cell, &mut slots);
        grid.insert_object(2, cell, &mut slots);

        // Chain: head=2 -> 1 -> 0 -> None
        assert_eq!(grid.cell_head(cell), Some(2));
        assert_eq!(get_next(&slots, 2), Some(1));
        assert_eq!(get_next(&slots, 1), Some(0));
        assert_eq!(get_next(&slots, 0), None);
        // Reverse: None <- 2 <- 1 <- 0
        assert_eq!(get_prev(&slots, 2), None);
        assert_eq!(get_prev(&slots, 1), Some(2));
        assert_eq!(get_prev(&slots, 0), Some(1));
    }

    #[test]
    fn remove_middle_object_from_chain() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0), make_slot(1), make_slot(2)];
        let cell = 5;

        grid.insert_object(0, cell, &mut slots);
        grid.insert_object(1, cell, &mut slots);
        grid.insert_object(2, cell, &mut slots);
        // Chain: 2 -> 1 -> 0

        grid.remove_object(1, cell, &mut slots);

        // Chain should be: 2 -> 0 -> None
        assert_eq!(grid.cell_head(cell), Some(2));
        assert_eq!(get_next(&slots, 2), Some(0));
        assert_eq!(get_next(&slots, 0), None);
        assert_eq!(get_prev(&slots, 2), None);
        assert_eq!(get_prev(&slots, 0), Some(2));
        // Removed object's links cleared
        assert_eq!(get_next(&slots, 1), None);
        assert_eq!(get_prev(&slots, 1), None);
    }

    #[test]
    fn remove_head_object_updates_cell_head() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0), make_slot(1)];
        let cell = 3;

        grid.insert_object(0, cell, &mut slots);
        grid.insert_object(1, cell, &mut slots);
        // Chain: 1 -> 0

        grid.remove_object(1, cell, &mut slots);

        assert_eq!(grid.cell_head(cell), Some(0));
        assert_eq!(get_prev(&slots, 0), None);
        assert_eq!(get_next(&slots, 0), None);
    }

    #[test]
    fn remove_tail_object() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0), make_slot(1)];
        let cell = 3;

        grid.insert_object(0, cell, &mut slots);
        grid.insert_object(1, cell, &mut slots);
        // Chain: 1 -> 0

        grid.remove_object(0, cell, &mut slots);

        assert_eq!(grid.cell_head(cell), Some(1));
        assert_eq!(get_next(&slots, 1), None);
        assert_eq!(get_prev(&slots, 1), None);
    }

    #[test]
    fn remove_only_object_empties_cell() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0)];
        let cell = 7;

        grid.insert_object(0, cell, &mut slots);
        grid.remove_object(0, cell, &mut slots);

        assert_eq!(grid.cell_head(cell), None);
        assert_eq!(get_next(&slots, 0), None);
        assert_eq!(get_prev(&slots, 0), None);
    }

    #[test]
    fn set_position_same_cell_is_noop() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0)];

        // Two positions that map to the same cell
        let pos1 = WorldCoord::new(256, 256);
        let pos2 = WorldCoord::new(300, 300);
        assert_eq!(
            pos1.to_tile().cell_index(),
            pos2.to_tile().cell_index(),
            "positions must map to same cell for this test"
        );

        let cell = pos1.to_tile().cell_index();
        grid.insert_object(0, cell, &mut slots);

        // set_position should not modify anything
        grid.set_position(0, &pos1, &pos2, &mut slots);

        assert_eq!(grid.cell_head(cell), Some(0));
        assert_eq!(get_next(&slots, 0), None);
        assert_eq!(get_prev(&slots, 0), None);
    }

    #[test]
    fn set_position_different_cell_moves_object() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0)];

        // Two positions in different cells
        let pos1 = WorldCoord::new(256, 256);
        let pos2 = WorldCoord::new(256 + 512, 256);
        let cell1 = pos1.to_tile().cell_index();
        let cell2 = pos2.to_tile().cell_index();
        assert_ne!(cell1, cell2, "positions must map to different cells");

        grid.insert_object(0, cell1, &mut slots);
        grid.set_position(0, &pos1, &pos2, &mut slots);

        assert_eq!(grid.cell_head(cell1), None);
        assert_eq!(grid.cell_head(cell2), Some(0));
    }

    #[test]
    fn cell_objects_iteration_yields_all_objects() {
        let mut grid = CellGrid::new();
        let mut slots = vec![make_slot(0), make_slot(1), make_slot(2)];
        let cell = 20;

        grid.insert_object(0, cell, &mut slots);
        grid.insert_object(1, cell, &mut slots);
        grid.insert_object(2, cell, &mut slots);

        // Walk the linked list manually
        let mut collected = Vec::new();
        let mut current = grid.cell_head(cell);
        while let Some(handle) = current {
            collected.push(handle);
            current = get_next(&slots, handle);
        }

        assert_eq!(collected, vec![2, 1, 0]);
    }

    #[test]
    fn empty_cell_iteration_yields_nothing() {
        let grid = CellGrid::new();
        assert_eq!(grid.cell_head(0), None);
    }

    #[test]
    fn cell_index_from_world_matches_manual() {
        let pos = WorldCoord::new(512, 1024);
        let expected = pos.to_tile().cell_index();
        assert_eq!(CellGrid::cell_index_from_world(&pos), expected);
    }
}
