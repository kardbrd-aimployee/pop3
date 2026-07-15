use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use image::{imageops, RgbaImage};
use serde::Serialize;

use crate::data::animation::{
    anim_shape, build_direct_sprite_atlas, build_tribe_atlas, AnimationSequence, AnimationsData,
    NUM_TRIBES, SHAMAN_ANIMS, STORED_DIRECTIONS,
};
use crate::data::psfb::ContainerPSFB;
use crate::data::types::BinDeserializer;

use super::structures::make_contact_sheet;
use super::units::unit_combo_for_animation;

const ANIMATION_SCHEMA_VERSION: u32 = 1;
const PREVIEW_SIZE: u32 = 160;
const PREVIEW_ANIMATIONS: [&str; 5] = ["idle", "walk", "carry", "dig", "build"];

const NON_SHAMAN_ANIMATION_GROUPS: [(&str, [u16; 5]); 22] = [
    ("idle", [15, 16, 17, 18, 19]),
    ("walk", [21, 22, 23, 24, 25]),
    ("die", [27, 28, 29, 31, 30]),
    ("action", [32, 33, 34, 35, 36]),
    ("celebrate", [38, 39, 40, 41, 42]),
    ("spell-idle", [43, 44, 45, 46, 47]),
    ("spell-walk", [48, 49, 50, 51, 52]),
    ("work1", [53, 54, 55, 56, 57]),
    ("work2", [58, 59, 60, 61, 62]),
    ("work3", [63, 64, 65, 66, 67]),
    ("work4", [68, 69, 70, 71, 72]),
    ("work5", [73, 74, 75, 76, 77]),
    ("vehicle", [78, 79, 80, 81, 82]),
    ("swim", [83, 84, 85, 86, 87]),
    ("carry", [88, 89, 90, 91, 92]),
    ("ride", [110, 111, 112, 113, 114]),
    ("dig", [115, 116, 117, 118, 119]),
    ("build", [120, 121, 122, 123, 124]),
    ("sit1", [131, 132, 133, 134, 135]),
    ("sit2", [136, 137, 138, 139, 140]),
    ("sit3", [141, 142, 143, 144, 145]),
    ("sit4", [146, 147, 148, 149, 150]),
];

const SHAMAN_ANIMATION_GROUPS: [(&str, u16); 12] = [
    ("idle", 20),
    ("walk", 26),
    ("action", 37),
    ("special", 94),
    ("work1", 106),
    ("vehicle", 107),
    ("unknown-109", 109),
    ("swim", 125),
    ("dig", 126),
    ("carry", 127),
    ("build", 128),
    ("ride", 129),
];

const NON_SHAMAN_EXTRA_ANIMATIONS: [(&str, u16, usize); 2] =
    [("special", 100, 0), ("special", 101, 4)];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitAnimationSpec {
    pub id: String,
    pub unit_id: String,
    pub unit_name: String,
    pub animation_name: String,
    pub subtype: u8,
    pub animation_id: u16,
    pub vstart_base: usize,
    pub unit_combo: Option<(u16, u16)>,
    pub sprite_source: &'static str,
}

#[derive(Debug)]
pub struct UnitAnimationRequest {
    pub base: PathBuf,
    pub output: PathBuf,
    pub landscape: String,
}

#[derive(Debug)]
pub struct UnitAnimationExport {
    pub manifest_path: PathBuf,
    pub contact_sheet_path: PathBuf,
    pub animation_count: usize,
}

#[derive(Serialize)]
struct Manifest {
    schema_version: u32,
    kind: &'static str,
    source: SourceManifest,
    atlas: AtlasManifest,
    items: Vec<AnimationManifest>,
}

#[derive(Serialize)]
struct SourceManifest {
    base: String,
    sprite_bank: &'static str,
    vele: &'static str,
    vfra: &'static str,
    vstart: &'static str,
    palette: String,
    landscape: String,
}

#[derive(Serialize)]
struct AtlasManifest {
    tribes: usize,
    stored_directions: usize,
    display_directions: usize,
    row_order: &'static str,
    column_order: &'static str,
    mirroring: &'static str,
}

#[derive(Serialize)]
struct AnimationManifest {
    id: String,
    unit: String,
    unit_name: String,
    animation: String,
    subtype: u8,
    animation_id: u16,
    vstart_base: usize,
    unit_combo: Option<(u16, u16)>,
    sprite_source: &'static str,
    frame_width: u32,
    frame_height: u32,
    frames_per_direction: u32,
    baseline_y: i32,
    atlas: String,
}

pub fn unit_animation_specs() -> Vec<UnitAnimationSpec> {
    let units = [
        ("brave", "Brave", 2u8, 15u16),
        ("warrior", "Warrior", 3u8, 16u16),
        ("preacher", "Preacher", 4u8, 17u16),
        ("spy", "Spy", 5u8, 18u16),
        ("firewarrior", "Firewarrior", 6u8, 19u16),
    ];
    let mut specs = Vec::new();

    for (unit_index, (unit_id, unit_name, subtype, idle_id)) in units.into_iter().enumerate() {
        let unit_combo = unit_combo_for_animation(idle_id);
        for (animation_name, animation_ids) in NON_SHAMAN_ANIMATION_GROUPS {
            let animation_id = animation_ids[unit_index];
            specs.push(make_spec(
                unit_id,
                unit_name,
                subtype,
                animation_name,
                animation_id,
                unit_combo,
                "composited",
            ));
        }
    }

    for (animation_name, animation_id, unit_index) in NON_SHAMAN_EXTRA_ANIMATIONS {
        let (unit_id, unit_name, subtype, idle_id) = units[unit_index];
        specs.push(make_spec(
            unit_id,
            unit_name,
            subtype,
            animation_name,
            animation_id,
            unit_combo_for_animation(idle_id),
            "composited",
        ));
    }

    for (animation_name, animation_id) in SHAMAN_ANIMATION_GROUPS {
        specs.push(make_spec(
            "shaman",
            "Shaman",
            7,
            animation_name,
            animation_id,
            None,
            if matches!(animation_id, 20 | 26) {
                "direct"
            } else {
                "composited"
            },
        ));
    }

    specs
}

fn make_spec(
    unit_id: &'static str,
    unit_name: &'static str,
    subtype: u8,
    animation_name: &'static str,
    animation_id: u16,
    unit_combo: Option<(u16, u16)>,
    sprite_source: &'static str,
) -> UnitAnimationSpec {
    let (vstart_base, _) = anim_shape(animation_id);
    UnitAnimationSpec {
        id: format!("{unit_id}-{animation_name}"),
        unit_id: unit_id.to_string(),
        unit_name: unit_name.to_string(),
        animation_name: animation_name.to_string(),
        subtype,
        animation_id,
        vstart_base,
        unit_combo,
        sprite_source,
    }
}

pub fn export_unit_animations(
    request: &UnitAnimationRequest,
) -> Result<UnitAnimationExport, Box<dyn Error>> {
    if request.landscape.len() != 1
        || !request
            .landscape
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric())
    {
        return Err(invalid_input("landscape must be one alphanumeric bank key").into());
    }

    let data_dir = request.base.join("data");
    let palette_path = data_dir.join(format!("pal0-{}.dat", request.landscape));
    let sprite_path = data_dir.join("HSPR0-0.DAT");
    let vele_path = data_dir.join("VELE-0.ANI");
    let vfra_path = data_dir.join("VFRA-0.ANI");
    let vstart_path = data_dir.join("VSTART-0.ANI");
    for path in [
        &palette_path,
        &sprite_path,
        &vele_path,
        &vfra_path,
        &vstart_path,
    ] {
        ensure_file(path)?;
    }

    let palette = load_palette(&palette_path)?;
    let container = ContainerPSFB::from_file(&sprite_path).ok_or_else(|| {
        invalid_data(format!(
            "could not parse sprite bank: {}",
            sprite_path.display()
        ))
    })?;
    let animations = AnimationsData::from_path(&data_dir);
    let sequences = AnimationSequence::from_data(&animations);
    let specs = unit_animation_specs();
    let animations_dir = request.output.join("animations");
    fs::create_dir_all(&animations_dir)?;

    let mut manifest_items = Vec::with_capacity(specs.len());
    let mut previews = Vec::new();
    for spec in specs {
        let (atlas_width, atlas_height, atlas, frame_width, frame_height, frames, baseline_y) =
            build_animation_atlas(&spec, &sequences, &container, &palette)?;
        let unit_dir = animations_dir.join(&spec.unit_id);
        fs::create_dir_all(&unit_dir)?;
        let file_name = format!("{}.png", spec.animation_name);
        let atlas_path = unit_dir.join(&file_name);
        let image = RgbaImage::from_raw(atlas_width, atlas_height, atlas)
            .ok_or_else(|| invalid_data("generated animation atlas has invalid dimensions"))?;
        if PREVIEW_ANIMATIONS.contains(&spec.animation_name.as_str()) {
            let cell = atlas_cell(&image, frame_width, frame_height, 0, 0)?;
            previews.push((
                format!("{} {}", spec.unit_name, spec.animation_name),
                fit_preview(&cell, PREVIEW_SIZE),
            ));
        }
        image.save(&atlas_path)?;
        let atlas_relative = format!("animations/{}/{}", spec.unit_id, file_name);
        manifest_items.push(AnimationManifest {
            id: spec.id,
            unit: spec.unit_id,
            unit_name: spec.unit_name,
            animation: spec.animation_name,
            subtype: spec.subtype,
            animation_id: spec.animation_id,
            vstart_base: spec.vstart_base,
            unit_combo: spec.unit_combo,
            sprite_source: spec.sprite_source,
            frame_width,
            frame_height,
            frames_per_direction: frames,
            baseline_y,
            atlas: atlas_relative,
        });
    }

    let contact_sheet = make_contact_sheet(&previews, PREVIEW_SIZE);
    let contact_sheet_path = request.output.join("contact-sheet.png");
    fs::create_dir_all(&request.output)?;
    contact_sheet.save(&contact_sheet_path)?;

    let manifest = Manifest {
        schema_version: ANIMATION_SCHEMA_VERSION,
        kind: "unit-animations",
        source: SourceManifest {
            base: request.base.display().to_string(),
            sprite_bank: "data/HSPR0-0.DAT",
            vele: "data/VELE-0.ANI",
            vfra: "data/VFRA-0.ANI",
            vstart: "data/VSTART-0.ANI",
            palette: relative_source(&request.base, &palette_path),
            landscape: request.landscape.clone(),
        },
        atlas: AtlasManifest {
            tribes: NUM_TRIBES,
            stored_directions: STORED_DIRECTIONS,
            display_directions: 8,
            row_order: "tribe-major, stored-direction-minor",
            column_order: "animation-frame",
            mirroring: "renderer mirrors the two directions absent from the five stored rows",
        },
        items: manifest_items,
    };
    let manifest_path = request.output.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    Ok(UnitAnimationExport {
        manifest_path,
        contact_sheet_path,
        animation_count: manifest.items.len(),
    })
}

fn build_animation_atlas(
    spec: &UnitAnimationSpec,
    sequences: &[AnimationSequence],
    container: &ContainerPSFB,
    palette: &[[u8; 4]],
) -> Result<(u32, u32, Vec<u8>, u32, u32, u32, i32), io::Error> {
    if spec.sprite_source == "direct" {
        let (starts, frames) = SHAMAN_ANIMS
            .iter()
            .find(|(animation_id, _, _)| *animation_id == spec.animation_id)
            .map(|(_, starts, frames)| (starts, *frames))
            .ok_or_else(|| invalid_data("direct unit animation has no source sprite range"))?;
        return build_direct_sprite_atlas(container, palette, starts, frames)
            .ok_or_else(|| invalid_data("could not build direct unit animation atlas"));
    }

    build_tribe_atlas(
        sequences,
        container,
        palette,
        spec.vstart_base,
        Some(spec.unit_combo),
        None,
    )
    .ok_or_else(|| invalid_data(format!("could not build animation atlas for {}", spec.id)))
}

fn atlas_cell(
    atlas: &RgbaImage,
    cell_width: u32,
    cell_height: u32,
    column: u32,
    row: u32,
) -> Result<RgbaImage, io::Error> {
    let x0 = column
        .checked_mul(cell_width)
        .ok_or_else(|| invalid_data("animation atlas column overflow"))?;
    let y0 = row
        .checked_mul(cell_height)
        .ok_or_else(|| invalid_data("animation atlas row overflow"))?;
    if cell_width == 0
        || cell_height == 0
        || x0 + cell_width > atlas.width()
        || y0 + cell_height > atlas.height()
    {
        return Err(invalid_data(
            "animation atlas cell is outside the generated atlas",
        ));
    }

    Ok(imageops::crop_imm(atlas, x0, y0, cell_width, cell_height).to_image())
}

fn fit_preview(source: &RgbaImage, size: u32) -> RgbaImage {
    let padding = (size as f32 * 0.08).round() as u32;
    let drawable = size.saturating_sub(padding * 2).max(1);
    let scale =
        (drawable as f32 / source.width() as f32).min(drawable as f32 / source.height() as f32);
    let width = (source.width() as f32 * scale).round().max(1.0) as u32;
    let height = (source.height() as f32 * scale).round().max(1.0) as u32;
    let scaled = imageops::resize(source, width, height, imageops::FilterType::Nearest);
    let mut image = RgbaImage::new(size, size);
    imageops::overlay(
        &mut image,
        &scaled,
        ((size - width) / 2) as i64,
        ((size - height) / 2) as i64,
    );
    image
}

fn load_palette(path: &Path) -> Result<Vec<[u8; 4]>, io::Error> {
    let data = fs::read(path)?;
    if data.len() < 1024 {
        return Err(invalid_data(format!(
            "palette is shorter than 1024 bytes: {}",
            path.display()
        )));
    }
    Ok((0..256)
        .map(|index| {
            let offset = index * 4;
            [data[offset], data[offset + 1], data[offset + 2], 255]
        })
        .collect())
}

fn ensure_file(path: &Path) -> Result<(), io::Error> {
    if path.is_file() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("required original-game file not found: {}", path.display()),
        ))
    }
}

fn relative_source(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_contains_construction_animation_ids() {
        let specs = unit_animation_specs();
        let brave_dig = specs.iter().find(|spec| spec.id == "brave-dig").unwrap();
        let brave_build = specs.iter().find(|spec| spec.id == "brave-build").unwrap();
        let brave_carry = specs.iter().find(|spec| spec.id == "brave-carry").unwrap();
        assert_eq!(brave_dig.animation_id, 115);
        assert_eq!(brave_build.animation_id, 120);
        assert_eq!(brave_carry.animation_id, 88);
    }

    #[test]
    fn catalog_preserves_type_layers_and_direct_shaman_sprites() {
        let specs = unit_animation_specs();
        assert_eq!(
            specs
                .iter()
                .find(|spec| spec.id == "warrior-build")
                .unwrap()
                .unit_combo,
            Some((2, 2))
        );
        assert_eq!(
            specs
                .iter()
                .find(|spec| spec.id == "firewarrior-build")
                .unwrap()
                .unit_combo,
            Some((2, 1))
        );
        assert_eq!(
            specs
                .iter()
                .find(|spec| spec.id == "shaman-idle")
                .unwrap()
                .sprite_source,
            "direct"
        );
        assert_eq!(
            specs
                .iter()
                .find(|spec| spec.id == "shaman-build")
                .unwrap()
                .sprite_source,
            "composited"
        );
    }

    #[test]
    fn catalog_has_expected_animation_count() {
        assert_eq!(unit_animation_specs().len(), 124);
    }
}
