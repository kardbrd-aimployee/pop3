use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use image::{imageops, Rgba, RgbaImage};
use serde::Serialize;

use crate::data::animation::{
    anim_shape, build_direct_sprite_atlas, build_tribe_atlas, AnimationSequence, AnimationsData,
    SHAMAN_ANIMS, STORED_DIRECTIONS,
};
use crate::data::psfb::ContainerPSFB;
use crate::data::types::BinDeserializer;

use super::structures::make_contact_sheet;

const ICON_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitIconSpec {
    pub id: String,
    pub name: String,
    pub short_label: String,
    pub subtype: u8,
    pub animation_id: u16,
    pub tribe: Tribe,
    pub vstart_base: usize,
    pub unit_combo: Option<(u16, u16)>,
    pub source_sprite_start: Option<u16>,
    pub sprite_source: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tribe {
    Blue,
    Red,
    Yellow,
    Green,
}

impl Tribe {
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "blue" => Some(Self::Blue),
            "red" => Some(Self::Red),
            "yellow" => Some(Self::Yellow),
            "green" => Some(Self::Green),
            _ => None,
        }
    }

    pub fn index(self) -> u8 {
        match self {
            Self::Blue => 0,
            Self::Red => 1,
            Self::Yellow => 2,
            Self::Green => 3,
        }
    }
}

#[derive(Debug)]
pub struct UnitIconRequest {
    pub base: PathBuf,
    pub output: PathBuf,
    pub landscape: String,
    pub tribe: Tribe,
    pub size: u32,
}

#[derive(Debug)]
pub struct UnitIconExport {
    pub manifest_path: PathBuf,
    pub contact_sheet_path: PathBuf,
    pub icon_count: usize,
}

#[derive(Serialize)]
struct Manifest {
    schema_version: u32,
    kind: &'static str,
    source: SourceManifest,
    render: RenderManifest,
    items: Vec<IconManifest>,
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
    tribe: Tribe,
}

#[derive(Serialize)]
struct RenderManifest {
    width: u32,
    height: u32,
    pose: &'static str,
    stored_direction: usize,
    frame: usize,
    background: &'static str,
}

#[derive(Serialize)]
struct IconManifest {
    id: String,
    name: String,
    short_label: String,
    subtype: u8,
    tribe: u8,
    animation_id: u16,
    vstart_base: usize,
    unit_combo: Option<(u16, u16)>,
    sprite_source: &'static str,
    source_sprite_start: Option<u16>,
    icon: String,
}

pub fn unit_icon_specs(tribe: Tribe) -> Vec<UnitIconSpec> {
    let tribe_index = tribe.index() as usize;
    let shaman_start = SHAMAN_ANIMS
        .iter()
        .find(|(animation_id, _, _)| *animation_id == 20)
        .map(|(_, starts, _)| starts[tribe_index]);

    let mut specs = Vec::with_capacity(6);
    for (name, subtype, animation_id) in [
        ("Brave", 2u8, 15u16),
        ("Warrior", 3u8, 16u16),
        ("Preacher", 4u8, 17u16),
        ("Spy", 5u8, 18u16),
        ("Firewarrior", 6u8, 19u16),
        ("Shaman", 7u8, 20u16),
    ] {
        let is_shaman = animation_id == 20;
        specs.push(UnitIconSpec {
            id: format!("{}-{}", name.to_ascii_lowercase(), tribe_slug(tribe)),
            name: format!("{name} ({})", tribe_name(tribe)),
            short_label: name.to_string(),
            subtype,
            animation_id,
            tribe,
            vstart_base: anim_shape(animation_id).0,
            unit_combo: unit_combo_for_animation(animation_id),
            source_sprite_start: if is_shaman { shaman_start } else { None },
            sprite_source: if is_shaman { "direct" } else { "composited" },
        });
    }
    specs
}

pub fn export_unit_icons(request: &UnitIconRequest) -> Result<UnitIconExport, Box<dyn Error>> {
    if !(64..=1024).contains(&request.size) {
        return Err(invalid_input("icon size must be between 64 and 1024 pixels").into());
    }
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
    let specs = unit_icon_specs(request.tribe);

    let shaman_atlas = SHAMAN_ANIMS
        .iter()
        .find(|(animation_id, _, _)| *animation_id == 20)
        .and_then(|(_, starts, frames)| {
            build_direct_sprite_atlas(&container, &palette, starts, *frames)
        })
        .ok_or_else(|| invalid_data("could not build shaman idle atlas"))?;

    let icons_dir = request.output.join("icons");
    fs::create_dir_all(&icons_dir)?;
    let mut rendered = Vec::with_capacity(specs.len());
    let mut manifest_items = Vec::with_capacity(specs.len());
    let row = request.tribe.index() as u32 * STORED_DIRECTIONS as u32;

    for spec in specs {
        let cell = if spec.animation_id == 20 {
            let (_, _, atlas, frame_width, frame_height, _, _) = &shaman_atlas;
            atlas_cell(
                atlas,
                *frame_width,
                *frame_height,
                *frame_width,
                *frame_height,
                0,
                row,
            )?
        } else {
            let (atlas_width, atlas_height, atlas, frame_width, frame_height, _, _) =
                build_tribe_atlas(
                    &sequences,
                    &container,
                    &palette,
                    spec.vstart_base,
                    Some(spec.unit_combo),
                    None,
                )
                .ok_or_else(|| invalid_data("could not build unit idle atlas"))?;
            atlas_cell(
                &atlas,
                atlas_width,
                atlas_height,
                frame_width,
                frame_height,
                0,
                row,
            )?
        };
        let icon = fit_icon(&cell, request.size);
        let file_name = format!("{}.png", spec.id);
        icon.save(icons_dir.join(&file_name))?;
        rendered.push((spec.short_label.clone(), icon));
        manifest_items.push(IconManifest {
            id: spec.id,
            name: spec.name,
            short_label: spec.short_label,
            subtype: spec.subtype,
            tribe: spec.tribe.index(),
            animation_id: spec.animation_id,
            vstart_base: spec.vstart_base,
            unit_combo: spec.unit_combo,
            sprite_source: spec.sprite_source,
            source_sprite_start: spec.source_sprite_start,
            icon: format!("icons/{file_name}"),
        });
    }

    let contact_sheet = make_contact_sheet(&rendered, request.size);
    let contact_sheet_path = request.output.join("contact-sheet.png");
    contact_sheet.save(&contact_sheet_path)?;

    let manifest = Manifest {
        schema_version: ICON_SCHEMA_VERSION,
        kind: "unit-icons",
        source: SourceManifest {
            base: request.base.display().to_string(),
            sprite_bank: "data/HSPR0-0.DAT",
            vele: "data/VELE-0.ANI",
            vfra: "data/VFRA-0.ANI",
            vstart: "data/VSTART-0.ANI",
            palette: relative_source(&request.base, &palette_path),
            landscape: request.landscape.clone(),
            tribe: request.tribe,
        },
        render: RenderManifest {
            width: request.size,
            height: request.size,
            pose: "idle",
            stored_direction: 0,
            frame: 0,
            background: "transparent",
        },
        items: manifest_items,
    };
    let manifest_path = request.output.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    Ok(UnitIconExport {
        manifest_path,
        contact_sheet_path,
        icon_count: rendered.len(),
    })
}

fn atlas_cell(
    atlas: &[u8],
    atlas_width: u32,
    atlas_height: u32,
    cell_width: u32,
    cell_height: u32,
    column: u32,
    row: u32,
) -> Result<RgbaImage, io::Error> {
    let x0 = column
        .checked_mul(cell_width)
        .ok_or_else(|| invalid_data("unit atlas column overflow"))?;
    let y0 = row
        .checked_mul(cell_height)
        .ok_or_else(|| invalid_data("unit atlas row overflow"))?;
    if cell_width == 0
        || cell_height == 0
        || x0 + cell_width > atlas_width
        || y0 + cell_height > atlas_height
    {
        return Err(invalid_data(
            "unit atlas cell is outside the generated atlas",
        ));
    }

    let mut cell = RgbaImage::new(cell_width, cell_height);
    for y in 0..cell_height {
        for x in 0..cell_width {
            let source = ((y0 + y) * atlas_width + x0 + x) as usize * 4;
            if source + 4 > atlas.len() {
                return Err(invalid_data(
                    "unit atlas buffer is shorter than its dimensions",
                ));
            }
            cell.put_pixel(
                x,
                y,
                Rgba([
                    atlas[source],
                    atlas[source + 1],
                    atlas[source + 2],
                    atlas[source + 3],
                ]),
            );
        }
    }
    Ok(cell)
}

fn fit_icon(source: &RgbaImage, size: u32) -> RgbaImage {
    let padding = (size as f32 * 0.08).round() as u32;
    let drawable = size.saturating_sub(padding * 2).max(1);
    let scale =
        (drawable as f32 / source.width() as f32).min(drawable as f32 / source.height() as f32);
    let width = (source.width() as f32 * scale).round().max(1.0) as u32;
    let height = (source.height() as f32 * scale).round().max(1.0) as u32;
    let scaled = imageops::resize(source, width, height, imageops::FilterType::Nearest);
    let mut icon = RgbaImage::new(size, size);
    imageops::overlay(
        &mut icon,
        &scaled,
        ((size - width) / 2) as i64,
        ((size - height) / 2) as i64,
    );
    icon
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

fn tribe_name(tribe: Tribe) -> &'static str {
    match tribe {
        Tribe::Blue => "Blue",
        Tribe::Red => "Red",
        Tribe::Yellow => "Yellow",
        Tribe::Green => "Green",
    }
}

fn unit_combo_for_animation(animation_id: u16) -> Option<(u16, u16)> {
    match animation_id {
        15 => None,         // Brave: common body layer
        16 => Some((2, 2)), // Warrior
        17 => Some((3, 1)), // Preacher
        18 => Some((2, 3)), // Spy
        19 => Some((2, 1)), // Firewarrior
        20 => None,         // Shaman: direct per-tribe sprites
        _ => None,
    }
}

fn tribe_slug(tribe: Tribe) -> &'static str {
    match tribe {
        Tribe::Blue => "blue",
        Tribe::Red => "red",
        Tribe::Yellow => "yellow",
        Tribe::Green => "green",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_catalog_contains_idle_person_types() {
        let specs = unit_icon_specs(Tribe::Blue);
        assert_eq!(specs.len(), 6);
        assert_eq!(specs[0].id, "brave-blue");
        assert_eq!(specs[0].animation_id, 15);
        assert_eq!(specs[0].unit_combo, None);
        assert_eq!(specs[1].unit_combo, Some((2, 2)));
        assert_eq!(specs[2].unit_combo, Some((3, 1)));
        assert_eq!(specs[3].unit_combo, Some((2, 3)));
        assert_eq!(specs[4].unit_combo, Some((2, 1)));
        assert_eq!(specs[4].sprite_source, "composited");
        assert_eq!(specs[5].sprite_source, "direct");
        assert_eq!(specs[5].source_sprite_start, Some(6879));
    }

    #[test]
    fn unit_catalog_offsets_shaman_sprites_by_tribe() {
        let specs = unit_icon_specs(Tribe::Green);
        assert_eq!(specs[5].source_sprite_start, Some(6939));
        assert_eq!(specs[5].tribe.index(), 3);
    }

    #[test]
    fn atlas_cell_copies_a_single_rgba_cell() {
        let atlas = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
        let cell = atlas_cell(&atlas, 2, 2, 1, 1, 1, 0).expect("cell");
        assert_eq!(cell.get_pixel(0, 0).0, [5, 6, 7, 8]);
    }
}
