use std::collections::HashMap;

use crate::data::objects::{Object3D, ShapeFootprints};
use crate::data::units::building_obj_index;

use super::BuildingSubtype;

#[derive(Debug, Clone, Default)]
pub struct BuildingCatalog {
    footprints: HashMap<(u8, u8), Vec<(i16, i16)>>,
}

impl BuildingCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, subtype: BuildingSubtype, rotation: u8, cells: Vec<(i16, i16)>) {
        self.footprints.insert((subtype as u8, rotation & 3), cells);
    }

    pub fn footprint(&self, subtype: BuildingSubtype, rotation: u8) -> Option<&[(i16, i16)]> {
        self.footprints
            .get(&(subtype as u8, rotation & 3))
            .map(Vec::as_slice)
    }

    pub fn from_assets(objects: &[Option<Object3D>], footprints: &ShapeFootprints) -> Self {
        let mut catalog = Self::new();
        for raw_subtype in 1..=18 {
            let Ok(subtype) = BuildingSubtype::try_from(raw_subtype) else {
                continue;
            };
            let Some(object_index) = building_obj_index(raw_subtype, 0) else {
                continue;
            };
            let Some(Some(object)) = objects.get(object_index) else {
                continue;
            };
            for rotation in 0..4u8 {
                let shape_index = object.footprint_index(rotation as usize);
                if shape_index < 0 {
                    continue;
                }
                if let Some(cells) = footprints.occupied_offsets(shape_index as usize) {
                    if !cells.is_empty() {
                        catalog.insert(subtype, rotation, cells);
                    }
                }
            }
        }
        catalog
    }
}
