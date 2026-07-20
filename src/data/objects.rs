use core::mem::size_of;
use core::slice::Iter;
use std::collections::BTreeSet;
use std::io::Read;
use std::path::Path;

use cgmath::{InnerSpace, Vector2, Vector3};

use crate::data::level::ObjectPaths;
use crate::data::types::{from_reader, BinDeserializer};
use crate::render::model::{MeshModel, VertexModel};
use crate::render::tex_model::{TexModel, TexVertex};

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct ObjectRaw {
    flags: u16,
    facs_num: u16,
    pnts_num: u16,
    f1: u8,
    morph_index: u8,
    f2: u32,
    coord_scale: u32,
    facs_ptr: u32,
    facs_ptr_end: u32,
    pnts_ptr: u32,
    pnts_ptr_end: u32,
    f4: i16,
    f5: i16,
    f6: i16,
    f7: u16,
    f8: u16,
    f9: u16,
    fp_idx: [i8; 4], // SHAPES.DAT footprint index per rotation (0-3), at OBJS offset 0x2c
    f11: u16,
    f12: u16,
    f13: u16,
}

impl BinDeserializer for ObjectRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<ObjectRaw, { size_of::<ObjectRaw>() }, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct Shape {
    pub width: u8,
    pub height: u8,
    pub origin_x: u8,
    pub origin_z: u8,
    pub cell_mask: [u8; 40],
    pub shape_ref: u32,
}

impl BinDeserializer for Shape {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<Self, { size_of::<Self>() }, R>(reader)
    }
}

/******************************************************************************/

/// Parsed building footprint data from SHAPES.DAT.
/// File layout: [64 entries × 48 bytes] [1532 bytes bitmap data].
/// Each bitmap byte's bit 0 indicates whether the cell is occupied.
pub struct ShapeFootprints {
    shapes: Vec<Shape>,
    bitmap_data: Vec<u8>,
}

/// Number of valid shape entries in SHAPES.DAT (entries 64+ have garbage shape_ref).
const SHAPE_ENTRY_COUNT: usize = 64;

impl ShapeFootprints {
    pub fn empty() -> Self {
        ShapeFootprints {
            shapes: Vec::new(),
            bitmap_data: Vec::new(),
        }
    }

    pub fn from_file(path: &Path) -> Self {
        let data = std::fs::read(path).unwrap();
        let entry_bytes = SHAPE_ENTRY_COUNT * size_of::<Shape>();
        let mut shapes = Vec::with_capacity(SHAPE_ENTRY_COUNT);
        let mut cursor = std::io::Cursor::new(&data[..entry_bytes]);
        for _ in 0..SHAPE_ENTRY_COUNT {
            if let Some(s) = Shape::from_reader(&mut cursor) {
                shapes.push(s);
            }
        }
        let bitmap_data = data[entry_bytes..].to_vec();
        ShapeFootprints {
            shapes,
            bitmap_data,
        }
    }

    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }

    /// Check if cell (dx, dy) within a shape's bounding box is actually occupied.
    /// Returns false for out-of-bounds or empty bitmap cells.
    pub fn is_cell_occupied(&self, shape_idx: usize, dx: usize, dy: usize) -> bool {
        if let Some(s) = self.shapes.get(shape_idx) {
            let w = s.width as usize;
            let offset = s.shape_ref as usize + dy * w + dx;
            offset < self.bitmap_data.len() && (self.bitmap_data[offset] & 1) != 0
        } else {
            false
        }
    }

    pub fn occupied_offsets(&self, shape_idx: usize) -> Option<Vec<(i16, i16)>> {
        let shape = self.shapes.get(shape_idx)?;
        let origin_x = (shape.origin_x / 2) as i16;
        let origin_z = (shape.origin_z / 2) as i16;
        let mut result = Vec::new();
        for dy in 0..shape.height as usize {
            for dx in 0..shape.width as usize {
                if self.is_cell_occupied(shape_idx, dx, dy) {
                    result.push((dx as i16 - origin_x, dy as i16 - origin_z));
                }
            }
        }
        Some(result)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct PointRaw {
    x: i16,
    y: i16,
    z: i16,
}

impl BinDeserializer for PointRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<Self, { size_of::<Self>() }, R>(reader)
    }
}

/******************************************************************************/

#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct FaceRaw {
    f0: u16,
    tex_index: i16,
    flags1: i16,
    num_points: u8,
    f11: u8,
    point_1_u: u32,
    point_1_v: u32,
    point_2_u: u32,
    point_2_v: u32,
    point_3_u: u32,
    point_3_v: u32,
    point_4_u: u32,
    point_4_v: u32,
    point_1: u16,
    point_2: u16,
    point_3: u16,
    point_4: u16,
    f6: u16,
    ff1: u16,
    ff2: u16,
    ff3: u16,
    ff4: u16,
    f8: u8,
    flags2: u8,
}

impl BinDeserializer for FaceRaw {
    fn from_reader<R: Read>(reader: &mut R) -> Option<Self> {
        from_reader::<Self, { size_of::<Self>() }, R>(reader)
    }
}

/******************************************************************************/

const XYZ_SCALE: f32 = 1.0 / 300.0;
const UV_SCALE: f32 = 4.768372e-7;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
}

impl Vertex {
    pub fn new() -> Self {
        Vertex {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            u: 0.0,
            v: 0.0,
        }
    }

    pub fn from_point(&mut self, point: &PointRaw, u: u32, v: u32) {
        self.x = point.x as f32 * XYZ_SCALE;
        self.y = point.y as f32 * XYZ_SCALE;
        self.z = point.z as f32 * XYZ_SCALE;
        self.u = u as f32 * UV_SCALE;
        self.v = v as f32 * UV_SCALE;
    }
}

impl Default for Vertex {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Face {
    pub texture_index: i16,
    pub vertex_num: usize,
    pub vertex: [Vertex; 4],
    /// Construction-phase visibility/material mask from the 60-byte face record.
    pub flags2: u8,
}

impl Face {
    pub fn new(texture_index: i16, vertex_num: usize, flags2: u8) -> Self {
        Face {
            texture_index,
            vertex_num,
            vertex: [Vertex::default(); 4],
            flags2,
        }
    }
}

/******************************************************************************/

#[derive(Debug)]
pub struct Object3D {
    object: ObjectRaw,
    faces: Vec<FaceRaw>,
    points: Vec<PointRaw>,
}

impl Object3D {
    pub fn create(object: &ObjectRaw, faces: &[FaceRaw], points: &[PointRaw]) -> Self {
        let mut object_3d = Object3D {
            object: *object,
            faces: Vec::new(),
            points: Vec::new(),
        };
        for i in object.pnts_ptr..object.pnts_ptr_end {
            object_3d.points.push(points[i as usize - 1]);
        }
        for i in object.facs_ptr..object.facs_ptr_end {
            object_3d.faces.push(faces[i as usize - 1]);
        }
        object_3d
    }

    pub fn create_objects(
        objects: &[ObjectRaw],
        faces: &[FaceRaw],
        points: &[PointRaw],
    ) -> Vec<Self> {
        let mut objects_3d = Vec::new();
        for object in objects {
            if object.facs_num > 0 {
                objects_3d.push(Self::create(object, faces, points));
            }
        }
        objects_3d
    }

    pub fn from_file(base: &Path, bank_num: &str) -> Vec<Self> {
        let paths = ObjectPaths::from_default_dir(base, bank_num);
        let objects = ObjectRaw::from_file_vec(&paths.objs0_dat);
        let points = PointRaw::from_file_vec(&paths.pnts0);
        let faces = FaceRaw::from_file_vec(&paths.facs0);
        Self::create_objects(&objects, &faces, &points)
    }

    /// Like `create_objects` but preserves file indices: returns `None` for
    /// objects with no faces instead of dropping them.
    pub fn create_objects_all(
        objects: &[ObjectRaw],
        faces: &[FaceRaw],
        points: &[PointRaw],
    ) -> Vec<Option<Self>> {
        objects
            .iter()
            .map(|object| {
                if object.facs_num > 0 {
                    Some(Self::create(object, faces, points))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn from_file_all(base: &Path, bank_num: &str) -> Vec<Option<Self>> {
        let paths = ObjectPaths::from_default_dir(base, bank_num);
        let objects = ObjectRaw::from_file_vec(&paths.objs0_dat);
        let points = PointRaw::from_file_vec(&paths.pnts0);
        let faces = FaceRaw::from_file_vec(&paths.facs0);
        Self::create_objects_all(&objects, &faces, &points)
    }

    /// Load both OBJS banks needed for a level.
    /// Bank 0 contains building models (indices 117-193 via building_obj_index).
    /// Level banks (2-8) contain scenery models at different indices.
    /// Shape_LoadBank @ 0x49b990 remaps bank 0 → 2.
    /// Returns (building_bank, scenery_bank).
    pub fn load_dual_banks(base: &Path, level_bank: u8) -> (Vec<Option<Self>>, Vec<Option<Self>>) {
        let building_bank = Self::from_file_all(base, "0");
        let scenery_bank_num = if level_bank == 0 { 2 } else { level_bank };
        let scenery_bank = Self::from_file_all(base, &scenery_bank_num.to_string());
        (building_bank, scenery_bank)
    }

    pub fn iter_face(&self) -> FaceIter<Iter<FaceRaw>> {
        FaceIter {
            iter: self.faces.iter(),
            points: &self.points,
        }
    }

    pub fn face_count(&self) -> usize {
        self.faces.len()
    }

    pub fn point_count(&self) -> usize {
        self.points.len()
    }

    pub fn coord_scale(&self) -> f32 {
        self.object.coord_scale as f32
    }

    /// Returns the SHAPES.DAT footprint index for the given rotation (0-3).
    /// Read from the OBJS entry at offset 0x2c (4 signed bytes, one per rotation).
    /// Returns i8 since negative values mean no footprint.
    pub fn footprint_index(&self, rotation: usize) -> i8 {
        self.object.fp_idx[rotation & 3]
    }
}

/******************************************************************************/

pub struct FaceIter<'a, I>
where
    I: Iterator<Item = &'a FaceRaw>,
{
    iter: I,
    points: &'a [PointRaw],
}

impl<'a, I> Iterator for FaceIter<'a, I>
where
    I: Iterator<Item = &'a FaceRaw>,
{
    type Item = Face;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(f) => {
                let num_points = std::cmp::min(f.num_points as usize, 4);
                let mut face_3d = Face::new(f.tex_index, num_points, f.flags2);
                face_3d.vertex[0].from_point(
                    &self.points[f.point_1 as usize],
                    f.point_1_u,
                    f.point_1_v,
                );
                face_3d.vertex[1].from_point(
                    &self.points[f.point_2 as usize],
                    f.point_2_u,
                    f.point_2_v,
                );
                face_3d.vertex[2].from_point(
                    &self.points[f.point_3 as usize],
                    f.point_3_u,
                    f.point_3_v,
                );
                if num_points == 4 {
                    face_3d.vertex[3].from_point(
                        &self.points[f.point_4 as usize],
                        f.point_4_u,
                        f.point_4_v,
                    );
                }
                Some(face_3d)
            }
            None => None,
        }
    }
}

/******************************************************************************/

pub fn mk_tex_vertex(tex_index: i16, v: &Vertex) -> TexVertex {
    TexVertex {
        coord: Vector3::new(v.x, v.y, v.z),
        uv: Vector2::new(v.u, v.v),
        tex_id: tex_index,
    }
}

pub fn mk_pop_object(object: &Object3D) -> TexModel {
    mk_pop_object_for_phase(object, None)
}

pub fn construction_face_visible(flags2: u8, phase: u8) -> bool {
    flags2 & (1 << phase.min(4)) != 0
}

fn push_scaffold_quad(model: &mut TexModel, corners: [Vector3<f32>; 4], tex_id: i16) {
    let uvs = [
        Vector2::new(0.0, 0.0),
        Vector2::new(1.0, 0.0),
        Vector2::new(1.0, 1.0),
        Vector2::new(0.0, 1.0),
    ];
    let base = model.vertices.len() as u16;
    for (coord, uv) in corners.into_iter().zip(uvs) {
        model.push_vertex(TexVertex { coord, uv, tex_id });
    }
    // The native transition faces are visible from either side while they
    // assemble. Emit both windings so the narrow beams behave the same way.
    model.indices.extend_from_slice(&[
        base,
        base + 1,
        base + 2,
        base,
        base + 2,
        base + 3,
        base + 2,
        base + 1,
        base,
        base + 3,
        base + 2,
        base,
    ]);
}

fn push_scaffold_beam(model: &mut TexModel, start: Vector3<f32>, end: Vector3<f32>) {
    let direction = end - start;
    if direction.magnitude2() < f32::EPSILON {
        return;
    }
    let direction = direction.normalize();
    let reference = if direction.y.abs() < 0.9 {
        Vector3::unit_y()
    } else {
        Vector3::unit_x()
    };
    // The native temporary face objects rasterize as roughly three-pixel
    // timbers at the standard gameplay zoom. A slightly wider prism keeps
    // that silhouette legible after perspective projection and filtering.
    let half_width = 0.035;
    let side = direction.cross(reference).normalize() * half_width;
    let up = direction.cross(side).normalize() * half_width;
    let a = [
        start + side + up,
        start - side + up,
        start - side - up,
        start + side - up,
    ];
    let b = [
        end + side + up,
        end - side + up,
        end - side - up,
        end + side - up,
    ];
    const SCAFFOLD_TEXTURE: i16 = 21;
    push_scaffold_quad(model, [a[0], b[0], b[1], a[1]], SCAFFOLD_TEXTURE);
    push_scaffold_quad(model, [a[1], b[1], b[2], a[2]], SCAFFOLD_TEXTURE);
    push_scaffold_quad(model, [a[2], b[2], b[3], a[3]], SCAFFOLD_TEXTURE);
    push_scaffold_quad(model, [a[3], b[3], b[0], a[0]], SCAFFOLD_TEXTURE);
    push_scaffold_quad(model, [a[3], a[2], a[1], a[0]], SCAFFOLD_TEXTURE);
    push_scaffold_quad(model, [b[0], b[1], b[2], b[3]], SCAFFOLD_TEXTURE);
}

fn scaffold_point_key(point: Vector3<f32>) -> (i32, i32, i32) {
    (
        (point.x * 10_000.0).round() as i32,
        (point.y * 10_000.0).round() as i32,
        (point.z * 10_000.0).round() as i32,
    )
}

/// Builds the bare cage shown by the original construction transition.
///
/// The cage is derived from the selected native hut family's phase-zero body
/// edges and uses its native scaffold texture. The original creates temporary
/// render-type-10 face objects; keeping the resulting beams in one mesh gives
/// the same visible silhouette without introducing replacement art.
pub fn mk_pop_object_construction_scaffold(object: &Object3D) -> TexModel {
    let mut model = MeshModel::new();
    let mut edges = BTreeSet::new();
    for face in object.iter_face() {
        if !construction_face_visible(face.flags2, 0) {
            continue;
        }
        let vertices = &face.vertex[..face.vertex_num];
        let is_hut_body = vertices.iter().all(|vertex| {
            vertex.x >= -1.15 && vertex.x <= 1.15 && vertex.z >= 0.30 && vertex.z <= 1.80
        });
        if !is_hut_body {
            continue;
        }
        for index in 0..vertices.len() {
            let start = vertices[index];
            let end = vertices[(index + 1) % vertices.len()];
            let start_key = scaffold_point_key(Vector3::new(start.x, start.y, start.z));
            let end_key = scaffold_point_key(Vector3::new(end.x, end.y, end.z));
            let key = if start_key <= end_key {
                (start_key, end_key)
            } else {
                (end_key, start_key)
            };
            if edges.insert(key) {
                push_scaffold_beam(
                    &mut model,
                    Vector3::new(start.x, start.y, start.z),
                    Vector3::new(end.x, end.y, end.z),
                );
            }
        }
    }
    model
}

/// Builds the original construction/demolition view of an OBJS mesh. A normal
/// building ignores the face masks; render type 10 shows only faces whose
/// phase bit is enabled.
pub fn mk_pop_object_for_phase(object: &Object3D, phase: Option<u8>) -> TexModel {
    let mut model: TexModel = MeshModel::new();
    for face in object.iter_face() {
        if phase.is_some_and(|phase| !construction_face_visible(face.flags2, phase)) {
            continue;
        }
        if face.vertex_num == 3 {
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[0]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[1]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[2]));
        } else {
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[0]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[1]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[2]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[2]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[3]));
            model.push_vertex(mk_tex_vertex(face.texture_index, &face.vertex[0]));
        }
    }
    model
}

#[cfg(test)]
mod tests {
    use super::{construction_face_visible, push_scaffold_beam};
    use crate::render::model::MeshModel;
    use crate::render::tex_model::TexModel;
    use cgmath::Vector3;

    #[test]
    fn construction_face_masks_select_the_current_phase_bit() {
        assert!(construction_face_visible(0b0000_0001, 0));
        assert!(!construction_face_visible(0b0000_0001, 1));
        assert!(construction_face_visible(0b0001_0100, 2));
        assert!(construction_face_visible(0b0001_0100, 4));
        assert!(!construction_face_visible(0b0000_0100, 4));
    }

    #[test]
    fn scaffold_beam_is_a_double_sided_six_face_prism() {
        let mut model: TexModel = MeshModel::new();
        push_scaffold_beam(
            &mut model,
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        );
        assert_eq!(model.vertices.len(), 24);
        assert_eq!(model.indices.len(), 72);
        assert!(model.vertices.iter().all(|vertex| vertex.tex_id == 21));
    }
}
