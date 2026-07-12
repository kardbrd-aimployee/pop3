use cgmath::{Vector2, Vector3};

use crate::render::envelop::GpuModel;
use crate::render::model::MeshModel;

/******************************************************************************/

pub struct TexVertex {
    pub coord: Vector3<f32>,
    pub uv: Vector2<f32>,
    pub tex_id: i16,
}

pub type TexModel = MeshModel<TexVertex, u16>;

/******************************************************************************/

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TexVertexGpu {
    pub coord: [f32; 3],
    pub uv: [f32; 2],
    pub tex_id: i32,
}

impl From<&TexVertex> for TexVertexGpu {
    fn from(v: &TexVertex) -> Self {
        TexVertexGpu {
            coord: [v.coord.x, v.coord.y, v.coord.z],
            uv: [v.uv.x, v.uv.y],
            tex_id: v.tex_id as i32,
        }
    }
}

/******************************************************************************/

impl GpuModel for TexModel {
    fn vertex_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        vec![wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TexVertexGpu>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 12,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Sint32,
                    offset: 20,
                    shader_location: 2,
                },
            ],
        }]
    }

    fn vertex_data(&self) -> Vec<u8> {
        let gpu_vertices: Vec<TexVertexGpu> =
            self.vertices.iter().map(TexVertexGpu::from).collect();
        bytemuck::cast_slice(&gpu_vertices).to_vec()
    }

    fn index_data(&self) -> Option<Vec<u8>> {
        if self.indices.is_empty() {
            None
        } else {
            Some(bytemuck::cast_slice(&self.indices).to_vec())
        }
    }

    fn index_format() -> wgpu::IndexFormat {
        wgpu::IndexFormat::Uint16
    }

    fn vertex_count(&self) -> u32 {
        self.vertices.len() as u32
    }

    fn index_count(&self) -> u32 {
        self.indices.len() as u32
    }

    fn is_indexed(&self) -> bool {
        !self.indices.is_empty()
    }
}
