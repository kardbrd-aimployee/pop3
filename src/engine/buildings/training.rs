use super::types::*;
use crate::engine::objects::ObjectHandle;

// Conversion times (ticks)
pub const CONV_TIME_WARRIOR: u16 = 256;
pub const CONV_TIME_SPY: u16 = 192;
pub const CONV_TIME_PREACHER: u16 = 192;
pub const CONV_TIME_SUPER_WARRIOR: u16 = 384;

// Mana costs for training
pub const MANA_COST_WARRIOR: u32 = 500;
pub const MANA_COST_SPY: u32 = 300;
pub const MANA_COST_PREACHER: u32 = 400;
pub const MANA_COST_SUPER_WARRIOR: u32 = 800;

/// What unit type a training building produces.
pub fn training_output_subtype(building_subtype: BuildingSubtype) -> Option<u8> {
    match building_subtype {
        BuildingSubtype::WarriorTrain => Some(3),     // Warrior
        BuildingSubtype::SpyTrain => Some(5),          // Spy
        BuildingSubtype::Temple => Some(4),            // Preacher (Religious)
        BuildingSubtype::SuperWarriorTrain => Some(6), // Super Warrior
        _ => None,
    }
}

pub fn conversion_time(target_subtype: u8) -> u16 {
    match target_subtype {
        3 => CONV_TIME_WARRIOR,
        4 => CONV_TIME_PREACHER,
        5 => CONV_TIME_SPY,
        6 => CONV_TIME_SUPER_WARRIOR,
        _ => 256,
    }
}

pub fn training_mana_cost(target_subtype: u8) -> u32 {
    match target_subtype {
        3 => MANA_COST_WARRIOR,
        4 => MANA_COST_PREACHER,
        5 => MANA_COST_SPY,
        6 => MANA_COST_SUPER_WARRIOR,
        _ => 0,
    }
}

#[derive(Debug, PartialEq)]
pub enum ConvertAction {
    None,
    ConvertUnit { handle: ObjectHandle, new_subtype: u8 },
}

/// Tick training building. Returns action when conversion completes.
pub fn tick_convert(building: &mut BuildingData) -> ConvertAction {
    if building.behavior_flags & 0x01 == 0 {
        return ConvertAction::None;
    }
    if building.state != BuildingState::Active {
        return ConvertAction::None;
    }
    if building.conversion_countdown == 0 {
        return ConvertAction::None;
    }

    building.conversion_countdown -= 1;
    if building.conversion_countdown == 0 {
        // Find the occupant being trained (first occupied slot)
        if let Some(handle) = building.occupant_slots.iter().filter_map(|s| *s).next() {
            let target = training_output_subtype(building.building_subtype).unwrap_or(2);
            return ConvertAction::ConvertUnit {
                handle,
                new_subtype: target,
            };
        }
    }
    ConvertAction::None
}

/// Start training: set countdown, returns true if training started.
/// Caller must check mana and wood costs before calling.
pub fn start_training(building: &mut BuildingData) -> bool {
    if let Some(target) = training_output_subtype(building.building_subtype) {
        building.conversion_countdown = conversion_time(target);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_active_trainer(subtype: BuildingSubtype) -> BuildingData {
        let mut b = BuildingData::default();
        b.state = BuildingState::Active;
        b.building_subtype = subtype;
        b.behavior_flags = 0x01; // training flag
        b.occupant_slots[0] = Some(42); // an occupant to train
        b.occupant_count = 1;
        b
    }

    #[test]
    fn conversion_time_warrior() {
        assert_eq!(conversion_time(3), 256);
    }

    #[test]
    fn conversion_time_spy() {
        assert_eq!(conversion_time(5), 192);
    }

    #[test]
    fn conversion_time_preacher() {
        assert_eq!(conversion_time(4), 192);
    }

    #[test]
    fn conversion_time_super_warrior() {
        assert_eq!(conversion_time(6), 384);
    }

    #[test]
    fn mana_cost_warrior() {
        assert_eq!(training_mana_cost(3), 500);
    }

    #[test]
    fn mana_cost_spy() {
        assert_eq!(training_mana_cost(5), 300);
    }

    #[test]
    fn mana_cost_preacher() {
        assert_eq!(training_mana_cost(4), 400);
    }

    #[test]
    fn mana_cost_super_warrior() {
        assert_eq!(training_mana_cost(6), 800);
    }

    #[test]
    fn training_output_warrior_train() {
        assert_eq!(training_output_subtype(BuildingSubtype::WarriorTrain), Some(3));
    }

    #[test]
    fn training_output_spy_train() {
        assert_eq!(training_output_subtype(BuildingSubtype::SpyTrain), Some(5));
    }

    #[test]
    fn training_output_temple() {
        assert_eq!(training_output_subtype(BuildingSubtype::Temple), Some(4));
    }

    #[test]
    fn training_output_super_warrior_train() {
        assert_eq!(training_output_subtype(BuildingSubtype::SuperWarriorTrain), Some(6));
    }

    #[test]
    fn training_output_non_trainer_returns_none() {
        assert_eq!(training_output_subtype(BuildingSubtype::SmallHut), None);
        assert_eq!(training_output_subtype(BuildingSubtype::DrumTower), None);
    }

    #[test]
    fn tick_convert_countdown_decrements() {
        let mut b = make_active_trainer(BuildingSubtype::WarriorTrain);
        b.conversion_countdown = 10;
        let action = tick_convert(&mut b);
        assert_eq!(action, ConvertAction::None);
        assert_eq!(b.conversion_countdown, 9);
    }

    #[test]
    fn tick_convert_emits_at_zero() {
        let mut b = make_active_trainer(BuildingSubtype::WarriorTrain);
        b.conversion_countdown = 1; // will hit zero this tick
        let action = tick_convert(&mut b);
        assert_eq!(
            action,
            ConvertAction::ConvertUnit {
                handle: 42,
                new_subtype: 3
            }
        );
    }

    #[test]
    fn tick_convert_spy_subtype() {
        let mut b = make_active_trainer(BuildingSubtype::SpyTrain);
        b.conversion_countdown = 1;
        let action = tick_convert(&mut b);
        assert_eq!(
            action,
            ConvertAction::ConvertUnit {
                handle: 42,
                new_subtype: 5
            }
        );
    }

    #[test]
    fn tick_convert_skips_without_training_flag() {
        let mut b = make_active_trainer(BuildingSubtype::WarriorTrain);
        b.behavior_flags = 0;
        b.conversion_countdown = 1;
        let action = tick_convert(&mut b);
        assert_eq!(action, ConvertAction::None);
    }

    #[test]
    fn tick_convert_skips_non_active() {
        let mut b = make_active_trainer(BuildingSubtype::WarriorTrain);
        b.state = BuildingState::Init;
        b.conversion_countdown = 1;
        let action = tick_convert(&mut b);
        assert_eq!(action, ConvertAction::None);
    }

    #[test]
    fn tick_convert_skips_zero_countdown() {
        let mut b = make_active_trainer(BuildingSubtype::WarriorTrain);
        b.conversion_countdown = 0;
        let action = tick_convert(&mut b);
        assert_eq!(action, ConvertAction::None);
    }

    #[test]
    fn start_training_sets_countdown() {
        let mut b = make_active_trainer(BuildingSubtype::WarriorTrain);
        assert!(start_training(&mut b));
        assert_eq!(b.conversion_countdown, 256); // CONV_TIME_WARRIOR
    }

    #[test]
    fn start_training_spy() {
        let mut b = make_active_trainer(BuildingSubtype::SpyTrain);
        assert!(start_training(&mut b));
        assert_eq!(b.conversion_countdown, 192); // CONV_TIME_SPY
    }

    #[test]
    fn start_training_fails_non_trainer() {
        let mut b = make_active_trainer(BuildingSubtype::SmallHut);
        assert!(!start_training(&mut b));
    }
}
