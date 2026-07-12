use std::iter::Zip;
use std::ops::RangeFrom;
use std::slice::Chunks;

use cgmath::{Vector2, Vector3, Vector4};

use crate::data::objects::{Shape, ShapeFootprints};
use crate::render::envelop::{GpuModel, ModelEnvelop, RenderType};
use crate::render::model::{MeshModel, Triangle, VertexModel};

pub type LandscapeModel = MeshModel<Vector2<u8>, u16>;

impl GpuModel for LandscapeModel {
    fn vertex_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        // wgpu requires vertex stride to be a multiple of 4 (VERTEX_ALIGNMENT)
        vec![wgpu::VertexBufferLayout {
            array_stride: 4,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint8x2,
                offset: 0,
                shader_location: 0,
            }],
        }]
    }

    fn vertex_data(&self) -> Vec<u8> {
        // Pad each 2-byte vertex to 4 bytes for alignment
        self.vertices
            .iter()
            .flat_map(|v| [v.x, v.y, 0, 0])
            .collect()
    }

    fn index_data(&self) -> Option<Vec<u8>> {
        None
    }

    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }

    fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    fn index_count(&self) -> u32 {
        0
    }

    fn is_indexed(&self) -> bool {
        false
    }
}

pub struct LandscapeMesh<const N: usize> {
    vertices: Vec<Vector2<u8>>,
    step: f32,
    height_scale: f32,
    shift_x: usize,
    shift_y: usize,
    heights: [[u16; N]; N],
}

impl<const N: usize> LandscapeMesh<N> {
    fn gen_mesh() -> Vec<Vector2<u8>> {
        let vertices_num: usize = N * N * 6;
        let mut vertices = vec![Vector2 { x: 0, y: 0 }; vertices_num];
        for i in 0..(N - 1) {
            for j in 0..(N - 1) {
                let index = (i * N + j) * 6;
                let i_u8 = i as u8;
                let j_u8 = j as u8;
                vertices[index] = Vector2 { x: i_u8, y: j_u8 };
                vertices[index + 1] = Vector2 {
                    x: i_u8,
                    y: j_u8 + 1,
                };
                vertices[index + 2] = Vector2 {
                    x: i_u8 + 1,
                    y: j_u8,
                };
                vertices[index + 3] = Vector2 {
                    x: i_u8 + 1,
                    y: j_u8 + 1,
                };
                vertices[index + 4] = Vector2 {
                    x: i_u8,
                    y: j_u8 + 1,
                };
                vertices[index + 5] = Vector2 {
                    x: i_u8 + 1,
                    y: j_u8,
                };
            }
        }
        vertices
    }

    fn shift_n(n: usize, max_n: usize, shift: i32) -> usize {
        let shift_n = (n as i32) + shift;
        if shift_n >= 0 {
            (shift_n as usize) % max_n
        } else {
            (((max_n as i32) + shift_n) as usize) % max_n
        }
    }

    pub fn width(&self) -> usize {
        N
    }

    pub fn new(step: f32, height_scale: f32) -> Self {
        let vertices = Self::gen_mesh();
        Self {
            vertices,
            step,
            height_scale,
            shift_x: 0,
            shift_y: 0,
            heights: [[0u16; N]; N],
        }
    }

    pub fn set_heights(&mut self, heights: &[[u16; N]; N]) {
        self.heights = *heights;
    }

    pub fn height_at(&self, x: usize, y: usize) -> u16 {
        self.heights[y % N][x % N]
    }

    pub fn set_height_at(&mut self, x: usize, y: usize, h: u16) {
        self.heights[y % N][x % N] = h;
    }

    /// Bilinear height interpolation at fractional cell position.
    /// cell_x/cell_y are in cell units (0.0 to N-1). Handles toroidal wrapping.
    /// Returns height in world units (already multiplied by height_scale).
    ///
    /// Matches the original game's Terrain_GetHeightAtPoint (0x004E8E50).
    /// Uses "/" diagonal split consistent with gen_mesh() triangle layout.
    pub fn interpolate_height_at(&self, cell_x: f32, cell_y: f32) -> f32 {
        let ix = cell_x.floor() as i32;
        let iy = cell_y.floor() as i32;
        let fx = cell_x - ix as f32; // fractional 0..1
        let fy = cell_y - iy as f32;

        let x0 = ((ix % N as i32 + N as i32) as usize) % N;
        let y0 = ((iy % N as i32 + N as i32) as usize) % N;
        let x1 = (x0 + 1) % N;
        let y1 = (y0 + 1) % N;

        let h00 = self.heights[y0][x0] as f32;
        let h10 = self.heights[y0][x1] as f32;
        let h01 = self.heights[y1][x0] as f32;
        let h11 = self.heights[y1][x1] as f32;

        // "/" diagonal split: same as gen_mesh() triangle layout
        let h = if fx + fy <= 1.0 {
            // Lower-left triangle: (0,0)-(0,1)-(1,0)
            h00 + (h10 - h00) * fx + (h01 - h00) * fy
        } else {
            // Upper-right triangle: (1,1)-(0,1)-(1,0)
            h11 + (h01 - h11) * (1.0 - fx) + (h10 - h11) * (1.0 - fy)
        };

        h * self.height_scale
    }

    /// Compute the curved terrain surface height at a visual grid position,
    /// matching the GPU shader: curvature is applied per-vertex BEFORE interpolation.
    ///
    /// `vis_x, vis_y`: visual grid position (0..N), same as the shader's `in.coord_in`.
    /// The shader uses the raw vertex index for position/curvature and the shifted
    /// index for height lookup.
    pub fn curved_height_at(&self, vis_x: f32, vis_y: f32, curvature_scale: f32) -> f32 {
        let step = self.step;
        let w = N as f32;
        let center = (w - 1.0) * step / 2.0;
        let sx = self.shift_x;
        let sy = self.shift_y;

        let ix = vis_x.floor() as i32;
        let iy = vis_y.floor() as i32;
        let fx = vis_x - ix as f32;
        let fy = vis_y - iy as f32;

        // Visual vertex indices (grid positions for curvature)
        let vx0 = ((ix % N as i32 + N as i32) as usize) % N;
        let vy0 = ((iy % N as i32 + N as i32) as usize) % N;
        let vx1 = (vx0 + 1) % N;
        let vy1 = (vy0 + 1) % N;

        // Apply curvature per-vertex matching the shader:
        //   coord3d.x = vertex_index * step  (visual position, for curvature)
        //   height = heights[(vertex_index + shift) % w]  (shifted, for height)
        let curv = |vx: usize, vy: usize| -> f32 {
            let hx = (vx + sx) % N;
            let hy = (vy + sy) % N;
            let h = self.heights[hy][hx] as f32 * self.height_scale;
            let gx = vx as f32 * step;
            let gy = vy as f32 * step;
            let dx = gx - center;
            let dy = gy - center;
            h - (dx * dx + dy * dy) * curvature_scale
        };

        let c00 = curv(vx0, vy0);
        let c10 = curv(vx1, vy0);
        let c01 = curv(vx0, vy1);
        let c11 = curv(vx1, vy1);

        // "/" diagonal split — same as gen_mesh() and interpolate_height_at
        if fx + fy <= 1.0 {
            c00 + (c10 - c00) * fx + (c01 - c00) * fy
        } else {
            c11 + (c01 - c11) * (1.0 - fx) + (c10 - c11) * (1.0 - fy)
        }
    }

    /// Flatten terrain under a building footprint.
    ///
    /// Matches Building_FlattenTerrain (0x0042F2A0):
    /// 1. Samples all cell heights in footprint
    /// 2. Computes average (or min if use_average=false)
    /// 3. Writes uniform height to all footprint cells
    /// 4. Smooths surrounding terrain
    pub fn flatten_building_footprint(
        &mut self,
        cell_x: i32,
        cell_y: i32, // building center cell
        shape: &Shape,
        shape_idx: usize,
        footprints: &ShapeFootprints,
        use_average: bool,
    ) {
        let w = shape.width as i32;
        let h = shape.height as i32;
        if w == 0 || h == 0 {
            return;
        }
        let ni = N as i32;

        // Corner = center - origin (signed, clipped to map bounds)
        // Origin is in tile units (2 per cell), convert to cell units
        let base_cx = cell_x - shape.origin_x as i32 / 2;
        let base_cy = cell_y - shape.origin_z as i32 / 2;

        // Pass 1: Sample heights and compute target
        let mut sum: u64 = 0;
        let mut min_h: u16 = u16::MAX;
        let mut count: u32 = 0;
        for dy in 0..h {
            for dx in 0..w {
                if footprints.is_cell_occupied(shape_idx, dx as usize, dy as usize) {
                    let cx = ((base_cx + dx) % ni + ni) % ni;
                    let cy = ((base_cy + dy) % ni + ni) % ni;
                    let cx = cx as usize;
                    let cy = cy as usize;
                    let cx1 = (cx + 1) % N;
                    let cy1 = (cy + 1) % N;
                    let h00 = self.heights[cy][cx];
                    let h10 = self.heights[cy][cx1];
                    let h01 = self.heights[cy1][cx];
                    let h11 = self.heights[cy1][cx1];
                    sum += h00 as u64 + h10 as u64 + h01 as u64 + h11 as u64;
                    min_h = min_h.min(h00).min(h10).min(h01).min(h11);
                    count += 4;
                }
            }
        }

        if count == 0 {
            return;
        }

        let target = if use_average {
            (sum / count as u64) as u16
        } else {
            min_h
        };
        let target = target.max(1);

        // Pass 2: Write target height to all footprint cells
        for dy in 0..h {
            for dx in 0..w {
                if footprints.is_cell_occupied(shape_idx, dx as usize, dy as usize) {
                    let cx = ((base_cx + dx) % ni + ni) % ni;
                    let cy = ((base_cy + dy) % ni + ni) % ni;
                    let cx = cx as usize;
                    let cy = cy as usize;
                    let cx1 = (cx + 1) % N;
                    let cy1 = (cy + 1) % N;
                    self.heights[cy][cx] = target;
                    self.heights[cy][cx1] = target;
                    self.heights[cy1][cx] = target;
                    self.heights[cy1][cx1] = target;
                }
            }
        }

        // Pass 3: Smooth surrounding terrain
        let radius = (w.max(h) / 2 + 1) as usize;
        self.smooth_terrain_area(
            cell_x, cell_y, radius, base_cx, base_cy, w, h, shape_idx, footprints,
        );
    }

    /// Smooth terrain transitions around a flattened area.
    /// Averages heights at border cells to create gradual transitions.
    fn smooth_terrain_area(
        &mut self,
        center_x: i32,
        center_y: i32,
        radius: usize,
        base_cx: i32,
        base_cy: i32,
        fp_w: i32,
        fp_h: i32,
        shape_idx: usize,
        footprints: &ShapeFootprints,
    ) {
        let r = radius as i32 + 1;
        let ni = N as i32;

        for dy in -r..=r {
            for dx in -r..=r {
                let cx = ((center_x + dx) % ni + ni) % ni;
                let cy = ((center_y + dy) % ni + ni) % ni;

                // Skip cells inside the footprint
                let rel_x = cx - ((base_cx % ni + ni) % ni);
                let rel_y = cy - ((base_cy % ni + ni) % ni);
                if rel_x >= 0 && rel_y >= 0 && rel_x < fp_w && rel_y < fp_h {
                    if footprints.is_cell_occupied(shape_idx, rel_x as usize, rel_y as usize) {
                        continue;
                    }
                }

                let cx = cx as usize;
                let cy = cy as usize;
                let cx1 = (cx + 1) % N;
                let cy1 = (cy + 1) % N;
                let cxm = (cx + N - 1) % N;
                let cym = (cy + N - 1) % N;

                let h_center = self.heights[cy][cx] as u32;
                let h_right = self.heights[cy][cx1] as u32;
                let h_left = self.heights[cy][cxm] as u32;
                let h_down = self.heights[cy1][cx] as u32;
                let h_up = self.heights[cym][cx] as u32;

                let avg = (h_center * 4 + h_right + h_left + h_down + h_up) / 8;
                self.heights[cy][cx] = avg as u16;
            }
        }
    }

    pub fn heights(&self) -> &[[u16; N]; N] {
        &self.heights
    }

    /// Export heights as Vec<u32> for GPU buffer upload (matches Landscape::to_vec format).
    pub fn heights_to_gpu_vec(&self) -> Vec<u32> {
        let mut vec = vec![0u32; N * N];
        for i in 0..N {
            for j in 0..N {
                vec[i * N + j] = self.heights[i][j] as u32;
            }
        }
        vec
    }

    pub fn step(&self) -> f32 {
        self.step
    }

    pub fn height_scale(&self) -> f32 {
        self.height_scale
    }

    pub fn shift_x(&mut self, shift: i32) -> usize {
        self.shift_x = Self::shift_n(self.shift_x, N, shift);
        self.shift_x
    }

    pub fn shift_y(&mut self, shift: i32) -> usize {
        self.shift_y = Self::shift_n(self.shift_y, N, shift);
        self.shift_y
    }

    pub fn set_shift(&mut self, sx: usize, sy: usize) {
        self.shift_x = sx % N;
        self.shift_y = sy % N;
    }

    pub fn get_shift_vector(&self) -> Vector4<i32> {
        Vector4::new(self.shift_x as i32, self.shift_y as i32, 0, 0)
    }

    pub fn to_model(&self, m: &mut LandscapeModel) {
        for v2 in &self.vertices {
            m.push_vertex(*v2);
        }
    }

    pub fn iter(&self) -> LandscapeTriangleIterator<N> {
        let iter_internal = (0..).zip(self.vertices.chunks(3));
        LandscapeTriangleIterator {
            landscape: self,
            iter_internal,
        }
    }

    fn make_vec3(&self, v: &Vector2<u8>) -> Vector3<f32> {
        let x = v.x as f32 * self.step;
        let y = v.y as f32 * self.step;
        let index_x = (v.x as usize + self.shift_x) % N;
        let index_y = (v.y as usize + self.shift_y) % N;
        let z = self.heights[index_y][index_x] as f32 * self.height_scale;
        Vector3 { x, y, z }
    }
}

pub struct LandscapeTriangleIterator<'a, const N: usize> {
    landscape: &'a LandscapeMesh<N>,
    iter_internal: Zip<RangeFrom<usize>, Chunks<'a, Vector2<u8>>>,
}

impl<'a, const N: usize> Iterator for LandscapeTriangleIterator<'a, N> {
    type Item = (usize, Triangle<f32>);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter_internal.next() {
            Some((n, c)) => {
                if c.len() != 3 {
                    return None;
                }
                let a = self.landscape.make_vec3(&c[0]);
                let b = self.landscape.make_vec3(&c[1]);
                let c = self.landscape.make_vec3(&c[2]);
                let t: Triangle<f32> = Triangle { a, b, c };
                Some((n, t))
            }
            None => None,
        }
    }
}

/******************************************************************************/

/// Landscape model transform: world = LANDSCAPE_SCALE * model + LANDSCAPE_OFFSET.
pub const LANDSCAPE_SCALE: f32 = 2.5;
pub const LANDSCAPE_OFFSET: f32 = -2.0;

/// Packed landscape uniform data matching the WGSL LandscapeParams struct.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LandscapeUniformData {
    pub level_shift: [i32; 4],
    pub height_scale: f32,
    pub step: f32,
    pub width: i32,
    pub _pad_width: i32,
    pub sunlight: [f32; 4],
    pub wat_offset: i32,
    pub curvature_scale: f32,
    pub camera_focus: [f32; 2],
    pub viewport_radius: f32,
    pub _pad2: [f32; 3],
}

/// A landscape program variant with its own pipeline and group-1 bind group.
pub struct LandscapeVariant {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_1: wgpu::BindGroup,
}

pub struct LandscapeProgramContainer {
    variants: Vec<LandscapeVariant>,
    index: usize,
}

impl LandscapeProgramContainer {
    pub fn new() -> Self {
        Self {
            variants: Vec::new(),
            index: 0,
        }
    }

    pub fn add(&mut self, variant: LandscapeVariant) {
        self.variants.push(variant);
    }

    pub fn next(&mut self) {
        if !self.variants.is_empty() {
            self.index = (self.index + 1) % self.variants.len();
        }
    }

    pub fn prev(&mut self) {
        if self.variants.is_empty() {
            return;
        }
        self.index = if self.index == 0 {
            self.variants.len() - 1
        } else {
            self.index - 1
        };
    }

    pub fn current(&self) -> Option<&LandscapeVariant> {
        self.variants.get(self.index)
    }
}

pub fn make_landscape_model<const N: usize>(
    device: &wgpu::Device,
    landscape_mesh: &LandscapeMesh<N>,
) -> ModelEnvelop<LandscapeModel> {
    let mut model: LandscapeModel = MeshModel::new();
    landscape_mesh.to_model(&mut model);
    log::debug!(
        "Landscape mesh - vertices={:?}, indices={:?}",
        model.vertices.len(),
        model.indices.len()
    );
    let m = vec![(RenderType::Triangles, model)];
    let mut model_main = ModelEnvelop::<LandscapeModel>::new(device, m);
    if let Some(m) = model_main.get(0) {
        m.location.x = LANDSCAPE_OFFSET;
        m.location.y = LANDSCAPE_OFFSET;
        m.scale = LANDSCAPE_SCALE;
    }
    eprintln!(
        "[landscape] model transform: location=({0},{0},0) scale={1}",
        LANDSCAPE_OFFSET, LANDSCAPE_SCALE
    );
    model_main
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Compare sprite z_base computation against the terrain shader's curved surface.
    ///
    /// Sprite method: interpolate raw height at cell coords, then subtract curvature
    ///   using visual position: vis = (cell - shift) % w.
    /// Shader method: apply curvature per-vertex at visual position, height from
    ///   shifted index, then GPU interpolates curved vertices.
    ///
    /// Tests both shift=0 and shift≠0 to catch coordinate mapping bugs.
    #[test]
    fn sprite_vs_terrain_curvature_drift() {
        let mut mesh = LandscapeMesh::<128>::new(0.0625, 0.0004);

        // Create a sloped terrain (height increases with x and y)
        let mut heights = [[0u16; 128]; 128];
        for y in 0..128 {
            for x in 0..128 {
                heights[y][x] = (x as u16) * 4 + (y as u16) * 2;
            }
        }
        mesh.set_heights(&heights);

        let step = mesh.step();
        let w = 128.0_f32;
        let center = (w - 1.0) * step / 2.0;
        let curvature_scale: f32 = 0.0512;

        // Test cells (absolute cell positions)
        let test_cells: Vec<(f32, f32)> = vec![
            (64.0, 64.0),   // near center (when shift=0)
            (80.5, 64.5),   // moderate distance
            (100.5, 100.5), // far
            (120.5, 120.5), // very far (edge)
            (10.5, 10.5),   // other edge
            (64.5, 120.5),  // far in one axis
        ];

        // Test with multiple shift values
        let shifts: Vec<(usize, usize)> = vec![
            (0, 0),     // no shift
            (77, 33),   // typical game shift
            (120, 120), // large shift near wrap
        ];

        let mut overall_max_delta: f32 = 0.0;

        for (sx, sy) in &shifts {
            mesh.set_shift(*sx, *sy);
            eprintln!("\n--- shift=({},{}) ---", sx, sy);
            eprintln!(
                "{:<20} {:>8} {:>10} {:>10} {:>10} {:>10}",
                "cell(vis)", "dist", "sprite_z", "shader_z", "delta", "delta_px"
            );

            for (cx, cy) in &test_cells {
                // Sprite code: vis = (cell - shift) % w
                let vis_x = ((*cx - *sx as f32) % w + w) % w;
                let vis_y = ((*cy - *sy as f32) % w + w) % w;
                let gx = vis_x * step;
                let gy = vis_y * step;

                // Sprite method: interpolate height at cell coords, curvature at visual pos
                let gz = mesh.interpolate_height_at(*cx, *cy);
                let dx = gx - center;
                let dy = gy - center;
                let curv_off = (dx * dx + dy * dy) * curvature_scale;
                let sprite_z = gz - curv_off;

                // Shader method: curvature per-vertex at visual pos, then interpolate
                let shader_z = mesh.curved_height_at(vis_x, vis_y, curvature_scale);

                let delta = sprite_z - shader_z;
                let delta_px = delta * LANDSCAPE_SCALE * 600.0;
                overall_max_delta = overall_max_delta.max(delta.abs());

                let dist = (dx * dx + dy * dy).sqrt();
                eprintln!("({:>5.1},{:>5.1})({:>5.1},{:>5.1}) {:>6.2} {:>10.6} {:>10.6} {:>10.6} {:>8.1}px",
                    cx, cy, vis_x, vis_y, dist, sprite_z, shader_z, delta, delta_px);
            }
        }

        eprintln!(
            "\noverall max delta = {:.6} ({:.1}px)",
            overall_max_delta,
            overall_max_delta * LANDSCAPE_SCALE * 600.0
        );

        // If delta > ~0.001 (≈1.5px), sprites visibly float.
        assert!(overall_max_delta < 0.1, "curvature drift sanity check");
    }
}
