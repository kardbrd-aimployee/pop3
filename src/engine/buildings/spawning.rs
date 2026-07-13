use super::types::*;

// Values loaded by the original game from LEVELS/constant.dat.
pub const HUT_SPROG_TIME_1: u16 = 4000;
pub const HUT_SPROG_TIME_2: u16 = 3000;
pub const HUT_SPROG_TIME_3: u16 = 2000;

const POPULATION_BAND_PERCENT: [u16; 20] = [
    30, 35, 40, 50, 60, 70, 80, 90, 100, 110, 120, 130, 140, 150, 160, 170, 180, 190, 195, 200,
];

pub fn sprog_time_for_level(subtype: BuildingSubtype) -> u16 {
    match subtype {
        BuildingSubtype::SmallHut => HUT_SPROG_TIME_1,
        BuildingSubtype::MediumHut => HUT_SPROG_TIME_2,
        BuildingSubtype::LargeHut => HUT_SPROG_TIME_3,
        _ => u16::MAX, // non-hut buildings don't spawn
    }
}

/// Population weight used by `Building_CalcPopGrowthRate` at 0x00426220.
/// Shamans and wild people are not included in the five population counters.
pub const fn population_growth_weight(person_subtype: u8) -> u32 {
    match person_subtype {
        2 => 15,    // brave
        3..=6 => 4, // warrior, preacher, spy, firewarrior
        _ => 0,
    }
}

pub const fn population_band(weighted_population: u32) -> usize {
    let band = ((weighted_population + 1) / 10) as usize;
    if band < POPULATION_BAND_PERCENT.len() {
        band
    } else {
        POPULATION_BAND_PERCENT.len() - 1
    }
}

/// Original percent constants are converted to 8.8 fixed point with integer
/// truncation by the constant.dat loader.
pub const fn population_scale_fixed(weighted_population: u32) -> u32 {
    let percent = POPULATION_BAND_PERCENT[population_band(weighted_population)] as u32;
    (percent << 8) / 100
}

pub fn spawn_threshold(subtype: BuildingSubtype, weighted_population: u32) -> u16 {
    let base = sprog_time_for_level(subtype) as u32;
    if base == u16::MAX as u32 {
        return u16::MAX;
    }
    ((base * population_scale_fixed(weighted_population)) >> 8) as u16
}

/// Result of spawn tick check.
#[derive(Debug, PartialEq)]
pub enum SpawnAction {
    None,
    SpawnBrave,
}

/// Check if hut should spawn a brave this tick.
/// Caller creates the brave in the pool when this returns `SpawnBrave`.
pub fn tick_spawn(
    building: &mut BuildingData,
    weighted_population: u32,
    has_population_capacity: bool,
) -> SpawnAction {
    if !building.population_timer_enabled {
        return SpawnAction::None;
    }
    if building.behavior_flags & 0x20 == 0 {
        return SpawnAction::None;
    }
    if building.state != BuildingState::Active {
        return SpawnAction::None;
    }

    // Building_UpdatePopGrowth adds 2 * (occupants + 1) each update.
    let growth = 2 * (u16::from(building.occupant_count) + 1);
    building.construction_progress = building.construction_progress.saturating_add(growth);
    let threshold = spawn_threshold(building.building_subtype, weighted_population);
    if building.construction_progress >= threshold {
        if !has_population_capacity {
            building.construction_progress = threshold;
            return SpawnAction::None;
        }
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
        assert_eq!(sprog_time_for_level(BuildingSubtype::SmallHut), 4000);
    }

    #[test]
    fn sprog_time_medium_hut() {
        assert_eq!(sprog_time_for_level(BuildingSubtype::MediumHut), 3000);
    }

    #[test]
    fn sprog_time_large_hut() {
        assert_eq!(sprog_time_for_level(BuildingSubtype::LargeHut), 2000);
    }

    #[test]
    fn sprog_time_non_hut_is_max() {
        assert_eq!(sprog_time_for_level(BuildingSubtype::DrumTower), u16::MAX);
        assert_eq!(sprog_time_for_level(BuildingSubtype::Temple), u16::MAX);
    }

    #[test]
    fn tick_spawn_increments_timer() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        let action = tick_spawn(&mut b, 0, true);
        assert_eq!(action, SpawnAction::None);
        assert_eq!(b.construction_progress, 2);
    }

    #[test]
    fn tick_spawn_triggers_at_threshold() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        b.construction_progress = spawn_threshold(BuildingSubtype::SmallHut, 0) - 2;
        let action = tick_spawn(&mut b, 0, true);
        assert_eq!(action, SpawnAction::SpawnBrave);
        assert_eq!(b.construction_progress, 0); // reset
    }

    #[test]
    fn tick_spawn_large_hut_threshold() {
        let mut b = make_active_hut(BuildingSubtype::LargeHut);
        b.construction_progress = spawn_threshold(BuildingSubtype::LargeHut, 0) - 2;
        let action = tick_spawn(&mut b, 0, true);
        assert_eq!(action, SpawnAction::SpawnBrave);
        assert_eq!(b.construction_progress, 0);
    }

    #[test]
    fn tick_spawn_skips_without_spawn_flag() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        b.behavior_flags = 0; // no spawn flag
        let action = tick_spawn(&mut b, 0, true);
        assert_eq!(action, SpawnAction::None);
        assert_eq!(b.construction_progress, 0); // not incremented
    }

    #[test]
    fn tick_spawn_skips_non_active_state() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        b.state = BuildingState::Init;
        let action = tick_spawn(&mut b, 0, true);
        assert_eq!(action, SpawnAction::None);
        assert_eq!(b.construction_progress, 0);
    }

    #[test]
    fn population_bands_match_original_integer_math() {
        assert_eq!(population_band(0), 0);
        assert_eq!(population_band(9), 1);
        assert_eq!(population_band(198), 19);
        assert_eq!(population_band(999), 19);
        assert_eq!(population_scale_fixed(0), 76);
        assert_eq!(spawn_threshold(BuildingSubtype::SmallHut, 0), 1187);
        assert_eq!(spawn_threshold(BuildingSubtype::SmallHut, 198), 8000);
    }

    #[test]
    fn occupants_accelerate_growth() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        b.occupant_count = 3;
        assert_eq!(tick_spawn(&mut b, 0, true), SpawnAction::None);
        assert_eq!(b.construction_progress, 8);
    }

    #[test]
    fn completed_growth_waits_at_population_capacity() {
        let mut b = make_active_hut(BuildingSubtype::SmallHut);
        let threshold = spawn_threshold(BuildingSubtype::SmallHut, 0);
        b.construction_progress = threshold - 2;
        assert_eq!(tick_spawn(&mut b, 0, false), SpawnAction::None);
        assert_eq!(b.construction_progress, threshold);
        assert_eq!(tick_spawn(&mut b, 0, true), SpawnAction::SpawnBrave);
    }
}
