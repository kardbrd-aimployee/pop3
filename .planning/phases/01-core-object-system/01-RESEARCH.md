# Phase 1: Core Object System - Research

**Researched:** 2026-03-17
**Domain:** Fixed-size object pool with spatial indexing, migrating from Vec-based unit storage
**Confidence:** HIGH

## Summary

Phase 1 replaces `UnitCoordinator`'s `Vec<Unit>` with a unified 1101-slot fixed-size object pool that matches the original Populous: The Beginning binary's allocation behavior. The pool uses two-tier singly-linked free lists (high-priority for units/buildings, low-priority for effects) and a 128x128 cell grid for spatial indexing via doubly-linked per-cell object lists.

The codebase is well-structured for this migration. The `Unit` struct (49 lines) contains exactly the person-specific fields that will become `PersonData`. The `RegionMapCell` already has padding bytes (`_pad1`, `_pad2`, `_pad3`) that can be repurposed for object list head pointers. The `UnitCoordinator` accesses units via index (`self.units[i]`) throughout tick/combat, which maps directly to pool index access. There are 11 places in `app.rs` that iterate `unit_coordinator.units` -- these all need migration to pool iterators.

**Primary recommendation:** Build the ObjectPool as a new module (`src/engine/objects/`), implement Person allocation first, then incrementally migrate UnitCoordinator from `Vec<Unit>` to pool references. Keep all existing public APIs stable via iterator adapters until every consumer is migrated.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- Two-tier free list matching original binary: high-priority pool for units/buildings (first ~461 slots), low-priority pool for effects/particles (remaining ~640 slots)
- Fixed-size array, not Vec -- `[Option<GameObject>; 1101]` or similar fixed layout
- Allocation order must match original binary to preserve determinism (RNG usage, AI decisions cascade from allocation order)
- Object handles are pool indices (u16), not pointers -- stable across pool operations
- Free list is a singly-linked list through the pool slots (matching original's free list at 0x008788B4/B8)
- Use Rust enum for type-specific data: `GameObjectData { Person(PersonData), Building(BuildingData), ... }`
- Common fields in a shared header struct: position, angle, model_type, subtype, tribe, flags, health, object_index
- Header matches original binary's 179-byte object layout for the common prefix fields
- Type-specific data lives after the header via the enum variant
- For Phase 1, only `Person` variant needs full implementation -- other variants are empty stubs
- Incremental migration, not big-bang replacement
- Step 1: Create ObjectPool with Person allocation support
- Step 2: UnitCoordinator stores pool reference + list of person object IDs instead of Vec<Unit>
- Step 3: All access goes through pool.get(id) / pool.get_mut(id) -- iterator adapter for rendering
- Step 4: Update FrameState to expose pool iterator instead of Vec reference
- Preserve all existing Unit fields -- PersonData contains exactly what Unit currently has
- Keep UnitCoordinator's selection, region_map, segment_pool, failure_cache unchanged
- Extend existing RegionMapCell with object linked-list head pointer (add `first_object: Option<u16>` field)
- Objects get `next_in_cell: Option<u16>` and `prev_in_cell: Option<u16>` fields for doubly-linked list
- Object_SetPosition updates cell linkage: remove from old cell, insert into new cell
- RegionMap already has the right 128x128 structure and is owned by UnitCoordinator -- minimize disruption
- Pathfinding code continues using RegionMap's existing walkability/region APIs unchanged

### Claude's Discretion
- Exact memory layout and padding of the pool struct
- Whether to use `unsafe` for pool indexing performance or stay safe with bounds checks
- Test organization -- new test file vs extending existing test modules
- Whether to introduce a `Handle<T>` wrapper type for pool indices

### Deferred Ideas (OUT OF SCOPE)
- Building object type implementation (Phase 2 -- only stub variant in Phase 1)
- Effect/Shot/Spell object types (Phases 2-3 -- only stub variants in Phase 1)
- Cell-grid-based spatial queries for combat range detection (Phase 2)
- Object destruction effects and debris spawning (Phase 2)
- Full 179-byte binary-compatible serialization for save/load (Phase 4)
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| OBJ-01 | Unified object pool with 1101 max active objects, 179 bytes per object, two-tier free list | Pool architecture pattern below; original binary specs at docs/specs/object_system.md confirm 0x44D slots, free lists at 0x008788B4/B8 |
| OBJ-02 | Object lifecycle (create, destroy, reinitialize) matching original binary's allocation order | Object_Create/Object_Destroy specs confirmed; allocation order from free list heads; lifecycle pattern documented |
| OBJ-03 | Cell-based spatial grid (128x128, 16 bytes/cell) with per-cell object linked lists | RegionMapCell already exists with padding fields repurposable; cell list uses offsets 0x20/0x22 in original |
| OBJ-04 | Object position updates that maintain cell linkage (Object_SetPosition) | Doubly-linked cell list pattern documented; position-to-cell conversion via existing WorldCoord::to_tile() and TileCoord::cell_index() |
| OBJ-05 | UnitCoordinator migration to borrow from unified pool instead of owning Vec<Unit> | 11 access sites in app.rs identified; incremental migration strategy with iterator adapters |
</phase_requirements>

## Standard Stack

This is a pure Rust game engine project. No external crates are needed for Phase 1 -- all data structures are hand-built to match original binary behavior.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust std | 1.x | Fixed arrays, Option, iterators | Language standard library only |

### Supporting
No additional crates needed. The object pool, free lists, and cell grid are fundamental game engine data structures that must precisely match the original binary's behavior.

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Custom pool | `slotmap` crate | slotmap uses generational indices which don't match original binary's allocation order; custom pool is required for determinism |
| Custom linked list | `Vec<Option<T>>` with separate free Vec | Doesn't match original's singly-linked free list through slots; allocation order would differ |

## Architecture Patterns

### Recommended Project Structure
```
src/engine/
  objects/
    mod.rs           # pub use exports
    pool.rs          # ObjectPool, GameObject, ObjectHeader
    types.rs         # GameObjectData enum, PersonData, stub variants
    cell_grid.rs     # CellGrid integration (extends RegionMapCell)
    handle.rs        # ObjectHandle type alias or newtype wrapper
  units/
    coordinator.rs   # Modified: stores pool ref + person IDs
    unit.rs          # Kept temporarily for backward compat, then removed
    ...              # Other files unchanged
  movement/
    types.rs         # RegionMapCell extended with first_object field
    ...
```

### Pattern 1: Fixed-Size Pool with Intrusive Free Lists
**What:** A `[MaybeUninit<PoolSlot>; 1101]` or `[Option<GameObject>; 1101]` array where free slots form a singly-linked list via a `next_free: Option<u16>` field stored in the slot itself. Two list heads track high-priority (index 0..460) and low-priority (index 461..1100) pools.
**When to use:** Always -- this is the core data structure.
**Example:**
```rust
// Source: docs/specs/object_system.md - Object Pool section
pub const MAX_OBJECTS: usize = 1101;       // 0x44D
pub const LOW_PRIORITY_START: usize = 461; // 0x280 - effects/particles boundary

pub type ObjectHandle = u16;

pub struct ObjectHeader {
    pub model_type: ModelType,
    pub subtype: u8,
    pub tribe: u8,
    pub state: u8,
    pub state_phase: u8,
    pub flags1: u32,
    pub flags2: u32,
    pub flags3: u32,
    pub object_index: ObjectHandle,
    pub angle: u16,
    pub position: WorldCoord,
    pub velocity: WorldCoord,  // stub for Phase 1
    pub health: u16,
    pub max_health: u16,
    // Cell linkage
    pub next_in_cell: Option<u16>,
    pub prev_in_cell: Option<u16>,
}

pub enum GameObjectData {
    Person(PersonData),
    Building(()),
    Creature(()),
    Vehicle(()),
    Scenery(()),
    General(()),
    Effect(()),
    Shot(()),
    Shape(()),
    Internal(()),
    Spell(()),
}

pub struct GameObject {
    pub header: ObjectHeader,
    pub data: GameObjectData,
}

enum PoolSlot {
    Occupied(GameObject),
    Free { next_free: Option<u16> },
}

pub struct ObjectPool {
    slots: Box<[PoolSlot; MAX_OBJECTS]>,
    free_high: Option<u16>,  // Head of high-priority free list (slots 0..461)
    free_low: Option<u16>,   // Head of low-priority free list (slots 461..1101)
    active_count: u16,
}
```

### Pattern 2: PersonData Extracted from Unit
**What:** Move all current `Unit` fields into `PersonData`, keeping `ObjectHeader` for the shared fields. `PersonData` contains `PersonMovement`, `AnimationState`, combat stats, state machine fields.
**When to use:** When allocating Person objects from the pool.
**Example:**
```rust
pub struct PersonData {
    pub movement: PersonMovement,
    pub anim: AnimationState,
    pub state: PersonState,
    pub prev_state: PersonState,
    pub state_timer: u16,
    pub state_counter: u8,
    pub target_unit: Option<ObjectHandle>,
    pub attacker_unit: Option<ObjectHandle>,
    pub alive: bool,
    pub home_pos: WorldCoord,
    pub behavior_flags: u16,
    pub wander_duration: u8,
    pub wander_range: u8,
    pub linked_obj_id: Option<ObjectHandle>,
    pub bloodlust: bool,
    pub shielded: bool,
    // Rendering cache
    pub cell_x: f32,
    pub cell_y: f32,
}
```

### Pattern 3: Iterator Adapters for Migration
**What:** Provide iterator methods on ObjectPool that return person objects, allowing rendering code to iterate without knowing about pool internals. UnitCoordinator exposes `persons()` and `persons_mut()` methods.
**When to use:** All rendering and tick code that currently iterates `units`.
**Example:**
```rust
impl ObjectPool {
    pub fn persons(&self) -> impl Iterator<Item = (ObjectHandle, &ObjectHeader, &PersonData)> {
        self.slots.iter().filter_map(|slot| {
            if let PoolSlot::Occupied(obj) = slot {
                if let GameObjectData::Person(ref person) = obj.data {
                    Some((obj.header.object_index, &obj.header, person))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }
}
```

### Pattern 4: Cell Grid Doubly-Linked List
**What:** Each RegionMapCell gets a `first_object: Option<u16>` field. Each object's header has `next_in_cell` / `prev_in_cell`. Inserting/removing from cell list is O(1).
**When to use:** Object_SetPosition and any spatial query.
**Example:**
```rust
impl ObjectPool {
    pub fn set_position(
        &mut self,
        handle: ObjectHandle,
        new_pos: WorldCoord,
        cell_grid: &mut CellGrid,
    ) {
        let obj = self.get_mut(handle).unwrap();
        let old_tile = obj.header.position.to_tile();
        let new_tile = new_pos.to_tile();

        if old_tile != new_tile {
            cell_grid.remove_object(handle, old_tile, &mut self.slots);
            cell_grid.insert_object(handle, new_tile, &mut self.slots);
        }

        obj.header.position = new_pos;
    }
}
```

### Anti-Patterns to Avoid
- **Storing references/borrows in pool objects:** Pool objects reference each other by `ObjectHandle` (u16 index), never by Rust references. This avoids borrow checker issues entirely.
- **Using HashMap for handle lookup:** The pool IS the lookup table. `pool.get(handle)` is a direct array index, O(1). A HashMap would add overhead and break allocation order determinism.
- **Migrating all consumers at once:** The rendering code (app.rs) has 11 distinct access patterns. Migrate incrementally with iterator adapters.
- **Making ObjectPool own the RegionMap:** Keep RegionMap ownership in UnitCoordinator. Pass cell grid as parameter to pool methods that need it.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Intrusive linked lists | Safe Rust doubly-linked list from scratch | Store `Option<u16>` indices in object headers + helper methods | Avoids unsafe, matches original binary's index-based linking |
| Generation counters | ABA-problem detection for stale handles | Not needed in Phase 1 -- original binary doesn't use generational indices | Adds complexity; original game never needed it |

**Key insight:** The original binary uses raw indices everywhere. Matching this means simpler code. Generation-based handles can be added later if needed, but the original game survived 25+ years without them.

## Common Pitfalls

### Pitfall 1: Breaking the 260 existing tests
**What goes wrong:** Changing `UnitCoordinator.units` from `Vec<Unit>` to pool-backed storage breaks all test code that constructs units directly.
**Why it happens:** Tests use `make_unit()` helpers that construct `Unit` structs directly, and some tests access `coordinator.units` as a Vec.
**How to avoid:** Keep the `Unit` type alias or provide a compatibility method. Tests that construct `UnitCoordinator::new()` and call `.units.is_empty()` (line 485 of coordinator.rs) need the fewest changes if `units` becomes an accessor method.
**Warning signs:** `cargo test` fails after pool changes.

### Pitfall 2: Free list initialization order
**What goes wrong:** If the free list is initialized in wrong order (e.g., slot 1100 first instead of slot 0 first), allocation order diverges from original binary, causing different RNG consumption and cascade failures in determinism.
**Why it happens:** The original binary initializes the free list by linking slot 0 -> 1 -> 2 -> ... -> 460 (high) and 461 -> 462 -> ... -> 1100 (low). Reversing this is easy to do accidentally.
**How to avoid:** Initialize with explicit order matching: `free_high` points to slot 0, which points to slot 1, etc.
**Warning signs:** Frida trace fixture tests show different allocation order.

### Pitfall 3: Cell list corruption during iteration
**What goes wrong:** Iterating objects in a cell while also moving objects (which updates cell linkage) corrupts the linked list.
**Why it happens:** Moving an object removes it from its current cell list and inserts into a new one. If you're iterating the old cell list, the next pointer may be stale.
**How to avoid:** Collect handles first, then mutate. Or use a two-phase approach (read phase, write phase) as the tick loop already does.
**Warning signs:** Objects disappear from cells or cells contain stale handles.

### Pitfall 4: Borrow checker conflicts with pool + cell grid
**What goes wrong:** Methods that need mutable access to both the pool and the cell grid hit Rust's aliasing rules since both are owned by the same struct.
**Why it happens:** `set_position` needs `&mut pool.slots` and `&mut cell_grid`, but if both are fields of `ObjectPool`, you can't borrow both mutably.
**How to avoid:** Keep `CellGrid` (or `RegionMap`) separate from `ObjectPool`. Pass cell grid as a separate `&mut` parameter. This matches the existing pattern where `UnitCoordinator` owns `RegionMap` separately from units.
**Warning signs:** "cannot borrow `self` as mutable more than once" errors.

### Pitfall 5: Option<u16> vs sentinel values
**What goes wrong:** The original binary uses 0xFFFF or 0 as "no object" sentinel. Using `Option<u16>` is safer but means different representations.
**Why it happens:** Mixing Rust idioms with C-style sentinels.
**How to avoid:** Use `Option<u16>` consistently in Rust code. Only convert to/from sentinel values at serialization boundaries (Phase 4). Define `pub const NO_OBJECT: u16 = 0xFFFF;` for the rare cases where raw u16 is needed.
**Warning signs:** Off-by-one errors when slot 0 is a valid object but 0 is also used as "none".

### Pitfall 6: Rendering code accesses `.units` field directly
**What goes wrong:** app.rs lines 379, 403, 414, 459, 470, 1132, 1255, 1410, 3128 all access `unit_coordinator.units` directly. Changing the field type breaks compilation in 9 places.
**Why it happens:** `units` is a public field, not a method.
**How to avoid:** Step 1: Add a `pub fn units(&self) -> &[Unit]` method. Step 2: Change `pub units: Vec<Unit>` to private. Step 3: Have the method return a pool-backed slice or iterator. Alternatively, expose a thin `PersonView` that provides the same fields the rendering code needs.
**Warning signs:** Compilation errors in app.rs after coordinator changes.

## Code Examples

### Object Pool Initialization
```rust
// Source: docs/specs/object_system.md - InitObjectPointerArray (0x004afbf0)
impl ObjectPool {
    pub fn new() -> Self {
        let mut slots: Box<[PoolSlot; MAX_OBJECTS]> = {
            let mut v = Vec::with_capacity(MAX_OBJECTS);
            for _ in 0..MAX_OBJECTS {
                v.push(PoolSlot::Free { next_free: None });
            }
            v.into_boxed_slice().try_into().unwrap()
        };

        // Link high-priority free list: 0 -> 1 -> ... -> 460 -> None
        for i in 0..LOW_PRIORITY_START {
            let next = if i + 1 < LOW_PRIORITY_START { Some((i + 1) as u16) } else { None };
            slots[i] = PoolSlot::Free { next_free: next };
        }

        // Link low-priority free list: 461 -> 462 -> ... -> 1100 -> None
        for i in LOW_PRIORITY_START..MAX_OBJECTS {
            let next = if i + 1 < MAX_OBJECTS { Some((i + 1) as u16) } else { None };
            slots[i] = PoolSlot::Free { next_free: next };
        }

        ObjectPool {
            slots,
            free_high: Some(0),
            free_low: Some(LOW_PRIORITY_START as u16),
            active_count: 0,
        }
    }
}
```

### Object Creation (matching Object_Create)
```rust
// Source: docs/specs/object_system.md - Object_Create (0x004afc70)
impl ObjectPool {
    pub fn create(
        &mut self,
        model_type: ModelType,
        subtype: u8,
        tribe: u8,
        position: WorldCoord,
    ) -> Option<ObjectHandle> {
        // High-priority types: Person, Building, Creature, Vehicle, Scenery
        let is_high_priority = matches!(model_type,
            ModelType::Person | ModelType::Building | ModelType::Creature |
            ModelType::Vehicle | ModelType::Scenery
        );

        let free_head = if is_high_priority {
            &mut self.free_high
        } else {
            &mut self.free_low
        };

        let slot_idx = (*free_head)?;
        let next = match &self.slots[slot_idx as usize] {
            PoolSlot::Free { next_free } => *next_free,
            PoolSlot::Occupied(_) => panic!("free list points to occupied slot"),
        };
        *free_head = next;

        let header = ObjectHeader {
            model_type,
            subtype,
            tribe,
            state: 0,
            state_phase: 0,
            flags1: 0,
            flags2: 0,
            flags3: 0,
            object_index: slot_idx,
            angle: 0,
            position,
            velocity: WorldCoord::default(),
            health: 0,
            max_health: 0,
            next_in_cell: None,
            prev_in_cell: None,
        };

        let data = match model_type {
            ModelType::Person => GameObjectData::Person(PersonData::default()),
            ModelType::Building => GameObjectData::Building(()),
            ModelType::Creature => GameObjectData::Creature(()),
            ModelType::Vehicle => GameObjectData::Vehicle(()),
            ModelType::Scenery => GameObjectData::Scenery(()),
            ModelType::General => GameObjectData::General(()),
            ModelType::Effect => GameObjectData::Effect(()),
            ModelType::Shot => GameObjectData::Shot(()),
            ModelType::Shape => GameObjectData::Shape(()),
            ModelType::Internal => GameObjectData::Internal(()),
            ModelType::Spell => GameObjectData::Spell(()),
        };

        self.slots[slot_idx as usize] = PoolSlot::Occupied(GameObject { header, data });
        self.active_count += 1;

        Some(slot_idx)
    }
}
```

### Object Destruction (matching Object_Destroy)
```rust
// Source: docs/specs/object_system.md - Object_Destroy (0x004b00c0)
impl ObjectPool {
    pub fn destroy(&mut self, handle: ObjectHandle) -> bool {
        let idx = handle as usize;
        if idx >= MAX_OBJECTS {
            return false;
        }

        let is_high = idx < LOW_PRIORITY_START;

        match &self.slots[idx] {
            PoolSlot::Occupied(_) => {}
            PoolSlot::Free { .. } => return false,
        }

        // Return to appropriate free list (push to front)
        let free_head = if is_high {
            &mut self.free_high
        } else {
            &mut self.free_low
        };

        self.slots[idx] = PoolSlot::Free { next_free: *free_head };
        *free_head = Some(handle);
        self.active_count -= 1;

        true
    }
}
```

### Cell Grid Insert/Remove
```rust
// Source: docs/specs/object_system.md - cell_next (0x20), cell_prev (0x22)
impl RegionMap {
    pub fn insert_object(&mut self, pool: &mut [PoolSlot], handle: u16, tile: TileCoord) {
        let cell_idx = tile.cell_index();
        let old_head = self.cells[cell_idx].first_object;

        // New object points to old head
        if let PoolSlot::Occupied(ref mut obj) = pool[handle as usize] {
            obj.header.next_in_cell = old_head;
            obj.header.prev_in_cell = None;
        }

        // Old head's prev points to new object
        if let Some(old_handle) = old_head {
            if let PoolSlot::Occupied(ref mut old_obj) = pool[old_handle as usize] {
                old_obj.header.prev_in_cell = Some(handle);
            }
        }

        self.cells[cell_idx].first_object = Some(handle);
    }

    pub fn remove_object(&mut self, pool: &mut [PoolSlot], handle: u16, tile: TileCoord) {
        let (prev, next) = if let PoolSlot::Occupied(ref obj) = pool[handle as usize] {
            (obj.header.prev_in_cell, obj.header.next_in_cell)
        } else {
            return;
        };

        // Unlink: prev.next = next
        if let Some(prev_handle) = prev {
            if let PoolSlot::Occupied(ref mut prev_obj) = pool[prev_handle as usize] {
                prev_obj.header.next_in_cell = next;
            }
        } else {
            // Was head of list
            let cell_idx = tile.cell_index();
            self.cells[cell_idx].first_object = next;
        }

        // Unlink: next.prev = prev
        if let Some(next_handle) = next {
            if let PoolSlot::Occupied(ref mut next_obj) = pool[next_handle as usize] {
                next_obj.header.prev_in_cell = prev;
            }
        }

        // Clear own linkage
        if let PoolSlot::Occupied(ref mut obj) = pool[handle as usize] {
            obj.header.next_in_cell = None;
            obj.header.prev_in_cell = None;
        }
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Vec<Unit>` owned by UnitCoordinator | ObjectPool with fixed-size array + free lists | Phase 1 (now) | Enables all future object types, deterministic allocation |
| UnitId = usize (Vec index) | ObjectHandle = u16 (pool index) | Phase 1 (now) | Stable handles across insert/remove operations |
| No spatial indexing for objects | Per-cell doubly-linked object lists | Phase 1 (now) | Enables O(1) spatial queries for combat/spells in Phase 2+ |

## Open Questions

1. **Box vs inline array for 1101 PoolSlots**
   - What we know: `[PoolSlot; 1101]` on the stack would be large (~200KB+ depending on PersonData size). `Box<[PoolSlot; 1101]>` puts it on the heap.
   - What's unclear: Exact size of PoolSlot with the GameObjectData enum. PersonMovement alone is ~140 bytes.
   - Recommendation: Use `Box<[PoolSlot; MAX_OBJECTS]>` to avoid stack overflow. Verify with `std::mem::size_of::<PoolSlot>()` in a test.

2. **Handle<T> wrapper vs raw u16**
   - What we know: A newtype `ObjectHandle(u16)` prevents accidentally passing a raw u16 where a handle is expected.
   - What's unclear: Whether the additional type ceremony helps or hinders in a codebase where handles are passed everywhere.
   - Recommendation: Start with `pub type ObjectHandle = u16;` for simplicity. Upgrade to newtype if misuse becomes a problem.

3. **first_object field in RegionMapCell: repurpose padding or add new field**
   - What we know: RegionMapCell has `_pad1: [u8; 6]`, `_pad2: [u8; 2]`, `_pad3: [u8; 3]` -- plenty of space. The original binary stores cell list head at 0x00888982 (separate from RegionMap).
   - What's unclear: Whether repurposing padding maintains the `#[repr(C)]` layout needed for binary data loading.
   - Recommendation: Add `first_object: Option<u16>` as a new Rust-only field after the repr(C) fields, or use a separate `CellObjectHeads: [Option<u16>; 128*128]` array to avoid touching RegionMapCell's binary layout at all. The separate array approach is safer.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` with `cargo test` |
| Config file | Cargo.toml (already configured) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| OBJ-01 | Pool allocates from correct free list tier, 1101 max | unit | `cargo test pool` | No -- Wave 0 |
| OBJ-01 | Two-tier free list ordering matches original | unit | `cargo test free_list` | No -- Wave 0 |
| OBJ-02 | Object_Create initializes header fields correctly | unit | `cargo test object_create` | No -- Wave 0 |
| OBJ-02 | Object_Destroy returns slot to correct free list | unit | `cargo test object_destroy` | No -- Wave 0 |
| OBJ-02 | Create-destroy-create reuses same slot (LIFO) | unit | `cargo test reuse_slot` | No -- Wave 0 |
| OBJ-03 | Cell grid insert/remove maintains linked list integrity | unit | `cargo test cell_grid` | No -- Wave 0 |
| OBJ-03 | Multiple objects in same cell form correct chain | unit | `cargo test cell_chain` | No -- Wave 0 |
| OBJ-04 | set_position updates cell linkage on cell change | unit | `cargo test set_position` | No -- Wave 0 |
| OBJ-04 | set_position no-ops when cell unchanged | unit | `cargo test set_position_same_cell` | No -- Wave 0 |
| OBJ-05 | All 260 existing tests still pass | integration | `cargo test` | Yes -- 260 tests across 24 files |
| OBJ-05 | UnitCoordinator.tick() works with pool-backed storage | unit | `cargo test coordinator_tick` | Partially -- existing tick tests |

### Sampling Rate
- **Per task commit:** `cargo test`
- **Per wave merge:** `cargo test`
- **Phase gate:** All 260 existing tests pass + new pool/cell tests pass

### Wave 0 Gaps
- [ ] `src/engine/objects/pool.rs` tests -- covers OBJ-01, OBJ-02
- [ ] `src/engine/objects/cell_grid.rs` tests -- covers OBJ-03, OBJ-04
- [ ] No framework install needed -- Rust's built-in test framework already configured

## Sources

### Primary (HIGH confidence)
- `docs/specs/object_system.md` -- Object pool limits (0x44D), free lists (0x008788B4/B8), cell linkage (offsets 0x20/0x22), Object_Create/Destroy specs
- `src/engine/units/coordinator.rs` -- Current UnitCoordinator implementation (553 lines), 6 tests
- `src/engine/units/unit.rs` -- Current Unit struct (49 lines)
- `src/engine/movement/types.rs` -- RegionMapCell, WorldCoord, PersonMovement structs
- `src/engine/movement/region.rs` -- RegionMap implementation (291 lines)
- `src/render/app.rs` -- 11 direct access sites to `unit_coordinator.units`
- `things-to-implement.md` Section 1 -- Object pool status and field offsets

### Secondary (MEDIUM confidence)
- `docs/specs/object_system.md` Appendix BI -- Linked list system details, cell grid base address 0x00888982
- `1-CONTEXT.md` -- User decisions constraining architecture

### Tertiary (LOW confidence)
- None -- all findings verified against reverse engineering specs and source code

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no external dependencies, pure Rust data structures
- Architecture: HIGH -- verified against both original binary specs and current codebase structure
- Pitfalls: HIGH -- identified from reading actual code access patterns (11 sites in app.rs, test construction patterns)

**Research date:** 2026-03-17
**Valid until:** 2026-04-17 (stable domain -- game engine data structures don't change)
