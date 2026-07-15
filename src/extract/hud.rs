use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use image::{imageops, Rgba, RgbaImage};
use serde::Serialize;

use crate::data::psfb::{ContainerPSFB, SpritePSFB};
use crate::data::types::{BinDeserializer, Image};

use super::structures::make_contact_sheet;

const SCHEMA_VERSION: u32 = 1;
const CONTACT_SHEET_ITEMS: usize = 64;
const TABLE_RECORD_SIZE: usize = 16;

#[derive(Debug)]
pub struct HudSpriteCandidateRequest {
    pub base: PathBuf,
    pub output: PathBuf,
    pub bank: HudSpriteBank,
    pub landscape: String,
    pub size: u32,
    pub min_dimension: u16,
    pub max_dimension: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudSpriteBank {
    Primary,
    Extension,
    Hspr1,
    Hspr2,
    Mspr,
    MsprExtension,
    Point,
    Point1,
    Point2,
    Panel,
}

impl HudSpriteBank {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "primary" => Some(Self::Primary),
            "extension" => Some(Self::Extension),
            "hspr1" => Some(Self::Hspr1),
            "hspr2" => Some(Self::Hspr2),
            "mspr" => Some(Self::Mspr),
            "mspr-extension" => Some(Self::MsprExtension),
            "point" => Some(Self::Point),
            "point1" => Some(Self::Point1),
            "point2" => Some(Self::Point2),
            "panel" => Some(Self::Panel),
            _ => None,
        }
    }

    fn data_file(self) -> &'static str {
        match self {
            Self::Primary => "HSPR0-0.DAT",
            Self::Extension => "HSPR0-1.DAT",
            Self::Hspr1 => "HSPR1-0.DAT",
            Self::Hspr2 => "hspr2-0.dat",
            Self::Mspr => "MSPR0-0.DAT",
            Self::MsprExtension => "MSPR0-1.DAT",
            Self::Point => "POINT0-0.DAT",
            Self::Point1 => "POINT0-1.DAT",
            Self::Point2 => "POINT0-2.DAT",
            Self::Panel => "plspanel.spr",
        }
    }

    fn table_file(self) -> Option<&'static str> {
        match self {
            Self::Primary => None,
            Self::Extension => Some("HSPR0-1.TAB"),
            Self::MsprExtension => Some("MSPR0-1.TAB"),
            Self::Hspr1
            | Self::Hspr2
            | Self::Mspr
            | Self::Point
            | Self::Point1
            | Self::Point2
            | Self::Panel => None,
        }
    }

    fn palette_file(self, landscape: &str) -> String {
        match self {
            Self::Point | Self::Point1 | Self::Point2 => "PAL1-0.DAT".to_owned(),
            Self::Panel => "plspal.dat".to_owned(),
            Self::Hspr1 | Self::Hspr2 => "PAL1-0.DAT".to_owned(),
            Self::Primary | Self::Extension | Self::Mspr | Self::MsprExtension => {
                format!("pal0-{landscape}.dat")
            }
        }
    }
}

#[derive(Debug)]
pub struct HudSpriteCandidateExport {
    pub manifest_path: PathBuf,
    pub contact_sheet_paths: Vec<PathBuf>,
    pub candidate_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TableSprite {
    index: usize,
    offset: usize,
    width: u16,
    height: u16,
    flags: u32,
}

#[derive(Serialize)]
struct Manifest {
    schema_version: u32,
    kind: &'static str,
    source: SourceManifest,
    filter: FilterManifest,
    items: Vec<CandidateManifest>,
}

#[derive(Serialize)]
struct SourceManifest {
    sprite_data: String,
    sprite_table: Option<String>,
    palette: String,
    table_record_size: Option<usize>,
}

#[derive(Serialize)]
struct FilterManifest {
    min_dimension: u16,
    max_dimension: u16,
}

#[derive(Serialize)]
struct CandidateManifest {
    sprite_index: usize,
    data_offset: usize,
    width: u16,
    height: u16,
    flags: u32,
    image: String,
    contact_sheet: String,
}

pub fn export_hud_sprite_candidates(
    request: &HudSpriteCandidateRequest,
) -> Result<HudSpriteCandidateExport, Box<dyn Error>> {
    validate_request(request)?;

    let data_dir = request.base.join("data");
    let sprite_data_path = data_dir.join(request.bank.data_file());
    let sprite_table_path = request.bank.table_file().map(|name| data_dir.join(name));
    let palette_path = data_dir.join(request.bank.palette_file(&request.landscape));
    ensure_file(&sprite_data_path)?;
    if let Some(path) = &sprite_table_path {
        ensure_file(path)?;
    }
    ensure_file(&palette_path)?;

    let sprite_bank = load_sprite_bank(
        request.bank,
        &sprite_data_path,
        sprite_table_path.as_deref(),
    )?;
    let palette = load_palette(&palette_path)?;
    let candidates = candidate_sprites(
        &sprite_bank.sprites(),
        request.min_dimension,
        request.max_dimension,
    );

    let sprites_dir = request.output.join("sprites");
    let sheets_dir = request.output.join("contact-sheets");
    fs::create_dir_all(&sprites_dir)?;
    fs::create_dir_all(&sheets_dir)?;

    let mut manifest_items = Vec::with_capacity(candidates.len());
    let mut contact_sheet_paths = Vec::new();

    for (sheet_number, chunk) in candidates.chunks(CONTACT_SHEET_ITEMS).enumerate() {
        let first_index = chunk.first().map(|sprite| sprite.index).unwrap_or(0);
        let last_index = chunk
            .last()
            .map(|sprite| sprite.index)
            .unwrap_or(first_index);
        let sheet_name = format!("{sheet_number:03}-{first_index:03}-{last_index:03}.png");
        let mut rendered = Vec::with_capacity(chunk.len());

        for sprite in chunk {
            let indexed = sprite_bank.decode(*sprite)?;
            let native = indexed_to_rgba(&indexed, &palette);
            let icon = fit_icon(&native, request.size);
            let image_name = format!("{:03}.png", sprite.index);
            native.save(sprites_dir.join(&image_name))?;
            rendered.push((
                format!("#{} {}x{}", sprite.index, sprite.width, sprite.height),
                icon,
            ));
            manifest_items.push(CandidateManifest {
                sprite_index: sprite.index,
                data_offset: sprite.offset,
                width: sprite.width,
                height: sprite.height,
                flags: sprite.flags,
                image: format!("sprites/{image_name}"),
                contact_sheet: format!("contact-sheets/{sheet_name}"),
            });
        }

        let sheet_path = sheets_dir.join(&sheet_name);
        make_contact_sheet(&rendered, request.size).save(&sheet_path)?;
        contact_sheet_paths.push(sheet_path);
    }

    let manifest = Manifest {
        schema_version: SCHEMA_VERSION,
        kind: "hud-sprite-candidates",
        source: SourceManifest {
            sprite_data: relative_source(&request.base, &sprite_data_path),
            sprite_table: sprite_table_path
                .as_ref()
                .map(|path| relative_source(&request.base, path)),
            palette: relative_source(&request.base, &palette_path),
            table_record_size: sprite_table_path.as_ref().map(|_| TABLE_RECORD_SIZE),
        },
        filter: FilterManifest {
            min_dimension: request.min_dimension,
            max_dimension: request.max_dimension,
        },
        items: manifest_items,
    };
    let manifest_path = request.output.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    Ok(HudSpriteCandidateExport {
        manifest_path,
        contact_sheet_paths,
        candidate_count: candidates.len(),
    })
}

enum LoadedSpriteBank {
    Container(ContainerPSFB),
    Extension {
        data: Vec<u8>,
        sprites: Vec<TableSprite>,
    },
}

impl LoadedSpriteBank {
    fn sprites(&self) -> Vec<TableSprite> {
        match self {
            Self::Container(container) => container
                .sprites_info()
                .iter()
                .map(|sprite| TableSprite {
                    index: sprite.index,
                    offset: sprite.offset,
                    width: sprite.width,
                    height: sprite.height,
                    flags: 0,
                })
                .collect(),
            Self::Extension { sprites, .. } => sprites.clone(),
        }
    }

    fn decode(&self, sprite: TableSprite) -> Result<Image, io::Error> {
        match self {
            Self::Container(container) => container.get_image(sprite.index).ok_or_else(|| {
                invalid_data(format!(
                    "sprite {} is missing from container bank",
                    sprite.index
                ))
            }),
            Self::Extension { data, .. } => decode_table_sprite(sprite, data),
        }
    }
}

fn load_sprite_bank(
    bank: HudSpriteBank,
    data_path: &Path,
    table_path: Option<&Path>,
) -> Result<LoadedSpriteBank, Box<dyn Error>> {
    match bank {
        HudSpriteBank::Primary
        | HudSpriteBank::Hspr1
        | HudSpriteBank::Hspr2
        | HudSpriteBank::Mspr
        | HudSpriteBank::Point
        | HudSpriteBank::Point1
        | HudSpriteBank::Point2
        | HudSpriteBank::Panel => ContainerPSFB::from_file(data_path)
            .map(LoadedSpriteBank::Container)
            .ok_or_else(|| {
                invalid_data(format!(
                    "could not parse PSFB bank: {}",
                    data_path.display()
                ))
                .into()
            }),
        HudSpriteBank::Extension | HudSpriteBank::MsprExtension => {
            let table_path =
                table_path.ok_or_else(|| invalid_data("extension bank requires a table"))?;
            Ok(LoadedSpriteBank::Extension {
                data: fs::read(data_path)?,
                sprites: parse_sprite_table(&fs::read(table_path)?)?,
            })
        }
    }
}

fn parse_sprite_table(data: &[u8]) -> Result<Vec<TableSprite>, io::Error> {
    if data.len() % TABLE_RECORD_SIZE != 0 {
        return Err(invalid_data(format!(
            "HSPR table length {} is not divisible by {TABLE_RECORD_SIZE}",
            data.len()
        )));
    }

    data.chunks_exact(TABLE_RECORD_SIZE)
        .enumerate()
        .map(|(index, record)| {
            let offset = u32::from_le_bytes(record[0..4].try_into().unwrap()) as usize;
            let width = u32::from_le_bytes(record[4..8].try_into().unwrap());
            let height = u32::from_le_bytes(record[8..12].try_into().unwrap());
            let flags = u32::from_le_bytes(record[12..16].try_into().unwrap());
            Ok(TableSprite {
                index,
                offset,
                width: width
                    .try_into()
                    .map_err(|_| invalid_data(format!("sprite {index} width is too large")))?,
                height: height
                    .try_into()
                    .map_err(|_| invalid_data(format!("sprite {index} height is too large")))?,
                flags,
            })
        })
        .collect()
}

fn candidate_sprites(
    sprites: &[TableSprite],
    min_dimension: u16,
    max_dimension: u16,
) -> Vec<TableSprite> {
    sprites
        .iter()
        .copied()
        .filter(|sprite| {
            sprite.width >= min_dimension
                && sprite.height >= min_dimension
                && sprite.width <= max_dimension
                && sprite.height <= max_dimension
        })
        .collect()
}

fn decode_table_sprite(sprite: TableSprite, data: &[u8]) -> Result<Image, io::Error> {
    let encoded = data.get(sprite.offset..).ok_or_else(|| {
        invalid_data(format!(
            "sprite {} offset {} is outside HSPR0-1.DAT",
            sprite.index, sprite.offset
        ))
    })?;
    let mut image = Image::new(
        sprite.width as usize,
        sprite.height as usize,
        vec![255; sprite.width as usize * sprite.height as usize],
    );
    SpritePSFB {
        index: sprite.index,
        offset: 0,
        width: sprite.width,
        height: sprite.height,
    }
    .to_storage(&mut image, encoded);
    Ok(image)
}

fn validate_request(request: &HudSpriteCandidateRequest) -> Result<(), io::Error> {
    if !(48..=1024).contains(&request.size) {
        return Err(invalid_input(
            "sprite size must be between 48 and 1024 pixels",
        ));
    }
    if request.min_dimension == 0 || request.min_dimension > request.max_dimension {
        return Err(invalid_input(
            "minimum dimension must be non-zero and no greater than maximum dimension",
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

pub(crate) fn indexed_to_rgba(source: &Image, palette: &[[u8; 4]]) -> RgbaImage {
    let mut image = RgbaImage::new(source.width as u32, source.height as u32);
    for (position, palette_index) in source.data.iter().copied().enumerate() {
        let x = position % source.width;
        let y = position / source.width;
        let color = palette[palette_index as usize];
        image.put_pixel(
            x as u32,
            y as u32,
            Rgba([
                color[0],
                color[1],
                color[2],
                if palette_index == 255 { 0 } else { 255 },
            ]),
        );
    }
    image
}

pub(crate) fn fit_icon(source: &RgbaImage, size: u32) -> RgbaImage {
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

pub(crate) fn load_palette(path: &Path) -> Result<Vec<[u8; 4]>, io::Error> {
    let data = fs::read(path)?;
    match data.len() {
        768 => Ok(data
            .chunks_exact(3)
            .map(|color| [color[0], color[1], color[2], 255])
            .collect()),
        length if length >= 1024 => Ok((0..256)
            .map(|index| {
                let offset = index * 4;
                [data[offset], data[offset + 1], data[offset + 2], 255]
            })
            .collect()),
        _ => Err(invalid_data(format!(
            "palette must contain 256 RGB or RGBA entries: {}",
            path.display()
        ))),
    }
}

pub(crate) fn ensure_file(path: &Path) -> Result<(), io::Error> {
    if path.is_file() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("required original-game file not found: {}", path.display()),
        ))
    }
}

pub(crate) fn relative_source(base: &Path, path: &Path) -> String {
    path.strip_prefix(base)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

pub(crate) fn invalid_input(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

pub(crate) fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_external_hspr_table_records() {
        let mut data = vec![0; 32];
        data[16..20].copy_from_slice(&144u32.to_le_bytes());
        data[20..24].copy_from_slice(&10u32.to_le_bytes());
        data[24..28].copy_from_slice(&18u32.to_le_bytes());
        data[28..32].copy_from_slice(&7u32.to_le_bytes());
        let sprites = parse_sprite_table(&data).expect("table");
        assert_eq!(sprites.len(), 2);
        assert_eq!(
            sprites[1],
            TableSprite {
                index: 1,
                offset: 144,
                width: 10,
                height: 18,
                flags: 7,
            }
        );
    }

    #[test]
    fn candidates_filter_dimensions() {
        let sprites = [
            TableSprite {
                index: 10,
                offset: 0,
                width: 20,
                height: 20,
                flags: 0,
            },
            TableSprite {
                index: 11,
                offset: 0,
                width: 8,
                height: 20,
                flags: 0,
            },
        ];
        let result = candidate_sprites(&sprites, 12, 64);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].index, 10);
    }

    #[test]
    fn indexed_sprite_uses_palette_255_as_transparency() {
        let source = Image::new(2, 1, vec![0, 255]);
        let mut palette = vec![[0, 0, 0, 255]; 256];
        palette[0] = [10, 20, 30, 255];
        palette[255] = [40, 50, 60, 255];
        let image = indexed_to_rgba(&source, &palette);
        assert_eq!(image.get_pixel(0, 0).0, [10, 20, 30, 255]);
        assert_eq!(image.get_pixel(1, 0).0, [40, 50, 60, 0]);
    }

    #[test]
    fn point_and_panel_banks_use_ui_specific_palettes() {
        assert_eq!(HudSpriteBank::Point.palette_file("7"), "PAL1-0.DAT");
        assert_eq!(HudSpriteBank::Point1.palette_file("7"), "PAL1-0.DAT");
        assert_eq!(HudSpriteBank::Panel.palette_file("7"), "plspal.dat");
        assert_eq!(HudSpriteBank::Hspr1.palette_file("7"), "PAL1-0.DAT");
        assert_eq!(HudSpriteBank::Mspr.palette_file("7"), "pal0-7.dat");
        assert_eq!(HudSpriteBank::Primary.palette_file("7"), "pal0-7.dat");
    }
}
