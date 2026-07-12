use crate::engine::objects::handle::ObjectHandle;

/// Maximum number of occupants in any building.
pub const MAX_OCCUPANTS: usize = 6;

/// Wood consumed per construction tick.
pub const CONSTRUCTION_WOOD_PER_TICK: u16 = 1;

/// Building state machine values from the original binary (offset +0x2C).
/// See BLD.4 in docs/specs/buildings.md.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BuildingState {
    Init = 0x01,
    ConstructionDone = 0x02,
    Active = 0x03,
    Destroying = 0x04,
    Sinking = 0x05,
    FinalTeardown = 0x06,
}

/// Building subtypes from BLD.1 in docs/specs/buildings.md.
/// Values match the original binary's building_type field at offset +0x2B.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BuildingSubtype {
    SmallHut = 1,
    MediumHut = 2,
    LargeHut = 3,
    DrumTower = 4,
    Temple = 5,
    SpyTrain = 6,
    WarriorTrain = 7,
    SuperWarriorTrain = 8,
    Reconversion = 9,
    WallPiece = 10,
    Gateway = 11,
    BoatHut = 13,
    AirshipHut = 14,
    GuardPost = 15,
    Prison = 16,
    Vault = 17,
    HeadQuarters = 18,
}

impl TryFrom<u8> for BuildingSubtype {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            1 => Self::SmallHut,
            2 => Self::MediumHut,
            3 => Self::LargeHut,
            4 => Self::DrumTower,
            5 => Self::Temple,
            6 => Self::SpyTrain,
            7 => Self::WarriorTrain,
            8 => Self::SuperWarriorTrain,
            9 => Self::Reconversion,
            10 => Self::WallPiece,
            11 => Self::Gateway,
            13 => Self::BoatHut,
            14 => Self::AirshipHut,
            15 => Self::GuardPost,
            16 => Self::Prison,
            17 => Self::Vault,
            18 => Self::HeadQuarters,
            other => return Err(other),
        })
    }
}

/// Building-specific data, stored in GameObjectData::Building.
/// Field layout matches the original struct at BLD.2.
#[derive(Debug, Clone)]
pub struct BuildingData {
    pub state: BuildingState,
    pub building_subtype: BuildingSubtype,
    pub wood_stored: u16,
    pub occupant_slots: [Option<ObjectHandle>; MAX_OCCUPANTS],
    pub occupant_count: u8,
    pub construction_progress: u16,
    pub conversion_countdown: u16,
    pub training_countdown: u16,
    pub damage_accumulated: u16,
    pub damage_cooldown: u8,
    pub behavior_flags: u32,
    pub shake_x: i16,
    pub shake_z: i16,
    pub num_fighting: u8,
    pub target_person: Option<ObjectHandle>,
    /// Imported level huts are active housing but do not begin a synchronized
    /// population cycle until broader economy initialization is implemented.
    pub population_timer_enabled: bool,
}

impl Default for BuildingData {
    fn default() -> Self {
        Self {
            state: BuildingState::Init,
            building_subtype: BuildingSubtype::SmallHut,
            wood_stored: 0,
            occupant_slots: [None; MAX_OCCUPANTS],
            occupant_count: 0,
            construction_progress: 0,
            conversion_countdown: 0,
            training_countdown: 0,
            damage_accumulated: 0,
            damage_cooldown: 0,
            behavior_flags: 0,
            shake_x: 0,
            shake_z: 0,
            num_fighting: 0,
            target_person: None,
            population_timer_enabled: true,
        }
    }
}

/// Returns behavior flags for a building subtype.
/// Flags from BLD.3/BLD.5: 0x20 = spawns braves (housing), 0x01 = trains units.
pub fn building_behavior_flags(subtype: BuildingSubtype) -> u32 {
    match subtype {
        BuildingSubtype::SmallHut | BuildingSubtype::MediumHut | BuildingSubtype::LargeHut => 0x20, // Housing: spawns braves
        BuildingSubtype::SpyTrain
        | BuildingSubtype::WarriorTrain
        | BuildingSubtype::SuperWarriorTrain
        | BuildingSubtype::Reconversion => 0x01, // Training: converts units
        BuildingSubtype::BoatHut | BuildingSubtype::AirshipHut => 0x40, // Vehicle factory
        BuildingSubtype::Temple => 0x0400, // Has conversion timer (spell recharge)
        BuildingSubtype::DrumTower | BuildingSubtype::GuardPost => 0x08, // Has occupant fighting
        _ => 0,
    }
}

/// Returns max health for a building subtype.
/// Values approximate the original binary's type properties table.
pub fn building_max_health(subtype: BuildingSubtype) -> u16 {
    match subtype {
        BuildingSubtype::SmallHut => 600,
        BuildingSubtype::MediumHut => 800,
        BuildingSubtype::LargeHut => 1000,
        BuildingSubtype::DrumTower => 1200,
        BuildingSubtype::Temple => 1500,
        BuildingSubtype::SpyTrain => 800,
        BuildingSubtype::WarriorTrain => 800,
        BuildingSubtype::SuperWarriorTrain => 800,
        BuildingSubtype::Reconversion => 800,
        BuildingSubtype::WallPiece => 400,
        BuildingSubtype::Gateway => 600,
        BuildingSubtype::BoatHut => 800,
        BuildingSubtype::AirshipHut => 800,
        BuildingSubtype::GuardPost => 1000,
        BuildingSubtype::Prison => 600,
        BuildingSubtype::Vault => 800,
        BuildingSubtype::HeadQuarters => 2000,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_state_repr_values() {
        assert_eq!(BuildingState::Init as u8, 0x01);
        assert_eq!(BuildingState::ConstructionDone as u8, 0x02);
        assert_eq!(BuildingState::Active as u8, 0x03);
        assert_eq!(BuildingState::Destroying as u8, 0x04);
        assert_eq!(BuildingState::Sinking as u8, 0x05);
        assert_eq!(BuildingState::FinalTeardown as u8, 0x06);
    }

    #[test]
    fn building_subtype_repr_values() {
        assert_eq!(BuildingSubtype::SmallHut as u8, 1);
        assert_eq!(BuildingSubtype::MediumHut as u8, 2);
        assert_eq!(BuildingSubtype::LargeHut as u8, 3);
        assert_eq!(BuildingSubtype::DrumTower as u8, 4);
        assert_eq!(BuildingSubtype::Temple as u8, 5);
        assert_eq!(BuildingSubtype::SpyTrain as u8, 6);
        assert_eq!(BuildingSubtype::WarriorTrain as u8, 7);
        assert_eq!(BuildingSubtype::SuperWarriorTrain as u8, 8);
        assert_eq!(BuildingSubtype::Reconversion as u8, 9);
        assert_eq!(BuildingSubtype::WallPiece as u8, 10);
        assert_eq!(BuildingSubtype::Gateway as u8, 11);
        assert_eq!(BuildingSubtype::BoatHut as u8, 13);
        assert_eq!(BuildingSubtype::AirshipHut as u8, 14);
        assert_eq!(BuildingSubtype::GuardPost as u8, 15);
        assert_eq!(BuildingSubtype::Prison as u8, 16);
        assert_eq!(BuildingSubtype::Vault as u8, 17);
        assert_eq!(BuildingSubtype::HeadQuarters as u8, 18);
    }

    #[test]
    fn building_data_default() {
        let bd = BuildingData::default();
        assert_eq!(bd.state, BuildingState::Init);
        assert_eq!(bd.building_subtype, BuildingSubtype::SmallHut);
        assert_eq!(bd.wood_stored, 0);
        assert_eq!(bd.occupant_count, 0);
        assert_eq!(bd.construction_progress, 0);
        assert_eq!(bd.damage_accumulated, 0);
        assert_eq!(bd.damage_cooldown, 0);
        assert_eq!(bd.behavior_flags, 0);
        assert_eq!(bd.shake_x, 0);
        assert_eq!(bd.shake_z, 0);
        assert_eq!(bd.num_fighting, 0);
        assert!(bd.target_person.is_none());
        for slot in &bd.occupant_slots {
            assert!(slot.is_none());
        }
    }

    #[test]
    fn max_occupants_is_six() {
        assert_eq!(MAX_OCCUPANTS, 6);
    }

    #[test]
    fn behavior_flags_housing() {
        assert_eq!(building_behavior_flags(BuildingSubtype::SmallHut), 0x20);
        assert_eq!(building_behavior_flags(BuildingSubtype::MediumHut), 0x20);
        assert_eq!(building_behavior_flags(BuildingSubtype::LargeHut), 0x20);
    }

    #[test]
    fn behavior_flags_training() {
        assert_eq!(building_behavior_flags(BuildingSubtype::SpyTrain), 0x01);
        assert_eq!(building_behavior_flags(BuildingSubtype::WarriorTrain), 0x01);
        assert_eq!(
            building_behavior_flags(BuildingSubtype::SuperWarriorTrain),
            0x01
        );
        assert_eq!(building_behavior_flags(BuildingSubtype::Reconversion), 0x01);
    }

    #[test]
    fn behavior_flags_vehicle() {
        assert_eq!(building_behavior_flags(BuildingSubtype::BoatHut), 0x40);
        assert_eq!(building_behavior_flags(BuildingSubtype::AirshipHut), 0x40);
    }

    #[test]
    fn max_health_values() {
        assert_eq!(building_max_health(BuildingSubtype::SmallHut), 600);
        assert_eq!(building_max_health(BuildingSubtype::MediumHut), 800);
        assert_eq!(building_max_health(BuildingSubtype::LargeHut), 1000);
        assert_eq!(building_max_health(BuildingSubtype::DrumTower), 1200);
        assert_eq!(building_max_health(BuildingSubtype::Temple), 1500);
        assert_eq!(building_max_health(BuildingSubtype::HeadQuarters), 2000);
    }
}
