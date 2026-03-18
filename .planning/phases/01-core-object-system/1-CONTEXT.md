# Phase 1: Core Object System - Context

**Gathered:** 2026-03-17
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement a unified object pool (1101 slots, two-tier free lists) with a 128x128 cell-based spatial grid, and migrate the existing UnitCoordinator from owning `Vec<Unit>` to borrowing from the pool. All 260 existing tests must pass unchanged. This phase delivers the foundation every other gameplay system depends on.

</domain>

<decisions>
## Implementation Decisions

### Pool allocation strategy
- Two-tier free list matching original binary: high-priority pool for units/buildings (first ~461 slots), low-priority pool for effects/particles (remaining ~640 slots)
- Fixed-size array, not Vec — `[Option<GameObject>; 1101]` or similar fixed layout
- Allocation order must match original binary to preserve determinism (RNG usage, AI decisions cascade from allocation order)
- Object handles are pool indices (u16), not pointers — stable across pool operations
- Free list is a singly-linked list through the pool slots (matching original's free list at 0x008788B4/B8)

### Object struct design
- Use Rust enum for type-specific data: `GameObjectData { Person(PersonData), Building(BuildingData), ... }`
- Common fields in a shared header struct: position, angle, model_type, subtype, tribe, flags, health, object_index
- Header matches original binary's 179-byte object layout for the common prefix fields
- Type-specific data lives after the header via the enum variant
- For Phase 1, only `Person` variant needs full implementation — other variants are empty stubs (`Building(())`, `Creature(())`, etc.)

### UnitCoordinator migration approach
- Incremental migration, not big-bang replacement
- Step 1: Create ObjectPool with Person allocation support
- Step 2: UnitCoordinator stores pool reference + list of person object IDs instead of Vec<Unit>
- Step 3: All access goes through pool.get(id) / pool.get_mut(id) — iterator adapter for rendering
- Step 4: Update FrameState to expose pool iterator instead of Vec reference
- Preserve all existing Unit fields — PersonData contains exactly what Unit currently has
- Keep UnitCoordinator's selection, region_map, segment_pool, failure_cache unchanged

### Cell grid integration
- Extend existing RegionMapCell with object linked-list head pointer (add `first_object: Option<u16>` field)
- Objects get `next_in_cell: Option<u16>` and `prev_in_cell: Option<u16>` fields for doubly-linked list
- Object_SetPosition updates cell linkage: remove from old cell, insert into new cell
- RegionMap already has the right 128x128 structure and is owned by UnitCoordinator — minimize disruption
- Pathfinding code continues using RegionMap's existing walkability/region APIs unchanged

### Claude's Discretion
- Exact memory layout and padding of the pool struct
- Whether to use `unsafe` for pool indexing performance or stay safe with bounds checks
- Test organization — new test file vs extending existing test modules
- Whether to introduce a `Handle<T>` wrapper type for pool indices

</decisions>

<specifics>
## Specific Ideas

- The original binary's object pool is at 0x008c89c0, 179 bytes per object, with specific free list heads at 0x008788B4 (high-priority) and 0x008788B8 (low-priority)
- Current Unit struct is ~210-220 bytes (larger than original due to Rust enum/Option overhead) — this is acceptable since we're not binary-compatible at struct level, only behavior-compatible
- The existing `ModelType` enum in `src/data/units.rs` already defines all 11 types — reuse it
- PersonMovement (~140 bytes) must stay embedded in the person data, not externalized

</specifics>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Object system spec
- `docs/specs/` — Reverse engineering specs for the original binary's object system (if object system spec exists)
- `things-to-implement.md` §1 "Core Object System" — Object pool structure, cell grid, model types, instance fields

### Current implementation (must understand before modifying)
- `src/engine/units/coordinator.rs` — UnitCoordinator owns Vec<Unit>, tick loop, selection, movement infra
- `src/engine/units/unit.rs` — Current Unit struct (49 lines, ~210 bytes)
- `src/engine/movement/types.rs` — WorldCoord, PersonMovement, RegionMapCell structs
- `src/engine/movement/region.rs` — RegionMap 128x128 grid (291 lines)
- `src/engine/frame.rs` — FrameState references UnitCoordinator (43 lines)
- `src/render/app.rs` lines 150-190 — GameEngine struct owns UnitCoordinator
- `src/data/units.rs` lines 8-21 — ModelType enum (11 types)

### Test files (must not break)
- `src/engine/units/coordinator.rs` tests at lines 476-552
- All 260 `#[test]` functions across 24 files (run `cargo test` to verify)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ModelType` enum (src/data/units.rs:8-21): Already defines all 11 object types — use directly in pool
- `RegionMap` (src/engine/movement/region.rs): 128x128 grid with 16 bytes/cell — extend for object linkage
- `RegionMapCell` (src/engine/movement/types.rs:78-94): Has padding bytes (_pad fields) that could be repurposed for object list pointers
- `WorldCoord` (src/engine/movement/types.rs:9-39): Position type with toroidal wrapping — use in pool object header
- `PersonMovement` (src/engine/movement/types.rs:242-277): ~140 bytes of movement state — embed in PersonData variant
- `AnimationState` (src/engine/units/animation.rs:58-71): 8-byte animation tracking — embed in PersonData

### Established Patterns
- `BinDeserializer` trait for binary data parsing — not directly applicable but shows the pattern for matching original binary layout
- Trait-based tick subsystems (src/engine/state/traits.rs) — pool should integrate with existing tick loop via these traits
- `FrameState` as read-only snapshot — pool must support immutable iteration for rendering while mutable access for tick
- `GameCommand` input boundary — pool operations triggered through commands, not direct mutation

### Integration Points
- `UnitCoordinator::load_level()` (coordinator.rs:65-129): Currently pushes to Vec, must allocate from pool
- `UnitCoordinator::tick()` (coordinator.rs:162-211): Iterates units for state/movement/combat — must iterate pool person objects
- `GameEngine::frame_state()` (app.rs:651): Borrows UnitCoordinator — FrameState must expose pool iteration
- `App::render()` (app.rs:2037+): Accesses units for sprite rendering — must use pool iterator
- `App` minimap rendering (app.rs:379-403): Iterates unit_coordinator.units — update to pool

</code_context>

<deferred>
## Deferred Ideas

- Building object type implementation (Phase 2 — only stub variant in Phase 1)
- Effect/Shot/Spell object types (Phases 2-3 — only stub variants in Phase 1)
- Cell-grid-based spatial queries for combat range detection (Phase 2)
- Object destruction effects and debris spawning (Phase 2)
- Full 179-byte binary-compatible serialization for save/load (Phase 4)

</deferred>

---

*Phase: 01-core-object-system*
*Context gathered: 2026-03-17*
