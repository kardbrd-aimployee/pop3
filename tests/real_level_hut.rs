use std::path::PathBuf;

use pop3::data::level::{LevelDefinition, LevelRes, ObjectPaths};
use pop3::data::objects::{Object3D, ShapeFootprints};
use pop3::data::psfb::ContainerPSFB;
use pop3::data::types::BinDeserializer;
use pop3::data::units::ModelType;
use pop3::engine::buildings::{BuildingCatalog, BuildingSubtype};
use pop3::engine::objects::CellGrid;
use pop3::engine::{GameAction, GameSession};
use pop3::render::hud::{
    HFX_CONSTRUCTION_BLOCKED_OVERLAY, HFX_CONSTRUCTION_ICONS, HFX_CONSTRUCTION_ICONS_HOVER,
    HFX_HUD_SPRITE_IDS, HSPR_HUD_SPRITE_IDS,
};

fn assert_native_construction_hud_assets(base: &std::path::Path) {
    let data = base.join("data");
    let hfx = ContainerPSFB::from_file(&data.join("hfx0-0.dat"))
        .expect("original HFX HUD bank must decode");
    let hspr = ContainerPSFB::from_file(&data.join("HSPR0-0.DAT"))
        .expect("original HSPR status bank must decode");

    for &sprite_id in HFX_HUD_SPRITE_IDS {
        let image = hfx
            .get_image(sprite_id as usize)
            .unwrap_or_else(|| panic!("original HFX sprite {sprite_id} must decode"));
        assert!(
            image.width > 0 && image.height > 0,
            "original HFX sprite {sprite_id} must have an image extent"
        );
    }
    for &sprite_id in &HFX_CONSTRUCTION_ICONS {
        let image = hfx
            .get_image(sprite_id as usize)
            .unwrap_or_else(|| panic!("original HFX construction icon {sprite_id} must decode"));
        assert!(
            image.width > 0 && image.height > 0,
            "original HFX construction icon {sprite_id} must have an image extent"
        );
    }
    for &sprite_id in &HFX_CONSTRUCTION_ICONS_HOVER {
        let image = hfx.get_image(sprite_id as usize).unwrap_or_else(|| {
            panic!("original HFX highlighted construction icon {sprite_id} must decode")
        });
        assert!(
            image.width > 0 && image.height > 0,
            "original HFX highlighted construction icon {sprite_id} must have an image extent"
        );
    }
    let blocked_overlay = hfx
        .get_image(HFX_CONSTRUCTION_BLOCKED_OVERLAY as usize)
        .expect("original HFX blocked construction overlay must decode");
    assert!(
        blocked_overlay.width > 0 && blocked_overlay.height > 0,
        "original HFX blocked construction overlay must have an image extent"
    );
    for &sprite_id in &HSPR_HUD_SPRITE_IDS {
        assert!(
            hspr.get_image(sprite_id as usize).is_some(),
            "original HSPR status sprite {sprite_id} must decode"
        );
    }
}

#[test]
#[ignore = "requires legally owned POP3_DATA_DIR assets"]
fn real_level_one_hut_vertical_slice() {
    let base = PathBuf::from(
        std::env::var("POP3_DATA_DIR").expect("set POP3_DATA_DIR to the Populous 3 data root"),
    );
    assert_native_construction_hud_assets(&base);
    let level = LevelRes::new(&base, 1, None);
    let obj_bank = level.obj_bank;
    let expected = level
        .units
        .iter()
        .filter(|raw| raw.model_type().is_some() && raw.loc_x() != 0 && raw.loc_y() != 0)
        .count();
    let expected_people = level
        .units
        .iter()
        .filter(|raw| {
            raw.model_type() == Some(ModelType::Person) && raw.loc_x() != 0 && raw.loc_y() != 0
        })
        .count();

    let (building_objects, _) = Object3D::load_dual_banks(&base, obj_bank);
    let bank = if obj_bank == 0 { 2 } else { obj_bank };
    let paths = ObjectPaths::from_default_dir(&base, &bank.to_string());
    let footprints = ShapeFootprints::from_file(&paths.shapes);
    let catalog = BuildingCatalog::from_assets(&building_objects, &footprints);
    let mut session = GameSession::from_level(LevelDefinition::from(level), catalog)
        .expect("Level 1 must instantiate");
    assert_eq!(session.world.pool().active_count() as usize, expected);
    let original_people: std::collections::HashSet<_> = session
        .world
        .pool()
        .persons()
        .map(|(handle, _, _)| handle)
        .collect();
    let builder = session
        .world
        .pool()
        .persons()
        .find(|(_, header, person)| header.tribe == 0 && header.subtype == 2 && person.alive)
        .map(|(handle, _, _)| handle)
        .expect("Level 1 must contain a blue brave builder");

    let cell = (0..128)
        .flat_map(|y| (0..128).map(move |x| (x, y)))
        .find(|&cell| {
            session
                .validate_building_placement(BuildingSubtype::SmallHut, cell, 0)
                .is_ok()
        })
        .expect("Level 1 must contain a valid hut site");
    session.enqueue(GameAction::PlaceBuilding {
        subtype: BuildingSubtype::SmallHut,
        owner: 0,
        cell,
        rotation: 0,
    });
    assert!(session.step().actions[0].clone().is_applied());
    let placed_hut = session
        .world
        .terrain
        .occupant(cell.0, cell.1)
        .expect("placed hut must occupy its footprint");
    let hut_position = session.world.get(placed_hut).unwrap().header.position;
    let expected_spawn =
        pop3::engine::movement::WorldCoord::new(hut_position.x.wrapping_add(512), hut_position.z);
    session.enqueue(GameAction::AssignConstruction {
        units: vec![builder],
        building: placed_hut,
    });
    assert!(session.step().actions[0].clone().is_applied());
    for _ in 0..30_000 {
        session.step();
        if session.world.pool().persons().count() >= expected_people + 1 {
            break;
        }
    }

    assert!(session.world.pool().persons().count() >= expected_people + 1);
    let snapshot = session.snapshot();
    assert_eq!(
        snapshot.persons.len(),
        session.world.pool().persons().count()
    );
    let new_people: Vec<_> = snapshot
        .persons
        .iter()
        .filter(|record| !original_people.contains(&record.handle))
        .collect();
    let spawned_here: Vec<_> = new_people
        .into_iter()
        .filter(|record| {
            session
                .world
                .get(record.handle)
                .is_some_and(|object| object.header.position == expected_spawn)
        })
        .collect();
    assert_eq!(spawned_here.len(), 1);
    let brave = spawned_here
        .into_iter()
        .find(|record| record.subtype == 2 && record.tribe == 0)
        .expect("spawned brave must be rendered")
        .handle;
    let position = session.world.get(brave).unwrap().header.position;
    assert_eq!(
        session
            .world
            .cell_head(CellGrid::cell_index_from_world(&position)),
        Some(brave)
    );
    assert_eq!(
        snapshot.tribes[0].population as usize,
        session
            .world
            .pool()
            .persons()
            .filter(|(_, h, p)| h.tribe == 0 && p.alive)
            .count()
    );
}

trait Applied {
    fn is_applied(&self) -> bool;
}

impl Applied for pop3::engine::session::ActionEvent {
    fn is_applied(&self) -> bool {
        matches!(self, Self::Applied(_))
    }
}
