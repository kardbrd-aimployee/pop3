use core::mem::size_of;
use std::io::Read;

use crate::data::types::{from_reader, BinDeserializer};

/******************************************************************************/

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ModelType {
    Person = 1,
    Building = 2,
    Creature = 3,
    Vehicle = 4,
    Scenery = 5,
    General = 6,
    Effect = 7,
    Shot = 8,
    Shape = 9,
    Internal = 10,
    Spell = 11,
}

impl ModelType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Person),
            2 => Some(Self::Building),
            3 => Some(Self::Creature),
            4 => Some(Self::Vehicle),
            5 => Some(Self::Scenery),
            6 => Some(Self::General),
            7 => Some(Self::Effect),
            8 => Some(Self::Shot),
            9 => Some(Self::Shape),
            10 => Some(Self::Internal),
            11 => Some(Self::Spell),
            _ => None,
        }
    }

    pub fn is_visible(&self) -> bool {
        matches!(
            self,
            Self::Person
                | Self::Building
                | Self::Creature
                | Self::Vehicle
                | Self::Scenery
                | Self::General
                | Self::Shape
        )
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct UnitRaw {
    pub subtype: u8, // byte 0: subtype within model (e.g. Brave=2, Shaman=7)
    pub model: u8,   // byte 1: model type (1=Person, 2=Building, 3=Creature, ...)
    tribe_index: u8, // byte 2: owner tribe (0-3, or 255=unowned)
    loc_x: u16,      // bytes 3-4: world X position
    loc_y: u16,      // bytes 5-6: world Y position
    angle: u32,      // bytes 7-10: rotation angle (game uses angle/512 for buildings)
    f2: u16,
    f3: u16,
    fd: [u8; 40],
}

impl UnitRaw {
    pub fn tribe_index(&self) -> u8 {
        self.tribe_index
    }
    pub fn loc_x(&self) -> u16 {
        self.loc_x
    }
    pub fn loc_y(&self) -> u16 {
        self.loc_y
    }
    pub fn model_type(&self) -> Option<ModelType> {
        ModelType::from_u8(self.model)
    }
    pub fn angle(&self) -> u32 {
        self.angle
    }
    pub fn f2(&self) -> u16 {
        self.f2
    }
    pub fn f3(&self) -> u16 {
        self.f3
    }
    pub fn fd(&self) -> &[u8; 40] {
        &self.fd
    }
}

impl BinDeserializer for UnitRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<UnitRaw, { size_of::<UnitRaw>() }, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct TribeConfigRaw {
    pub data: [u8; 16],
}

impl BinDeserializer for TribeConfigRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<TribeConfigRaw, { size_of::<TribeConfigRaw>() }, R>(reader)
    }
}

/******************************************************************************/

/// Returns the OBJS object index for a completed building.
/// tribe_index: 0-3 (Blue, Red, Yellow, Green)
///
/// Huts use 3 consecutive indices per tribe (Small/Medium/Large).
/// Other buildings use 1 index per tribe in blocks of 4.
pub fn building_obj_index(subtype: u8, tribe_index: u8) -> Option<usize> {
    let tribe = tribe_index.min(3) as usize;
    match subtype {
        1 => Some(145 + tribe * 3), // Small Hut
        2 => Some(146 + tribe * 3), // Medium Hut
        3 => Some(147 + tribe * 3), // Large Hut
        4 => Some(117 + tribe),     // Guard Tower (DrumTower)
        5 => Some(133 + tribe),     // Temple (Preacher Training)
        6 => Some(129 + tribe),     // Spy Training
        7 => Some(141 + tribe),     // Warrior Training
        8 => Some(137 + tribe),     // Firewarrior Training
        13 => Some(121 + tribe),    // Boat Hut
        15 => Some(125 + tribe),    // Balloon Hut (Airship)
        18 => Some(190),            // Vault of Knowledge
        _ => None,
    }
}

/// Returns the OBJS object index for a scenery object.
/// Derived from the scenery data table at 0x5a0790 (field +0x0a per 0x18-byte entry)
/// and Object_InitShapeData @ 0x4bd5b0 which applies subtype-specific overrides.
pub fn scenery_obj_index(subtype: u8) -> Option<usize> {
    match subtype {
        1 => Some(13),  // Mass Tree
        2 => Some(14),  // Special Tree 1
        3 => Some(15),  // Special Tree 2
        4 => Some(16),  // Mass Fruit Tree
        5 => Some(17),  // Special Fruit Tree 1
        6 => Some(18),  // Special Fruit Tree 2
        7 => Some(2),   // Tree variant 7
        8 => Some(3),   // Tree variant 8
        9 => Some(45),  // Stone Head (worship site)
        10 => Some(5),  // Obelisk
        11 => Some(23), // Totem Pole
        12 => Some(30), // Discovery Pillar (Reincarnation Site)
        14 => Some(26), // Additional Tree (Object_InitShapeData forces 0x1a)
        15 => Some(12), // Bridge/Island
        16 => Some(18), // Portal/Trigger scenery
        18 => Some(44), // Vegetation
        19 => Some(39), // Sub-level Scenery
        _ => None,      // 0, 13 (position-variant), 17 (no model)
    }
}

/// Returns the OBJS object index for any model type + subtype combination.
pub fn object_3d_index(model_type: &ModelType, subtype: u8, tribe_index: u8) -> Option<usize> {
    match model_type {
        ModelType::Building => building_obj_index(subtype, tribe_index),
        ModelType::Scenery => scenery_obj_index(subtype),
        _ => None,
    }
}

/******************************************************************************/
