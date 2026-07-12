// UnitCoordinator — owns all live units and movement infrastructure.
//
// Provides the bridge between user input (selection, move orders) and
// the movement system (pathfinding, per-tick position updates).

use super::animation::{select_animation, tick_animation, AnimationState};
use super::coords::{cell_to_tile, cell_to_world, toroidal_delta, world_to_render_pos};
use super::person_state::{
    apply_damage, calculate_melee_damage, enter_state, person_type_defaults, tick_state,
    CombatPhase, DeferredAction, PersonState, TickResult, COMBAT_DETECT_RANGE, COMBAT_MELEE_RANGE,
    SWING_READY_TICKS,
};
use super::selection::{DragState, SelectionState};
use super::unit::Unit;
use crate::data::units::{ModelType, UnitRaw};
use crate::engine::buildings::{
    self, tick::BuildingTickActions, BuildingCombatAction, BuildingState, ConvertAction,
    SpawnAction,
};
use crate::engine::combat;
use crate::engine::effects::types::EffectType;
use crate::engine::effects::EffectAction;
use crate::engine::movement::{
    atan2, move_point_by_angle, process_route_movement, state_goto, FailureCache, PersonMovement,
    RegionMap, RouteResult, SegmentPool, UsedTargetsCache, WorldCoord,
};
use crate::engine::objects::{CellGrid, GameObjectData, ObjectHandle, ObjectPool};
use crate::engine::state::rng::GameRng;
use crate::engine::state::traits::ObjectTick;

pub struct UnitCoordinator {
    units: Vec<Unit>,
    pub selection: SelectionState,
    pub drag: DragState,

    // Object pool and spatial grid (source of truth for allocation)
    pool: ObjectPool,
    cell_grid: CellGrid,
    person_handles: Vec<ObjectHandle>,

    // Movement infrastructure
    region_map: RegionMap,
    segment_pool: SegmentPool,
    failure_cache: FailureCache,
    used_targets: UsedTargetsCache,

    landscape_size: f32,

    // Animation frame counts indexed by animation ID.
    // Populated from animation data during atlas rebuild.
    pub anim_frame_counts: Vec<u8>,

    // State machine RNG (same LCG as original binary)
    pub rng: GameRng,

    // Deferred effect actions collected during tick, drained by app loop
    pending_effect_actions: Vec<EffectAction>,
}

impl UnitCoordinator {
    pub fn new() -> Self {
        Self {
            units: Vec::new(),
            selection: SelectionState::new(),
            drag: DragState::None,
            pool: ObjectPool::new(),
            cell_grid: CellGrid::new(),
            person_handles: Vec::new(),
            region_map: RegionMap::new(),
            segment_pool: SegmentPool::new(),
            failure_cache: FailureCache::new(),
            used_targets: UsedTargetsCache::new(),
            landscape_size: 128.0,
            anim_frame_counts: Vec::new(),
            rng: GameRng::new(0x1234),
            pending_effect_actions: Vec::new(),
        }
    }

    /// Extract person units from level data into live units.
    /// Non-person objects remain as static LevelObjects in main.rs.
    pub fn load_level(
        &mut self,
        units_raw: &[UnitRaw],
        landscape_height: &[[u16; 128]; 128],
        landscape_size: usize,
    ) {
        self.units.clear();
        self.selection.clear();
        self.landscape_size = landscape_size as f32;

        // Reset pool and cell grid
        self.pool.clear();
        self.cell_grid.clear();
        self.person_handles.clear();

        // Reset movement infrastructure
        self.segment_pool = SegmentPool::new();
        self.failure_cache = FailureCache::new();
        self.region_map = RegionMap::new();

        Self::populate_water(&mut self.region_map, landscape_height, landscape_size);
        self.region_map.set_terrain_flags(2, 0x00); // terrain class 2 = building = unwalkable

        log::info!(
            "[unit-ctrl] load_level: {} raw units, landscape_size={}",
            units_raw.len(),
            landscape_size
        );

        for raw in units_raw {
            if raw.model_type() != Some(ModelType::Person) {
                continue;
            }
            if raw.loc_x() == 0 && raw.loc_y() == 0 {
                continue;
            }

            let defaults = person_type_defaults(raw.subtype);
            let mut movement = PersonMovement::default();
            movement.position = WorldCoord::new(raw.loc_x() as i16, raw.loc_y() as i16);
            movement.facing_angle = (raw.angle() & 0x7FF) as u16;
            movement.unit_type = raw.subtype;
            movement.speed = defaults.speed;

            let home = movement.position;
            let pos = movement.position;
            let (cx, cy) = world_to_render_pos(&movement.position, self.landscape_size);

            // Allocate in the object pool (source of truth for allocation)
            if let Ok(handle) =
                self.pool
                    .create(ModelType::Person, raw.subtype, raw.tribe_index(), pos)
            {
                if let Some(obj) = self.pool.get_mut(handle) {
                    obj.header.health = defaults.max_health;
                    obj.header.max_health = defaults.max_health;
                    obj.header.angle = (raw.angle() & 0x7FF) as u16;
                    if let crate::engine::objects::GameObjectData::Person(ref mut pd) = obj.data {
                        pd.movement = movement.clone();
                        pd.cell_x = cx;
                        pd.cell_y = cy;
                        pd.state = PersonState::Idle;
                        pd.prev_state = PersonState::Idle;
                        pd.alive = true;
                        pd.home_pos = home;
                    }
                }
                // Insert into spatial grid
                let cell_idx = CellGrid::cell_index_from_world(&pos);
                self.cell_grid
                    .insert_object(handle, cell_idx, self.pool.slots_mut());
                self.person_handles.push(handle);
            }

            // Also populate the Vec<Unit> compatibility shim
            self.units.push(Unit {
                id: self.units.len(),
                handle: *self
                    .person_handles
                    .last()
                    .expect("person pool allocation succeeded"),
                model_type: ModelType::Person,
                subtype: raw.subtype,
                tribe_index: raw.tribe_index(),
                movement,
                cell_x: cx,
                cell_y: cy,
                state: PersonState::Idle,
                prev_state: PersonState::Idle,
                state_timer: 0,
                state_counter: 0,
                health: defaults.max_health,
                max_health: defaults.max_health,
                target_unit: None,
                attacker_unit: None,
                alive: true,
                home_pos: home,
                behavior_flags: 0,
                wander_duration: 0,
                wander_range: 0,
                linked_obj_id: None,
                bloodlust: false,
                shielded: false,
                anim: AnimationState::default(),
                building_handle: None,
                wood_carried: 0,
                guard_position: None,
                gather_target: None,
            });
            // Initialize idle state with a random timer (matches Person_Init calling Person_SetState)
            let idx = self.units.len() - 1;
            enter_state(&mut self.units[idx], PersonState::Idle, &mut self.rng);
            select_animation(
                &mut self.units[idx].anim,
                PersonState::Idle,
                raw.subtype,
                &self.anim_frame_counts,
                false,
            );
        }
        log::info!("[unit-ctrl] loaded {} person units", self.units.len());
    }

    /// Issue move orders to all selected units targeting `target_world`.
    /// Transitions units into GoToPoint state and calls state_goto.
    pub fn order_move(&mut self, target_world: WorldCoord) {
        self.used_targets.clear();
        for &unit_id in &self.selection.selected {
            if let Some(unit) = self.units.get_mut(unit_id) {
                if !unit.alive {
                    continue;
                }
                let result = state_goto(
                    &self.region_map,
                    &mut self.segment_pool,
                    &self.failure_cache,
                    &mut unit.movement,
                    target_world,
                    &mut self.used_targets,
                );
                if result == RouteResult::NoRoute {
                    unit.movement.flags1 &= !0x1000;
                } else {
                    unit.state = PersonState::GoToPoint;
                    unit.target_unit = None; // Cancel combat
                                             // Restore subtype speed (enter_idle sets it to 0)
                    unit.movement.speed = person_type_defaults(unit.subtype).speed;
                }
                log::info!(
                    "[move-order] unit {} result={:?} state={:?} target=({}, {})",
                    unit_id,
                    result,
                    unit.state,
                    unit.movement.target_pos.x,
                    unit.movement.target_pos.z
                );
            }
        }
    }

    /// Advance all units by one tick: state machine + movement + combat + drowning.
    pub fn tick(&mut self) {
        let unit_count = self.units.len();

        // Collect deferred actions and dead handles during person iteration
        let mut deferred_actions: Vec<(usize, DeferredAction)> = Vec::new();
        let mut dead_indices: Vec<usize> = Vec::new();

        // Phase 1: State machine tick + movement for each unit
        for i in 0..unit_count {
            let unit = &mut self.units[i];
            if !unit.alive {
                continue;
            }

            // Run state machine tick
            let (result, deferred) = tick_state(unit, &mut self.rng);
            if let TickResult::Transition(new_state) = result {
                enter_state(unit, new_state, &mut self.rng);
            }

            // Collect deferred actions for post-loop processing
            if deferred != DeferredAction::None {
                deferred_actions.push((i, deferred));
            }

            // Check if unit just became dead (alive=false after tick_dead countdown)
            if !unit.alive {
                dead_indices.push(i);
            }

            // Select animation every tick (matches decomp — walk→idle override needs movement check)
            select_animation(
                &mut unit.anim,
                unit.state,
                unit.subtype,
                &self.anim_frame_counts,
                unit.movement.is_moving(),
            );

            // Advance animation frame
            tick_animation(&mut unit.anim);

            // Process movement for moving states
            if unit.movement.is_moving() {
                Self::advance_movement(&mut self.segment_pool, unit, self.landscape_size);
            }

            // Update rendering cache
            let (cx, cy) = world_to_render_pos(&unit.movement.position, self.landscape_size);
            unit.cell_x = cx;
            unit.cell_y = cy;
        }

        // Phase 2: Drowning detection
        for i in 0..unit_count {
            let unit = &self.units[i];
            if !unit.alive {
                continue;
            }
            if unit.state == PersonState::Drowning || unit.state == PersonState::Dead {
                continue;
            }

            let tile = unit.movement.position.to_tile();
            if !self.region_map.is_walkable(tile) {
                let unit = &mut self.units[i];
                enter_state(unit, PersonState::Drowning, &mut self.rng);
            }
        }

        // Phase 3: Combat detection — idle/wander units auto-engage nearby enemies
        self.detect_combat();

        // Phase 4: Process combat damage for fighting units
        self.process_combat();

        // Phase 5: Process deferred actions (building occupancy, wood deposit, etc.)
        // CRITICAL: Must happen after combat so dying units' final actions are processed.
        self.process_deferred_actions(deferred_actions);

        // Phase 6: Process deaths — units whose alive flag was cleared by tick_dead
        self.process_dead_units(dead_indices);
    }

    /// Process deferred actions collected during person tick loop.
    fn process_deferred_actions(&mut self, actions: Vec<(usize, DeferredAction)>) {
        for (unit_idx, action) in actions {
            match action {
                DeferredAction::None => {}
                DeferredAction::AddToBuilding { person, building } => {
                    if let Some(building_obj) = self.pool.get_mut(building) {
                        if let GameObjectData::Building(ref mut bd) = building_obj.data {
                            let _ = buildings::add_occupant(bd, person);
                        }
                    }
                }
                DeferredAction::RemoveFromBuilding { person, building } => {
                    if let Some(building_obj) = self.pool.get_mut(building) {
                        if let GameObjectData::Building(ref mut bd) = building_obj.data {
                            buildings::remove_occupant(bd, person);
                        }
                    }
                }
                DeferredAction::DepositWood { building, amount } => {
                    if let Some(building_obj) = self.pool.get_mut(building) {
                        if let GameObjectData::Building(ref mut bd) = building_obj.data {
                            bd.wood_stored += amount;
                        }
                    }
                }
                DeferredAction::SpawnAtBuilding { building: _ } => {
                    // Spawn processing handled by population tick subsystem
                    // (coordinator just records the intent; actual spawn happens in
                    // tick_update_population to avoid re-entrant pool mutation)
                }
                DeferredAction::FindNearestTree { unit_index } => {
                    if let Some(unit) = self.units.get(unit_index) {
                        let pos = unit.movement.position;
                        if let Some(tree_pos) =
                            crate::engine::economy::wood::find_nearest_tree_position(
                                &pos,
                                &self.cell_grid,
                                &self.pool,
                            )
                        {
                            if let Some(unit) = self.units.get_mut(unit_index) {
                                unit.gather_target = Some(tree_pos);
                                unit.state_timer = 1; // mark as "has target, navigating"
                            }
                        }
                        // If no tree found, state_timer stays 0 and next tick will retry
                    }
                }
            }
        }
    }

    /// Process dead units: call process_death, remove from cell grid, destroy in pool.
    fn process_dead_units(&mut self, dead_indices: Vec<usize>) {
        for idx in dead_indices.iter().rev() {
            let unit = &self.units[*idx];
            let tribe = unit.tribe_index;
            let last_attacker = None; // attacker tracking is coordinator-level
            let pos = unit.movement.position;

            // Spawn death puff effect at unit's position
            self.pending_effect_actions.push(EffectAction::SpawnAt {
                effect_type: EffectType::DeathPuff as u8,
                x: pos.x as i32,
                y: pos.z as i32, // z in WorldCoord maps to y in effect space
                z: 0,
                owner: tribe,
            });

            // Get death actions (kill tracking)
            let handle = self.person_handles.get(*idx).copied();
            let Some(handle) = handle else { continue };
            let death_actions = combat::process_death(handle, tribe, last_attacker);

            // If unit has a pool handle in person_handles, clean up pool and grid
            // (For units also tracked in pool, handle cleanup)
            let cell_idx = CellGrid::cell_index_from_world(&pos);

            // Find matching person handle by index
            if *idx < self.person_handles.len() {
                self.cell_grid
                    .remove_object(handle, cell_idx, self.pool.slots_mut());
                self.pool.destroy(handle);
            }

            // Track kill for killer tribe if applicable
            if let Some(_killer_tribe) = death_actions.last_attacker_tribe {
                // Kill count tracking deferred to stats subsystem
            }
        }

        // Remove dead entries from person_handles (reverse order to preserve indices)
        for idx in dead_indices.iter().rev() {
            if *idx < self.person_handles.len() {
                self.person_handles.remove(*idx);
            }
        }
    }

    /// Move a unit one step along its path (waypoint advancement + position update).
    fn advance_movement(segment_pool: &mut SegmentPool, unit: &mut Unit, _landscape_size: f32) {
        // Waypoint advancement for pathfind-routed movement
        if unit.state == PersonState::GoToPoint
            || unit.state == PersonState::GoToMarker
            || unit.state == PersonState::Moving
        {
            process_route_movement(segment_pool, &mut unit.movement);
        }

        // Compute facing angle toward next waypoint (for routed movement)
        // or use existing facing_angle (for wander/flee)
        if unit.state == PersonState::GoToPoint
            || unit.state == PersonState::GoToMarker
            || unit.state == PersonState::Moving
        {
            let dx = toroidal_delta(unit.movement.position.x, unit.movement.next_waypoint.x);
            let dz = toroidal_delta(unit.movement.position.z, unit.movement.next_waypoint.z);

            // Check arrival at destination
            if dx.abs() < 0x48 && dz.abs() < 0x48 {
                if unit.movement.segment_index == 0 {
                    unit.movement.position = unit.movement.target_pos;
                    unit.movement.flags1 &= !0x1000; // Clear MOVING
                }
                return;
            }
            unit.movement.facing_angle = atan2(dx, -dz);
        }

        // Advance position by speed in facing direction
        move_point_by_angle(
            &mut unit.movement.position,
            unit.movement.facing_angle,
            unit.movement.speed as i16,
        );
    }

    /// Detect nearby enemies and enter combat for idle/wandering units.
    fn detect_combat(&mut self) {
        // Collect (unit_index, target_index) pairs to avoid borrow issues
        let mut engagements: Vec<(usize, usize)> = Vec::new();

        for i in 0..self.units.len() {
            let unit = &self.units[i];
            if !unit.alive {
                continue;
            }
            // Only idle/wandering units auto-engage
            if unit.state != PersonState::Idle && unit.state != PersonState::Wander {
                continue;
            }

            let mut best_dist = COMBAT_DETECT_RANGE as i32 + 1;
            let mut best_target: Option<usize> = None;

            for j in 0..self.units.len() {
                if i == j {
                    continue;
                }
                let other = &self.units[j];
                if !other.alive {
                    continue;
                }
                if other.tribe_index == unit.tribe_index {
                    continue;
                } // Same tribe
                if other.state == PersonState::Dead {
                    continue;
                }

                let dx = toroidal_delta(unit.movement.position.x, other.movement.position.x) as i32;
                let dz = toroidal_delta(unit.movement.position.z, other.movement.position.z) as i32;
                let dist = dx.abs() + dz.abs(); // Manhattan distance (fast approximation)

                if dist < best_dist {
                    best_dist = dist;
                    best_target = Some(j);
                }
            }

            if let Some(target) = best_target {
                engagements.push((i, target));
            }
        }

        // Apply engagements
        for (attacker_idx, target_idx) in engagements {
            let target_id = self.units[target_idx].id;
            let target_pos = self.units[target_idx].movement.position;
            let unit = &mut self.units[attacker_idx];
            unit.target_unit = Some(target_id);
            enter_state(unit, PersonState::Fighting, &mut self.rng);

            // Face toward target
            let dx = toroidal_delta(unit.movement.position.x, target_pos.x);
            let dz = toroidal_delta(unit.movement.position.z, target_pos.z);
            unit.movement.facing_angle = atan2(dx, -dz);
        }
    }

    /// Process combat: drive sub-phase transitions based on distance to target.
    /// Original: Person_ProcessCombatState routes through sub-phases at offset 0x2D.
    /// - Seek/Approach: chase when out of melee range
    /// - SwingReady→Strike: pause then deal damage when in melee range
    /// - Lunge/Recovering: managed by tick_fighting in person_state.rs
    fn process_combat(&mut self) {
        // Collect damage events: (target_index, damage, attacker_tribe)
        let mut damage_events: Vec<(usize, u16, u8)> = Vec::new();

        for i in 0..self.units.len() {
            let unit = &self.units[i];
            if !unit.alive || unit.state != PersonState::Fighting {
                continue;
            }

            let target_id = match unit.target_unit {
                Some(id) => id,
                None => continue,
            };

            // Find target by ID
            let target_idx = match self.units.iter().position(|u| u.id == target_id) {
                Some(idx) => idx,
                None => continue,
            };

            let target = &self.units[target_idx];
            if !target.alive || target.health == 0 {
                continue;
            }

            let target_pos = target.movement.position;
            let dx = toroidal_delta(unit.movement.position.x, target_pos.x) as i32;
            let dz = toroidal_delta(unit.movement.position.z, target_pos.z) as i32;
            let dist = dx.abs() + dz.abs();

            let phase = CombatPhase::from_counter(self.units[i].state_counter);

            match phase {
                CombatPhase::Seek => {
                    // Start approaching if within detect range
                    if dist <= COMBAT_DETECT_RANGE as i32 {
                        self.units[i].state_counter = CombatPhase::Approach as u8;
                    } else {
                        // Target escaped detect range — disengage
                        self.units[i].target_unit = None;
                    }
                }
                CombatPhase::Approach => {
                    if dist <= COMBAT_MELEE_RANGE as i32 {
                        // Arrived in melee range — stop and prepare to swing
                        self.units[i].movement.flags1 &= !0x1000;
                        self.units[i].movement.speed = 0;
                        self.units[i].state_counter = CombatPhase::SwingReady as u8;
                        self.units[i].state_timer = SWING_READY_TICKS;
                    } else if dist <= COMBAT_DETECT_RANGE as i32 {
                        // Chase: walk toward target
                        let defaults = person_type_defaults(self.units[i].subtype);
                        self.units[i].movement.speed = defaults.speed;
                        self.units[i].movement.flags1 |= 0x1080;
                        self.units[i].movement.facing_angle = atan2(
                            toroidal_delta(self.units[i].movement.position.x, target_pos.x),
                            -toroidal_delta(self.units[i].movement.position.z, target_pos.z),
                        );
                    } else {
                        self.units[i].target_unit = None;
                    }
                }
                CombatPhase::Strike => {
                    // tick_fighting sets Strike phase; we apply damage here
                    let damage = calculate_melee_damage(&self.units[i]);
                    damage_events.push((target_idx, damage, self.units[i].tribe_index));
                    // tick_fighting will advance to LungeBack on next tick
                }
                CombatPhase::SwingReady
                | CombatPhase::LungeBack
                | CombatPhase::LungeFwd
                | CombatPhase::Recovering => {
                    // These phases are timer-driven by tick_fighting — no coordinator action
                    // Face target while waiting
                    self.units[i].movement.facing_angle = atan2(
                        toroidal_delta(self.units[i].movement.position.x, target_pos.x),
                        -toroidal_delta(self.units[i].movement.position.z, target_pos.z),
                    );
                }
            }
        }

        // Apply damage
        for (target_idx, damage, _attacker_tribe) in damage_events {
            let target = &mut self.units[target_idx];
            let target_pos = target.movement.position;
            // Spawn hit spark effect at target position
            self.pending_effect_actions.push(EffectAction::SpawnAt {
                effect_type: EffectType::HitSpark as u8,
                x: target_pos.x as i32,
                y: target_pos.z as i32,
                z: 0,
                owner: target.tribe_index,
            });
            apply_damage(target, damage);
            if target.health == 0 {
                // Spawn blood spray on fatal hit
                self.pending_effect_actions.push(EffectAction::SpawnAt {
                    effect_type: EffectType::BloodSpray as u8,
                    x: target_pos.x as i32,
                    y: target_pos.z as i32,
                    z: 0,
                    owner: target.tribe_index,
                });
                enter_state(target, PersonState::Dead, &mut self.rng);
            }
        }

        // Clear target for units whose target died
        for i in 0..self.units.len() {
            if self.units[i].state != PersonState::Fighting {
                continue;
            }
            if let Some(target_id) = self.units[i].target_unit {
                if let Some(target) = self.units.iter().find(|u| u.id == target_id) {
                    if !target.alive || target.state == PersonState::Dead {
                        self.units[i].target_unit = None;
                    }
                }
            }
        }
    }

    /// Tick all buildings in the pool. Processes spawn, convert, and combat actions.
    pub fn tick_buildings(&mut self) {
        let building_handles: Vec<ObjectHandle> =
            self.pool.buildings().map(|(h, _, _)| h).collect();

        // Phase 1: tick each building, collect actions + effect spawns for state transitions
        let mut all_actions: Vec<(ObjectHandle, BuildingTickActions)> = Vec::new();
        for handle in building_handles {
            if let Some(obj) = self.pool.get_mut(handle) {
                if let GameObjectData::Building(ref mut bd) = obj.data {
                    let state_before = bd.state;
                    let actions = buildings::tick::tick_building(bd, &mut obj.header, handle);
                    let state_after = bd.state;
                    let pos = obj.header.position;
                    let tribe = obj.header.tribe;
                    // Spawn effects on state transitions
                    if state_before == BuildingState::Init && state_after != BuildingState::Init {
                        // Construction completing -> construction dust
                        self.pending_effect_actions.push(EffectAction::SpawnAt {
                            effect_type: EffectType::ConstructionDust as u8,
                            x: pos.x as i32,
                            y: pos.z as i32,
                            z: 0,
                            owner: tribe,
                        });
                    }
                    if state_before == BuildingState::Active
                        && state_after == BuildingState::Destroying
                    {
                        // Building destroyed -> destruction collapse
                        self.pending_effect_actions.push(EffectAction::SpawnAt {
                            effect_type: EffectType::DestructionCollapse as u8,
                            x: pos.x as i32,
                            y: pos.z as i32,
                            z: 0,
                            owner: tribe,
                        });
                    }
                    if state_after == BuildingState::Destroying && bd.damage_accumulated > 0 {
                        // Building on fire while destroying
                        self.pending_effect_actions.push(EffectAction::SpawnAt {
                            effect_type: EffectType::BuildingFire as u8,
                            x: pos.x as i32,
                            y: pos.z as i32,
                            z: 0,
                            owner: tribe,
                        });
                    }
                    all_actions.push((handle, actions));
                }
            }
        }

        // Phase 2: process spawn actions
        for (building_handle, actions) in &all_actions {
            if actions.spawn == SpawnAction::SpawnBrave {
                if let Some(obj) = self.pool.get(*building_handle) {
                    let pos = obj.header.position;
                    let tribe = obj.header.tribe;
                    self.spawn_brave_near(pos, tribe);
                }
            }
        }

        // Phase 3: process convert actions
        for (_building_handle, actions) in &all_actions {
            if let ConvertAction::ConvertUnit {
                handle,
                new_subtype,
            } = &actions.convert
            {
                if let Some(obj) = self.pool.get_mut(*handle) {
                    obj.header.subtype = *new_subtype;
                }
            }
        }

        // Phase 4: process building combat actions
        for (_building_handle, actions) in &all_actions {
            for combat_action in &actions.combat {
                if let BuildingCombatAction::AttackTarget { target, damage, .. } = combat_action {
                    if let Some(target_obj) = self.pool.get_mut(*target) {
                        if let GameObjectData::Building(ref mut bd) = target_obj.data {
                            buildings::apply_building_damage(bd, &mut target_obj.header, *damage);
                        } else {
                            let dmg = (*damage).min(target_obj.header.health);
                            target_obj.header.health -= dmg;
                        }
                    }
                }
            }
        }
    }

    /// Spawn a brave person near a building position.
    fn spawn_brave_near(&mut self, building_pos: WorldCoord, tribe: u8) {
        // Offset spawn position slightly from building (one cell = 128 world units)
        let spawn_pos = WorldCoord::new(
            building_pos.x.wrapping_add(128),
            building_pos.z.wrapping_add(64),
        );
        let subtype = 2; // Brave
        let defaults = person_type_defaults(subtype);

        if let Ok(handle) = self
            .pool
            .create(ModelType::Person, subtype, tribe, spawn_pos)
        {
            if let Some(obj) = self.pool.get_mut(handle) {
                obj.header.health = defaults.max_health;
                obj.header.max_health = defaults.max_health;
                if let GameObjectData::Person(ref mut pd) = obj.data {
                    pd.movement.position = spawn_pos;
                    pd.movement.unit_type = subtype;
                    pd.movement.speed = defaults.speed;
                    pd.state = PersonState::Idle;
                    pd.prev_state = PersonState::Idle;
                    pd.alive = true;
                    pd.home_pos = building_pos;
                    let (cx, cy) = world_to_render_pos(&spawn_pos, self.landscape_size);
                    pd.cell_x = cx;
                    pd.cell_y = cy;
                }
            }
            // Insert into spatial grid
            let cell_idx = CellGrid::cell_index_from_world(&spawn_pos);
            self.cell_grid
                .insert_object(handle, cell_idx, self.pool.slots_mut());
            self.person_handles.push(handle);
        }
    }

    /// Tick all projectiles in the pool. Returns list of impacts:
    /// (position, damage, aoe_radius, knockback_force).
    pub fn tick_projectiles(&mut self) -> Vec<(WorldCoord, u16, u16, u16)> {
        let shot_handles: Vec<ObjectHandle> = self.pool.shots().map(|(h, _, _)| h).collect();

        let mut impacts = Vec::new();
        let mut expired = Vec::new();

        for handle in shot_handles {
            if let Some(obj) = self.pool.get_mut(handle) {
                if let GameObjectData::Shot(ref mut sd) = obj.data {
                    match combat::tick_projectile(sd, &mut obj.header) {
                        combat::ProjectileResult::Impact {
                            position,
                            damage,
                            aoe_radius,
                            knockback_force,
                        } => {
                            impacts.push((position, damage, aoe_radius, knockback_force));
                            expired.push(handle);
                        }
                        combat::ProjectileResult::Expired => {
                            expired.push(handle);
                        }
                        combat::ProjectileResult::Continue => {}
                    }
                }
            }
        }

        for handle in expired {
            self.pool.destroy(handle);
        }

        impacts
    }

    /// Process projectile impacts: apply knockback and AOE damage to nearby persons.
    /// For each impact, queries the CellGrid for persons within the AOE radius,
    /// then applies knockback velocity and damage to each affected unit.
    fn process_projectile_impacts(&mut self, impacts: Vec<(WorldCoord, u16, u16, u16)>) {
        for (impact_pos, damage, aoe_radius, knockback_force) in &impacts {
            if *aoe_radius == 0 && *knockback_force == 0 {
                continue;
            }

            // Find all persons within aoe_radius of impact_pos using cell grid
            let tile = impact_pos.to_tile();
            let cell_radius = (*aoe_radius as i32 / 128).max(1); // convert world units to cells
            let mut affected: Vec<ObjectHandle> = Vec::new();

            for dx in -cell_radius..=cell_radius {
                for dz in -cell_radius..=cell_radius {
                    let cx = ((tile.x as i32 + dx) & 127) as u8;
                    let cz = ((tile.z as i32 + dz) & 127) as u8;
                    let cell_idx = cz as usize * 128 + cx as usize;
                    let mut current = self.cell_grid.cell_head(cell_idx);
                    while let Some(handle) = current {
                        if let Some(obj) = self.pool.get(handle) {
                            if obj.header.model_type == ModelType::Person
                                || obj.header.model_type == ModelType::Building
                            {
                                affected.push(handle);
                            }
                            current = obj.header.next_in_cell;
                        } else {
                            break;
                        }
                    }
                }
            }

            // Apply knockback and AOE damage to each affected object
            for handle in affected {
                if let Some(obj) = self.pool.get_mut(handle) {
                    // Apply knockback (persons only)
                    if *knockback_force > 0 && obj.header.model_type == ModelType::Person {
                        combat::apply_knockback(
                            &obj.header.position,
                            &mut obj.header.velocity,
                            impact_pos,
                            *knockback_force,
                        );
                    }
                    // Apply AOE damage
                    if *damage > 0 {
                        if let GameObjectData::Building(ref mut bd) = obj.data {
                            buildings::apply_building_damage(bd, &mut obj.header, *damage);
                        } else {
                            let dmg = (*damage).min(obj.header.health);
                            obj.header.health -= dmg;
                        }
                    }
                }
            }
        }
    }

    /// Mark height-0 cells as water (unwalkable) in the region map,
    /// then erode one cell inward so shore-adjacent land is also unwalkable.
    /// Water cells get region_id=1 so `same_region` returns false when
    /// routing between land (region 0) and water, forcing the pathfinder
    /// to engage and reject the unwalkable target.
    fn populate_water(
        region_map: &mut RegionMap,
        landscape_height: &[[u16; 128]; 128],
        size: usize,
    ) {
        region_map.set_terrain_flags(1, 0x00); // terrain class 1 = water = unwalkable
        region_map.set_terrain_flags(3, 0x00); // terrain class 3 = shore buffer = unwalkable
        let ni = size as i32;
        let n = size;

        // Pass 1: mark fully-submerged cells as water
        let mut is_water = vec![false; n * n];
        for cell_y in 0..size {
            for cell_x in 0..size {
                let cy1 = (cell_y + 1) % n;
                let cx1 = (cell_x + 1) % n;
                let all_water = landscape_height[cell_y][cell_x] == 0
                    && landscape_height[cell_y][cx1] == 0
                    && landscape_height[cy1][cell_x] == 0
                    && landscape_height[cy1][cx1] == 0;
                if all_water {
                    is_water[cell_y * n + cell_x] = true;
                    let tile = cell_to_tile(cell_x as i32, cell_y as i32, ni);
                    let cell = region_map.get_cell_mut(tile);
                    cell.terrain_type = 1;
                    region_map.set_cell_region(tile, 1); // water region
                }
            }
        }

        // Pass 2: erode — mark non-water cells adjacent to water as shore buffer
        for cell_y in 0..size {
            for cell_x in 0..size {
                if is_water[cell_y * n + cell_x] {
                    continue; // already water
                }
                let neighbors = [
                    (cell_x, (cell_y + n - 1) % n), // north
                    (cell_x, (cell_y + 1) % n),     // south
                    ((cell_x + n - 1) % n, cell_y), // west
                    ((cell_x + 1) % n, cell_y),     // east
                ];
                let adjacent_to_water = neighbors.iter().any(|&(nx, ny)| is_water[ny * n + nx]);
                if adjacent_to_water {
                    let tile = cell_to_tile(cell_x as i32, cell_y as i32, ni);
                    let cell = region_map.get_cell_mut(tile);
                    cell.terrain_type = 3; // shore buffer
                }
            }
        }
    }

    /// Rebuild the Vec<Unit> compatibility shim from pool persons.
    /// Called at end of tick() so rendering consumers see current state.
    fn sync_units_from_pool(&mut self) {
        self.units.clear();
        for (handle, header, person) in self.pool.persons() {
            self.units.push(Unit {
                id: handle.slot() as usize,
                handle,
                model_type: header.model_type,
                subtype: header.subtype,
                tribe_index: header.tribe,
                movement: person.movement.clone(),
                cell_x: person.cell_x,
                cell_y: person.cell_y,
                state: person.state,
                prev_state: person.prev_state,
                state_timer: person.state_timer,
                state_counter: person.state_counter,
                health: header.health,
                max_health: header.max_health,
                target_unit: person.target_unit.map(|h| h.slot() as usize),
                attacker_unit: person.attacker_unit.map(|h| h.slot() as usize),
                alive: person.alive,
                home_pos: person.home_pos,
                behavior_flags: person.behavior_flags,
                wander_duration: person.wander_duration,
                wander_range: person.wander_range,
                linked_obj_id: person.linked_obj_id.map(|h| h.slot() as usize),
                bloodlust: person.bloodlust,
                shielded: person.shielded,
                anim: person.anim,
                building_handle: person.building_handle,
                wood_carried: person.wood_carried,
                guard_position: person.guard_position,
                gather_target: person.gather_target,
            });
        }
    }

    /// Read-only access to the units Vec (compatibility shim).
    pub fn units(&self) -> &[Unit] {
        &self.units
    }

    /// Access the object pool.
    /// Drain pending effect actions collected during the last tick.
    pub fn drain_effect_actions(&mut self) -> Vec<EffectAction> {
        std::mem::take(&mut self.pending_effect_actions)
    }

    pub fn pool(&self) -> &ObjectPool {
        &self.pool
    }

    /// Mutable access to the object pool.
    pub fn pool_mut(&mut self) -> &mut ObjectPool {
        &mut self.pool
    }

    /// Access the cell grid.
    pub fn cell_grid(&self) -> &CellGrid {
        &self.cell_grid
    }

    pub fn region_map(&self) -> &RegionMap {
        &self.region_map
    }

    pub fn region_map_mut(&mut self) -> &mut RegionMap {
        &mut self.region_map
    }
}

/// ObjectTick implementation — plugs UnitCoordinator into GameWorld's tick loop.
/// Original: Tick_UpdateObjects (0x004a7550) processes persons, buildings, projectiles.
impl ObjectTick for UnitCoordinator {
    fn tick_update_objects(&mut self) {
        // 5a. Person state ticks, movement, combat, deferred actions, death handling
        self.tick();
        // 5b. Building state ticks (construction, spawn timers, damage)
        self.tick_buildings();
        // 5c. Projectile ticks (movement, impact detection, expiry)
        let impacts = self.tick_projectiles();
        // 5d. Process projectile impacts: knockback + AOE damage to nearby persons
        self.process_projectile_impacts(impacts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_level_filters_persons() {
        // UnitRaw is repr(C, packed) — we can't easily construct one in tests
        // without unsafe. Test via the coordinator's public interface instead.
        let coord = UnitCoordinator::new();
        assert!(coord.units().is_empty());
        assert_eq!(coord.landscape_size, 128.0);
    }

    #[test]
    fn populate_water_marks_unwalkable() {
        // Create a water block: cells (10,10)..(13,13) all have height 0
        // so cells (10,10), (10,11), (10,12), (11,10), (11,11), (11,12),
        // (12,10), (12,11), (12,12) have all 4 corners at 0 → water.
        let mut height = [[100u16; 128]; 128];
        for y in 10..=13 {
            for x in 10..=13 {
                height[y][x] = 0;
            }
        }

        let mut map = RegionMap::new();
        UnitCoordinator::populate_water(&mut map, &height, 128);

        // Interior water cell should be unwalkable
        let water_tile = cell_to_tile(11, 11, 128);
        assert!(!map.is_walkable(water_tile));

        // Far-away land cell should remain walkable
        let land = cell_to_tile(50, 50, 128);
        assert!(map.is_walkable(land));
    }

    #[test]
    fn populate_water_all_land() {
        // No water at all — everything should be walkable
        let height = [[50u16; 128]; 128];
        let mut map = RegionMap::new();
        UnitCoordinator::populate_water(&mut map, &height, 128);
        let t1 = cell_to_tile(0, 0, 128);
        assert!(map.is_walkable(t1));
        let t2 = cell_to_tile(127, 127, 128);
        assert!(map.is_walkable(t2));
    }

    #[test]
    fn populate_water_shore_erosion() {
        // Create a water block at cells (10,10)..(13,13)
        // Water cells: (10..12, 10..12) — all 4 corners at height 0
        // Shore buffer: cells adjacent to water but not water themselves
        let mut height = [[100u16; 128]; 128];
        for y in 10..=13 {
            for x in 10..=13 {
                height[y][x] = 0;
            }
        }

        let mut map = RegionMap::new();
        UnitCoordinator::populate_water(&mut map, &height, 128);

        // Cell (9, 11) is land but adjacent to water cell (10, 11) → shore buffer
        let shore_tile = cell_to_tile(9, 11, 128);
        assert!(
            !map.is_walkable(shore_tile),
            "shore-adjacent cell should be unwalkable"
        );

        // Cell (13, 11) is land but adjacent to water cell (12, 11) → shore buffer
        let shore_tile2 = cell_to_tile(13, 11, 128);
        assert!(
            !map.is_walkable(shore_tile2),
            "shore-adjacent cell should be unwalkable"
        );

        // Cell (8, 11) is 2 cells away from water → should remain walkable
        let far_land = cell_to_tile(8, 11, 128);
        assert!(
            map.is_walkable(far_land),
            "cell 2 away from water should be walkable"
        );
    }
}
