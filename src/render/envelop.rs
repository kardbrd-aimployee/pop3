use cgmath::{Vector3, Matrix4, Rad, Deg};

use crate::render::model::{IterTriangleModel, TriangleIteratorVector3};
use crate::render::picking::intersect_iter;
use crate::render::gpu::buffer::GpuBuffer;

pub trait GpuModel {
    /// Vertex buffer layouts for pipeline creation
    fn vertex_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>>;
    /// Vertex data as raw bytes
    fn vertex_data(&self) -> Vec<u8>;
    /// Index data as raw bytes, None if non-indexed
    fn index_data(&self) -> Option<Vec<u8>>;
    /// Index format
    fn index_format() -> wgpu::IndexFormat;
    /// Number of vertices
    fn vertex_count(&self) -> u32;
    /// Number of indices (0 if not indexed)
    fn index_count(&self) -> u32;
    /// Whether model uses indexed drawing
    fn is_indexed(&self) -> bool;
}

pub enum RenderType {
    Triangles,
    Lines,
}

pub struct EModel<M> {
    pub model: M,
    pub location: Vector3<f32>,
    pub angles: Vector3<f32>,
    pub scale: f32,
    render: RenderType,
}

impl<M> EModel<M> {
    pub fn new(model: M, render: RenderType) -> Self {
        let location = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
        let angles = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
        let scale = 1.0;
        Self { model, location, angles, scale, render }
    }

    pub fn transform(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.location)
            * Matrix4::from_angle_x(Rad::from(Deg(self.angles.x)))
            * Matrix4::from_angle_y(Rad::from(Deg(self.angles.y)))
            * Matrix4::from_angle_z(Rad::from(Deg(self.angles.z)))
            * Matrix4::from_scale(self.scale)
    }
}

pub type ModelInit<M> = (RenderType, M);

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TransformRaw {
    pub data: [[f32; 4]; 4],
}

impl From<Matrix4<f32>> for TransformRaw {
    fn from(m: Matrix4<f32>) -> Self {
        TransformRaw {
            data: [
                [m.x.x, m.x.y, m.x.z, m.x.w],
                [m.y.x, m.y.y, m.y.z, m.y.w],
                [m.z.x, m.z.y, m.z.z, m.z.w],
                [m.w.x, m.w.y, m.w.z, m.w.w],
            ],
        }
    }
}

pub struct ModelEnvelop<M> {
    models: Vec<EModel<M>>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: Option<wgpu::Buffer>,
    vertex_offsets: Vec<u64>,
    index_offsets: Vec<u64>,
}

impl<M: GpuModel> ModelEnvelop<M> {
    pub fn new(device: &wgpu::Device, models: Vec<ModelInit<M>>) -> Self {
        let models_e: Vec<EModel<M>> = models
            .into_iter()
            .map(|(render, model)| EModel::new(model, render))
            .collect();

        // Collect all vertex data and compute per-model byte offsets
        let mut all_vertex_data: Vec<u8> = Vec::new();
        let mut vertex_offsets: Vec<u64> = Vec::new();
        for e in &models_e {
            vertex_offsets.push(all_vertex_data.len() as u64);
            all_vertex_data.extend_from_slice(&e.model.vertex_data());
        }

        // Collect all index data and compute per-model byte offsets
        let mut all_index_data: Vec<u8> = Vec::new();
        let mut index_offsets: Vec<u64> = Vec::new();
        let mut has_any_index = false;
        for e in &models_e {
            index_offsets.push(all_index_data.len() as u64);
            if let Some(idx_data) = e.model.index_data() {
                has_any_index = true;
                all_index_data.extend_from_slice(&idx_data);
            }
        }

        let vertex_buffer = GpuBuffer::new_vertex(device, &all_vertex_data, "model_vertex_buffer");
        let index_buffer = if has_any_index {
            Some(GpuBuffer::new_index(device, &all_index_data, "model_index_buffer"))
        } else {
            None
        };

        ModelEnvelop {
            models: models_e,
            vertex_buffer: vertex_buffer.buffer,
            index_buffer: index_buffer.map(|b| b.buffer),
            vertex_offsets,
            index_offsets,
        }
    }

    /// Write the transform of model at `index` into the given buffer at offset 0.
    pub fn write_transform(&self, queue: &wgpu::Queue, buffer: &wgpu::Buffer, index: usize) {
        if let Some(e) = self.models.get(index) {
            let transform_raw: TransformRaw = e.transform().into();
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(&transform_raw));
        }
    }

    /// Draw all models. Caller is responsible for setting bind groups before this call.
    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for (i, e) in self.models.iter().enumerate() {
            if e.model.vertex_count() == 0 && e.model.index_count() == 0 { continue; }
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(self.vertex_offsets[i]..));

            if e.model.is_indexed() {
                if let Some(ref idx_buf) = self.index_buffer {
                    render_pass.set_index_buffer(
                        idx_buf.slice(self.index_offsets[i]..),
                        M::index_format(),
                    );
                    render_pass.draw_indexed(0..e.model.index_count(), 0, 0..1);
                }
            } else {
                render_pass.draw(0..e.model.vertex_count(), 0..1);
            }
        }
    }

    /// Draw a single model by index. Caller is responsible for setting bind groups.
    pub fn draw_single<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, index: usize) {
        if let Some(e) = self.models.get(index) {
            if e.model.vertex_count() == 0 && e.model.index_count() == 0 { return; }
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(self.vertex_offsets[index]..));
            if e.model.is_indexed() {
                if let Some(ref idx_buf) = self.index_buffer {
                    render_pass.set_index_buffer(
                        idx_buf.slice(self.index_offsets[index]..),
                        M::index_format(),
                    );
                    render_pass.draw_indexed(0..e.model.index_count(), 0, 0..1);
                }
            } else {
                render_pass.draw(0..e.model.vertex_count(), 0..1);
            }
        }
    }

    /// Number of models in the envelop.
    pub fn len(&self) -> usize {
        self.models.len()
    }

    pub fn get(&mut self, index: usize) -> Option<&mut EModel<M>> {
        if index >= self.models.len() {
            return None;
        }
        Some(&mut self.models[index])
    }

    pub fn update_model_buffers(&mut self, device: &wgpu::Device, index: usize) {
        if index >= self.models.len() {
            return;
        }

        // Rebuild all vertex data to keep offsets consistent
        let mut all_vertex_data: Vec<u8> = Vec::new();
        self.vertex_offsets.clear();
        for e in &self.models {
            self.vertex_offsets.push(all_vertex_data.len() as u64);
            all_vertex_data.extend_from_slice(&e.model.vertex_data());
        }

        // Recreate vertex buffer
        let new_vertex_buffer = GpuBuffer::new_vertex(device, &all_vertex_data, "model_vertex_buffer");
        self.vertex_buffer = new_vertex_buffer.buffer;

        // Rebuild index data
        let mut all_index_data: Vec<u8> = Vec::new();
        let mut has_any_index = false;
        self.index_offsets.clear();
        for e in &self.models {
            self.index_offsets.push(all_index_data.len() as u64);
            if let Some(idx_data) = e.model.index_data() {
                has_any_index = true;
                all_index_data.extend_from_slice(&idx_data);
            }
        }

        self.index_buffer = if has_any_index {
            Some(GpuBuffer::new_index(device, &all_index_data, "model_index_buffer").buffer)
        } else {
            None
        };
    }
}

pub fn intersect_models<'a, M>(
    envelop: &'a ModelEnvelop<M>,
    vec_s: Vector3<f32>,
    vec_e: Vector3<f32>,
) -> Option<usize>
where
    M: IterTriangleModel<'a, Vector3<f32>>,
{
    let r = envelop.models.iter().rfold(None, |r, e| {
        let mvp = e.transform();
        let iter = TriangleIteratorVector3::from_model(&e.model);
        match (r, intersect_iter(iter, &mvp, vec_s, vec_e)) {
            (r1 @ Some((_, t1)), r2 @ Some((_, t2))) => {
                if t1 > t2 {
                    r1
                } else {
                    r2
                }
            }
            (None, r2) => r2,
            (r1, _) => r1,
        }
    });
    r.map(|(index, _)| index)
}
