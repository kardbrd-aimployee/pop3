use cgmath::Vector3;

use crate::data::objects::{mk_pop_object, mk_pop_object_for_phase, Object3D, Shape};
use crate::data::units::{
    building_obj_index, building_obj_index_with_variant, scenery_obj_index, ModelType,
};
use crate::render::envelop::{ModelEnvelop, RenderType};
use crate::render::model::{MeshModel, VertexModel};
use crate::render::terrain::LandscapeMesh;
use crate::render::tex_model::{TexModel, TexVertex};

fn building_angle_radians(angle: u32) -> f32 {
    -std::f32::consts::FRAC_PI_2 - (angle as f32) * std::f32::consts::TAU / 2048.0
}

use crate::render::sprites::LevelObject;

/// Build a single-building mesh for ghost preview at a specific cell position.
/// Returns a ModelEnvelop with exactly one model entry, or None if the building
/// type/tribe combination doesn't map to a valid Object3D.
pub fn build_ghost_building_mesh(
    device: &wgpu::Device,
    building_type: u8,
    tribe_index: u8,
    cell_x: f32,
    cell_y: f32,
    rotation: u8,
    building_bank: &[Option<Object3D>],
    landscape: &LandscapeMesh<128>,
    curvature_scale: f32,
) -> Option<ModelEnvelop<TexModel>> {
    let idx = building_obj_index(building_type, tribe_index)?;
    let obj3d = building_bank.get(idx)?.as_ref()?;

    let local_model = mk_pop_object(obj3d);
    let step = landscape.step();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();
    let center = (w - 1.0) * step / 2.0;
    let scale = step * (obj3d.coord_scale() / 300.0);

    let vis_x = ((cell_x - shift.x as f32) % w + w) % w;
    let vis_y = ((cell_y - shift.y as f32) % w + w) % w;
    let gx = vis_x * step;
    let gy = vis_y * step;

    let angle = (rotation & 3) as u32 * 512;
    let angle_rad = building_angle_radians(angle);
    let cos_a = angle_rad.cos();
    let sin_a = angle_rad.sin();

    let mut combined: TexModel = MeshModel::new();
    let base_idx = combined.vertices.len() as u16;
    for v in &local_model.vertices {
        let rx = v.coord.x * cos_a - v.coord.z * sin_a;
        let rz = v.coord.x * sin_a + v.coord.z * cos_a;

        let vx_gpu = gx + rx * scale;
        let vy_gpu = gy + rz * scale;

        // Per-vertex curvature (matching landscape shader)
        let vdx = vx_gpu - center;
        let vdy = vy_gpu - center;
        let vertex_curvature = (vdx * vdx + vdy * vdy) * curvature_scale;

        // Per-vertex terrain height sampling
        let vert_cell_x = vis_x + rx * scale / step;
        let vert_cell_y = vis_y + rz * scale / step;
        let abs_cell_x = ((vert_cell_x % w + w) % w) + shift.x as f32;
        let abs_cell_y = ((vert_cell_y % w + w) % w) + shift.y as f32;
        let vertex_gz = landscape.interpolate_height_at(abs_cell_x, abs_cell_y);
        let vertex_z = vertex_gz - vertex_curvature + v.coord.y * scale;

        combined.push_vertex(TexVertex {
            coord: Vector3::new(vx_gpu, vy_gpu, vertex_z),
            uv: v.uv,
            tex_id: v.tex_id,
        });
    }
    for &idx16 in &local_model.indices {
        combined.indices.push(base_idx + idx16);
    }

    if combined.vertices.is_empty() {
        return None;
    }

    let m = vec![(RenderType::Triangles, combined)];
    Some(ModelEnvelop::<TexModel>::new(device, m))
}

pub fn build_building_meshes(
    device: &wgpu::Device,
    objects: &[LevelObject],
    building_bank: &[Option<Object3D>],
    scenery_bank: &[Option<Object3D>],
    _shapes: &[Shape],
    landscape: &LandscapeMesh<128>,
    curvature_scale: f32,
) -> ModelEnvelop<TexModel> {
    let mut combined: TexModel = MeshModel::new();
    let step = landscape.step();
    let w = landscape.width() as f32;
    let shift = landscape.get_shift_vector();
    let center = (w - 1.0) * step / 2.0;

    let mut building_count = 0;
    for obj in objects {
        if obj.building_state == Some(crate::engine::buildings::BuildingState::FinalTeardown) {
            continue;
        }
        if obj.building_state == Some(crate::engine::buildings::BuildingState::Init)
            && obj.construction_progress == 0
        {
            continue;
        }
        let phased = matches!(
            obj.building_state,
            Some(crate::engine::buildings::BuildingState::Init)
                | Some(crate::engine::buildings::BuildingState::Destroying)
                | Some(crate::engine::buildings::BuildingState::Sinking)
        ) && obj.construction_phase < 4;
        // Look up model index and select the right bank based on model type
        let (idx, bank): (Option<usize>, &[Option<Object3D>]) = match obj.model_type {
            ModelType::Building => (
                building_obj_index_with_variant(obj.subtype, obj.tribe_index, obj.visual_variant),
                building_bank,
            ),
            ModelType::Scenery => (scenery_obj_index(obj.subtype), scenery_bank),
            _ => (None, building_bank),
        };
        let idx = match idx {
            Some(i) => i,
            None => {
                eprintln!(
                    "[3d-obj] UNMAPPED type={:?} subtype={} tribe={} cell=({:.1},{:.1}) angle={}",
                    obj.model_type, obj.subtype, obj.tribe_index, obj.cell_x, obj.cell_y, obj.angle
                );
                continue;
            }
        };
        building_count += 1;
        eprintln!(
            "[3d-obj] type={:?} subtype={} tribe={} angle={} -> idx={}",
            obj.model_type, obj.subtype, obj.tribe_index, obj.angle, idx
        );
        let obj3d = match idx < bank.len() {
            true => match &bank[idx] {
                Some(o) => o,
                None => {
                    eprintln!("  -> object at {} is None", idx);
                    continue;
                }
            },
            false => continue,
        };

        let local_model = if phased {
            mk_pop_object_for_phase(obj3d, Some(obj.construction_phase))
        } else {
            mk_pop_object(obj3d)
        };
        let scale = step * (obj3d.coord_scale() / 300.0);

        let vis_x = ((obj.cell_x - shift.x as f32) % w + w) % w;
        let vis_y = ((obj.cell_y - shift.y as f32) % w + w) % w;
        let gx = vis_x * step;
        let gy = vis_y * step;

        // Skip buildings outside the visible globe disc (matching landscape viewport fade)
        let dx_cull = gx - center;
        let dy_cull = gy - center;
        let viewport_radius = center * 1.4;
        if dx_cull * dx_cull + dy_cull * dy_cull > viewport_radius * viewport_radius {
            continue;
        }

        // Rotate model vertices in the horizontal plane (model X/Z -> world X/Y)
        let angle_rad = building_angle_radians(obj.angle);
        let cos_a = angle_rad.cos();
        let sin_a = angle_rad.sin();

        let base_idx = combined.vertices.len() as u16;
        for v in &local_model.vertices {
            let rx = v.coord.x * cos_a - v.coord.z * sin_a;
            let rz = v.coord.x * sin_a + v.coord.z * cos_a;

            let vx_gpu = gx + rx * scale;
            let vy_gpu = gy + rz * scale;

            // Per-vertex curvature (matching landscape shader: dist_sq * curvature_scale)
            let vdx = vx_gpu - center;
            let vdy = vy_gpu - center;
            let vertex_curvature = (vdx * vdx + vdy * vdy) * curvature_scale;

            // Per-vertex terrain height sampling (matching Model3D_RenderObject Phase 4)
            let vert_cell_x = vis_x + rx * scale / step;
            let vert_cell_y = vis_y + rz * scale / step;
            let abs_cell_x = ((vert_cell_x % w + w) % w) + shift.x as f32;
            let abs_cell_y = ((vert_cell_y % w + w) % w) + shift.y as f32;
            let vertex_gz = landscape.interpolate_height_at(abs_cell_x, abs_cell_y);
            let vertex_z = vertex_gz - vertex_curvature + v.coord.y * scale;

            combined.push_vertex(TexVertex {
                coord: Vector3::new(vx_gpu, vy_gpu, vertex_z),
                uv: v.uv,
                tex_id: v.tex_id,
            });
        }
        for &idx16 in &local_model.indices {
            combined.indices.push(base_idx + idx16);
        }
    }
    eprintln!(
        "[buildings] total={} vertices={} indices={} step={:.4} center={:.4}",
        building_count,
        combined.vertices.len(),
        combined.indices.len(),
        step,
        center
    );
    if !combined.vertices.is_empty() {
        let (mut min_x, mut min_y, mut min_z) = (f32::MAX, f32::MAX, f32::MAX);
        let (mut max_x, mut max_y, mut max_z) = (f32::MIN, f32::MIN, f32::MIN);
        for v in &combined.vertices {
            min_x = min_x.min(v.coord.x);
            max_x = max_x.max(v.coord.x);
            min_y = min_y.min(v.coord.y);
            max_y = max_y.max(v.coord.y);
            min_z = min_z.min(v.coord.z);
            max_z = max_z.max(v.coord.z);
        }
        eprintln!(
            "[buildings] bbox x=[{:.3}..{:.3}] y=[{:.3}..{:.3}] z=[{:.3}..{:.3}]",
            min_x, max_x, min_y, max_y, min_z, max_z
        );
    }
    let m = vec![(RenderType::Triangles, combined)];
    ModelEnvelop::<TexModel>::new(device, m)
}

#[cfg(test)]
mod tests {
    use super::building_angle_radians;

    #[test]
    fn building_rotation_corrects_model_space_quarter_turn() {
        assert!((building_angle_radians(0) + std::f32::consts::FRAC_PI_2).abs() < 0.000_001);
        assert!((building_angle_radians(512) + std::f32::consts::PI).abs() < 0.000_001);
        assert!(
            (building_angle_radians(1024) + 3.0 * std::f32::consts::FRAC_PI_2).abs() < 0.000_001
        );
    }
}
