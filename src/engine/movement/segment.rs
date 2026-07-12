// Route segment pool — 400 slots of 109 bytes each.
// Binary base: 0x93E1C1
// Original: pool used by FindExistingSegment @ 0x4d85f0
//           and FindOrCreateSegment @ 0x4d8a10
//
// Route segments store cached paths (sequences of tile waypoints)
// between two map locations. Multiple units can share the same
// segment via reference counting.

use super::constants::*;
use super::types::{FailureCacheEntry, RouteSegment, TileCoord, WorldCoord};

/// The route segment pool: circular buffer of MAX_SEGMENTS slots.
/// Globals at 0x93DD44-0x93DD70 in the binary.
pub struct SegmentPool {
    /// The segment slots
    pub segments: Vec<RouteSegment>,
    /// Circular scan position for next free slot (0x93DD44)
    pub next_free_hint: i16,
    /// Last assigned segment index (0x93DD46)
    pub last_assigned: i16,
    /// Number of active (ref_count > 0) segments (0x93DD48)
    pub active_count: i16,
    /// Force route flag (0x93DD58)
    pub force_route: bool,
}

impl SegmentPool {
    pub fn new() -> Self {
        Self {
            segments: (0..MAX_SEGMENTS).map(|_| RouteSegment::default()).collect(),
            // Index 0 is reserved (sentinel for "no segment")
            next_free_hint: 1,
            last_assigned: 0,
            active_count: 0,
            force_route: false,
        }
    }

    /// Find an existing segment matching src→dst (or dst→src if bidirectional).
    /// Original: FindExistingSegment @ 0x4d85f0
    /// Returns segment index if found, None otherwise.
    /// Skips index 0 (reserved sentinel).
    pub fn find_existing(&self, src: TileCoord, dst: TileCoord) -> Option<u16> {
        for (i, seg) in self.segments.iter().enumerate().skip(1) {
            if seg.ref_count > 0 && seg.matches_bidirectional(src, dst) {
                return Some(i as u16);
            }
        }
        None
    }

    /// Allocate a new segment slot, returning its index.
    /// Uses circular scanning from next_free_hint.
    /// Index 0 is reserved as the "no segment" sentinel — never allocated.
    /// Returns None if the pool is full.
    pub fn allocate(&mut self) -> Option<u16> {
        let start = self.next_free_hint.max(1) as usize;
        for offset in 0..(MAX_SEGMENTS - 1) {
            let idx = ((start - 1 + offset) % (MAX_SEGMENTS - 1)) + 1;
            if self.segments[idx].is_free() {
                self.segments[idx].ref_count = 0;
                self.next_free_hint = ((idx % (MAX_SEGMENTS - 1)) + 1) as i16;
                self.last_assigned = idx as i16;
                self.active_count += 1;
                return Some(idx as u16);
            }
        }
        None // Pool full
    }

    /// Increment reference count for a segment.
    pub fn add_ref(&mut self, index: u16) {
        if (index as usize) < MAX_SEGMENTS {
            self.segments[index as usize].ref_count += 1;
        }
    }

    /// Decrement reference count. If it reaches 0, the slot becomes free.
    pub fn release(&mut self, index: u16) {
        let idx = index as usize;
        if idx < MAX_SEGMENTS {
            self.segments[idx].ref_count -= 1;
            if self.segments[idx].ref_count <= 0 {
                self.segments[idx].ref_count = 0;
                self.active_count -= 1;
            }
        }
    }

    /// Get a segment by index.
    pub fn get(&self, index: u16) -> Option<&RouteSegment> {
        self.segments.get(index as usize)
    }

    /// Get a mutable segment by index.
    pub fn get_mut(&mut self, index: u16) -> Option<&mut RouteSegment> {
        self.segments.get_mut(index as usize)
    }

    /// Extract waypoint N from a segment as world coordinates.
    /// Original: ExtractWaypoint @ 0x4d8560
    pub fn get_waypoint(&self, seg_index: u16, wp_index: u8) -> Option<WorldCoord> {
        self.get(seg_index)
            .and_then(|seg| seg.get_waypoint_world(wp_index))
    }
}

impl Default for SegmentPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Failure cache: remembers recently-failed A* searches.
/// Binary: 8 entries × 10 bytes at 0x93E171.
/// Prevents repeating expensive pathfinding for known-impossible routes.
pub struct FailureCache {
    entries: [FailureCacheEntry; FAILURE_CACHE_SIZE],
}

impl FailureCache {
    pub fn new() -> Self {
        Self {
            entries: [FailureCacheEntry::default(); FAILURE_CACHE_SIZE],
        }
    }

    /// Check if a src→dst route recently failed.
    /// Returns true if found in the cache (meaning: don't bother running A*).
    pub fn is_failed(&self, src: TileCoord, dst: TileCoord) -> bool {
        self.entries.iter().any(|e| e.matches(src, dst))
    }

    /// Record a failed A* search.
    /// Overwrites the least-used entry (or first empty slot).
    pub fn record_failure(&mut self, src: TileCoord, dst: TileCoord) {
        // First, try to find an empty slot
        if let Some(entry) = self.entries.iter_mut().find(|e| e.is_empty()) {
            entry.usage_count = 1;
            entry.src_tile_x = src.x;
            entry.src_tile_z = src.z;
            entry.dst_tile_x = dst.x;
            entry.dst_tile_z = dst.z;
            return;
        }

        // No empty slot — overwrite the entry with the lowest usage count
        let min_idx = self
            .entries
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| e.usage_count)
            .map(|(i, _)| i)
            .unwrap();

        self.entries[min_idx] = FailureCacheEntry {
            usage_count: 1,
            src_tile_x: src.x,
            src_tile_z: src.z,
            dst_tile_x: dst.x,
            dst_tile_z: dst.z,
        };
    }

    /// Increment the usage count of an existing failure entry.
    pub fn increment(&mut self, src: TileCoord, dst: TileCoord) {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.matches(src, dst)) {
            entry.usage_count = entry.usage_count.saturating_add(1);
        }
    }

    /// Clear all failure cache entries.
    pub fn clear(&mut self) {
        self.entries = [FailureCacheEntry::default(); FAILURE_CACHE_SIZE];
    }
}

impl Default for FailureCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_starts_empty() {
        let pool = SegmentPool::new();
        assert_eq!(pool.active_count, 0);
        assert_eq!(pool.segments.len(), MAX_SEGMENTS);
    }

    #[test]
    fn allocate_and_release() {
        let mut pool = SegmentPool::new();

        // Allocate a slot
        let idx = pool.allocate().unwrap();
        assert_eq!(pool.active_count, 1);

        // Add a reference
        pool.add_ref(idx);
        assert_eq!(pool.segments[idx as usize].ref_count, 1);

        // Release
        pool.release(idx);
        assert_eq!(pool.segments[idx as usize].ref_count, 0);
        assert_eq!(pool.active_count, 0);
    }

    #[test]
    fn circular_allocation() {
        let mut pool = SegmentPool::new();

        // Allocate first slot
        let idx1 = pool.allocate().unwrap();
        pool.add_ref(idx1);

        // Allocate second — should be next slot
        let idx2 = pool.allocate().unwrap();
        assert_eq!(idx2, idx1 + 1);
        pool.add_ref(idx2);

        // Release first, allocate again — circular scan finds slot after idx2
        pool.release(idx1);
        let idx3 = pool.allocate().unwrap();
        // Next free hint was after idx2, so idx3 should be idx2+1
        assert_eq!(idx3, idx2 + 1);
    }

    #[test]
    fn find_existing_segment() {
        let mut pool = SegmentPool::new();

        let idx = pool.allocate().unwrap();
        pool.add_ref(idx);

        let seg = pool.get_mut(idx).unwrap();
        seg.start_tile_x = 10;
        seg.start_tile_z = 20;
        seg.end_tile_x = 30;
        seg.end_tile_z = 40;

        // Forward match
        let found = pool.find_existing(TileCoord::new(10, 20), TileCoord::new(30, 40));
        assert_eq!(found, Some(idx));

        // No reverse match (not bidirectional)
        let found = pool.find_existing(TileCoord::new(30, 40), TileCoord::new(10, 20));
        assert_eq!(found, None);

        // Set bidirectional flag
        pool.get_mut(idx).unwrap().flags = 0x02;
        let found = pool.find_existing(TileCoord::new(30, 40), TileCoord::new(10, 20));
        assert_eq!(found, Some(idx));
    }

    #[test]
    fn waypoint_extraction() {
        let mut pool = SegmentPool::new();
        let idx = pool.allocate().unwrap();
        pool.add_ref(idx);

        let seg = pool.get_mut(idx).unwrap();
        seg.waypoint_count = 2;
        seg.waypoints[0].tile_x = 0x10;
        seg.waypoints[0].tile_z = 0x20;
        seg.waypoints[1].tile_x = 0x30;
        seg.waypoints[1].tile_z = 0x40;

        let wp0 = pool.get_waypoint(idx, 0).unwrap();
        assert_eq!(wp0.x, 0x1100); // ((0x10 & 0xFE) + 1) << 8
        assert_eq!(wp0.z, 0x2100);

        let wp1 = pool.get_waypoint(idx, 1).unwrap();
        assert_eq!(wp1.x, 0x3100);
        assert_eq!(wp1.z, 0x4100);

        // Out of bounds
        assert!(pool.get_waypoint(idx, 2).is_none());
    }

    // === Failure cache tests ===

    #[test]
    fn failure_cache_starts_empty() {
        let cache = FailureCache::new();
        assert!(!cache.is_failed(TileCoord::new(1, 2), TileCoord::new(3, 4)));
    }

    #[test]
    fn record_and_check_failure() {
        let mut cache = FailureCache::new();
        let src = TileCoord::new(10, 20);
        let dst = TileCoord::new(30, 40);

        cache.record_failure(src, dst);
        assert!(cache.is_failed(src, dst));
        assert!(!cache.is_failed(dst, src)); // Not bidirectional
    }

    #[test]
    fn failure_cache_eviction() {
        let mut cache = FailureCache::new();

        // Fill all 8 slots
        for i in 0..FAILURE_CACHE_SIZE {
            cache.record_failure(TileCoord::new(i as u8, 0), TileCoord::new(i as u8, 1));
        }

        // Increment usage on first entry to protect it
        cache.increment(TileCoord::new(0, 0), TileCoord::new(0, 1));

        // Add a 9th entry — should evict the lowest-usage one (entries 1-7 have count=1)
        cache.record_failure(TileCoord::new(99, 0), TileCoord::new(99, 1));
        assert!(cache.is_failed(TileCoord::new(99, 0), TileCoord::new(99, 1)));
        // Entry 0 should survive (usage_count=2)
        assert!(cache.is_failed(TileCoord::new(0, 0), TileCoord::new(0, 1)));
    }

    #[test]
    fn failure_cache_clear() {
        let mut cache = FailureCache::new();
        cache.record_failure(TileCoord::new(1, 2), TileCoord::new(3, 4));
        assert!(cache.is_failed(TileCoord::new(1, 2), TileCoord::new(3, 4)));

        cache.clear();
        assert!(!cache.is_failed(TileCoord::new(1, 2), TileCoord::new(3, 4)));
    }
}
