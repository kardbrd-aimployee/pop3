// Selection state and hit-testing for unit control.

use cgmath::Point2;
use super::unit::{Unit, UnitId};

pub struct SelectionState {
    pub selected: Vec<UnitId>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self { selected: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.selected.clear();
    }

    pub fn select_single(&mut self, id: UnitId) {
        self.selected.clear();
        self.selected.push(id);
    }

    pub fn select_multiple(&mut self, ids: Vec<UnitId>) {
        self.selected = ids;
    }

    pub fn is_selected(&self, id: UnitId) -> bool {
        self.selected.contains(&id)
    }
}

/// Drag-box state machine for rubber-band multi-select.
pub enum DragState {
    None,
    /// Left button pressed — not yet dragging (waiting for threshold).
    PendingDrag { start: Point2<f32> },
    /// Actively dragging — rubber band visible.
    Dragging { start: Point2<f32>, current: Point2<f32> },
}

/// Find the nearest unit to a cell-space position within `threshold` distance.
pub fn find_unit_at_cell(units: &[Unit], cell_x: f32, cell_y: f32, threshold: f32) -> Option<UnitId> {
    let threshold_sq = threshold * threshold;
    let mut best: Option<(UnitId, f32)> = None;
    for unit in units {
        let dx = unit.cell_x - cell_x;
        let dy = unit.cell_y - cell_y;
        let dist_sq = dx * dx + dy * dy;
        if dist_sq < threshold_sq {
            if best.map_or(true, |(_, d)| dist_sq < d) {
                best = Some((unit.id, dist_sq));
            }
        }
    }
    best.map(|(id, _)| id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::movement::PersonMovement;
    use crate::data::units::ModelType;

    fn make_unit(id: usize, cx: f32, cy: f32) -> Unit {
        use super::super::person_state::PersonState;
        use crate::engine::movement::WorldCoord;
        Unit {
            id,
            model_type: ModelType::Person,
            subtype: 2,
            tribe_index: 0,
            movement: PersonMovement::default(),
            cell_x: cx,
            cell_y: cy,
            state: PersonState::Idle,
            prev_state: PersonState::Idle,
            state_timer: 0,
            state_counter: 0,
            health: 1400,
            max_health: 1400,
            target_unit: None,
            attacker_unit: None,
            alive: true,
            home_pos: WorldCoord::new(0, 0),
            behavior_flags: 0,
            wander_duration: 0,
            wander_range: 0,
            linked_obj_id: None,
            bloodlust: false,
            shielded: false,
            anim: super::super::animation::AnimationState::default(),
            building_handle: None,
            wood_carried: 0,
            guard_position: None,
        }
    }

    #[test]
    fn find_nearest_unit() {
        let units = vec![
            make_unit(0, 10.0, 20.0),
            make_unit(1, 15.0, 20.0),
            make_unit(2, 50.0, 50.0),
        ];
        // Click near unit 1
        assert_eq!(find_unit_at_cell(&units, 14.5, 20.0, 2.0), Some(1));
        // Click near unit 0
        assert_eq!(find_unit_at_cell(&units, 10.2, 20.1, 2.0), Some(0));
        // Click far from any unit
        assert_eq!(find_unit_at_cell(&units, 80.0, 80.0, 2.0), None);
    }

    #[test]
    fn selection_state_basics() {
        let mut sel = SelectionState::new();
        assert!(sel.selected.is_empty());

        sel.select_single(5);
        assert!(sel.is_selected(5));
        assert!(!sel.is_selected(3));

        sel.select_multiple(vec![1, 2, 3]);
        assert!(sel.is_selected(2));
        assert!(!sel.is_selected(5));

        sel.clear();
        assert!(sel.selected.is_empty());
    }
}
