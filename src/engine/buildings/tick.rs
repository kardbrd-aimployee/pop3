use super::combat::{tick_building_combat, BuildingCombatAction};
use super::spawning::{tick_spawn, SpawnAction};
use super::state_machine::*;
use super::training::{tick_convert, ConvertAction};
use super::types::*;
use crate::engine::objects::{ObjectHandle, ObjectHeader};

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
            tick_constructing(building, header);
            BuildingTickActions::none()
        }
        BuildingState::ConstructionDone => {
            on_construction_complete(building);
            transition_building_state(building, BuildingState::Active);
            BuildingTickActions::none()
        }
        BuildingState::Active => tick_active(building, header, handle),
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

fn tick_constructing(building: &mut BuildingData, _header: &mut ObjectHeader) {
    if building.wood_stored > 0 {
        let consume = 1.min(building.wood_stored);
        building.wood_stored -= consume;
        building.construction_progress += 1;
    }
    // Transition when fully constructed (progress reaches target for type)
    if building.construction_progress >= construction_target(building.building_subtype) {
        transition_building_state(building, BuildingState::ConstructionDone);
    }
}

fn tick_active(
    building: &mut BuildingData,
    _header: &mut ObjectHeader,
    handle: ObjectHandle,
) -> BuildingTickActions {
    let spawn = tick_spawn(building);
    let convert = tick_convert(building);
    let combat = tick_building_combat(building, handle);
    BuildingTickActions {
        spawn,
        convert,
        combat,
    }
}

fn tick_destroying(building: &mut BuildingData, _header: &mut ObjectHeader) {
    // Decrement health toward 0, transition to sinking
    if building.damage_accumulated >= 100 {
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
/// Values approximate the original binary's wood cost configuration.
pub fn construction_target(subtype: BuildingSubtype) -> u16 {
    match subtype {
        BuildingSubtype::SmallHut => 3,
        BuildingSubtype::MediumHut => 5,
        BuildingSubtype::LargeHut => 7,
        BuildingSubtype::DrumTower => 5,
        BuildingSubtype::Temple => 6,
        BuildingSubtype::SpyTrain
        | BuildingSubtype::WarriorTrain
        | BuildingSubtype::SuperWarriorTrain => 5,
        _ => 4,
    }
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
    fn construction_consumes_wood_and_increments_progress() {
        let mut b = BuildingData::default();
        b.building_subtype = BuildingSubtype::SmallHut;
        b.wood_stored = 10;
        b.construction_progress = 0;
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.wood_stored, 9);
        assert_eq!(b.construction_progress, 1);
    }

    #[test]
    fn construction_completes_at_target() {
        let mut b = BuildingData::default();
        b.building_subtype = BuildingSubtype::SmallHut; // target = 3
        b.wood_stored = 10;
        b.construction_progress = 2; // one more tick to reach 3
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.state, BuildingState::ConstructionDone);
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
        let mut h = make_header();
        tick_building(&mut b, &mut h, DUMMY_HANDLE);
        assert_eq!(b.state, BuildingState::Destroying);
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
        assert_eq!(construction_target(BuildingSubtype::MediumHut), 5);
        assert_eq!(construction_target(BuildingSubtype::LargeHut), 7);
        assert_eq!(construction_target(BuildingSubtype::DrumTower), 5);
        assert_eq!(construction_target(BuildingSubtype::Temple), 6);
        assert_eq!(construction_target(BuildingSubtype::WarriorTrain), 5);
        assert_eq!(construction_target(BuildingSubtype::WallPiece), 4); // default
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
