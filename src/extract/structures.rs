use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use cgmath::{InnerSpace, Vector2, Vector3};
use image::{imageops, Rgba, RgbaImage};
use serde::Serialize;

use crate::data::bl320::make_bl320_texture_rgba;
use crate::data::level::{LevelPaths, ObjectPaths};
use crate::data::objects::{mk_pop_object, Object3D};
use crate::render::hud::FONT_8X8;
use crate::render::tex_model::TexModel;

const ICON_SCHEMA_VERSION: u32 = 1;
const TEXTURE_COLUMNS: usize = 8;
const TEXTURE_ROWS: usize = 32;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StructureIconSpec {
    pub id: String,
    pub name: String,
    pub short_label: String,
    pub subtype: u8,
    pub visual_variant: Option<u8>,
    pub object_index: usize,
}

#[derive(Debug)]
pub struct StructureIconRequest {
    pub base: PathBuf,
    pub output: PathBuf,
    pub landscape: String,
    pub tribe: Tribe,
    pub size: u32,
}

#[derive(Debug)]
pub struct StructureIconExport {
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
    object_bank: &'static str,
    point_bank: &'static str,
    face_bank: &'static str,
    texture_atlas: String,
    palette: String,
    landscape: String,
    tribe: Tribe,
}

#[derive(Serialize)]
struct RenderManifest {
    width: u32,
    height: u32,
    projection: &'static str,
    texture_filter: &'static str,
    background: &'static str,
}

#[derive(Serialize)]
struct IconManifest {
    id: String,
    name: String,
    short_label: String,
    subtype: u8,
    tribe: u8,
    visual_variant: Option<u8>,
    object_index: usize,
    face_count: usize,
    point_count: usize,
    icon: String,
}

#[derive(Clone, Copy)]
struct ProjectedVertex {
    x: f32,
    y: f32,
    depth: f32,
    uv: Vector2<f32>,
}

pub fn structure_icon_specs(tribe: Tribe) -> Vec<StructureIconSpec> {
    let tribe_index = tribe.index() as usize;
    let mut specs = Vec::with_capacity(16);

    for (subtype, stage_name, stage_label) in [
        (1u8, "Small Hut", "Hut I"),
        (2u8, "Medium Hut", "Hut II"),
        (3u8, "Large Hut", "Hut III"),
    ] {
        for (variant, family) in ['A', 'B', 'C'].into_iter().enumerate() {
            specs.push(StructureIconSpec {
                id: format!("{}-{}", slug(stage_name), family.to_ascii_lowercase()),
                name: format!("{stage_name} ({family})"),
                short_label: format!("{stage_label} {family}"),
                subtype,
                visual_variant: Some(variant as u8),
                object_index: 145 + variant * 12 + tribe_index * 3 + subtype as usize - 1,
            });
        }
    }

    for (id, name, short_label, subtype, blue_object_index) in [
        ("drum-tower", "Drum Tower", "Drum Tower", 4u8, 117usize),
        ("temple", "Temple", "Temple", 5u8, 133usize),
        (
            "spy-training-hut",
            "Spy Training Hut",
            "Spy Hut",
            6u8,
            129usize,
        ),
        (
            "warrior-training-hut",
            "Warrior Training Hut",
            "Warrior Hut",
            7u8,
            141usize,
        ),
        (
            "firewarrior-training-hut",
            "Firewarrior Training Hut",
            "Firewarrior",
            8u8,
            137usize,
        ),
        ("boat-hut", "Boat Hut", "Boat Hut", 13u8, 121usize),
        ("airship-hut", "Airship Hut", "Airship Hut", 14u8, 125usize),
    ] {
        specs.push(StructureIconSpec {
            id: id.to_string(),
            name: name.to_string(),
            short_label: short_label.to_string(),
            subtype,
            visual_variant: None,
            object_index: blue_object_index + tribe_index,
        });
    }

    specs
}

pub fn export_structure_icons(
    request: &StructureIconRequest,
) -> Result<StructureIconExport, Box<dyn Error>> {
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

    let level_paths = LevelPaths::from_default_dir(&request.base, &request.landscape);
    let object_paths = ObjectPaths::from_default_dir(&request.base, "0");
    for path in [
        &level_paths.palette,
        &level_paths.bl320,
        &object_paths.objs0_dat,
        &object_paths.pnts0,
        &object_paths.facs0,
    ] {
        ensure_file(path)?;
    }

    let palette = fs::read(&level_paths.palette)?;
    let (atlas_width, atlas_height, atlas) = make_bl320_texture_rgba(&level_paths.bl320, &palette);
    let object_bank = Object3D::from_file_all(&request.base, "0");
    let specs = structure_icon_specs(request.tribe);

    let icons_dir = request.output.join("icons");
    fs::create_dir_all(&icons_dir)?;
    let mut rendered = Vec::with_capacity(specs.len());
    let mut manifest_items = Vec::with_capacity(specs.len());

    for spec in specs {
        let object = object_bank
            .get(spec.object_index)
            .and_then(Option::as_ref)
            .ok_or_else(|| {
                invalid_data(format!(
                    "OBJS0-0 has no rendered object at index {} ({})",
                    spec.object_index, spec.name
                ))
            })?;
        let icon = render_model_icon(
            &mk_pop_object(object),
            &atlas,
            atlas_width,
            atlas_height,
            request.size,
        );
        let file_name = format!("{}.png", spec.id);
        icon.save(icons_dir.join(&file_name))?;
        rendered.push((spec.short_label.clone(), icon));
        manifest_items.push(IconManifest {
            id: spec.id,
            name: spec.name,
            short_label: spec.short_label,
            subtype: spec.subtype,
            tribe: request.tribe.index(),
            visual_variant: spec.visual_variant,
            object_index: spec.object_index,
            face_count: object.face_count(),
            point_count: object.point_count(),
            icon: format!("icons/{file_name}"),
        });
    }

    let contact_sheet = make_contact_sheet(&rendered, request.size);
    let contact_sheet_path = request.output.join("contact-sheet.png");
    contact_sheet.save(&contact_sheet_path)?;

    let manifest = Manifest {
        schema_version: ICON_SCHEMA_VERSION,
        kind: "structure-icons",
        source: SourceManifest {
            base: request.base.display().to_string(),
            object_bank: "objects/OBJS0-0.DAT",
            point_bank: "objects/PNTS0-0.DAT",
            face_bank: "objects/FACS0-0.DAT",
            texture_atlas: relative_source(&request.base, &level_paths.bl320),
            palette: relative_source(&request.base, &level_paths.palette),
            landscape: request.landscape.clone(),
            tribe: request.tribe,
        },
        render: RenderManifest {
            width: request.size,
            height: request.size,
            projection: "orthographic-isometric",
            texture_filter: "nearest",
            background: "transparent",
        },
        items: manifest_items,
    };
    let manifest_path = request.output.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_vec_pretty(&manifest)?)?;

    Ok(StructureIconExport {
        manifest_path,
        contact_sheet_path,
        icon_count: rendered.len(),
    })
}

fn render_model_icon(
    model: &TexModel,
    atlas: &[u8],
    atlas_width: usize,
    atlas_height: usize,
    size: u32,
) -> RgbaImage {
    let mut image = RgbaImage::new(size, size);
    if model.vertices.is_empty() {
        return image;
    }

    let view = Vector3::new(1.0, 0.85, 1.0).normalize();
    let right = Vector3::unit_y().cross(view).normalize();
    let up = view.cross(right).normalize();
    let mut projected = Vec::with_capacity(model.vertices.len());
    for vertex in &model.vertices {
        projected.push(ProjectedVertex {
            x: vertex.coord.dot(right),
            y: -vertex.coord.dot(up),
            depth: vertex.coord.dot(view),
            uv: vertex.uv,
        });
    }

    let min_x = projected.iter().map(|p| p.x).fold(f32::INFINITY, f32::min);
    let max_x = projected
        .iter()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = projected.iter().map(|p| p.y).fold(f32::INFINITY, f32::min);
    let max_y = projected
        .iter()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let padding = size as f32 * 0.08;
    let drawable = size as f32 - padding * 2.0;
    let scale = (drawable / (max_x - min_x).max(0.001)).min(drawable / (max_y - min_y).max(0.001));
    let offset_x = (size as f32 - (max_x - min_x) * scale) * 0.5 - min_x * scale;
    let offset_y = (size as f32 - (max_y - min_y) * scale) * 0.5 - min_y * scale;
    for point in &mut projected {
        point.x = point.x * scale + offset_x;
        point.y = point.y * scale + offset_y;
    }

    let mut depth = vec![f32::NEG_INFINITY; (size * size) as usize];
    let light = Vector3::new(-0.45, 1.0, -0.35).normalize();
    for (triangle_index, triangle) in model.vertices.chunks_exact(3).enumerate() {
        let points = &projected[triangle_index * 3..triangle_index * 3 + 3];
        let normal =
            (triangle[1].coord - triangle[0].coord).cross(triangle[2].coord - triangle[0].coord);
        let brightness = if normal.magnitude2() > f32::EPSILON {
            0.58 + 0.42 * normal.normalize().dot(light).abs()
        } else {
            0.75
        };
        rasterize_triangle(
            &mut image,
            &mut depth,
            points,
            triangle[0].tex_id,
            brightness,
            atlas,
            atlas_width,
            atlas_height,
        );
    }
    image
}

#[allow(clippy::too_many_arguments)]
fn rasterize_triangle(
    image: &mut RgbaImage,
    depth_buffer: &mut [f32],
    points: &[ProjectedVertex],
    texture_id: i16,
    brightness: f32,
    atlas: &[u8],
    atlas_width: usize,
    atlas_height: usize,
) {
    let area = edge(points[0], points[1], points[2].x, points[2].y);
    if area.abs() < 0.0001 {
        return;
    }
    let width = image.width() as i32;
    let height = image.height() as i32;
    let min_x = points
        .iter()
        .map(|p| p.x.floor() as i32)
        .min()
        .unwrap_or(0)
        .clamp(0, width - 1);
    let max_x = points
        .iter()
        .map(|p| p.x.ceil() as i32)
        .max()
        .unwrap_or(0)
        .clamp(0, width - 1);
    let min_y = points
        .iter()
        .map(|p| p.y.floor() as i32)
        .min()
        .unwrap_or(0)
        .clamp(0, height - 1);
    let max_y = points
        .iter()
        .map(|p| p.y.ceil() as i32)
        .max()
        .unwrap_or(0)
        .clamp(0, height - 1);

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let w0 = edge(points[1], points[2], px, py) / area;
            let w1 = edge(points[2], points[0], px, py) / area;
            let w2 = 1.0 - w0 - w1;
            if w0 < -0.0001 || w1 < -0.0001 || w2 < -0.0001 {
                continue;
            }
            let pixel_depth = w0 * points[0].depth + w1 * points[1].depth + w2 * points[2].depth;
            let index = y as usize * image.width() as usize + x as usize;
            if pixel_depth <= depth_buffer[index] {
                continue;
            }
            depth_buffer[index] = pixel_depth;
            let uv = points[0].uv * w0 + points[1].uv * w1 + points[2].uv * w2;
            let mut color = sample_texture(texture_id, uv, atlas, atlas_width, atlas_height);
            for channel in &mut color.0[..3] {
                *channel = (*channel as f32 * brightness).clamp(0.0, 255.0) as u8;
            }
            color.0[3] = 255;
            image.put_pixel(x as u32, y as u32, color);
        }
    }
}

fn sample_texture(
    texture_id: i16,
    uv: Vector2<f32>,
    atlas: &[u8],
    atlas_width: usize,
    atlas_height: usize,
) -> Rgba<u8> {
    if texture_id < 0 || texture_id as usize >= TEXTURE_COLUMNS * TEXTURE_ROWS {
        return Rgba([155, 145, 120, 255]);
    }
    let cell_width = atlas_width / TEXTURE_COLUMNS;
    let cell_height = atlas_height / TEXTURE_ROWS;
    if cell_width == 0 || cell_height == 0 {
        return Rgba([155, 145, 120, 255]);
    }
    let texture_id = texture_id as usize;
    let column = texture_id % TEXTURE_COLUMNS;
    let row = texture_id / TEXTURE_COLUMNS;
    let u = uv.x.clamp(0.0, 0.999_999);
    let v = uv.y.clamp(0.0, 0.999_999);
    let x = column * cell_width + (u * cell_width as f32) as usize;
    let y = row * cell_height + (v * cell_height as f32) as usize;
    let index = (y * atlas_width + x) * 4;
    if index + 3 >= atlas.len() {
        return Rgba([155, 145, 120, 255]);
    }
    Rgba([atlas[index], atlas[index + 1], atlas[index + 2], 255])
}

fn edge(a: ProjectedVertex, b: ProjectedVertex, x: f32, y: f32) -> f32 {
    (x - a.x) * (b.y - a.y) - (y - a.y) * (b.x - a.x)
}

pub(crate) fn make_contact_sheet(icons: &[(String, RgbaImage)], icon_size: u32) -> RgbaImage {
    let columns = 4u32;
    let rows = (icons.len() as u32).div_ceil(columns);
    let tile_width = icon_size + 20;
    let tile_height = icon_size + 42;
    let mut sheet = RgbaImage::from_pixel(
        columns * tile_width,
        rows * tile_height,
        Rgba([24, 24, 27, 255]),
    );
    for (index, (label, icon)) in icons.iter().enumerate() {
        let column = index as u32 % columns;
        let row = index as u32 / columns;
        let tile_x = column * tile_width;
        let tile_y = row * tile_height;
        draw_border(
            &mut sheet,
            tile_x + 4,
            tile_y + 4,
            tile_width - 8,
            tile_height - 8,
        );
        imageops::overlay(
            &mut sheet,
            icon,
            (tile_x + (tile_width - icon_size) / 2) as i64,
            (tile_y + 8) as i64,
        );
        draw_centered_label(
            &mut sheet,
            label,
            tile_x + 8,
            tile_y + icon_size + 14,
            tile_width - 16,
        );
    }
    sheet
}

fn draw_border(image: &mut RgbaImage, x: u32, y: u32, width: u32, height: u32) {
    let color = Rgba([111, 91, 50, 255]);
    for px in x..x + width {
        image.put_pixel(px, y, color);
        image.put_pixel(px, y + height - 1, color);
    }
    for py in y..y + height {
        image.put_pixel(x, py, color);
        image.put_pixel(x + width - 1, py, color);
    }
}

fn draw_centered_label(image: &mut RgbaImage, label: &str, x: u32, y: u32, width: u32) {
    let max_chars = (width / 8).max(1) as usize;
    let words: Vec<&str> = label.split_whitespace().collect();
    let mut lines = vec![String::new()];
    for word in words {
        let should_wrap = {
            let line = lines.last().unwrap();
            let extra = usize::from(!line.is_empty()) + word.len();
            !line.is_empty() && line.len() + extra > max_chars && lines.len() < 2
        };
        if should_wrap {
            lines.push(word.to_string());
        } else {
            let line = lines.last_mut().unwrap();
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
    }
    for (line_index, line) in lines.iter().take(2).enumerate() {
        let line_width = line.len() as u32 * 8;
        let line_x = x + width.saturating_sub(line_width) / 2;
        draw_text(
            image,
            line,
            line_x,
            y + line_index as u32 * 10,
            Rgba([232, 225, 204, 255]),
        );
    }
}

fn draw_text(image: &mut RgbaImage, text: &str, x: u32, y: u32, color: Rgba<u8>) {
    for (char_index, byte) in text.bytes().enumerate() {
        if !(32..128).contains(&byte) {
            continue;
        }
        let glyph = FONT_8X8[(byte - 32) as usize];
        for (row, bits) in glyph.into_iter().enumerate() {
            for column in 0..8u32 {
                if bits & (0x80 >> column) == 0 {
                    continue;
                }
                let px = x + char_index as u32 * 8 + column;
                let py = y + row as u32;
                if px < image.width() && py < image.height() {
                    image.put_pixel(px, py, color);
                }
            }
        }
    }
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

fn slug(value: &str) -> String {
    value.to_ascii_lowercase().replace(' ', "-")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::tex_model::TexVertex;

    #[test]
    fn structure_catalog_maps_all_hut_families() {
        let specs = structure_icon_specs(Tribe::Blue);
        assert_eq!(specs.len(), 16);
        assert_eq!(specs[0].object_index, 145);
        assert_eq!(specs[1].object_index, 157);
        assert_eq!(specs[2].object_index, 169);
        assert_eq!(specs[3].object_index, 146);
        assert_eq!(specs[6].object_index, 147);
    }

    #[test]
    fn structure_catalog_offsets_tribal_models() {
        let specs = structure_icon_specs(Tribe::Red);
        assert_eq!(specs[0].object_index, 148);
        assert_eq!(specs[9].object_index, 118);
        assert_eq!(specs.last().unwrap().object_index, 126);
    }

    #[test]
    fn rasterizer_draws_a_textured_triangle() {
        let mut model = TexModel::new();
        model.vertices = vec![
            TexVertex {
                coord: Vector3::new(-1.0, 0.0, 0.0),
                uv: Vector2::new(0.0, 0.0),
                tex_id: 0,
            },
            TexVertex {
                coord: Vector3::new(1.0, 0.0, 0.0),
                uv: Vector2::new(1.0, 0.0),
                tex_id: 0,
            },
            TexVertex {
                coord: Vector3::new(0.0, 1.0, 0.0),
                uv: Vector2::new(0.5, 1.0),
                tex_id: 0,
            },
        ];
        let atlas = vec![200u8; 256 * 1024 * 4];
        let image = render_model_icon(&model, &atlas, 256, 1024, 96);
        assert!(image.pixels().filter(|pixel| pixel.0[3] != 0).count() > 500);
    }
}
