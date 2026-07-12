// Movement system for Populous: The Beginning.
//
// Faithful reimplementation of the unit movement system from popTB.exe,
// using the REAL active movement functions (confirmed via Frida):
//
//   STATE_GOTO          @ 0x4d7e20 — thin dispatcher (34 calls per move order)
//   RouteTableLookup    @ 0x4d7f20 — 4-tier pathfinding cache
//   Object_UpdateMovement @ 0x4ed510 — formation manager (44 calls per tick)
//   MovePointByAngle    @ 0x4d4b20 — per-tick position update (~320/sec)
//
// NOTE: The Path_UpdateDirection (0x4248c0) / Path_FindBestDirection (0x424ed0)
// system was proven dormant — it never fires during normal gameplay. Those
// functions remain in the pathfinding worktree as reference only.
//
// Architecture:
//   Tier 1: Region Map (128×128) — same region = direct walk
//   Tier 2: Segment Pool (400 slots) — reuse cached path segments
//   Tier 3: Failure Cache (8 entries) — skip known-impossible routes
//   Tier 4: Dual-arm wall-following pathfinder (Bug2 variant)

pub mod constants;
pub mod math;
pub mod pathfinder;
pub mod region;
pub mod route;
pub mod segment;
pub mod tables;
pub mod types;
pub mod waypoint;

// Re-export primary API
pub use math::{
    angle_difference, atan2, distance, formation_rng_next, move_point_by_angle, rotation_direction,
};
pub use pathfinder::{
    pathfind, pathfind_debug, PathNode, PathfindDebug, PathfindResult, VisitedBitmap,
};
pub use region::RegionMap;
pub use route::{adjust_target_for_walkability, route_table_lookup, state_goto, RouteResult};
pub use segment::{FailureCache, SegmentPool};
pub use types::{PersonMovement, TileCoord, UsedTargetsCache, Waypoint, WorldCoord};
pub use waypoint::{process_route_movement, WaypointResult};
