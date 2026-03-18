use super::types::*;
use crate::engine::objects::handle::ObjectHandle;

/// Add an occupant to the building. Returns Ok(slot_index) on success,
/// Err(()) if the building is full (occupant_count >= MAX_OCCUPANTS).
pub fn add_occupant(building: &mut BuildingData, handle: ObjectHandle) -> Result<usize, ()> {
    if is_full(building) {
        return Err(());
    }
    for (i, slot) in building.occupant_slots.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(handle);
            building.occupant_count += 1;
            return Ok(i);
        }
    }
    Err(())
}

/// Remove a specific occupant by handle. Returns true if found and removed.
pub fn remove_occupant(building: &mut BuildingData, handle: ObjectHandle) -> bool {
    if let Some(idx) = find_occupant_slot(building, handle) {
        building.occupant_slots[idx] = None;
        building.occupant_count -= 1;
        true
    } else {
        false
    }
}

/// Eject occupant from a specific slot. Returns the handle if occupied.
pub fn eject_occupant(building: &mut BuildingData, slot: usize) -> Option<ObjectHandle> {
    if slot >= MAX_OCCUPANTS {
        return None;
    }
    let handle = building.occupant_slots[slot].take();
    if handle.is_some() {
        building.occupant_count -= 1;
    }
    handle
}

/// Check if the building is at maximum occupancy.
pub fn is_full(building: &BuildingData) -> bool {
    building.occupant_count as usize >= MAX_OCCUPANTS
}

/// Find which slot contains the given handle.
pub fn find_occupant_slot(building: &BuildingData, handle: ObjectHandle) -> Option<usize> {
    building
        .occupant_slots
        .iter()
        .position(|slot| *slot == Some(handle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_occupant_to_empty_building() {
        let mut b = BuildingData::default();
        let result = add_occupant(&mut b, 42);
        assert_eq!(result, Ok(0));
        assert_eq!(b.occupant_count, 1);
        assert_eq!(b.occupant_slots[0], Some(42));
    }

    #[test]
    fn add_occupant_fills_first_empty_slot() {
        let mut b = BuildingData::default();
        add_occupant(&mut b, 10).unwrap();
        add_occupant(&mut b, 20).unwrap();
        assert_eq!(b.occupant_slots[0], Some(10));
        assert_eq!(b.occupant_slots[1], Some(20));
        assert_eq!(b.occupant_count, 2);
    }

    #[test]
    fn add_occupant_fills_gap() {
        let mut b = BuildingData::default();
        add_occupant(&mut b, 10).unwrap();
        add_occupant(&mut b, 20).unwrap();
        add_occupant(&mut b, 30).unwrap();
        remove_occupant(&mut b, 20); // clears slot 1
        let result = add_occupant(&mut b, 40);
        assert_eq!(result, Ok(1)); // reuses slot 1
        assert_eq!(b.occupant_count, 3);
    }

    #[test]
    fn add_occupant_fails_when_full() {
        let mut b = BuildingData::default();
        for i in 0..6u16 {
            assert!(add_occupant(&mut b, i).is_ok());
        }
        assert!(is_full(&b));
        assert_eq!(add_occupant(&mut b, 99), Err(()));
    }

    #[test]
    fn remove_occupant_by_handle() {
        let mut b = BuildingData::default();
        add_occupant(&mut b, 42).unwrap();
        add_occupant(&mut b, 99).unwrap();
        assert!(remove_occupant(&mut b, 42));
        assert_eq!(b.occupant_count, 1);
        assert!(b.occupant_slots[0].is_none());
        assert_eq!(b.occupant_slots[1], Some(99));
    }

    #[test]
    fn remove_nonexistent_occupant_returns_false() {
        let mut b = BuildingData::default();
        add_occupant(&mut b, 42).unwrap();
        assert!(!remove_occupant(&mut b, 99));
        assert_eq!(b.occupant_count, 1);
    }

    #[test]
    fn eject_occupant_from_slot() {
        let mut b = BuildingData::default();
        add_occupant(&mut b, 42).unwrap();
        add_occupant(&mut b, 99).unwrap();
        let ejected = eject_occupant(&mut b, 0);
        assert_eq!(ejected, Some(42));
        assert_eq!(b.occupant_count, 1);
        assert!(b.occupant_slots[0].is_none());
    }

    #[test]
    fn eject_empty_slot_returns_none() {
        let mut b = BuildingData::default();
        assert_eq!(eject_occupant(&mut b, 0), None);
        assert_eq!(b.occupant_count, 0);
    }

    #[test]
    fn eject_out_of_bounds_returns_none() {
        let mut b = BuildingData::default();
        assert_eq!(eject_occupant(&mut b, 10), None);
    }

    #[test]
    fn find_occupant_slot_returns_index() {
        let mut b = BuildingData::default();
        add_occupant(&mut b, 42).unwrap();
        add_occupant(&mut b, 99).unwrap();
        assert_eq!(find_occupant_slot(&b, 42), Some(0));
        assert_eq!(find_occupant_slot(&b, 99), Some(1));
        assert_eq!(find_occupant_slot(&b, 1), None);
    }

    #[test]
    fn is_full_check() {
        let mut b = BuildingData::default();
        assert!(!is_full(&b));
        for i in 0..6u16 {
            add_occupant(&mut b, i).unwrap();
        }
        assert!(is_full(&b));
    }
}
