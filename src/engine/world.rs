use std::collections::HashMap;

use crate::data::level::{LevelDefinition, LevelObjectIndex};
use crate::data::units::ModelType;
use crate::engine::buildings::spawning::population_growth_weight;
use crate::engine::buildings::tick::{construction_target, tick_building_with_population};
use crate::engine::buildings::types::{building_behavior_flags, building_max_health};
use crate::engine::buildings::{
    BuildingCatalog, BuildingState, BuildingSubtype, PlacementError, SpawnAction,
};
use crate::engine::economy::population::{calculate_housing_capacity, can_spawn};
use crate::engine::movement::WorldCoord;
use crate::engine::objects::{CellGrid, GameObjectData, ObjectHandle, ObjectPool};
use crate::engine::state::tribe::TribeArray;
use crate::engine::units::coords::{cell_to_world, world_to_render_pos};
use crate::engine::units::person_state::{person_type_defaults, PersonState};

const WORLD_SIZE: usize = 128;

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

    fn occupy_and_flatten(
        &mut self,
        handle: ObjectHandle,
        cell: (i32, i32),
        footprint: &[(i16, i16)],
    ) {
        let base_height = self.heights[(cell.1 & 127) as usize][(cell.0 & 127) as usize];
        for &(dx, dy) in footprint {
            let x = ((cell.0 + dx as i32) & 127) as usize;
            let y = ((cell.1 + dy as i32) & 127) as usize;
            self.occupancy[y][x] = Some(handle);
            self.heights[y][x] = base_height;
            self.walkability[y][x] = 0;
        }
        self.revision = self.revision.wrapping_add(1).max(1);
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersonRenderRecord {
    pub handle: ObjectHandle,
    pub subtype: u8,
    pub tribe: u8,
    pub cell_x: f32,
    pub cell_y: f32,
    pub angle: u16,
    pub health: u16,
    pub max_health: u16,
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
                    building.construction_progress = 0;
                    building.population_timer_enabled = false;
                    object.header.health = building_max_health(subtype);
                    object.header.max_health = object.header.health;
                }
                world.terrain.occupy_and_flatten(handle, cell, &footprint);
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
        if let GameObjectData::Person(person) = &mut object.data {
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
            let (x, y) = world_to_render_pos(&object.header.position, 128.0);
            person.cell_x = x;
            person.cell_y = y;
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

    pub fn select(&mut self, handles: Vec<ObjectHandle>) {
        self.selected = handles
            .into_iter()
            .filter(|h| self.pool.get(*h).is_some())
            .collect();
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
            building.wood_stored = construction_target(subtype);
            building.population_timer_enabled = true;
        }
        self.link(handle);
        self.terrain.occupy_and_flatten(handle, cell, &footprint);
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

    pub fn tick_persons(&mut self) {
        let handles: Vec<_> = self.pool.persons().map(|(h, _, _)| h).collect();
        for handle in handles {
            let mut new_position = None;
            let stale_target = self
                .pool
                .get(handle)
                .and_then(|object| match &object.data {
                    GameObjectData::Person(person) => person.target_unit,
                    _ => None,
                })
                .is_some_and(|target| self.pool.get(target).is_none());
            if let Some(object) = self.pool.get_mut(handle) {
                let GameObjectData::Person(person) = &mut object.data else {
                    continue;
                };
                if stale_target {
                    person.target_unit = None;
                }
                if person.movement.is_moving() {
                    let target = person.movement.target_pos;
                    let mut pos = object.header.position;
                    pos.x = approach(pos.x, target.x, 32);
                    pos.z = approach(pos.z, target.z, 32);
                    new_position = Some(pos);
                    if pos == target {
                        person.movement.flags1 &= !0x1000;
                        person.state = PersonState::Idle;
                    }
                }
            }
            if let Some(position) = new_position {
                self.move_object(handle, position);
            }
        }
        self.selected.retain(|h| self.pool.get(*h).is_some());
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
        for (handle, header, _) in self.pool.persons() {
            let (cell_x, cell_y) = world_to_render_pos(&header.position, 128.0);
            persons.push(PersonRenderRecord {
                handle,
                subtype: header.subtype,
                tribe: header.tribe,
                cell_x,
                cell_y,
                angle: header.angle,
                health: header.health,
                max_health: header.max_health,
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
    let (x, y) = world_to_render_pos(&position, 128.0);
    (x.floor() as i32, y.floor() as i32)
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
        let before = world.terrain.revision();
        let hut = world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        assert!(world.terrain.revision() > before);
        let count = world.pool.active_count();
        assert_eq!(
            world.place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0),
            Err(WorldError::InvalidPlacement(PlacementError::Occupied))
        );
        assert_eq!(world.pool.active_count(), count);
        assert!(world
            .snapshot()
            .objects
            .iter()
            .any(|record| record.handle == hut));
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
    fn hut_constructs_then_spawns_one_canonical_brave() {
        let mut world = World::from_level(definition(Vec::new()), catalog()).unwrap();
        world
            .place_building(BuildingSubtype::SmallHut, 0, (12, 12), 0)
            .unwrap();
        for _ in 0..4 {
            world.tick_buildings();
        }
        assert_eq!(world.tribes.tribes[0].max_population, 3);
        // Empty small hut at the first population band: ceil(1187 / 2).
        for _ in 0..594 {
            world.tick_buildings();
        }
        assert_eq!(world.pool.persons().count(), 1);
        assert_eq!(world.tribes.tribes[0].population, 1);
        let snapshot = world.snapshot();
        assert_eq!(snapshot.persons.len(), 1);
        let brave = snapshot.persons[0].handle;
        let position = world.get(brave).unwrap().header.position;
        assert_eq!(
            world.cell_head(CellGrid::cell_index_from_world(&position)),
            Some(brave)
        );
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
