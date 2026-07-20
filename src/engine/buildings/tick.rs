use super::combat::{tick_building_combat, BuildingCombatAction};
use super::spawning::{tick_spawn, SpawnAction};
use super::state_machine::*;
use super::training::{tick_convert, ConvertAction};
use super::types::*;
use crate::engine::economy::wood::construction_wood_cost;
use crate::engine::objects::{ObjectHandle, ObjectHeader};

pub const CONSTRUCTION_UNITS_PER_WOOD: u16 = 100;
const DESTRUCTION_UNITS_PER_TICK: u16 = 4;

/// Aggregated actions emitted by a single building tick.
#[derive(Debug)]
pub struct BuildingTickActions {
    pub spawn: SpawnAction,
    pub convert: ConvertAction,
    pub combat: Vec<BuildingCombatAction>,
}

impl BuildingTickActions {
    pub fn none() -> Self {
        Self {
            spawn: SpawnAction::None,
            convert: ConvertAction::None,
            combat: Vec::new(),
        }
    }
}

/// Per-tick building update following original binary's BLD.7 pipeline order.
/// Original: Building_Update called from Tick_UpdateObjects at 0x0042E5F0.
pub fn tick_building(
    building: &mut BuildingData,
    header: &mut ObjectHeader,
    handle: ObjectHandle,
) -> BuildingTickActions {
    tick_building_with_population(building, header, handle, 0, true)
}

pub fn tick_building_with_population(
    building: &mut BuildingData,
    header: &mut ObjectHeader,
    handle: ObjectHandle,
    weighted_population: u32,
    has_population_capacity: bool,
) -> BuildingTickActions {
    // 1. Damage cooldown decrement
    if building.damage_cooldown > 0 {
        building.damage_cooldown -= 1;
    }

    // 2. Wobble animation decay (halves each tick)
    building.shake_x = building.shake_x / 2;
    building.shake_z = building.shake_z / 2;

    // 3. State dispatch
    match building.state {
        BuildingState::Init => {
            // Construction progress is driven by assigned brave work in World.
            BuildingTickActions::none()
        }
        BuildingState::ConstructionDone => {
            on_construction_complete(building);
            transition_building_state(building, BuildingState::Active);
            BuildingTickActions::none()
        }
        BuildingState::Active => tick_active(
            building,
            header,
            handle,
            weighted_population,
            has_population_capacity,
        ),
        BuildingState::Destroying => {
            tick_destroying(building, header);
            BuildingTickActions::none()
        }
        BuildingState::Sinking => {
            tick_sinking(building, header);
            BuildingTickActions::none()
        }
        BuildingState::FinalTeardown => BuildingTickActions::none(),
    }
}

fn tick_active(
    building: &mut BuildingData,
    _header: &mut ObjectHeader,
    handle: ObjectHandle,
    weighted_population: u32,
    has_population_capacity: bool,
) -> BuildingTickActions {
    let spawn = tick_spawn(building, weighted_population, has_population_capacity);
    let convert = tick_convert(building);
    let combat = tick_building_combat(building, handle);
    BuildingTickActions {
        spawn,
        convert,
        combat,
    }
}

fn tick_destroying(building: &mut BuildingData, _header: &mut ObjectHeader) {
    building.construction_progress = building
        .construction_progress
        .saturating_sub(DESTRUCTION_UNITS_PER_TICK);
    let target = construction_progress_target(building.building_subtype);
    building.construction_phase = construction_phase(building.construction_progress, target);
    if building.construction_progress == 0 {
        transition_building_state(building, BuildingState::Sinking);
    }
}

fn tick_sinking(building: &mut BuildingData, _header: &mut ObjectHeader) {
    // Sink animation counter, transition to teardown
    building.construction_progress += 1;
    if building.construction_progress >= 60 {
        transition_building_state(building, BuildingState::FinalTeardown);
    }
}

/// Wood units needed to complete construction, by subtype.
/// A carried wood piece is 100 original units. Hut upgrades each consume three
/// pieces; other values come from the same original constant.dat file.
pub fn construction_target(subtype: BuildingSubtype) -> u16 {
    construction_wood_cost(subtype as u8)
}

pub fn construction_progress_target(subtype: BuildingSubtype) -> u16 {
    construction_target(subtype).saturating_mul(CONSTRUCTION_UNITS_PER_WOOD)
}

/// Original shared phase calculation at 0x00491B40. Construction and
/// destruction both use this normalized 0..=4 phase value.
pub fn construction_phase(progress: u16, total: u16) -> u8 {
    if progress == 0 || total <= 1 {
        0
    } else if progress >= total {
        4
    } else {
        (((4 * progress as u32 - 1) / (total as u32 - 1)).min(3)) as u8
    }
}

/// Visible assembly stage after a discrete wood delivery. The original small
/// hut has three wood deliveries but four incomplete meshes: the first piece
/// reveals sparse phase zero, the last piece reaches phase two, and a final
/// building interval reveals phase three before the completed mesh.
pub fn construction_delivery_phase(delivered: u16, required: u16) -> u8 {
    if delivered == 0 {
        return 0;
    }
    if required <= 1 {
        return 2;
    }
    (((delivered.saturating_sub(1) as u32 * 2) / (required - 1) as u32).min(2)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::units::ModelType;
    use crate::engine::movement::WorldCoord;

    const fn h(slot: u16) -> ObjectHandle {
        ObjectHandle::new(slot, 1)
    }

    fn make_header() -> ObjectHeader {
        ObjectHeader {
            model_type: ModelType::Building,
            subtype: 1,
            tribe: 0,
            state: 0,
            state_phase: 0,
            flags1: 0,
            flags2: 0,
            flags3: 0,
            object_index: h(0),
            angle: 0,
            position: WorldCoord::default(),
            velocity: WorldCoord::default(),
            health: 600,
            max_health: 600,
            next_in_cell: None,
            prev_in_cell: None,
        }
    }

    const DUMMY_HANDLE: ObjectHandle = h(0);

    #[test]
    fn tick_decrements_damage_cooldown() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        b.damage_cooldown = 3;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.damage_cooldown, 2);
    }

    #[test]
    fn tick_decays_wobble() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        b.shake_x = 8;
        b.shake_z = -6;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.shake_x, 4);
        assert_eq!(b.shake_z, -3);
    }

    #[test]
    fn construction_does_not_advance_without_builder_work() {
        let mut b = BuildingData::default();
        b.building_subtype = BuildingSubtype::SmallHut;
        b.wood_stored = 10;
        b.construction_progress = 0;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.wood_stored, 10);
        assert_eq!(b.construction_progress, 0);
    }

    #[test]
    fn construction_cannot_complete_from_a_building_tick() {
        let mut b = BuildingData::default();
        b.building_subtype = BuildingSubtype::SmallHut; // target = 3
        b.wood_stored = 10;
        b.construction_progress = 2; // one more tick to reach 3
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.state, BuildingState::Init);
        assert_eq!(b.construction_progress, 2);
    }

    #[test]
    fn construction_done_transitions_to_active_on_tick() {
        let mut b = BuildingData::default();
        b.state = BuildingState::ConstructionDone;
        b.building_subtype = BuildingSubtype::SmallHut;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.state, BuildingState::Active);
        assert_eq!(b.behavior_flags, 0x20); // housing flag set
    }

    #[test]
    fn destroying_transitions_to_sinking() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Destroying;
        b.damage_accumulated = 100;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.state, BuildingState::Sinking);
    }

    #[test]
    fn destroying_stays_if_damage_low() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Destroying;
        b.damage_accumulated = 50;
        b.construction_progress = 100;
        b.construction_phase = 1;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.state, BuildingState::Destroying);
        assert_eq!(b.construction_progress, 96);
    }

    #[test]
    fn sinking_increments_progress_and_transitions() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Sinking;
        b.construction_progress = 59;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.state, BuildingState::FinalTeardown);
    }

    #[test]
    fn sinking_stays_if_not_done() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Sinking;
        b.construction_progress = 0;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.state, BuildingState::Sinking);
        assert_eq!(b.construction_progress, 1);
    }

    #[test]
    fn construction_target_values() {
        assert_eq!(construction_target(BuildingSubtype::SmallHut), 3);
        assert_eq!(construction_target(BuildingSubtype::MediumHut), 3);
        assert_eq!(construction_target(BuildingSubtype::LargeHut), 3);
        assert_eq!(construction_target(BuildingSubtype::DrumTower), 5);
        assert_eq!(construction_target(BuildingSubtype::Temple), 8);
        assert_eq!(construction_target(BuildingSubtype::WarriorTrain), 8);
        assert_eq!(construction_target(BuildingSubtype::WallPiece), 4); // default
    }

    #[test]
    fn normalized_construction_phases_match_original_thresholds() {
        let total = construction_progress_target(BuildingSubtype::SmallHut);
        assert_eq!(total, 300);
        assert_eq!(construction_phase(0, total), 0);
        assert_eq!(construction_phase(74, total), 0);
        assert_eq!(construction_phase(75, total), 1);
        assert_eq!(construction_phase(149, total), 1);
        assert_eq!(construction_phase(150, total), 2);
        assert_eq!(construction_phase(224, total), 2);
        assert_eq!(construction_phase(225, total), 3);
        assert_eq!(construction_phase(299, total), 3);
        assert_eq!(construction_phase(300, total), 4);
    }

    #[test]
    fn three_wood_deliveries_leave_room_for_the_final_scaffold_stage() {
        assert_eq!(construction_delivery_phase(0, 3), 0);
        assert_eq!(construction_delivery_phase(1, 3), 0);
        assert_eq!(construction_delivery_phase(2, 3), 1);
        assert_eq!(construction_delivery_phase(3, 3), 2);
    }

    #[test]
    fn no_wood_no_construction_progress() {
        let mut b = BuildingData::default();
        b.building_subtype = BuildingSubtype::SmallHut;
        b.wood_stored = 0;
        b.construction_progress = 0;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.construction_progress, 0);
        assert_eq!(b.state, BuildingState::Init); // stuck without wood
    }

    #[test]
    fn final_teardown_is_noop() {
        let mut b = BuildingData::default();
        b.state = BuildingState::FinalTeardown;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.state, BuildingState::FinalTeardown); // caller handles removal
    }

    #[test]
    fn tick_active_returns_building_tick_actions() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        b.building_subtype = BuildingSubtype::SmallHut;
        b.behavior_flags = 0x20; // housing flag
        b.construction_progress = 0;
        let mut h = make_header();
        let actions = tick_building(&mut b, &mut h, DUMMY_HANDLE);
        // Spawn timer just incremented, not at threshold yet
        assert_eq!(actions.spawn, SpawnAction::None);
        assert_eq!(actions.convert, ConvertAction::None);
        assert!(actions.combat.is_empty());
    }

    #[test]
    fn tick_active_returns_combat_actions() {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        b.building_subtype = BuildingSubtype::DrumTower;
        b.behavior_flags = 0x08; // fighting flag
        b.occupant_slots[0] = Some(h(10));
        b.occupant_count = 1;
        b.num_fighting = 1;
        b.target_person = Some(h(99));
        let mut h = make_header();
        let actions = tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(actions.combat.len(), 1);
    }
}
