use super::types::*;

// Spawn rate by hut level (ticks between spawns)
pub const HUT_SPROG_TIME_1: u16 = 1500;
pub const HUT_SPROG_TIME_2: u16 = 1200;
pub const HUT_SPROG_TIME_3: u16 = 900;

pub fn sprog_time_for_level(subtype: BuildingSubtype) -> u16 {
    match subtype {
        BuildingSubtype::SmallHut => HUT_SPROG_TIME_1,
        BuildingSubtype::MediumHut => HUT_SPROG_TIME_2,
        BuildingSubtype::LargeHut => HUT_SPROG_TIME_3,
        _ => u16::MAX, // non-hut buildings don't spawn
    }
}

/// Result of spawn tick check.
#[derive(Debug, PartialEq)]
pub enum SpawnAction {
    None,
    SpawnBrave,
}

/// Check if hut should spawn a brave this tick.
/// Caller must verify can_spawn() and create the brave in the pool.
pub fn tick_spawn(building: &mut BuildingData) -> SpawnAction {
    if !building.population_timer_enabled {
        return SpawnAction::None;
    }
    if building.behavior_flags & 0x20 == 0 {
        return SpawnAction::None;
    }
    if building.state != BuildingState::Active {
        return SpawnAction::None;
    }

    building.construction_progress += 1; // reuse as spawn timer in Active state
    let threshold = sprog_time_for_level(building.building_subtype);
    if building.construction_progress >= threshold {
        building.construction_progress = 0;
        SpawnAction::SpawnBrave
    } else {
        SpawnAction::None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_active_hut(subtype: BuildingSubtype) -> BuildingData {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        b.building_subtype = subtype;
        b.behavior_flags = 0x20; // housing flag
        b.construction_progress = 0;
        b
    }

    #[test]
    fn sprog_time_small_hut() {
        assert_eq!(sprog_time_for_level(BuildingSubtype::SmallHut), 1500);
    }

    #[test]
    fn sprog_time_medium_hut() {
        assert_eq!(sprog_time_for_level(BuildingSubtype::MediumHut), 1200);
    }

    #[test]
    fn sprog_time_large_hut() {
        assert_eq!(sprog_time_for_level(BuildingSubtype::LargeHut), 900);
    }

    #[test]
    fn sprog_time_non_hut_is_max() {
        assert_eq!(sprog_time_for_level(BuildingSubtype::DrumTower), u16::MAX);
        assert_eq!(sprog_time_for_level(BuildingSubtype::Temple), u16::MAX);
    }

    #[test]
    fn tick_spawn_increments_timer() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        let action = tick_spawn(&mut b);
        assert_eq!(action, SpawnAction::None);
        assert_eq!(b.construction_progress, 1);
    }

    #[test]
    fn tick_spawn_triggers_at_threshold() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        b.construction_progress = HUT_SPROG_TIME_1 - 1; // one tick away
        let action = tick_spawn(&mut b);
        assert_eq!(action, SpawnAction::SpawnBrave);
        assert_eq!(b.construction_progress, 0); // reset
    }

    #[test]
    fn tick_spawn_large_hut_threshold() {
        let mut b = make_active_hut(BuildingSubtype::LargeHut);
        b.construction_progress = HUT_SPROG_TIME_3 - 1;
        let action = tick_spawn(&mut b);
        assert_eq!(action, SpawnAction::SpawnBrave);
        assert_eq!(b.construction_progress, 0);
    }

    #[test]
    fn tick_spawn_skips_without_spawn_flag() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        b.behavior_flags = 0; // no spawn flag
        let action = tick_spawn(&mut b);
        assert_eq!(action, SpawnAction::None);
        assert_eq!(b.construction_progress, 0); // not incremented
    }

    #[test]
    fn tick_spawn_skips_non_active_state() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        b.state = BuildingState::Init;
        let action = tick_spawn(&mut b);
        assert_eq!(action, SpawnAction::None);
        assert_eq!(b.construction_progress, 0);
    }
}
