use std::collections::HashMap;

use crate::data::level::{LevelDefinition, LevelObjectIndex};
use crate::data::units::ModelType;
use crate::engine::buildings::spawning::population_growth_weight;
use crate::engine::buildings::tick::{
    construction_delivery_phase, construction_progress_target, construction_target,
    tick_building_with_population, CONSTRUCTION_UNITS_PER_WOOD,
};
use crate::engine::buildings::types::{building_behavior_flags, building_max_health};
use crate::engine::buildings::{
    BuildingCatalog, BuildingState, BuildingSubtype, PlacementError, SpawnAction,
};
use crate::engine::economy::population::{calculate_housing_capacity, can_spawn};
use crate::engine::movement::{atan2, WorldCoord};
use crate::engine::objects::{CellGrid, GameObjectData, ObjectHandle, ObjectPool};
use crate::engine::state::rng::GameRng;
use crate::engine::state::tribe::TribeArray;
use crate::engine::units::coords::{
    cell_to_world, toroidal_delta, world_to_cell, world_to_render_pos,
};
use crate::engine::units::person_state::{person_type_defaults, PersonState};

const WORLD_SIZE: usize = 128;
const BRAVE_SUBTYPE: u8 = 2;
const TREE_MAX_SUBTYPE: u8 = 8;
const TREE_WOOD_PIECES: u8 = 4;
const PERSON_MOVE_STEP: i16 = 32;
const FOUNDATION_STROKE_HEIGHT: u16 = 16;
const ORIGINAL_TICKS_PER_SECOND: u16 = 14;
const WORLD_TICKS_PER_SECOND: u16 = 30;
const CHOP_TICKS: u16 = original_ticks_to_world_ticks(20);
const DELIVERY_TICKS: u16 = original_ticks_to_world_ticks(8);
const FINAL_BUILD_TICKS: u16 = original_ticks_to_world_ticks(15);
const SITE_WORK_MIN_ORIGINAL_TICKS: u16 = 32;
const SITE_WORK_RANDOM_MASK: u32 = 0x3f;
const WORK_ANIMATION_TICKS_PER_FRAME: u8 = original_ticks_to_world_ticks(3) as u8;
const FOUNDATION_STROKE_TICKS: u16 = WORK_ANIMATION_TICKS_PER_FRAME as u16 * 5;

const BUILD_PHASE_TRAVEL_OR_FLATTEN: u8 = 0;
const BUILD_PHASE_DELIVER: u8 = 1;
const BUILD_PHASE_SITE_WORK: u8 = 2;
const BUILD_PHASE_WAIT_WOOD: u8 = 3;
const BUILD_PHASE_FINALIZE: u8 = 4;
const BUILD_PHASE_WAIT_FINALIZE: u8 = 5;

const fn original_ticks_to_world_ticks(ticks: u16) -> u16 {
    (ticks * WORLD_TICKS_PER_SECOND).div_ceil(ORIGINAL_TICKS_PER_SECOND)
}

#[derive(Debug, Clone)]
pub struct TerrainState {
    pub heights: Box<[[u16; WORLD_SIZE]; WORLD_SIZE]>,
    pub walkability: Box<[[u8; WORLD_SIZE]; WORLD_SIZE]>,
    occupancy: Box<[[Option<ObjectHandle>; WORLD_SIZE]; WORLD_SIZE]>,
    revision: u64,
}

impl TerrainState {
    pub fn new(heights: Box<[[u16; WORLD_SIZE]; WORLD_SIZE]>) -> Self {
        let mut walkability = Box::new([[0u8; WORLD_SIZE]; WORLD_SIZE]);
        for y in 0..WORLD_SIZE {
            for x in 0..WORLD_SIZE {
                if heights[y][x] == 0 {
                    walkability[y][x] |= 0x04;
                }
                let east = heights[y][(x + 1) & 127];
                let south = heights[(y + 1) & 127][x];
                let h = heights[y][x];
                if h.abs_diff(east).max(h.abs_diff(south)) > 256 {
                    walkability[y][x] |= 0x02;
                }
            }
        }
        Self {
            heights,
            walkability,
            occupancy: Box::new([[None; WORLD_SIZE]; WORLD_SIZE]),
            revision: 1,
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn occupant(&self, x: i32, y: i32) -> Option<ObjectHandle> {
        self.occupancy[(y & 127) as usize][(x & 127) as usize]
    }

    fn validate(&self, cell: (i32, i32), footprint: &[(i16, i16)]) -> Result<(), PlacementError> {
        for &(dx, dy) in footprint {
            let x = ((cell.0 + dx as i32) & 127) as usize;
            let y = ((cell.1 + dy as i32) & 127) as usize;
            if self.walkability[y][x] & 0x04 != 0 {
                return Err(PlacementError::Water);
            }
            if self.occupancy[y][x].is_some() {
                return Err(PlacementError::Occupied);
            }
            if self.walkability[y][x] & 0x02 != 0 {
                return Err(PlacementError::TooSteep);
            }
        }
        Ok(())
    }

    fn occupy(&mut self, handle: ObjectHandle, cell: (i32, i32), footprint: &[(i16, i16)]) {
        for &(dx, dy) in footprint {
            let x = ((cell.0 + dx as i32) & 127) as usize;
            let y = ((cell.1 + dy as i32) & 127) as usize;
            self.occupancy[y][x] = Some(handle);
            self.walkability[y][x] = 0;
        }
        self.revision = self.revision.wrapping_add(1).max(1);
    }

    fn flatten_immediately(&mut self, cell: (i32, i32), footprint: &[(i16, i16)], target: u16) {
        for &(dx, dy) in footprint {
            let x = ((cell.0 + dx as i32) & 127) as usize;
            let y = ((cell.1 + dy as i32) & 127) as usize;
            self.heights[y][x] = target;
        }
        self.revision = self.revision.wrapping_add(1).max(1);
    }

    fn uneven_footprint_offset(
        &self,
        cell: (i32, i32),
        footprint: &[(i16, i16)],
        target: u16,
        selector: u32,
    ) -> Option<(i16, i16)> {
        let uneven_count = footprint
            .iter()
            .filter(|&&offset| !self.footprint_offset_is_flat(cell, offset, target))
            .count();
        let selected = selector as usize % uneven_count.max(1);
        footprint
            .iter()
            .copied()
            .filter(|&offset| !self.footprint_offset_is_flat(cell, offset, target))
            .nth(selected)
    }

    fn footprint_offset_is_flat(&self, cell: (i32, i32), offset: (i16, i16), target: u16) -> bool {
        let x = ((cell.0 + offset.0 as i32) & 127) as usize;
        let y = ((cell.1 + offset.1 as i32) & 127) as usize;
        self.heights[y][x] == target
    }

    fn flatten_offset_one_step(
        &mut self,
        cell: (i32, i32),
        offset: (i16, i16),
        target: u16,
    ) -> bool {
        let (dx, dy) = offset;
        let x = ((cell.0 + dx as i32) & 127) as usize;
        let y = ((cell.1 + dy as i32) & 127) as usize;
        let height = &mut self.heights[y][x];
        if *height == target {
            return false;
        }
        *height = if *height < target {
            height.saturating_add(FOUNDATION_STROKE_HEIGHT).min(target)
        } else {
            height.saturating_sub(FOUNDATION_STROKE_HEIGHT).max(target)
        };
        self.revision = self.revision.wrapping_add(1).max(1);
        true
    }

    fn footprint_is_flat(&self, cell: (i32, i32), footprint: &[(i16, i16)], target: u16) -> bool {
        footprint.iter().all(|&(dx, dy)| {
            let x = ((cell.0 + dx as i32) & 127) as usize;
            let y = ((cell.1 + dy as i32) & 127) as usize;
            self.heights[y][x] == target
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LevelInitError {
    PoolExhausted {
        source: LevelObjectIndex,
    },
    InvalidBuildingSubtype {
        source: LevelObjectIndex,
        subtype: u8,
    },
    MissingBuildingFootprint {
        source: LevelObjectIndex,
        subtype: u8,
        rotation: u8,
    },
    OccupiedFootprint {
        source: LevelObjectIndex,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorldError {
    PoolExhausted,
    InvalidHandle(ObjectHandle),
    InvalidBuildingSubtype(u8),
    MissingBuildingFootprint { subtype: u8, rotation: u8 },
    InvalidPlacement(PlacementError),
    NotConstructionSite(ObjectHandle),
    InvalidBuilder(ObjectHandle),
    BuilderOwnerMismatch(ObjectHandle),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersonRenderRecord {
    pub handle: ObjectHandle,
    pub subtype: u8,
    pub tribe: u8,
    /// The sidebar's native person-class counters omit dead people even while
    /// their render records remain available for the death animation.
    pub alive: bool,
    pub cell_x: f32,
    pub cell_y: f32,
    pub angle: u16,
    pub health: u16,
    pub max_health: u16,
    pub animation_id: u16,
    pub animation_frame: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldObjectRenderRecord {
    pub handle: ObjectHandle,
    pub model_type: ModelType,
    pub subtype: u8,
    pub tribe: u8,
    pub cell_x: f32,
    pub cell_y: f32,
    pub angle: u16,
    pub health: u16,
    pub max_health: u16,
    pub building_state: Option<BuildingState>,
    pub construction_progress: u16,
    pub construction_phase: u8,
    pub visual_variant: u8,
    pub footprint: Vec<(i16, i16)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TribeSummary {
    pub tribe: u8,
    pub active: bool,
    pub population: u32,
    pub max_population: u16,
    pub mana: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldSnapshot {
    pub persons: Vec<PersonRenderRecord>,
    pub objects: Vec<WorldObjectRenderRecord>,
    pub tribes: Vec<TribeSummary>,
    pub selected: Vec<ObjectHandle>,
    pub terrain_revision: u64,
    pub object_revision: u64,
    pub effect_events: Vec<EffectEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectEvent {
    PersonSpawned(ObjectHandle),
    ObjectDestroyed(ObjectHandle),
}

pub struct World {
    pool: ObjectPool,
    cells: CellGrid,
    pub terrain: TerrainState,
    pub tribes: TribeArray,
    pub opaque_tribes: [[u8; 16]; 4],
    catalog: BuildingCatalog,
    source_handles: HashMap<LevelObjectIndex, ObjectHandle>,
    selected: Vec<ObjectHandle>,
    effects: Vec<EffectEvent>,
    object_revision: u64,
}

impl World {
    pub fn from_level(
        level: LevelDefinition,
        catalog: BuildingCatalog,
    ) -> Result<Self, LevelInitError> {
        let mut world = Self {
            pool: ObjectPool::new(),
            cells: CellGrid::new(),
            terrain: TerrainState::new(level.heights),
            tribes: TribeArray::new(),
            opaque_tribes: [[0; 16]; 4],
            catalog,
            source_handles: HashMap::new(),
            selected: Vec::new(),
            effects: Vec::new(),
            object_revision: 1,
        };
        for (i, tribe) in level.tribes.iter().take(4).enumerate() {
            world.opaque_tribes[i] = tribe.data;
        }

        for definition in level.objects {
            let position = WorldCoord::new(definition.position[0], definition.position[1]);
            let rotation = ((definition.angle / 512) & 3) as u8;
            let handle = world
                .pool
                .create(
                    definition.model_type,
                    definition.subtype,
                    definition.tribe,
                    position,
                )
                .map_err(|_| LevelInitError::PoolExhausted {
                    source: definition.source,
                })?;
            world.initialize_object(handle, definition.angle);

            if definition.model_type == ModelType::Building {
                let subtype = BuildingSubtype::try_from(definition.subtype).map_err(|subtype| {
                    LevelInitError::InvalidBuildingSubtype {
                        source: definition.source,
                        subtype,
                    }
                })?;
                let footprint = world
                    .catalog
                    .footprint(subtype, rotation)
                    .ok_or(LevelInitError::MissingBuildingFootprint {
                        source: definition.source,
                        subtype: definition.subtype,
                        rotation,
                    })?
                    .to_vec();
                let cell = render_cell(position);
                if let Some(object) = world.pool.get_mut(handle) {
                    let GameObjectData::Building(building) = &mut object.data else {
                        unreachable!()
                    };
                    building.building_subtype = subtype;
                    building.state = BuildingState::Active;
                    building.behavior_flags = building_behavior_flags(subtype);
                    building.wood_consumed = construction_target(subtype);
                    building.construction_progress = 0;
                    building.construction_phase = 4;
                    building.foundation_height =
                        world.terrain.heights[(cell.1 & 127) as usize][(cell.0 & 127) as usize];
                    building.population_timer_enabled = false;
                    object.header.health = building_max_health(subtype);
                    object.header.max_health = object.header.health;
                }
                let foundation_height =
                    world.terrain.heights[(cell.1 & 127) as usize][(cell.0 & 127) as usize];
                world.terrain.occupy(handle, cell, &footprint);
                world
                    .terrain
                    .flatten_immediately(cell, &footprint, foundation_height);
            }
            world.link(handle);
            world.source_handles.insert(definition.source, handle);
        }
        world.synchronize_tribes();
        Ok(world)
    }

    fn initialize_object(&mut self, handle: ObjectHandle, angle: u16) {
        let Some(object) = self.pool.get_mut(handle) else {
            return;
        };
        object.header.angle = angle;
        match &mut object.data {
            GameObjectData::Person(person) => {
                let defaults = person_type_defaults(object.header.subtype);
                object.header.health = defaults.max_health;
                object.header.max_health = defaults.max_health;
                person.movement.position = object.header.position;
                person.movement.facing_angle = angle;
                person.movement.unit_type = object.header.subtype;
                person.movement.speed = defaults.speed;
                person.state = PersonState::Idle;
                person.prev_state = PersonState::Idle;
                person.home_pos = object.header.position;
                person.alive = true;
                person.anim.animation_id = idle_animation(object.header.subtype);
                let (x, y) = world_to_render_pos(&object.header.position, 128.0);
                person.cell_x = x;
                person.cell_y = y;
            }
            GameObjectData::Scenery(scenery) if object.header.subtype <= TREE_MAX_SUBTYPE => {
                scenery.wood_remaining = TREE_WOOD_PIECES;
            }
            _ => {}
        }
    }

    fn link(&mut self, handle: ObjectHandle) {
        let Some(position) = self.pool.get(handle).map(|o| o.header.position) else {
            return;
        };
        let cell = CellGrid::cell_index_from_world(&position);
        self.cells
            .insert_object(handle, cell, self.pool.slots_mut());
    }

    pub fn pool(&self) -> &ObjectPool {
        &self.pool
    }
    pub fn get(&self, handle: ObjectHandle) -> Option<&crate::engine::objects::GameObject> {
        self.pool.get(handle)
    }
    pub(crate) fn get_mut_for_action(
        &mut self,
        handle: ObjectHandle,
    ) -> Option<&mut crate::engine::objects::GameObject> {
        self.pool.get_mut(handle)
    }
    pub fn source_handle(&self, source: LevelObjectIndex) -> Option<ObjectHandle> {
        self.source_handles.get(&source).copied()
    }
    pub fn cell_head(&self, cell: usize) -> Option<ObjectHandle> {
        self.cells
            .cell_head(cell)
            .filter(|h| self.pool.get(*h).is_some())
    }
    pub fn selected(&self) -> &[ObjectHandle] {
        &self.selected
    }

    pub fn construction_site_at(&self, cell: (i32, i32)) -> Option<ObjectHandle> {
        let handle = self.terrain.occupant(cell.0, cell.1)?;
        self.pool.get(handle).and_then(|object| match &object.data {
            GameObjectData::Building(building) if building.state == BuildingState::Init => {
                Some(handle)
            }
            _ => None,
        })
    }

    pub fn select(&mut self, handles: Vec<ObjectHandle>) {
        self.selected = handles
            .into_iter()
            .filter(|h| self.pool.get(*h).is_some())
            .collect();
    }

    pub fn assign_construction(
        &mut self,
        units: &[ObjectHandle],
        building: ObjectHandle,
    ) -> Result<(), WorldError> {
        let owner = self
            .pool
            .get(building)
            .and_then(|object| match &object.data {
                GameObjectData::Building(data) if data.state == BuildingState::Init => {
                    Some(object.header.tribe)
                }
                _ => None,
            })
            .ok_or(WorldError::NotConstructionSite(building))?;

        for &handle in units {
            let object = self
                .pool
                .get(handle)
                .ok_or(WorldError::InvalidBuilder(handle))?;
            let valid = matches!(&object.data, GameObjectData::Person(person) if person.alive)
                && object.header.subtype == BRAVE_SUBTYPE;
            if !valid {
                return Err(WorldError::InvalidBuilder(handle));
            }
            if object.header.tribe != owner {
                return Err(WorldError::BuilderOwnerMismatch(handle));
            }
        }

        let (_, site_cell, footprint, _) = self
            .construction_site_data(building)
            .ok_or(WorldError::NotConstructionSite(building))?;
        for (builder_index, &handle) in units.iter().enumerate() {
            self.cancel_construction_job(handle);
            let work_offset = footprint
                .get(builder_index % footprint.len().max(1))
                .copied()
                .unwrap_or((0, 0));
            let target = construction_work_position(site_cell, work_offset);
            if let Some(object) = self.pool.get_mut(handle) {
                if let GameObjectData::Person(person) = &mut object.data {
                    person.building_handle = Some(building);
                    person.gather_target = None;
                    person.construction_wood_reserved = false;
                    person.construction_work_offset = Some(work_offset);
                    person.construction_work_progress = 0;
                    person.wood_carried = 0;
                    person.prev_state = person.state;
                    person.state = PersonState::Building;
                    person.state_counter = BUILD_PHASE_TRAVEL_OR_FLATTEN;
                    person.state_timer = 0;
                    person.movement.target_pos = target;
                    person.movement.speed = person_type_defaults(object.header.subtype).speed;
                    person.movement.flags1 |= 0x1000;
                    set_animation(person, walk_animation(object.header.subtype));
                }
            }
        }
        Ok(())
    }

    pub(crate) fn cancel_construction_job(&mut self, person_handle: ObjectHandle) {
        let Some((building, tree, reserved, carrying)) =
            self.pool
                .get(person_handle)
                .and_then(|object| match &object.data {
                    GameObjectData::Person(person) => Some((
                        person.building_handle,
                        person.gather_target,
                        person.construction_wood_reserved,
                        person.wood_carried,
                    )),
                    _ => None,
                })
        else {
            return;
        };

        if reserved {
            if let Some(building) = building {
                if let Some(object) = self.pool.get_mut(building) {
                    if let GameObjectData::Building(data) = &mut object.data {
                        data.wood_reserved = data.wood_reserved.saturating_sub(1);
                    }
                }
            }
            if carrying == 0 {
                if let Some(tree) = tree {
                    if let Some(object) = self.pool.get_mut(tree) {
                        if let GameObjectData::Scenery(data) = &mut object.data {
                            data.wood_reserved = data.wood_reserved.saturating_sub(1);
                        }
                    }
                }
            }
        }

        if let Some(object) = self.pool.get_mut(person_handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                person.building_handle = None;
                person.gather_target = None;
                person.construction_wood_reserved = false;
                person.construction_work_offset = None;
                person.construction_work_progress = 0;
                person.wood_carried = 0;
                person.prev_state = person.state;
                person.state = PersonState::Idle;
                person.state_counter = 0;
                person.state_timer = 0;
                person.movement.flags1 &= !0x1000;
                person.movement.speed = 0;
                set_animation(person, idle_animation(object.header.subtype));
            }
        }
    }

    pub fn validate_building_placement(
        &self,
        subtype: BuildingSubtype,
        cell: (i32, i32),
        rotation: u8,
    ) -> Result<(), WorldError> {
        let footprint = self.catalog.footprint(subtype, rotation).ok_or(
            WorldError::MissingBuildingFootprint {
                subtype: subtype as u8,
                rotation: rotation & 3,
            },
        )?;
        self.terrain
            .validate(cell, footprint)
            .map_err(WorldError::InvalidPlacement)
    }

    pub fn place_building(
        &mut self,
        subtype: BuildingSubtype,
        owner: u8,
        cell: (i32, i32),
        rotation: u8,
    ) -> Result<ObjectHandle, WorldError> {
        self.validate_building_placement(subtype, cell, rotation)?;
        let footprint = self.catalog.footprint(subtype, rotation).unwrap().to_vec();
        let position = cell_to_world(cell.0 as f32, cell.1 as f32, 128.0);
        let foundation_height =
            self.terrain.heights[(cell.1 & 127) as usize][(cell.0 & 127) as usize];
        let handle = self
            .pool
            .create(ModelType::Building, subtype as u8, owner, position)
            .map_err(|_| WorldError::PoolExhausted)?;
        if let Some(object) = self.pool.get_mut(handle) {
            object.header.angle = (rotation as u16 & 3) * 512;
            object.header.health = building_max_health(subtype);
            object.header.max_health = object.header.health;
            let GameObjectData::Building(building) = &mut object.data else {
                unreachable!()
            };
            building.building_subtype = subtype;
            building.state = BuildingState::Init;
            building.behavior_flags = 0;
            building.wood_stored = 0;
            building.wood_reserved = 0;
            building.wood_consumed = 0;
            building.construction_progress = 0;
            building.construction_phase = 0;
            building.visual_variant = (handle.slot() % 3) as u8;
            building.foundation_height = foundation_height;
            building.population_timer_enabled = true;
        }
        self.link(handle);
        self.terrain.occupy(handle, cell, &footprint);
        self.object_revision = self.object_revision.wrapping_add(1).max(1);
        if owner < 4 {
            self.tribes.tribes[owner as usize].active = true;
        }
        Ok(handle)
    }

    pub fn spawn_person(
        &mut self,
        subtype: u8,
        tribe: u8,
        position: WorldCoord,
    ) -> Result<ObjectHandle, WorldError> {
        let handle = self
            .pool
            .create(ModelType::Person, subtype, tribe, position)
            .map_err(|_| WorldError::PoolExhausted)?;
        self.initialize_object(handle, 0);
        self.link(handle);
        if tribe < 4 {
            self.tribes.tribes[tribe as usize].active = true;
            self.tribes.tribes[tribe as usize].population += 1;
        }
        self.object_revision = self.object_revision.wrapping_add(1).max(1);
        self.effects.push(EffectEvent::PersonSpawned(handle));
        Ok(handle)
    }

    pub fn move_object(&mut self, handle: ObjectHandle, position: WorldCoord) -> bool {
        let Some(old) = self.pool.get(handle).map(|o| o.header.position) else {
            return false;
        };
        self.cells
            .set_position(handle, &old, &position, self.pool.slots_mut());
        let Some(object) = self.pool.get_mut(handle) else {
            return false;
        };
        object.header.position = position;
        if let GameObjectData::Person(person) = &mut object.data {
            person.movement.position = position;
            (person.cell_x, person.cell_y) = world_to_render_pos(&position, 128.0);
        }
        self.object_revision = self.object_revision.wrapping_add(1).max(1);
        true
    }

    pub fn tick_persons(&mut self, rng: &mut GameRng) {
        let handles: Vec<_> = self.pool.persons().map(|(h, _, _)| h).collect();
        for handle in handles {
            let construction_job = self.pool.get(handle).is_some_and(|object| {
                matches!(&object.data, GameObjectData::Person(person) if person.building_handle.is_some())
            });
            if construction_job {
                self.tick_construction_person(handle, rng);
            } else {
                self.tick_regular_person(handle);
            }
            self.tick_person_animation(handle);
        }
        self.selected.retain(|h| self.pool.get(*h).is_some());
    }

    fn tick_regular_person(&mut self, handle: ObjectHandle) {
        let stale_target = self
            .pool
            .get(handle)
            .and_then(|object| match &object.data {
                GameObjectData::Person(person) => person.target_unit,
                _ => None,
            })
            .is_some_and(|target| self.pool.get(target).is_none());
        let Some((moving, target, subtype)) =
            self.pool.get(handle).and_then(|object| match &object.data {
                GameObjectData::Person(person) => Some((
                    person.movement.is_moving(),
                    person.movement.target_pos,
                    object.header.subtype,
                )),
                _ => None,
            })
        else {
            return;
        };
        if stale_target {
            if let Some(object) = self.pool.get_mut(handle) {
                if let GameObjectData::Person(person) = &mut object.data {
                    person.target_unit = None;
                }
            }
        }
        if moving {
            let arrived = self.move_person_toward(handle, target);
            if let Some(object) = self.pool.get_mut(handle) {
                if let GameObjectData::Person(person) = &mut object.data {
                    if arrived {
                        person.movement.flags1 &= !0x1000;
                        person.prev_state = person.state;
                        person.state = PersonState::Idle;
                        person.movement.speed = 0;
                        set_animation(person, idle_animation(subtype));
                    } else {
                        set_animation(person, walk_animation(subtype));
                    }
                }
            }
        }
    }

    fn tick_construction_person(&mut self, handle: ObjectHandle, rng: &mut GameRng) {
        let Some((building_handle, person_state, state_counter, subtype, mut work_offset)) =
            self.pool.get(handle).and_then(|object| match &object.data {
                GameObjectData::Person(person) => Some((
                    person.building_handle?,
                    person.state,
                    person.state_counter,
                    object.header.subtype,
                    person.construction_work_offset,
                )),
                _ => None,
            })
        else {
            return;
        };

        let Some((site_position, site_cell, footprint, foundation_height)) =
            self.construction_site_data(building_handle)
        else {
            self.cancel_construction_job(handle);
            return;
        };

        match person_state {
            PersonState::Building => {
                let footprint_is_flat =
                    self.terrain
                        .footprint_is_flat(site_cell, &footprint, foundation_height);
                if state_counter == BUILD_PHASE_TRAVEL_OR_FLATTEN && !footprint_is_flat {
                    let needs_new_cell = work_offset.is_none_or(|offset| {
                        self.terrain
                            .footprint_offset_is_flat(site_cell, offset, foundation_height)
                    });
                    if needs_new_cell {
                        work_offset = self.terrain.uneven_footprint_offset(
                            site_cell,
                            &footprint,
                            foundation_height,
                            rng.next().wrapping_add(handle.slot() as u32),
                        );
                        self.set_construction_work_offset(handle, work_offset);
                        self.set_person_timer(handle, 0);
                    }
                }

                let work_position = construction_work_position(
                    site_cell,
                    work_offset
                        .or_else(|| footprint.first().copied())
                        .unwrap_or((0, 0)),
                );
                if !self.move_person_toward(handle, work_position) {
                    self.set_person_animation(handle, walk_animation(subtype));
                    return;
                }
                self.face_person_toward(handle, site_position);

                match state_counter {
                    BUILD_PHASE_DELIVER => {
                        self.set_person_animation(handle, 120);
                        if self.decrement_person_timer(handle) == 0 {
                            self.finish_construction_delivery(handle, building_handle, rng);
                        }
                    }
                    BUILD_PHASE_SITE_WORK => {
                        self.set_person_animation(handle, 120);
                        if self.decrement_person_timer(handle) == 0 {
                            self.finish_construction_site_work(handle, building_handle);
                        }
                    }
                    BUILD_PHASE_WAIT_WOOD => {
                        self.set_person_animation(handle, 120);
                        self.start_wood_trip(handle, building_handle);
                    }
                    BUILD_PHASE_FINALIZE => {
                        self.set_person_animation(handle, 120);
                        if self.decrement_person_timer(handle) == 0 {
                            self.complete_construction(building_handle);
                        }
                    }
                    BUILD_PHASE_WAIT_FINALIZE => {
                        self.set_person_animation(handle, 120);
                    }
                    _ if !footprint_is_flat => {
                        // Native animation 120 is the five-frame rising/jumping
                        // construction stroke. Animation 115 is only its
                        // downward digging counterpart; looping 115 made a
                        // builder repeatedly collapse without ever rising.
                        self.set_person_animation(handle, 120);
                        if self.person_timer(handle) == 0 {
                            self.set_person_timer(handle, FOUNDATION_STROKE_TICKS);
                        }
                        if self.decrement_person_timer(handle) == 0 {
                            if let Some(offset) = work_offset {
                                self.terrain.flatten_offset_one_step(
                                    site_cell,
                                    offset,
                                    foundation_height,
                                );
                            }
                        }
                    }
                    _ => self.start_wood_trip(handle, building_handle),
                }
            }
            PersonState::Gathering => {
                let tree = self.pool.get(handle).and_then(|object| match &object.data {
                    GameObjectData::Person(person) => person.gather_target,
                    _ => None,
                });
                let Some(tree) = tree else {
                    self.start_wood_trip(handle, building_handle);
                    return;
                };
                let tree_position = self.pool.get(tree).and_then(|object| match &object.data {
                    GameObjectData::Scenery(data)
                        if data.wood_remaining > 0 && data.wood_reserved > 0 =>
                    {
                        Some(object.header.position)
                    }
                    _ => None,
                });
                let Some(tree_position) = tree_position else {
                    self.release_wood_trip(handle, building_handle, true);
                    self.start_wood_trip(handle, building_handle);
                    return;
                };
                if self.move_person_toward(handle, tree_position) {
                    if let Some(object) = self.pool.get_mut(handle) {
                        if let GameObjectData::Person(person) = &mut object.data {
                            person.prev_state = person.state;
                            person.state = PersonState::GatheringWood;
                            person.state_timer = CHOP_TICKS;
                            person.movement.flags1 &= !0x1000;
                            person.movement.speed = 0;
                            set_animation(person, 73);
                        }
                    }
                } else {
                    self.set_person_animation(handle, walk_animation(subtype));
                }
            }
            PersonState::GatheringWood => {
                self.set_person_animation(handle, 73);
                if self.decrement_person_timer(handle) == 0 {
                    self.finish_chopping(handle, building_handle);
                }
            }
            PersonState::CarryingWood => {
                self.set_person_animation(handle, 88);
                let work_position = construction_work_position(
                    site_cell,
                    work_offset
                        .or_else(|| footprint.first().copied())
                        .unwrap_or((0, 0)),
                );
                if self.move_person_toward(handle, work_position) {
                    self.face_person_toward(handle, site_position);
                    self.deposit_construction_wood(handle, building_handle);
                }
            }
            _ => self.cancel_construction_job(handle),
        }
    }

    fn construction_site_data(
        &self,
        handle: ObjectHandle,
    ) -> Option<(WorldCoord, (i32, i32), Vec<(i16, i16)>, u16)> {
        let object = self.pool.get(handle)?;
        let GameObjectData::Building(building) = &object.data else {
            return None;
        };
        if building.state != BuildingState::Init {
            return None;
        }
        let rotation = ((object.header.angle / 512) & 3) as u8;
        let footprint = self
            .catalog
            .footprint(building.building_subtype, rotation)?
            .to_vec();
        Some((
            object.header.position,
            render_cell(object.header.position),
            footprint,
            building.foundation_height,
        ))
    }

    fn start_wood_trip(&mut self, person_handle: ObjectHandle, building_handle: ObjectHandle) {
        let needs_wood = self.pool.get(building_handle).is_some_and(|object| {
            matches!(&object.data, GameObjectData::Building(building)
                if building.state == BuildingState::Init
                    && building.wood_consumed
                        + building.wood_stored
                        + building.wood_reserved
                        < construction_target(building.building_subtype))
        });
        if !needs_wood {
            if let Some(object) = self.pool.get_mut(person_handle) {
                if let GameObjectData::Person(person) = &mut object.data {
                    person.prev_state = person.state;
                    person.state = PersonState::Building;
                    person.state_counter = BUILD_PHASE_WAIT_WOOD;
                    person.movement.flags1 &= !0x1000;
                    person.movement.speed = 0;
                    set_animation(person, 120);
                }
            }
            return;
        }

        let Some(person_position) = self.pool.get(person_handle).map(|o| o.header.position) else {
            return;
        };
        let Some(tree_handle) = self.find_nearest_available_tree(person_position) else {
            if let Some(object) = self.pool.get_mut(person_handle) {
                if let GameObjectData::Person(person) = &mut object.data {
                    person.prev_state = person.state;
                    person.state = PersonState::Building;
                    person.state_counter = BUILD_PHASE_WAIT_WOOD;
                    person.movement.flags1 &= !0x1000;
                    person.movement.speed = 0;
                    set_animation(person, 120);
                }
            }
            return;
        };
        let tree_position = self.pool.get(tree_handle).unwrap().header.position;
        if let Some(object) = self.pool.get_mut(tree_handle) {
            if let GameObjectData::Scenery(tree) = &mut object.data {
                tree.wood_reserved += 1;
            }
        }
        if let Some(object) = self.pool.get_mut(building_handle) {
            if let GameObjectData::Building(building) = &mut object.data {
                building.wood_reserved += 1;
            }
        }
        if let Some(object) = self.pool.get_mut(person_handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                person.prev_state = person.state;
                person.state = PersonState::Gathering;
                person.gather_target = Some(tree_handle);
                person.construction_wood_reserved = true;
                person.movement.target_pos = tree_position;
                person.movement.speed = person_type_defaults(object.header.subtype).speed;
                person.movement.flags1 |= 0x1000;
                set_animation(person, walk_animation(object.header.subtype));
            }
        }
    }

    fn find_nearest_available_tree(&self, position: WorldCoord) -> Option<ObjectHandle> {
        self.pool
            .slots()
            .iter()
            .filter_map(|slot| match slot {
                crate::engine::objects::PoolSlot::Occupied(object) => match &object.data {
                    GameObjectData::Scenery(tree)
                        if object.header.subtype <= TREE_MAX_SUBTYPE
                            && tree.wood_remaining > tree.wood_reserved =>
                    {
                        let dx = toroidal_world_distance(position.x, object.header.position.x);
                        let dz = toroidal_world_distance(position.z, object.header.position.z);
                        Some((object.header.object_index, dx + dz))
                    }
                    _ => None,
                },
                _ => None,
            })
            .min_by_key(|(_, distance)| *distance)
            .map(|(handle, _)| handle)
    }

    fn finish_chopping(&mut self, person_handle: ObjectHandle, building_handle: ObjectHandle) {
        let tree_handle = self
            .pool
            .get(person_handle)
            .and_then(|object| match &object.data {
                GameObjectData::Person(person) => person.gather_target,
                _ => None,
            });
        let Some(tree_handle) = tree_handle else {
            self.release_wood_trip(person_handle, building_handle, true);
            return;
        };
        let chopped = self.pool.get_mut(tree_handle).is_some_and(|object| {
            let GameObjectData::Scenery(tree) = &mut object.data else {
                return false;
            };
            tree.wood_reserved = tree.wood_reserved.saturating_sub(1);
            if tree.wood_remaining == 0 {
                return false;
            }
            tree.wood_remaining -= 1;
            true
        });
        if !chopped {
            self.release_wood_trip(person_handle, building_handle, false);
            self.start_wood_trip(person_handle, building_handle);
            return;
        }
        let Some(site_position) = self.construction_person_work_position(person_handle) else {
            self.release_wood_trip(person_handle, building_handle, false);
            return;
        };
        if let Some(object) = self.pool.get_mut(person_handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                person.gather_target = None;
                person.wood_carried = 1;
                person.prev_state = person.state;
                person.state = PersonState::CarryingWood;
                person.movement.target_pos = site_position;
                person.movement.speed = person_type_defaults(object.header.subtype).speed;
                person.movement.flags1 |= 0x1000;
                set_animation(person, 88);
            }
        }
    }

    fn deposit_construction_wood(
        &mut self,
        person_handle: ObjectHandle,
        building_handle: ObjectHandle,
    ) {
        let carrying_reserved_wood = self.pool.get(person_handle).is_some_and(|object| {
            matches!(&object.data, GameObjectData::Person(person)
                if person.wood_carried > 0 && person.construction_wood_reserved)
        });
        if !carrying_reserved_wood {
            self.cancel_construction_job(person_handle);
            return;
        }
        let valid_site = self.pool.get(building_handle).is_some_and(|object| {
            matches!(&object.data, GameObjectData::Building(building)
                if building.state == BuildingState::Init && building.wood_reserved > 0)
        });
        if !valid_site {
            self.cancel_construction_job(person_handle);
            return;
        }

        if let Some(object) = self.pool.get_mut(person_handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                person.prev_state = person.state;
                person.state = PersonState::Building;
                person.state_counter = BUILD_PHASE_DELIVER;
                person.state_timer = DELIVERY_TICKS;
                person.construction_work_progress = 0;
                person.movement.flags1 &= !0x1000;
                person.movement.speed = 0;
                set_animation(person, 120);
            }
        }
    }

    fn finish_construction_delivery(
        &mut self,
        person_handle: ObjectHandle,
        building_handle: ObjectHandle,
        rng: &mut GameRng,
    ) {
        let amount = self
            .pool
            .get(person_handle)
            .and_then(|object| match &object.data {
                GameObjectData::Person(person)
                    if person.construction_wood_reserved && person.wood_carried > 0 =>
                {
                    Some(person.wood_carried)
                }
                _ => None,
            });
        let Some(amount) = amount else {
            self.cancel_construction_job(person_handle);
            return;
        };

        let mut accepted = false;
        if let Some(object) = self.pool.get_mut(building_handle) {
            if let GameObjectData::Building(building) = &mut object.data {
                if building.state != BuildingState::Init || building.wood_reserved == 0 {
                    self.cancel_construction_job(person_handle);
                    return;
                }
                building.wood_reserved -= 1;
                building.wood_consumed = building.wood_consumed.saturating_add(amount);
                let target = construction_progress_target(building.building_subtype);
                let contribution = amount.saturating_mul(CONSTRUCTION_UNITS_PER_WOOD);
                building.construction_progress = building
                    .construction_progress
                    .saturating_add(contribution)
                    .min(target);
                building.construction_phase = construction_delivery_phase(
                    building.wood_consumed,
                    construction_target(building.building_subtype),
                );
                accepted = true;
            }
        }

        if !accepted {
            self.cancel_construction_job(person_handle);
            return;
        }
        if let Some(object) = self.pool.get_mut(person_handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                person.wood_carried = 0;
                person.construction_wood_reserved = false;
                person.state_counter = BUILD_PHASE_SITE_WORK;
                person.state_timer = next_site_work_ticks(rng);
                person.construction_work_progress = 0;
                set_animation(person, 120);
            }
        }
        self.object_revision = self.object_revision.wrapping_add(1).max(1);
    }

    fn finish_construction_site_work(
        &mut self,
        person_handle: ObjectHandle,
        building_handle: ObjectHandle,
    ) {
        let mut begin_final_interval = false;
        let mut wait_for_final_interval = false;
        let valid_site = if let Some(object) = self.pool.get_mut(building_handle) {
            if let GameObjectData::Building(building) = &mut object.data {
                if building.state != BuildingState::Init {
                    false
                } else {
                    let target = construction_progress_target(building.building_subtype);
                    if building.construction_progress >= target {
                        if building.construction_phase < 3 {
                            building.construction_phase = 3;
                            begin_final_interval = true;
                        } else {
                            wait_for_final_interval = true;
                        }
                    }
                    true
                }
            } else {
                false
            }
        } else {
            false
        };
        if !valid_site {
            self.cancel_construction_job(person_handle);
            return;
        }

        if begin_final_interval || wait_for_final_interval {
            if let Some(object) = self.pool.get_mut(person_handle) {
                if let GameObjectData::Person(person) = &mut object.data {
                    person.state_counter = if begin_final_interval {
                        BUILD_PHASE_FINALIZE
                    } else {
                        BUILD_PHASE_WAIT_FINALIZE
                    };
                    person.state_timer = if begin_final_interval {
                        FINAL_BUILD_TICKS
                    } else {
                        0
                    };
                    person.construction_work_progress = 0;
                    person.movement.flags1 &= !0x1000;
                    person.movement.speed = 0;
                    set_animation(person, 120);
                }
            }
            self.object_revision = self.object_revision.wrapping_add(1).max(1);
            return;
        }

        if let Some(object) = self.pool.get_mut(person_handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                person.state_counter = BUILD_PHASE_TRAVEL_OR_FLATTEN;
                person.construction_work_progress = 0;
            }
        }
        self.start_wood_trip(person_handle, building_handle);
    }

    fn complete_construction(&mut self, building_handle: ObjectHandle) {
        let Some((site_cell, footprint, rotation)) =
            self.pool.get(building_handle).and_then(|object| {
                let GameObjectData::Building(building) = &object.data else {
                    return None;
                };
                (building.state == BuildingState::Init).then(|| {
                    (
                        render_cell(object.header.position),
                        self.catalog
                            .footprint(
                                building.building_subtype,
                                ((object.header.angle / 512) & 3) as u8,
                            )
                            .unwrap_or_default()
                            .to_vec(),
                        ((object.header.angle / 512) & 3) as u8,
                    )
                })
            })
        else {
            return;
        };

        if let Some(object) = self.pool.get_mut(building_handle) {
            let GameObjectData::Building(building) = &mut object.data else {
                return;
            };
            building.construction_phase = 4;
            building.state = BuildingState::ConstructionDone;
        }

        let mut builders: Vec<_> = self
            .pool
            .persons()
            .filter_map(|(handle, _, person)| {
                (person.building_handle == Some(building_handle)).then_some(handle)
            })
            .collect();
        builders.sort_by_key(|handle| handle.slot());
        let exit_positions =
            construction_exit_positions(site_cell, &footprint, rotation, builders.len());
        for (handle, target) in builders.into_iter().zip(exit_positions) {
            if let Some(object) = self.pool.get_mut(handle) {
                if let GameObjectData::Person(person) = &mut object.data {
                    person.building_handle = None;
                    person.gather_target = None;
                    person.construction_wood_reserved = false;
                    person.construction_work_offset = None;
                    person.construction_work_progress = 0;
                    person.wood_carried = 0;
                    person.prev_state = person.state;
                    person.state = PersonState::GoToPoint;
                    person.state_counter = 0;
                    person.state_timer = 0;
                    person.movement.target_pos = target;
                    person.movement.speed = person_type_defaults(object.header.subtype).speed;
                    person.movement.flags1 |= 0x1000;
                    set_animation(person, walk_animation(object.header.subtype));
                }
            }
        }
        self.object_revision = self.object_revision.wrapping_add(1).max(1);
    }

    fn release_wood_trip(
        &mut self,
        person_handle: ObjectHandle,
        building_handle: ObjectHandle,
        release_tree: bool,
    ) {
        let tree_handle = self
            .pool
            .get(person_handle)
            .and_then(|object| match &object.data {
                GameObjectData::Person(person) => person.gather_target,
                _ => None,
            });
        if release_tree {
            if let Some(tree_handle) = tree_handle {
                if let Some(object) = self.pool.get_mut(tree_handle) {
                    if let GameObjectData::Scenery(tree) = &mut object.data {
                        tree.wood_reserved = tree.wood_reserved.saturating_sub(1);
                    }
                }
            }
        }
        if let Some(object) = self.pool.get_mut(building_handle) {
            if let GameObjectData::Building(building) = &mut object.data {
                building.wood_reserved = building.wood_reserved.saturating_sub(1);
            }
        }
        if let Some(object) = self.pool.get_mut(person_handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                person.gather_target = None;
                person.construction_wood_reserved = false;
                person.wood_carried = 0;
            }
        }
    }

    fn construction_person_work_position(&self, person_handle: ObjectHandle) -> Option<WorldCoord> {
        let (building_handle, work_offset) =
            self.pool
                .get(person_handle)
                .and_then(|object| match &object.data {
                    GameObjectData::Person(person) => {
                        Some((person.building_handle?, person.construction_work_offset))
                    }
                    _ => None,
                })?;
        let (_, site_cell, footprint, _) = self.construction_site_data(building_handle)?;
        Some(construction_work_position(
            site_cell,
            work_offset
                .or_else(|| footprint.first().copied())
                .unwrap_or((0, 0)),
        ))
    }

    fn set_construction_work_offset(&mut self, handle: ObjectHandle, offset: Option<(i16, i16)>) {
        if let Some(object) = self.pool.get_mut(handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                person.construction_work_offset = offset;
            }
        }
    }

    fn face_person_toward(&mut self, handle: ObjectHandle, target: WorldCoord) {
        let Some(position) = self.pool.get(handle).map(|object| object.header.position) else {
            return;
        };
        let dx = toroidal_delta(position.x, target.x);
        let dz = toroidal_delta(position.z, target.z);
        if dx == 0 && dz == 0 {
            return;
        }
        let angle = atan2(dx, -dz);
        if let Some(object) = self.pool.get_mut(handle) {
            object.header.angle = angle;
            if let GameObjectData::Person(person) = &mut object.data {
                person.movement.facing_angle = angle;
            }
        }
    }

    fn move_person_toward(&mut self, handle: ObjectHandle, target: WorldCoord) -> bool {
        let Some(position) = self.pool.get(handle).map(|object| object.header.position) else {
            return false;
        };
        self.face_person_toward(handle, target);
        let next = WorldCoord::new(
            approach(position.x, target.x, PERSON_MOVE_STEP),
            approach(position.z, target.z, PERSON_MOVE_STEP),
        );
        self.move_object(handle, next);
        next == target
    }

    fn decrement_person_timer(&mut self, handle: ObjectHandle) -> u16 {
        self.pool
            .get_mut(handle)
            .and_then(|object| match &mut object.data {
                GameObjectData::Person(person) => {
                    person.state_timer = person.state_timer.saturating_sub(1);
                    Some(person.state_timer)
                }
                _ => None,
            })
            .unwrap_or(0)
    }

    fn person_timer(&self, handle: ObjectHandle) -> u16 {
        self.pool
            .get(handle)
            .and_then(|object| match &object.data {
                GameObjectData::Person(person) => Some(person.state_timer),
                _ => None,
            })
            .unwrap_or(0)
    }

    fn set_person_timer(&mut self, handle: ObjectHandle, timer: u16) {
        if let Some(object) = self.pool.get_mut(handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                person.state_timer = timer;
            }
        }
    }

    fn set_person_animation(&mut self, handle: ObjectHandle, animation: u16) {
        if let Some(object) = self.pool.get_mut(handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                set_animation(person, animation);
            }
        }
    }

    fn tick_person_animation(&mut self, handle: ObjectHandle) {
        if let Some(object) = self.pool.get_mut(handle) {
            if let GameObjectData::Person(person) = &mut object.data {
                if person.anim.flags & 0x02 == 0 {
                    return;
                }
                person.anim.tick_counter = person.anim.tick_counter.saturating_add(1);
                if person.anim.tick_counter >= person.anim.ticks_per_frame as u16 {
                    person.anim.tick_counter = 0;
                    person.anim.frame_index = person.anim.frame_index.wrapping_add(1);
                    // Runtime-loaded idle/walk animations historically leave
                    // frame_count at one and are wrapped by the renderer's
                    // atlas metadata. Action fixtures and construction states
                    // do carry an exact count, which lets one-shot death
                    // animations hold their final frame here.
                    if person.anim.frame_count > 1
                        && person.anim.frame_index >= person.anim.frame_count
                    {
                        if person.anim.flags & 0x01 != 0 {
                            person.anim.frame_index = 0;
                        } else {
                            person.anim.frame_index = person.anim.frame_count - 1;
                        }
                    }
                }
            }
        }
    }

    pub fn tick_buildings(&mut self) {
        self.synchronize_tribes();
        let mut weighted_population = [0u32; 4];
        for (_, header, person) in self.pool.persons() {
            if person.alive && header.tribe < 4 {
                weighted_population[header.tribe as usize] +=
                    population_growth_weight(header.subtype);
            }
        }
        let mut remaining_population_capacity: [u16; 4] = std::array::from_fn(|index| {
            let tribe = &self.tribes.tribes[index];
            tribe.max_population.saturating_sub(tribe.population as u16)
        });
        let handles: Vec<_> = self.pool.buildings().map(|(h, _, _)| h).collect();
        let mut spawns = Vec::new();
        let mut attacks = Vec::new();
        for handle in handles {
            if let Some(object) = self.pool.get_mut(handle) {
                let position = object.header.position;
                let tribe = object.header.tribe;
                let GameObjectData::Building(building) = &mut object.data else {
                    continue;
                };
                let population = weighted_population
                    .get(tribe as usize)
                    .copied()
                    .unwrap_or(0);
                let remaining_capacity = remaining_population_capacity.get_mut(tribe as usize);
                let has_capacity = remaining_capacity.as_deref().copied().unwrap_or(0) > 0;
                let actions = tick_building_with_population(
                    building,
                    &mut object.header,
                    handle,
                    population,
                    has_capacity,
                );
                if actions.spawn == SpawnAction::SpawnBrave {
                    if let Some(remaining) = remaining_capacity {
                        *remaining = remaining.saturating_sub(1);
                    }
                    spawns.push((position, tribe));
                }
                attacks.extend(actions.combat);
            }
        }
        for action in attacks {
            if let crate::engine::buildings::BuildingCombatAction::AttackTarget {
                target,
                damage,
                ..
            } = action
            {
                if let Some(target) = self.pool.get_mut(target) {
                    target.header.health = target.header.health.saturating_sub(damage);
                }
            }
        }
        self.synchronize_tribes();
        for (position, tribe) in spawns {
            if tribe < 4
                && can_spawn(
                    self.tribes.tribes[tribe as usize].population as u16,
                    self.tribes.tribes[tribe as usize].max_population,
                )
            {
                let spawn = WorldCoord::new(position.x.wrapping_add(512), position.z);
                let _ = self.spawn_person(2, tribe, spawn);
            }
        }
        self.synchronize_tribes();
    }

    pub fn synchronize_tribes(&mut self) {
        for tribe in &mut self.tribes.tribes {
            tribe.population = 0;
            tribe.max_population = 0;
        }
        for (_, header, person) in self.pool.persons() {
            if person.alive && header.tribe < 4 {
                let tribe = &mut self.tribes.tribes[header.tribe as usize];
                tribe.active = true;
                tribe.population += 1;
            }
        }
        let mut huts = [[0u16; 3]; 4];
        for (_, header, building) in self.pool.buildings() {
            if header.tribe < 4 && building.state == BuildingState::Active {
                let tribe = &mut self.tribes.tribes[header.tribe as usize];
                tribe.active = true;
                match building.building_subtype {
                    BuildingSubtype::SmallHut => huts[header.tribe as usize][0] += 1,
                    BuildingSubtype::MediumHut => huts[header.tribe as usize][1] += 1,
                    BuildingSubtype::LargeHut => huts[header.tribe as usize][2] += 1,
                    _ => {}
                }
            }
        }
        for (i, counts) in huts.into_iter().enumerate() {
            self.tribes.tribes[i].max_population = calculate_housing_capacity(counts);
        }
    }

    pub fn add_mana(&mut self) {
        use crate::engine::economy::mana::{add_mana, mana_rate_for_person};
        let mut rates = [0u32; 4];
        for (_, header, person) in self.pool.persons() {
            if person.alive && header.tribe < 4 {
                rates[header.tribe as usize] += mana_rate_for_person(header.subtype);
            }
        }
        for (tribe, rate) in self.tribes.tribes.iter_mut().zip(rates) {
            add_mana(&mut tribe.mana, rate);
        }
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        let mut persons = Vec::new();
        let mut objects = Vec::new();
        for (handle, header, person) in self.pool.persons() {
            let (cell_x, cell_y) = world_to_render_pos(&header.position, 128.0);
            persons.push(PersonRenderRecord {
                handle,
                subtype: header.subtype,
                tribe: header.tribe,
                alive: person.alive,
                cell_x,
                cell_y,
                angle: header.angle,
                health: header.health,
                max_health: header.max_health,
                animation_id: person.anim.animation_id,
                animation_frame: person.anim.frame_index,
            });
        }
        for (i, slot) in self.pool.slots().iter().enumerate() {
            let crate::engine::objects::PoolSlot::Occupied(object) = slot else {
                continue;
            };
            if object.header.model_type == ModelType::Person {
                continue;
            }
            let handle = object.header.object_index;
            debug_assert_eq!(handle.slot() as usize, i);
            let (cell_x, cell_y) = world_to_render_pos(&object.header.position, 128.0);
            let (
                building_state,
                construction_progress,
                construction_phase,
                visual_variant,
                footprint,
            ) = match &object.data {
                GameObjectData::Building(building) => {
                    let rotation = ((object.header.angle / 512) & 3) as u8;
                    (
                        Some(building.state),
                        building.construction_progress,
                        building.construction_phase,
                        building.visual_variant,
                        self.catalog
                            .footprint(building.building_subtype, rotation)
                            .map(ToOwned::to_owned)
                            .unwrap_or_default(),
                    )
                }
                _ => (None, 0, 0, 0, Vec::new()),
            };
            objects.push(WorldObjectRenderRecord {
                handle,
                model_type: object.header.model_type,
                subtype: object.header.subtype,
                tribe: object.header.tribe,
                cell_x,
                cell_y,
                angle: object.header.angle,
                health: object.header.health,
                max_health: object.header.max_health,
                building_state,
                construction_progress,
                construction_phase,
                visual_variant,
                footprint,
            });
        }
        let tribes = self
            .tribes
            .tribes
            .iter()
            .map(|t| TribeSummary {
                tribe: t.index,
                active: t.active,
                population: t.population,
                max_population: t.max_population,
                mana: t.mana,
            })
            .collect();
        WorldSnapshot {
            persons,
            objects,
            tribes,
            selected: self.selected.clone(),
            terrain_revision: self.terrain.revision(),
            object_revision: self.object_revision,
            effect_events: self.effects.clone(),
        }
    }
}

fn render_cell(position: WorldCoord) -> (i32, i32) {
    let (x, y) = world_to_cell(&position, 128.0);
    (x.floor() as i32, y.floor() as i32)
}

fn construction_work_position(site_cell: (i32, i32), offset: (i16, i16)) -> WorldCoord {
    cell_to_world(
        (site_cell.0 as f32 + offset.0 as f32).rem_euclid(WORLD_SIZE as f32),
        (site_cell.1 as f32 + offset.1 as f32).rem_euclid(WORLD_SIZE as f32),
        WORLD_SIZE as f32,
    )
}

fn construction_exit_positions(
    site_cell: (i32, i32),
    footprint: &[(i16, i16)],
    rotation: u8,
    count: usize,
) -> Vec<WorldCoord> {
    let min_x = footprint.iter().map(|offset| offset.0).min().unwrap_or(0) as f32;
    let max_x = footprint.iter().map(|offset| offset.0).max().unwrap_or(0) as f32;
    let min_y = footprint.iter().map(|offset| offset.1).min().unwrap_or(0) as f32;
    let max_y = footprint.iter().map(|offset| offset.1).max().unwrap_or(0) as f32;
    let center_x = site_cell.0 as f32 + (min_x + max_x) * 0.5;
    let center_y = site_cell.1 as f32 + (min_y + max_y) * 0.5;

    (0..count)
        .map(|index| {
            let group = index / 6;
            let lane = index % 6;
            let group_len = (count - group * 6).min(6);
            let lateral = (lane as f32 - (group_len.saturating_sub(1) as f32 * 0.5)) * 0.5;
            // cell_to_world quantizes at half-cell resolution; 1.25 keeps the
            // regroup target visibly beyond the occupied edge after that
            // conversion rather than leaving the brave on the footprint.
            let outward = 1.25 + group as f32 * 0.75;
            let (cell_x, cell_y) = match rotation & 3 {
                0 => (center_x + lateral, site_cell.1 as f32 + min_y - outward),
                1 => (site_cell.0 as f32 + min_x - outward, center_y + lateral),
                2 => (center_x + lateral, site_cell.1 as f32 + max_y + outward),
                _ => (site_cell.0 as f32 + max_x + outward, center_y + lateral),
            };
            cell_to_world(
                cell_x.rem_euclid(WORLD_SIZE as f32),
                cell_y.rem_euclid(WORLD_SIZE as f32),
                WORLD_SIZE as f32,
            )
        })
        .collect()
}

fn approach(value: i16, target: i16, step: i16) -> i16 {
    let delta = target.wrapping_sub(value);
    if delta.unsigned_abs() <= step as u16 {
        target
    } else if delta > 0 {
        value.wrapping_add(step)
    } else {
        value.wrapping_sub(step)
    }
}

fn toroidal_world_distance(a: i16, b: i16) -> u32 {
    let direct = a.wrapping_sub(b).unsigned_abs() as u32;
    direct.min(65536 - direct)
}

fn next_site_work_ticks(rng: &mut GameRng) -> u16 {
    let original_ticks = SITE_WORK_MIN_ORIGINAL_TICKS + (rng.next() & SITE_WORK_RANDOM_MASK) as u16;
    original_ticks_to_world_ticks(original_ticks)
}

fn idle_animation(subtype: u8) -> u16 {
    match subtype {
        2 => 15,
        3 => 16,
        4 => 17,
        5 => 18,
        6 => 19,
        7 => 20,
        _ => 0,
    }
}

fn walk_animation(subtype: u8) -> u16 {
    match subtype {
        2 => 21,
        3 => 22,
        4 => 23,
        5 => 24,
        6 => 25,
        7 => 26,
        _ => 1,
    }
}

fn set_animation(person: &mut crate::engine::objects::PersonData, animation: u16) {
    if person.anim.animation_id != animation {
        person.anim.animation_id = animation;
        person.anim.frame_index = 0;
        person.anim.tick_counter = 0;
        person.anim.flags = 0x03;
        person.anim.frame_count = match animation {
            73 | 88 => 6,
            115 => 8,
            120 => 5,
            _ => 1,
        };
        person.anim.ticks_per_frame = match animation {
            73 | 88 | 115 | 120 => WORK_ANIMATION_TICKS_PER_FRAME,
            _ => 3,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::level::{LevelObjectDefinition, Sunlight};
    use crate::data::units::TribeConfigRaw;

    fn catalog() -> BuildingCatalog {
        let mut catalog = BuildingCatalog::new();
        for rotation in 0..4 {
            catalog.insert(
                BuildingSubtype::SmallHut,
                rotation,
                vec![(0, 0), (1, 0), (0, 1), (1, 1)],
            );
        }
        catalog
    }

    fn definition(objects: Vec<LevelObjectDefinition>) -> LevelDefinition {
        LevelDefinition {
            level_number: 1,
            heights: Box::new([[100; 128]; 128]),
            sunlight: Sunlight::new(1, 2, 3),
            tribes: vec![TribeConfigRaw { data: [7; 16] }; 4],
            objects,
        }
    }

    fn object(
        source: u16,
        model_type: ModelType,
        subtype: u8,
        tribe: u8,
        cell: (i32, i32),
    ) -> LevelObjectDefinition {
        let position = cell_to_world(cell.0 as f32, cell.1 as f32, 128.0);
        LevelObjectDefinition {
            source: LevelObjectIndex(source),
            model_type,
            subtype,
            tribe,
            position: [position.x, position.z],
            angle: 0,
        }
    }

    #[test]
    fn level_construction_instantiates_people_buildings_and_scenery() {
        let world = World::from_level(
            definition(vec![
                object(0, ModelType::Person, 2, 0, (10, 10)),
                object(1, ModelType::Building, 1, 0, (20, 20)),
                object(2, ModelType::Scenery, 1, 255, (30, 30)),
            ]),
            catalog(),
        )
        .unwrap();
        assert_eq!(world.pool.active_count(), 3);
        assert_eq!(world.pool.persons().count(), 1);
        assert_eq!(world.pool.buildings().count(), 1);
        assert_eq!(world.tribes.tribes[0].population, 1);
        assert_eq!(world.tribes.tribes[0].max_population, 3);
        let building = world.source_handle(LevelObjectIndex(1)).unwrap();
        assert_eq!(world.terrain.occupant(20, 20), Some(building));
        assert_eq!(world.opaque_tribes[0], [7; 16]);
    }

    #[test]
    fn placement_is_atomic_and_snapshot_uses_canonical_object() {
        let mut world = World::from_level(definition(Vec::new()), catalog()).unwrap();
        world.terrain.heights[12][13] = 132;
        let before = world.terrain.revision();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        assert_eq!(
            render_cell(world.get(hut).unwrap().header.position),
            (12, 12)
        );
        assert!(world.terrain.revision() > before);
        let count = world.pool.active_count();
        assert_eq!(
            world.place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0),
            Err(WorldError::InvalidPlacement(PlacementError::Occupied))
        );
        assert_eq!(world.pool.active_count(), count);
        assert_eq!(world.terrain.heights[12][13], 132);
        let record = world
            .snapshot()
            .objects
            .iter()
            .find(|record| record.handle == hut)
            .cloned()
            .unwrap();
        assert_eq!(record.building_state, Some(BuildingState::Init));
        assert_eq!(record.construction_phase, 0);
        assert_eq!(record.footprint.len(), 4);
        let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
            panic!("placed object must be a building");
        };
        assert_eq!(building.wood_stored, 0);
        assert_eq!(building.construction_progress, 0);
    }

    #[test]
    fn placed_building_preserves_clockwise_rotation() {
        let mut world = World::from_level(definition(Vec::new()), catalog()).unwrap();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 1)
            .unwrap();
        assert_eq!(world.get(hut).unwrap().header.angle, 512);
        assert_eq!(
            world
                .snapshot()
                .objects
                .iter()
                .find(|object| object.handle == hut)
                .unwrap()
                .angle,
            512
        );
    }

    #[test]
    fn movement_updates_both_render_and_simulation_facing() {
        let mut world = World::from_level(
            definition(vec![object(
                0,
                ModelType::Person,
                BRAVE_SUBTYPE,
                0,
                (10, 10),
            )]),
            catalog(),
        )
        .unwrap();
        let brave = world.source_handle(LevelObjectIndex(0)).unwrap();
        let start = world.get(brave).unwrap().header.position;
        let target = WorldCoord::new(start.x.wrapping_add(512), start.z.wrapping_add(256));
        let expected = atan2(
            toroidal_delta(start.x, target.x),
            -toroidal_delta(start.z, target.z),
        );

        assert!(!world.move_person_toward(brave, target));
        let object = world.get(brave).unwrap();
        let GameObjectData::Person(person) = &object.data else {
            unreachable!()
        };
        assert_eq!(object.header.angle, expected);
        assert_eq!(person.movement.facing_angle, expected);
    }

    #[test]
    fn builders_receive_distinct_footprint_work_cells() {
        let mut world = World::from_level(
            definition(vec![
                object(0, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 10)),
                object(1, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 11)),
                object(2, ModelType::Person, BRAVE_SUBTYPE, 0, (11, 10)),
            ]),
            catalog(),
        )
        .unwrap();
        let braves = [
            world.source_handle(LevelObjectIndex(0)).unwrap(),
            world.source_handle(LevelObjectIndex(1)).unwrap(),
            world.source_handle(LevelObjectIndex(2)).unwrap(),
        ];
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        world.assign_construction(&braves, hut).unwrap();

        let offsets: std::collections::HashSet<_> = braves
            .iter()
            .map(|handle| {
                let GameObjectData::Person(person) = &world.get(*handle).unwrap().data else {
                    unreachable!()
                };
                person.construction_work_offset.unwrap()
            })
            .collect();
        assert_eq!(offsets.len(), braves.len());
    }

    #[test]
    fn hut_constructs_then_spawns_one_canonical_brave() {
        let mut rng = GameRng::new(0);
        let mut world = World::from_level(
            definition(vec![
                object(0, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 10)),
                object(1, ModelType::Scenery, 1, 255, (11, 11)),
            ]),
            catalog(),
        )
        .unwrap();
        world.terrain.heights[12][13] = 140;
        let brave = world.source_handle(LevelObjectIndex(0)).unwrap();
        let tree = world.source_handle(LevelObjectIndex(1)).unwrap();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        world.assign_construction(&[brave], hut).unwrap();

        let mut observed_animations = std::collections::HashSet::new();
        let mut observed_animation_frames = std::collections::HashSet::new();
        let mut observed_phases = std::collections::HashSet::new();
        for _ in 0..5_000 {
            world.tick_persons(&mut rng);
            world.tick_buildings();
            let object = world.get(brave).unwrap();
            let GameObjectData::Person(person) = &object.data else {
                unreachable!()
            };
            observed_animations.insert(person.anim.animation_id);
            observed_animation_frames.insert((person.anim.animation_id, person.anim.frame_index));
            if let Some(object) = world.get(hut) {
                if let GameObjectData::Building(building) = &object.data {
                    observed_phases.insert(building.construction_phase);
                }
            }
            let active = world.get(hut).is_some_and(|object| {
                matches!(&object.data, GameObjectData::Building(building) if building.state == BuildingState::Active)
            });
            if active {
                break;
            }
        }
        let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
            unreachable!()
        };
        assert_eq!(building.state, BuildingState::Active);
        assert_eq!(building.construction_phase, 4);
        assert_eq!(building.construction_progress, 0);
        assert!(observed_animations.contains(&73));
        assert!(!observed_animations.contains(&115));
        assert!(observed_animations.contains(&88));
        assert!(observed_animations.contains(&120));
        for animation in [73, 120] {
            assert!(observed_animation_frames
                .iter()
                .any(|&(id, frame)| id == animation && frame > 0));
        }
        for phase in 0..=3 {
            assert!(observed_phases.contains(&phase));
        }
        let GameObjectData::Scenery(tree_data) = &world.get(tree).unwrap().data else {
            unreachable!()
        };
        assert_eq!(tree_data.wood_remaining, 1);
        assert_eq!(tree_data.wood_reserved, 0);
        assert_eq!(world.tribes.tribes[0].max_population, 3);
        // Empty small hut at the first population band: ceil(1187 / 2).
        for _ in 0..1_000 {
            world.tick_buildings();
            if world.pool.persons().count() == 2 {
                break;
            }
        }
        assert_eq!(world.pool.persons().count(), 2);
        assert_eq!(world.tribes.tribes[0].population, 2);
        let snapshot = world.snapshot();
        assert_eq!(snapshot.persons.len(), 2);
        let spawned = snapshot
            .persons
            .iter()
            .find(|person| person.handle != brave)
            .unwrap()
            .handle;
        let position = world.get(spawned).unwrap().header.position;
        assert_eq!(
            world.cell_head(CellGrid::cell_index_from_world(&position)),
            Some(spawned)
        );
    }

    #[test]
    fn construction_delivery_is_delayed_and_atomic() {
        let mut rng = GameRng::new(0);
        let mut world = World::from_level(
            definition(vec![
                object(0, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 10)),
                object(1, ModelType::Scenery, 1, 255, (11, 11)),
            ]),
            catalog(),
        )
        .unwrap();
        let brave = world.source_handle(LevelObjectIndex(0)).unwrap();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        world.assign_construction(&[brave], hut).unwrap();

        for _ in 0..500 {
            world.tick_persons(&mut rng);
            let delivering = matches!(
                &world.get(brave).unwrap().data,
                GameObjectData::Person(person)
                    if person.state == PersonState::Building
                        && person.state_counter == BUILD_PHASE_DELIVER
            );
            if delivering {
                break;
            }
        }
        let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
            unreachable!()
        };
        assert_eq!(person.state_counter, BUILD_PHASE_DELIVER);
        assert_eq!(person.state_timer, DELIVERY_TICKS);
        assert_eq!(person.wood_carried, 1);

        for _ in 0..DELIVERY_TICKS - 1 {
            world.tick_persons(&mut rng);
            let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
                unreachable!()
            };
            assert_eq!(building.construction_progress, 0);
            assert_eq!(building.wood_consumed, 0);
        }

        world.tick_persons(&mut rng);
        let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
            unreachable!()
        };
        assert_eq!(building.construction_progress, CONSTRUCTION_UNITS_PER_WOOD);
        assert_eq!(building.construction_phase, 0);
        assert_eq!(building.wood_consumed, 1);
        assert_eq!(building.wood_stored, 0);
        assert_eq!(building.wood_reserved, 0);

        let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
            unreachable!()
        };
        assert_eq!(person.state_counter, BUILD_PHASE_SITE_WORK);
        assert_eq!(person.wood_carried, 0);
        assert!(person.state_timer >= original_ticks_to_world_ticks(32));
        assert!(person.state_timer <= original_ticks_to_world_ticks(95));
        let site_work_ticks = person.state_timer;
        for _ in 0..site_work_ticks - 1 {
            world.tick_persons(&mut rng);
            let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
                unreachable!()
            };
            assert_eq!(person.state_counter, BUILD_PHASE_SITE_WORK);
        }
        world.tick_persons(&mut rng);
        let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
            unreachable!()
        };
        assert_eq!(person.state, PersonState::Gathering);
    }

    #[test]
    fn final_scaffold_interval_precedes_completion_and_builder_regroups() {
        let mut rng = GameRng::new(0);
        let mut world = World::from_level(
            definition(vec![
                object(0, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 10)),
                object(1, ModelType::Scenery, 1, 255, (11, 11)),
            ]),
            catalog(),
        )
        .unwrap();
        let brave = world.source_handle(LevelObjectIndex(0)).unwrap();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        world.assign_construction(&[brave], hut).unwrap();

        for _ in 0..5_000 {
            world.tick_persons(&mut rng);
            let in_final_interval = matches!(
                &world.get(brave).unwrap().data,
                GameObjectData::Person(person)
                    if person.state == PersonState::Building
                        && person.state_counter == BUILD_PHASE_FINALIZE
            );
            if in_final_interval {
                break;
            }
        }
        let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
            unreachable!()
        };
        assert_eq!(building.state, BuildingState::Init);
        assert_eq!(building.construction_phase, 3);
        let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
            unreachable!()
        };
        assert_eq!(person.state_timer, FINAL_BUILD_TICKS);

        for _ in 0..FINAL_BUILD_TICKS - 1 {
            world.tick_persons(&mut rng);
            let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
                unreachable!()
            };
            assert_eq!(building.state, BuildingState::Init);
            assert_eq!(building.construction_phase, 3);
        }
        world.tick_persons(&mut rng);
        let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
            unreachable!()
        };
        assert_eq!(building.state, BuildingState::ConstructionDone);
        assert_eq!(building.construction_phase, 4);
        let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
            unreachable!()
        };
        assert_eq!(person.building_handle, None);
        assert_eq!(person.state, PersonState::GoToPoint);
        assert!(person.movement.is_moving());
        let (_, exit_y) = world_to_render_pos(&person.movement.target_pos, WORLD_SIZE as f32);
        assert!(exit_y < 12.0, "rotation zero exits beyond the -Z entrance");
    }

    #[test]
    fn foundation_changes_only_after_a_discrete_work_stroke() {
        let mut rng = GameRng::new(0);
        let mut world = World::from_level(
            definition(vec![object(
                0,
                ModelType::Person,
                BRAVE_SUBTYPE,
                0,
                (12, 12),
            )]),
            catalog(),
        )
        .unwrap();
        world.terrain.heights[12][13] = 132;
        world.terrain.heights[13][12] = 68;
        let brave = world.source_handle(LevelObjectIndex(0)).unwrap();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        world.assign_construction(&[brave], hut).unwrap();

        for _ in 0..200 {
            world.tick_persons(&mut rng);
            let working = matches!(
                &world.get(brave).unwrap().data,
                GameObjectData::Person(person)
                    if person.state == PersonState::Building
                        && person.state_counter == BUILD_PHASE_TRAVEL_OR_FLATTEN
                        && person.anim.animation_id == 120
                        && person.state_timer > 0
            );
            if working {
                break;
            }
        }
        let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
            unreachable!()
        };
        let first_stroke_ticks = person.state_timer;
        assert_eq!(first_stroke_ticks, FOUNDATION_STROKE_TICKS - 1);
        assert_eq!(world.terrain.heights[12][13], 132);
        assert_eq!(world.terrain.heights[13][12], 68);
        let terrain_revision = world.terrain.revision();

        for _ in 0..first_stroke_ticks - 1 {
            world.tick_persons(&mut rng);
        }
        assert_eq!(world.terrain.heights[12][13], 132);
        assert_eq!(world.terrain.heights[13][12], 68);

        world.tick_persons(&mut rng);
        assert_eq!(world.terrain.revision(), terrain_revision + 1);
        let changed = [world.terrain.heights[12][13], world.terrain.heights[13][12]]
            .into_iter()
            .filter(|height| !matches!(height, 132 | 68))
            .count();
        assert_eq!(changed, 1, "one work stroke must change exactly one cell");
        assert!(matches!(world.terrain.heights[12][13], 132 | 116));
        assert!(matches!(world.terrain.heights[13][12], 68 | 84));
        let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
            unreachable!()
        };
        assert_eq!(person.anim.animation_id, 120);
        assert_eq!(person.anim.frame_index, 0);
    }

    #[test]
    fn one_brave_hut_construction_has_original_minimum_work_time() {
        let mut rng = GameRng::new(0);
        let mut world = World::from_level(
            definition(vec![
                object(0, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 10)),
                object(1, ModelType::Scenery, 1, 255, (11, 11)),
            ]),
            catalog(),
        )
        .unwrap();
        world.terrain.heights[12][13] = 164;
        let brave = world.source_handle(LevelObjectIndex(0)).unwrap();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        world.assign_construction(&[brave], hut).unwrap();

        let minimum_foundation_ticks = 4 * original_ticks_to_world_ticks(32);
        let minimum_work_ticks = minimum_foundation_ticks
            + 3 * (CHOP_TICKS
                + DELIVERY_TICKS
                + original_ticks_to_world_ticks(SITE_WORK_MIN_ORIGINAL_TICKS));
        let mut completed_at = None;
        for tick in 1..=5_000 {
            world.tick_persons(&mut rng);
            let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
                unreachable!()
            };
            assert_eq!(
                building.construction_progress % CONSTRUCTION_UNITS_PER_WOOD,
                0,
                "construction progress changed between discrete wood deliveries"
            );
            world.tick_buildings();
            if matches!(
                &world.get(hut).unwrap().data,
                GameObjectData::Building(building) if building.state == BuildingState::Active
            ) {
                completed_at = Some(tick);
                break;
            }
        }

        let completed_at = completed_at.expect("hut should complete");
        assert!(
            completed_at >= minimum_work_ticks,
            "hut completed in {completed_at} ticks, below the {minimum_work_ticks}-tick work minimum"
        );
    }

    #[test]
    fn construction_assignment_validates_builder_and_owner() {
        let mut world = World::from_level(
            definition(vec![
                object(0, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 10)),
                object(1, ModelType::Person, 3, 0, (11, 10)),
                object(2, ModelType::Person, BRAVE_SUBTYPE, 1, (12, 10)),
            ]),
            catalog(),
        )
        .unwrap();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (20, 20), 0)
            .unwrap();
        let brave = world.source_handle(LevelObjectIndex(0)).unwrap();
        let warrior = world.source_handle(LevelObjectIndex(1)).unwrap();
        let enemy = world.source_handle(LevelObjectIndex(2)).unwrap();

        assert_eq!(
            world.assign_construction(&[warrior], hut),
            Err(WorldError::InvalidBuilder(warrior))
        );
        assert_eq!(
            world.assign_construction(&[enemy], hut),
            Err(WorldError::BuilderOwnerMismatch(enemy))
        );
        world.assign_construction(&[brave], hut).unwrap();
        let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
            unreachable!()
        };
        assert_eq!(person.building_handle, Some(hut));
        assert_eq!(person.state, PersonState::Building);
    }

    #[test]
    fn multiple_builders_do_not_over_reserve_wood() {
        let mut rng = GameRng::new(0);
        let mut world = World::from_level(
            definition(vec![
                object(0, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 10)),
                object(1, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 11)),
                object(2, ModelType::Scenery, 1, 255, (11, 11)),
            ]),
            catalog(),
        )
        .unwrap();
        let braves = [
            world.source_handle(LevelObjectIndex(0)).unwrap(),
            world.source_handle(LevelObjectIndex(1)).unwrap(),
        ];
        let tree = world.source_handle(LevelObjectIndex(2)).unwrap();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        world.assign_construction(&braves, hut).unwrap();

        for _ in 0..5_000 {
            world.tick_persons(&mut rng);
            world.tick_buildings();
            let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
                unreachable!()
            };
            assert!(
                building.wood_consumed + building.wood_stored + building.wood_reserved
                    <= construction_target(BuildingSubtype::SmallHut)
            );
            let GameObjectData::Scenery(tree) = &world.get(tree).unwrap().data else {
                unreachable!()
            };
            assert!(tree.wood_reserved <= tree.wood_remaining);
            if building.state == BuildingState::Active {
                break;
            }
        }
        let GameObjectData::Building(building) = &world.get(hut).unwrap().data else {
            unreachable!()
        };
        assert_eq!(building.state, BuildingState::Active);
        assert_eq!(
            building.wood_consumed,
            construction_target(BuildingSubtype::SmallHut)
        );
    }

    #[test]
    fn missing_plan_cancels_builder_and_releases_tree_reservation() {
        let mut rng = GameRng::new(0);
        let mut world = World::from_level(
            definition(vec![
                object(0, ModelType::Person, BRAVE_SUBTYPE, 0, (10, 10)),
                object(1, ModelType::Scenery, 1, 255, (11, 11)),
            ]),
            catalog(),
        )
        .unwrap();
        let brave = world.source_handle(LevelObjectIndex(0)).unwrap();
        let tree = world.source_handle(LevelObjectIndex(1)).unwrap();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        world.assign_construction(&[brave], hut).unwrap();
        for _ in 0..200 {
            world.tick_persons(&mut rng);
            let reserved = matches!(
                &world.get(brave).unwrap().data,
                GameObjectData::Person(person) if person.construction_wood_reserved
            );
            if reserved {
                break;
            }
        }
        assert!(world.pool.destroy(hut));
        world.tick_persons(&mut rng);
        let GameObjectData::Person(person) = &world.get(brave).unwrap().data else {
            unreachable!()
        };
        assert_eq!(person.state, PersonState::Idle);
        assert_eq!(person.building_handle, None);
        let GameObjectData::Scenery(tree) = &world.get(tree).unwrap().data else {
            unreachable!()
        };
        assert_eq!(tree.wood_reserved, 0);
    }

    #[test]
    fn missing_footprint_reports_source_slot() {
        let error = World::from_level(
            definition(vec![object(77, ModelType::Building, 1, 0, (1, 1))]),
            BuildingCatalog::new(),
        )
        .err()
        .unwrap();
        assert_eq!(
            error,
            LevelInitError::MissingBuildingFootprint {
                source: LevelObjectIndex(77),
                subtype: 1,
                rotation: 0
            }
        );
    }
}
