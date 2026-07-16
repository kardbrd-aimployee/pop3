use std::error::Error;
use std::fs;
use std::path::PathBuf;

use serde::Serialize;

use crate::data::psfb::ContainerPSFB;
use crate::data::types::BinDeserializer;

use super::hud::{
    ensure_file, fit_icon, indexed_to_rgba, invalid_data, invalid_input, load_palette,
    relative_source,
};
use super::structures::make_contact_sheet;

const SCHEMA_VERSION: u32 = 1;
const HFX_BANK: &str = "hfx0-0.dat";
/// The native `?` overlay composed by the original blocked build-menu state.
const HFX_BLOCKED_OVERLAY: usize = 1055;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildingPanelIconSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub short_label: &'static str,
    pub building_subtype: u8,
    pub sprite_index: usize,
    pub highlight_sprite_index: usize,
}

/// Original house-tab element parameters from `0x576c20`.
///
/// The construction elements pass these HFX image numbers to
/// `FUN_004018a0`; they are not the smaller `POINT0-0.DAT` silhouettes.
/// Keep this order in sync with `render::hud::layout::CONSTRUCTION_PAGE`.
pub const BUILDING_PANEL_ICON_SPECS: [BuildingPanelIconSpec; 9] = [
    BuildingPanelIconSpec {
        id: "small-hut",
        name: "Small Hut",
        short_label: "Hut",
        building_subtype: 1,
        sprite_index: 1028,
        highlight_sprite_index: 1046,
    },
    BuildingPanelIconSpec {
        id: "drum-tower",
        name: "Drum Tower",
        short_label: "Drum Tower",
        building_subtype: 4,
        sprite_index: 1029,
        highlight_sprite_index: 1047,
    },
    BuildingPanelIconSpec {
        id: "warrior-training-hut",
        name: "Warrior Training Hut",
        short_label: "Warrior Hut",
        building_subtype: 7,
        sprite_index: 1030,
        highlight_sprite_index: 1048,
    },
    BuildingPanelIconSpec {
        id: "temple",
        name: "Temple",
        short_label: "Temple",
        building_subtype: 5,
        sprite_index: 1032,
        highlight_sprite_index: 1050,
    },
    BuildingPanelIconSpec {
        id: "spy-training-hut",
        name: "Spy Training Hut",
        short_label: "Spy Hut",
        building_subtype: 6,
        sprite_index: 1033,
        highlight_sprite_index: 1051,
    },
    BuildingPanelIconSpec {
        id: "firewarrior-training-hut",
        name: "Firewarrior Training Hut",
        short_label: "Firewarrior",
        building_subtype: 8,
        sprite_index: 1031,
        highlight_sprite_index: 1049,
    },
    BuildingPanelIconSpec {
        id: "boat-hut",
        name: "Boat Hut",
        short_label: "Boat Hut",
        building_subtype: 13,
        sprite_index: 1034,
        highlight_sprite_index: 1052,
    },
    BuildingPanelIconSpec {
        id: "guard-post",
        name: "Guard Post",
        short_label: "Guard Post",
        building_subtype: 15,
        sprite_index: 1035,
        highlight_sprite_index: 1053,
    },
    BuildingPanelIconSpec {
        id: "vault",
        name: "Vault",
        short_label: "Vault",
        building_subtype: 17,
        sprite_index: 1036,
        highlight_sprite_index: 1054,
    },
];

#[derive(Debug)]
pub struct BuildingPanelIconRequest {
    pub base: PathBuf,
    pub output: PathBuf,
    pub landscape: String,
    pub contact_sheet_size: u32,
}

#[derive(Debug)]
pub struct BuildingPanelIconExport {
    pub manifest_path: PathBuf,
    pub contact_sheet_path: PathBuf,
    pub icon_count: usize,
}

#[derive(Serialize)]
struct Manifest {
    schema_version: u32,
    kind: &'static str,
    source: SourceManifest,
    items: Vec<IconManifest>,
    blocked_overlay: OverlayManifest,
}

#[derive(Serialize)]
struct SourceManifest {
    sprite_bank: String,
    palette: String,
}

#[derive(Serialize)]
struct IconManifest {
    id: &'static str,
    name: &'static str,
    building_subtype: u8,
    sprite_index: usize,
    highlight_sprite_index: usize,
    width: u16,
    height: u16,
    icon: String,
    highlight_icon: String,
}

#[derive(Serialize)]
struct OverlayManifest {
    sprite_index: usize,
    width: u16,
    height: u16,
    icon: String,
}

pub fn export_building_panel_icons(
    request: &BuildingPanelIconRequest,
) -> Result<BuildingPanelIconExport, Box<dyn Error>> {
    validate_request(request)?;

    let data_dir = request.base.join("data");
    let sprite_bank_path = data_dir.join(HFX_BANK);
    let palette_path = data_dir.join(format!("pal0-{}.dat", request.landscape));
    ensure_file(&sprite_bank_path)?;
    ensure_file(&palette_path)?;

    let sprite_bank = ContainerPSFB::from_file(&sprite_bank_path).ok_or_else(|| {
        invalid_data(format!(
            "could not parse PSFB bank: {}",
            sprite_bank_path.display()
        ))
    })?;
    let palette = load_palette(&palette_path)?;
    let icons_dir = request.output.join("icons");
    fs::create_dir_all(&icons_dir)?;

    let mut rendered = Vec::with_capacity(BUILDING_PANEL_ICON_SPECS.len());
    let mut manifest_items = Vec::with_capacity(BUILDING_PANEL_ICON_SPECS.len());

    for spec in BUILDING_PANEL_ICON_SPECS {
        let source = sprite_bank.get_image(spec.sprite_index).ok_or_else(|| {
            invalid_data(format!(
                "{HFX_BANK} has no sprite {} ({})",
                spec.sprite_index, spec.name
            ))
        })?;
        let highlighted_source = sprite_bank
            .get_image(spec.highlight_sprite_index)
            .ok_or_else(|| {
                invalid_data(format!(
                    "{HFX_BANK} has no highlighted sprite {} ({})",
                    spec.highlight_sprite_index, spec.name
                ))
            })?;
        let icon = indexed_to_rgba(&source, &palette);
        let highlighted_icon = indexed_to_rgba(&highlighted_source, &palette);
        let file_name = format!("{}.png", spec.id);
        let highlighted_file_name = format!("{}-highlighted.png", spec.id);
        icon.save(icons_dir.join(&file_name))?;
        highlighted_icon.save(icons_dir.join(&highlighted_file_name))?;
        rendered.push((
            format!("{} (#{})", spec.short_label, spec.sprite_index),
            fit_icon(&icon, request.contact_sheet_size),
        ));
        rendered.push((
            format!(
                "{} highlighted (#{})",
                spec.short_label, spec.highlight_sprite_index
            ),
            fit_icon(&highlighted_icon, request.contact_sheet_size),
        ));
        manifest_items.push(IconManifest {
            id: spec.id,
            name: spec.name,
            building_subtype: spec.building_subtype,
            sprite_index: spec.sprite_index,
            highlight_sprite_index: spec.highlight_sprite_index,
            width: source.width as u16,
            height: source.height as u16,
            icon: format!("icons/{file_name}"),
            highlight_icon: format!("icons/{highlighted_file_name}"),
        });
    }

    // `FUN_004018a0` composites this native glyph over the corresponding
    // building icon for construction state 4. It is not a tenth building.
    let blocked_overlay_source = sprite_bank.get_image(HFX_BLOCKED_OVERLAY).ok_or_else(|| {
        invalid_data(format!(
            "{HFX_BANK} has no blocked construction overlay {HFX_BLOCKED_OVERLAY}"
        ))
    })?;
    let blocked_overlay = indexed_to_rgba(&blocked_overlay_source, &palette);
    let blocked_overlay_file_name = "blocked-overlay.png";
    blocked_overlay.save(icons_dir.join(blocked_overlay_file_name))?;
    rendered.push((
        format!("Blocked overlay (#{HFX_BLOCKED_OVERLAY})"),
        fit_icon(&blocked_overlay, request.contact_sheet_size),
    ));

    let contact_sheet_path = request.output.join("contact-sheet.png");
    make_contact_sheet(&rendered, request.contact_sheet_size).save(&contact_sheet_path)?;

    let manifest = Manifest {
        schema_version: SCHEMA_VERSION,
        kind: "building-panel-icons",
        source: SourceManifest {
            sprite_bank: relative_source(&request.base, &sprite_bank_path),
            palette: relative_source(&request.base, &palette_path),
        },
        items: manifest_items,
        blocked_overlay: OverlayManifest {
            sprite_index: HFX_BLOCKED_OVERLAY,
            width: blocked_overlay_source.width as u16,
            height: blocked_overlay_source.height as u16,
            icon: format!("icons/{blocked_overlay_file_name}"),
        },
    };
    let manifest_path = request.output.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    Ok(BuildingPanelIconExport {
        manifest_path,
        contact_sheet_path,
        icon_count: BUILDING_PANEL_ICON_SPECS.len(),
    })
}

fn validate_request(request: &BuildingPanelIconRequest) -> Result<(), std::io::Error> {
    if !(48..=512).contains(&request.contact_sheet_size) {
        return Err(invalid_input(
            "contact-sheet size must be between 48 and 512 pixels",
        ));
    }
    if request.landscape.len() != 1
        || !request
            .landscape
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric())
    {
        return Err(invalid_input("landscape must be one alphanumeric bank key"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_build_menu_uses_the_hfx_element_parameters() {
        let sprite_indices: Vec<_> = BUILDING_PANEL_ICON_SPECS
            .iter()
            .map(|spec| spec.sprite_index)
            .collect();
        assert_eq!(
            sprite_indices,
            [1028, 1029, 1030, 1032, 1033, 1031, 1034, 1035, 1036]
        );
    }

    #[test]
    fn native_build_menu_uses_the_hfx_highlight_parameters() {
        let highlighted_indices: Vec<_> = BUILDING_PANEL_ICON_SPECS
            .iter()
            .map(|spec| spec.highlight_sprite_index)
            .collect();
        assert_eq!(
            highlighted_indices,
            [1046, 1047, 1048, 1050, 1051, 1049, 1052, 1053, 1054]
        );
        assert!(BUILDING_PANEL_ICON_SPECS
            .iter()
            .all(|spec| spec.highlight_sprite_index == spec.sprite_index + 18));
    }

    #[test]
    fn native_build_menu_uses_the_blocked_question_overlay() {
        assert_eq!(HFX_BLOCKED_OVERLAY, 1055);
    }

    #[test]
    fn native_build_menu_uses_canonical_building_subtypes() {
        let subtypes: Vec<_> = BUILDING_PANEL_ICON_SPECS
            .iter()
            .map(|spec| spec.building_subtype)
            .collect();
        assert_eq!(subtypes, [1, 4, 7, 5, 6, 8, 13, 15, 17]);
    }
}
