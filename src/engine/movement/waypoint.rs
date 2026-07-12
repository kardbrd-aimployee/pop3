// Per-tick waypoint stepping — advances a unit through route segment waypoints.
// Original: ProcessRouteMovement @ 0x4d8e60 (~3KB)
//
// This function does NOT call MovePointByAngle. It only updates the
// next_waypoint field. The actual position advancement is done by
// Object_UpdateMovement (0x4ed510) which calls MovePointByAngle.
//
// Sequence per tick:
//   ProcessRouteMovement()  → updates next_waypoint (this function)
//   Object_UpdateMovement() → calls MovePointByAngle to advance position

use super::constants::*;
use super::segment::SegmentPool;
use super::types::PersonMovement;

/// Result of processing route movement for one tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaypointResult {
    /// No active segment — nothing to do
    NoSegment,
    /// Still moving toward current waypoint
    InProgress,
    /// Advanced to next waypoint in segment
    Advanced,
    /// Reached final waypoint — segment released
    Completed,
}

/// Per-tick waypoint stepping.
/// Original: ProcessRouteMovement @ 0x4d8e60
///
/// Checks distance to current waypoint, advances when close enough,
/// releases segment when final waypoint is reached.
pub fn process_route_movement(
    segment_pool: &mut SegmentPool,
    person: &mut PersonMovement,
) -> WaypointResult {
    // Phase 1: Check if unit has an active segment
    if person.segment_index == 0 {
        return WaypointResult::NoSegment;
    }

    let seg_idx = person.segment_index;

    // Load current waypoint from segment
    let (waypoint_count, current_wp) = {
        let seg = match segment_pool.get(seg_idx) {
            Some(s) => s,
            None => {
                // Invalid segment index — clear it
                person.segment_index = 0;
                person.waypoint_idx = 0;
                return WaypointResult::NoSegment;
            }
        };
        let wp_count = seg.waypoint_count;
        let wp = if person.waypoint_idx < wp_count {
            seg.get_waypoint_world(person.waypoint_idx)
        } else {
            // Index beyond waypoint count — treat as completed
            None
        };
        (wp_count, wp)
    };

    let current_wp = match current_wp {
        Some(wp) => wp,
        None => {
            // No valid waypoint — complete the segment
            release_segment(segment_pool, person);
            return WaypointResult::Completed;
        }
    };

    // Phase 2: Compute distance to current waypoint
    let dx = (person.position.x as i32 - current_wp.x as i32).abs();
    let dz = (person.position.z as i32 - current_wp.z as i32).abs();

    // Handle toroidal wrapping for distance
    let dx = if dx > WORLD_WRAP_THRESHOLD {
        WORLD_SIZE - dx
    } else {
        dx
    };
    let dz = if dz > WORLD_WRAP_THRESHOLD {
        WORLD_SIZE - dz
    } else {
        dz
    };

    // Phase 3: Check arrival threshold
    // Original: 0x240 for normal paths, 0xE0 for building entrances
    // TODO: Building entrance detection needs GetBuildingEntrance @ 0x42F850
    let threshold = WAYPOINT_ARRIVAL_THRESHOLD;

    if dx > threshold || dz > threshold {
        // Not close enough — keep moving toward current waypoint
        return WaypointResult::InProgress;
    }

    // Phase 4: Waypoint reached — advance or complete
    let next_idx = person.waypoint_idx + 1;

    if next_idx >= waypoint_count {
        // Last waypoint reached — release segment, switch to direct walk
        release_segment(segment_pool, person);
        WaypointResult::Completed
    } else {
        // Advance to next waypoint
        person.waypoint_idx = next_idx;

        // Extract next waypoint coordinates
        if let Some(next_wp) = segment_pool.get_waypoint(seg_idx, next_idx) {
            person.next_waypoint = next_wp;
            person.movement_dest = next_wp;
        }

        // Set movement flags
        person.set_goto_flags();

        WaypointResult::Advanced
    }
}

/// Release a segment when the final waypoint is reached.
/// Original: segment lifecycle in ProcessRouteMovement @ 0x4d8e60
///
/// Decrements ref_count. If ref_count hits 0 and segment is NOT persistent
/// (flag bit 2), the slot is freed. Copies target_pos to next_waypoint
/// so the unit walks the final stretch directly.
fn release_segment(segment_pool: &mut SegmentPool, person: &mut PersonMovement) {
    let seg_idx = person.segment_index;

    // Decrement reference count
    let should_deactivate = if let Some(seg) = segment_pool.get_mut(seg_idx) {
        seg.ref_count -= 1;
        if seg.ref_count <= 0 {
            // If not persistent, fully clear the segment
            if seg.flags & SEGMENT_FLAG_PERSISTENT == 0 {
                seg.ref_count = 0;
            }
            true
        } else {
            false
        }
    } else {
        false
    };
    if should_deactivate {
        segment_pool.active_count -= 1;
    }

    // Clear unit's segment assignment
    person.segment_index = 0;
    person.waypoint_idx = 0;

    // Switch to direct walk toward final target
    person.next_waypoint = person.target_pos;
    person.movement_dest = person.target_pos;

    // Set movement flags
    person.set_goto_flags();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::movement::types::{TileCoord, WorldCoord};

    fn setup_segment_pool_with_path(waypoints: &[(u8, u8)]) -> (SegmentPool, u16) {
        let mut pool = SegmentPool::new();
        let idx = pool.allocate().unwrap();
        pool.add_ref(idx);

        let seg = pool.get_mut(idx).unwrap();
        seg.start_tile_x = waypoints[0].0;
        seg.start_tile_z = waypoints[0].1;
        if let Some(last) = waypoints.last() {
            seg.end_tile_x = last.0;
            seg.end_tile_z = last.1;
        }
        seg.waypoint_count = waypoints.len() as u8;
        for (i, &(tx, tz)) in waypoints.iter().enumerate() {
            seg.waypoints[i].tile_x = tx;
            seg.waypoints[i].tile_z = tz;
        }

        (pool, idx)
    }

    #[test]
    fn no_segment_returns_immediately() {
        let mut pool = SegmentPool::new();
        let mut person = PersonMovement::default();
        person.segment_index = 0;

        let result = process_route_movement(&mut pool, &mut person);
        assert_eq!(result, WaypointResult::NoSegment);
    }

    #[test]
    fn in_progress_when_far_from_waypoint() {
        let (mut pool, idx) = setup_segment_pool_with_path(&[(0x10, 0x10), (0x30, 0x30)]);

        let mut person = PersonMovement::default();
        person.segment_index = idx;
        person.waypoint_idx = 0;
        // Position far from first waypoint
        person.position = WorldCoord::new(0x0100, 0x0100);

        let result = process_route_movement(&mut pool, &mut person);
        assert_eq!(result, WaypointResult::InProgress);
        assert_eq!(person.waypoint_idx, 0); // Not advanced
    }

    #[test]
    fn advances_when_close_to_waypoint() {
        let (mut pool, idx) = setup_segment_pool_with_path(&[(0x10, 0x10), (0x30, 0x30)]);

        let mut person = PersonMovement::default();
        person.segment_index = idx;
        person.waypoint_idx = 0;
        // Position very close to first waypoint center
        let wp0 = TileCoord::new(0x10, 0x10).to_world();
        person.position = WorldCoord::new(wp0.x + 10, wp0.z + 10);

        let result = process_route_movement(&mut pool, &mut person);
        assert_eq!(result, WaypointResult::Advanced);
        assert_eq!(person.waypoint_idx, 1);
        // next_waypoint should now be the second waypoint
        let wp1 = TileCoord::new(0x30, 0x30).to_world();
        assert_eq!(person.next_waypoint, wp1);
    }

    #[test]
    fn completes_at_final_waypoint() {
        let (mut pool, idx) = setup_segment_pool_with_path(&[(0x10, 0x10), (0x30, 0x30)]);

        let mut person = PersonMovement::default();
        person.segment_index = idx;
        person.waypoint_idx = 1; // Already at last waypoint
        person.target_pos = WorldCoord::new(0x5000, 0x6000);

        // Position close to last waypoint
        let wp1 = TileCoord::new(0x30, 0x30).to_world();
        person.position = WorldCoord::new(wp1.x + 5, wp1.z + 5);

        let result = process_route_movement(&mut pool, &mut person);
        assert_eq!(result, WaypointResult::Completed);
        assert_eq!(person.segment_index, 0);
        assert_eq!(person.waypoint_idx, 0);
        // next_waypoint should be the final target
        assert_eq!(person.next_waypoint, person.target_pos);
    }

    #[test]
    fn segment_ref_count_decremented_on_complete() {
        let (mut pool, idx) = setup_segment_pool_with_path(&[(0x10, 0x10)]);
        // Add an extra reference (simulating shared segment)
        pool.add_ref(idx);
        assert_eq!(pool.get(idx).unwrap().ref_count, 2);

        let mut person = PersonMovement::default();
        person.segment_index = idx;
        person.waypoint_idx = 0;
        person.target_pos = WorldCoord::new(0x5000, 0x6000);

        // Position close to the single waypoint
        let wp0 = TileCoord::new(0x10, 0x10).to_world();
        person.position = WorldCoord::new(wp0.x, wp0.z);

        process_route_movement(&mut pool, &mut person);
        // Ref count should be decremented by 1 (2 → 1)
        assert_eq!(pool.get(idx).unwrap().ref_count, 1);
    }

    #[test]
    fn single_waypoint_segment_completes() {
        let (mut pool, idx) = setup_segment_pool_with_path(&[(0x10, 0x10)]);

        let mut person = PersonMovement::default();
        person.segment_index = idx;
        person.waypoint_idx = 0;
        person.target_pos = WorldCoord::new(0x5000, 0x6000);

        let wp0 = TileCoord::new(0x10, 0x10).to_world();
        person.position = WorldCoord::new(wp0.x, wp0.z);

        let result = process_route_movement(&mut pool, &mut person);
        assert_eq!(result, WaypointResult::Completed);
    }

    #[test]
    fn multi_waypoint_full_traversal() {
        let (mut pool, idx) =
            setup_segment_pool_with_path(&[(0x10, 0x10), (0x20, 0x20), (0x30, 0x30)]);

        let mut person = PersonMovement::default();
        person.segment_index = idx;
        person.waypoint_idx = 0;
        person.target_pos = WorldCoord::new(0x5000, 0x6000);

        // Walk to waypoint 0
        let wp0 = TileCoord::new(0x10, 0x10).to_world();
        person.position = WorldCoord::new(wp0.x, wp0.z);
        assert_eq!(
            process_route_movement(&mut pool, &mut person),
            WaypointResult::Advanced
        );
        assert_eq!(person.waypoint_idx, 1);

        // Walk to waypoint 1
        let wp1 = TileCoord::new(0x20, 0x20).to_world();
        person.position = WorldCoord::new(wp1.x, wp1.z);
        assert_eq!(
            process_route_movement(&mut pool, &mut person),
            WaypointResult::Advanced
        );
        assert_eq!(person.waypoint_idx, 2);

        // Walk to waypoint 2 (last)
        let wp2 = TileCoord::new(0x30, 0x30).to_world();
        person.position = WorldCoord::new(wp2.x, wp2.z);
        assert_eq!(
            process_route_movement(&mut pool, &mut person),
            WaypointResult::Completed
        );
        assert_eq!(person.segment_index, 0);
        assert_eq!(person.next_waypoint, person.target_pos);
    }
}
