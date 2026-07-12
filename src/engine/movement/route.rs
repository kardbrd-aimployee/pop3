// Route lookup — the 4-tier pathfinding cache system.
// Original: RouteTableLookup @ 0x4d7f20
//
// Tier 1: Region map — same region → direct walk (O(1))
// Tier 2: Segment pool — reuse existing path (O(N) scan of 400 slots)
// Tier 3: Failure cache — recently failed? Skip A* (O(8))
// Tier 4: A* pathfinder — compute new path (expensive, deferred)
//
// Returns a segment index (0 = direct walk, >0 = follow segment waypoints).

use super::constants::*;
use super::region::{spiral_neighbors, RegionMap};
use super::segment::{FailureCache, SegmentPool};
use super::types::{PersonMovement, TileCoord, UsedTargetsCache, WorldCoord};

/// Result of a route lookup.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteResult {
    /// Same region: walk directly to target (segment_index = 0)
    DirectWalk,
    /// Existing or new segment found — follow its waypoints
    Segment(u16),
    /// No route found (all tiers exhausted)
    NoRoute,
}

/// Adjusts a target position to the nearest walkable cell.
/// Original: AdjustTargetForWalkability @ 0x4da480
///
/// Algorithm:
/// 1. Check if target tile is already walkable → return immediately (common case)
/// 2. Spiral search up to 32 neighbors for a walkable tile
/// 3. Skip tiles already in the used-targets cache (prevents pile-ups)
/// 4. Snap target to the walkable tile center, record in cache
pub fn adjust_target_for_walkability(
    region_map: &RegionMap,
    _person: &PersonMovement,
    target: &mut WorldCoord,
    used_targets: &mut UsedTargetsCache,
) {
    let target_tile = target.to_tile();

    // Fast path: target is already walkable
    if region_map.is_walkable(target_tile) {
        return;
    }

    // Slow path: spiral search for nearest walkable tile
    let offsets = spiral_neighbors(MAX_WALKABILITY_SEARCH);

    for (dx, dz) in offsets {
        // Compute candidate tile: offset × 2 (tile coords step by 2)
        let cx = (target_tile.x as i16) + (dx as i16) * 2;
        let cz = (target_tile.z as i16) + (dz as i16) * 2;

        // Bounds check (0..254, step 2)
        if !(0..=254).contains(&cx) || !(0..=254).contains(&cz) {
            continue;
        }

        let candidate = TileCoord::new((cx as u8) & 0xFE, (cz as u8) & 0xFE);

        // Check walkability
        if !region_map.is_walkable(candidate) {
            continue;
        }

        // Check used-targets cache (avoid pile-ups)
        if used_targets.contains(candidate) {
            continue;
        }

        // Found a walkable, unused tile — snap target to its center
        *target = candidate.to_world();
        used_targets.record(candidate);
        return;
    }

    // No walkable neighbor found — leave target unchanged
}

/// Core routing function. 4-tier pathfinding cache.
/// Original: RouteTableLookup @ 0x4d7f20
///
/// Args: person (unit to route), target (destination world coords)
/// Returns: RouteResult indicating how the unit should move.
///
/// Side effects: writes to person fields (target_pos, next_waypoint,
/// segment_index, waypoint_idx).
pub fn route_table_lookup(
    region_map: &RegionMap,
    segment_pool: &mut SegmentPool,
    failure_cache: &FailureCache,
    person: &mut PersonMovement,
    target: WorldCoord,
) -> RouteResult {
    // Step 1: Store target at unit+0x4F
    person.target_pos = target;

    // Step 2: Convert positions to tiles
    let src_tile = person.position.to_tile();
    let dst_tile = target.to_tile();

    // Step 3-4: Read region IDs — same region → direct walk
    let src_region = region_map.region_at(src_tile);
    let dst_region = region_map.region_at(dst_tile);

    if src_region == dst_region {
        // Tier 1 hit: same region, direct walk
        person.next_waypoint = target;
        person.segment_index = 0;
        person.waypoint_idx = 0;
        return RouteResult::DirectWalk;
    }

    // Step 5: Different region — search segment pool (Tier 2)
    // Original: FindExistingSegment @ 0x4d85f0
    if let Some(seg_idx) = segment_pool.find_existing(src_tile, dst_tile) {
        // Reuse existing segment
        segment_pool.add_ref(seg_idx);
        person.segment_index = seg_idx;
        person.waypoint_idx = 0;

        // Extract first waypoint
        if let Some(first_wp) = segment_pool.get_waypoint(seg_idx, 0) {
            person.next_waypoint = first_wp;
        } else {
            // Segment exists but has no waypoints — fallback to target
            person.next_waypoint = target;
        }
        return RouteResult::Segment(seg_idx);
    }

    // Step 6: Check failure cache (Tier 3)
    if failure_cache.is_failed(src_tile, dst_tile) {
        // Recently failed — don't bother with A*
        person.segment_index = 0;
        person.waypoint_idx = 0;
        return RouteResult::NoRoute;
    }

    // Step 7: Pathfinder (Tier 4) — dual-arm wall-following search.
    // Original: FindOrCreateSegment @ 0x4d8a10 → Pathfind @ 0x45d090
    let src_tile = person.position.to_tile();
    let dst_tile = target.to_tile();

    match super::pathfinder::pathfind(region_map, src_tile, dst_tile) {
        super::pathfinder::PathfindResult::Found(waypoints) if !waypoints.is_empty() => {
            // Allocate a new segment and populate it
            if let Some(seg_idx) = segment_pool.allocate() {
                segment_pool.add_ref(seg_idx);
                if let Some(seg) = segment_pool.get_mut(seg_idx) {
                    seg.start_tile_x = src_tile.x;
                    seg.start_tile_z = src_tile.z;
                    seg.end_tile_x = dst_tile.x;
                    seg.end_tile_z = dst_tile.z;
                    let count = waypoints.len().min(super::constants::MAX_WAYPOINTS);
                    seg.waypoint_count = count as u8;
                    for (i, wp) in waypoints.iter().take(count).enumerate() {
                        seg.waypoints[i] = *wp;
                    }
                }
                person.segment_index = seg_idx;
                person.waypoint_idx = 0;

                // Extract first waypoint
                if let Some(first_wp) = segment_pool.get_waypoint(seg_idx, 0) {
                    person.next_waypoint = first_wp;
                } else {
                    person.next_waypoint = target;
                }
                return RouteResult::Segment(seg_idx);
            }
            // Pool full — fall through to NoRoute
        }
        _ => {}
    }

    // Record failure in cache for future lookups
    // (failure_cache is immutable here — caller would need to handle this)
    person.segment_index = 0;
    person.waypoint_idx = 0;
    RouteResult::NoRoute
}

/// STATE_GOTO dispatcher — thin wrapper around route lookup.
/// Original: FUN_004d7e20 (76 bytes)
///
/// Called once per unit per move order.
/// Does NOT change the person state byte (+0x2B).
pub fn state_goto(
    region_map: &RegionMap,
    segment_pool: &mut SegmentPool,
    failure_cache: &FailureCache,
    person: &mut PersonMovement,
    target: WorldCoord,
    used_targets: &mut UsedTargetsCache,
) -> RouteResult {
    // Adjust target for walkability
    let mut adjusted_target = target;
    adjust_target_for_walkability(region_map, person, &mut adjusted_target, used_targets);

    // Route lookup (4-tier cache)
    let result = route_table_lookup(
        region_map,
        segment_pool,
        failure_cache,
        person,
        adjusted_target,
    );

    // Copy waypoint to movement_dest
    // Original: person[+0x57] = person[+0x53]
    person.movement_dest = person.next_waypoint;

    // Set movement flags
    // Original: flags1 |= 0x1000; flags1 &= ~0x80;
    person.set_goto_flags();

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_env() -> (RegionMap, SegmentPool, FailureCache, UsedTargetsCache) {
        (
            RegionMap::new(),
            SegmentPool::new(),
            FailureCache::new(),
            UsedTargetsCache::new(),
        )
    }

    #[test]
    fn same_region_direct_walk() {
        let (map, mut pool, cache, _) = make_test_env();
        let mut person = PersonMovement::default();
        person.position = WorldCoord::new(0x0500, 0x0500);
        let target = WorldCoord::new(0x2000, 0x3000);

        let result = route_table_lookup(&map, &mut pool, &cache, &mut person, target);
        assert_eq!(result, RouteResult::DirectWalk);
        assert_eq!(person.segment_index, 0);
        assert_eq!(person.next_waypoint, target);
        assert_eq!(person.target_pos, target);
    }

    #[test]
    fn cross_region_pathfinder_creates_segment() {
        let (mut map, mut pool, cache, _) = make_test_env();

        let src_tile = TileCoord::new(0x04, 0x04);
        let dst_tile = TileCoord::new(0x40, 0x40);
        map.set_cell_region(src_tile, 1);
        map.set_cell_region(dst_tile, 2);

        let mut person = PersonMovement::default();
        person.position = src_tile.to_world();
        let target = dst_tile.to_world();

        let result = route_table_lookup(&map, &mut pool, &cache, &mut person, target);
        // Pathfinder finds a path on walkable terrain → creates a segment
        match result {
            RouteResult::Segment(idx) => {
                assert!(idx > 0);
                assert_eq!(person.segment_index, idx);
                assert_eq!(person.waypoint_idx, 0);
            }
            other => panic!("Expected Segment, got {:?}", other),
        }
    }

    #[test]
    fn cross_region_reuse_existing_segment() {
        let (mut map, mut pool, cache, _) = make_test_env();

        let src_tile = TileCoord::new(0x04, 0x04);
        let dst_tile = TileCoord::new(0x40, 0x40);
        map.set_cell_region(src_tile, 1);
        map.set_cell_region(dst_tile, 2);

        // Pre-populate a segment in the pool
        let seg_idx = pool.allocate().unwrap();
        pool.add_ref(seg_idx);
        let seg = pool.get_mut(seg_idx).unwrap();
        seg.start_tile_x = src_tile.x;
        seg.start_tile_z = src_tile.z;
        seg.end_tile_x = dst_tile.x;
        seg.end_tile_z = dst_tile.z;
        seg.waypoint_count = 1;
        seg.waypoints[0].tile_x = 0x20;
        seg.waypoints[0].tile_z = 0x20;

        let mut person = PersonMovement::default();
        person.position = src_tile.to_world();
        let target = dst_tile.to_world();

        let result = route_table_lookup(&map, &mut pool, &cache, &mut person, target);
        assert_eq!(result, RouteResult::Segment(seg_idx));
        assert_eq!(person.segment_index, seg_idx);
        assert_eq!(person.waypoint_idx, 0);
        let expected_wp = TileCoord::new(0x20, 0x20).to_world();
        assert_eq!(person.next_waypoint, expected_wp);
    }

    #[test]
    fn cross_region_failure_cache_blocks_astar() {
        let (mut map, mut pool, mut cache, _) = make_test_env();

        let src_tile = TileCoord::new(0x04, 0x04);
        let dst_tile = TileCoord::new(0x40, 0x40);
        map.set_cell_region(src_tile, 1);
        map.set_cell_region(dst_tile, 2);

        cache.record_failure(src_tile, dst_tile);

        let mut person = PersonMovement::default();
        person.position = src_tile.to_world();
        let target = dst_tile.to_world();

        let result = route_table_lookup(&map, &mut pool, &cache, &mut person, target);
        assert_eq!(result, RouteResult::NoRoute);
    }

    #[test]
    fn state_goto_sets_flags() {
        let (map, mut pool, cache, mut used) = make_test_env();
        let mut person = PersonMovement::default();
        person.position = WorldCoord::new(0x0500, 0x0500);
        person.flags1 = FLAG1_BLOCKED;

        let target = WorldCoord::new(0x2000, 0x3000);
        let result = state_goto(&map, &mut pool, &cache, &mut person, target, &mut used);

        assert_eq!(result, RouteResult::DirectWalk);
        assert!(person.is_moving());
        assert!(!person.is_blocked());
        assert_eq!(person.movement_dest, person.next_waypoint);
    }

    // === AdjustTargetForWalkability tests ===

    #[test]
    fn walkable_target_unchanged() {
        let map = RegionMap::new(); // All terrain walkable by default
        let person = PersonMovement::default();
        let mut used = UsedTargetsCache::new();
        let mut target = WorldCoord::new(0x0500, 0x0500);
        let original = target;

        adjust_target_for_walkability(&map, &person, &mut target, &mut used);
        assert_eq!(target, original);
    }

    #[test]
    fn unwalkable_target_snaps_to_neighbor() {
        let mut map = RegionMap::new();
        let person = PersonMovement::default();
        let mut used = UsedTargetsCache::new();

        // Make terrain class 5 unwalkable
        map.set_terrain_flags(5, 0x00);
        // Set the target tile to terrain class 5
        let target_tile = TileCoord::new(0x10, 0x10);
        map.get_cell_mut(target_tile).terrain_type = 5;

        let mut target = target_tile.to_world();
        adjust_target_for_walkability(&map, &person, &mut target, &mut used);

        // Target should have been moved to a neighboring walkable tile
        assert_ne!(target, target_tile.to_world());
        // The new target should be walkable
        assert!(map.is_walkable_world(target));
    }

    #[test]
    fn used_targets_prevents_pileup() {
        let mut map = RegionMap::new();
        let person = PersonMovement::default();
        let mut used = UsedTargetsCache::new();

        // Make terrain class 5 unwalkable
        map.set_terrain_flags(5, 0x00);
        let target_tile = TileCoord::new(0x10, 0x10);
        map.get_cell_mut(target_tile).terrain_type = 5;

        // First unit snaps to some neighbor
        let mut target1 = target_tile.to_world();
        adjust_target_for_walkability(&map, &person, &mut target1, &mut used);

        // Second unit should snap to a DIFFERENT neighbor
        let mut target2 = target_tile.to_world();
        adjust_target_for_walkability(&map, &person, &mut target2, &mut used);

        assert_ne!(target1, target_tile.to_world());
        assert_ne!(target2, target_tile.to_world());
        assert_ne!(target1, target2);
    }
}
