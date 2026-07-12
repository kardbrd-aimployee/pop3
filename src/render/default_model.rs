use std::vec::Vec;

use cgmath::Vector3;

use crate::render::envelop::GpuModel;
use crate::render::model::{FromUsize, MeshModel};

/******************************************************************************/

pub type DefaultModel = MeshModel<Vector3<f32>, u16>;

impl FromUsize for u16 {
    fn from_usize(v: usize) -> Self {
        v as u16
    }
    fn to_usize(&self) -> usize {
        *self as usize
    }
}

/******************************************************************************/

impl GpuModel for DefaultModel {
    fn vertex_buffer_layouts() -> Vec<wgpu::VertexBufferLayout<'static>> {
        vec![wgpu::VertexBufferLayout {
            array_stride: 12,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        }]
    }

    fn vertex_data(&self) -> Vec<u8> {
        let floats: Vec<f32> = self.vertices.iter().flat_map(|v| [v.x, v.y, v.z]).collect();
        bytemuck::cast_slice(&floats).to_vec()
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
