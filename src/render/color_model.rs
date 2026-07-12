use cgmath::Vector3;

use crate::render::envelop::GpuModel;
use crate::render::model::MeshModel;

/******************************************************************************/

pub struct ColorVertex {
    pub coord: Vector3<f32>,
    pub color: Vector3<f32>,
}

pub type ColorModel = MeshModel<ColorVertex, u16>;

/******************************************************************************/

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ColorVertexGpu {
    pub coord: [f32; 3],
    pub color: [f32; 3],
}

impl From<&ColorVertex> for ColorVertexGpu {
    fn from(v: &ColorVertex) -> Self {
        ColorVertexGpu {
            coord: [v.coord.x, v.coord.y, v.coord.z],
            color: [v.color.x, v.color.y, v.color.z],
        }
    }
}

/******************************************************************************/

impl GpuModel for ColorModel {
    fn vertex_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        vec![wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColorVertexGpu>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: 12,
                    shader_location: 1,
                },
            ],
        }]
    }

    fn vertex_data(&self) -> Vec<u8> {
        let gpu_vertices: Vec<ColorVertexGpu> =
            self.vertices.iter().map(ColorVertexGpu::from).collect();
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
