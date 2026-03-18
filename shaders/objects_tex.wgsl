// objects_tex.wgsl — Textured 3D objects with directional lighting.

// Group 0: Transform matrices + lighting
struct Transforms {
    m_transform: mat4x4<f32>,
};

struct Transforms1 {
    m_transform1: mat4x4<f32>,
};

struct LightParams {
    sun_dir: vec3<f32>,
    ambient: f32,
    camera_focus: vec2<f32>,
    viewport_radius: f32,
    game_tick: f32,
};

@group(0) @binding(0) var<uniform> transforms: Transforms;
@group(0) @binding(1) var<uniform> transforms1: Transforms1;
@group(0) @binding(2) var<uniform> light: LightParams;

// Group 1: Texture and sampler
@group(1) @binding(0) var texture_main: texture_2d<f32>;
@group(1) @binding(1) var texture_sampler: sampler;

// Group 2: Shadow map
@group(2) @binding(0) var shadow_map: texture_depth_2d;
@group(2) @binding(1) var shadow_samp: sampler_comparison;
@group(2) @binding(2) var<uniform> shadow_light_mvp: mat4x4<f32>;

// Group 3: Ghost overlay (for placement preview)
struct GhostParams {
    tint: vec3<f32>,
    alpha: f32,
};
@group(3) @binding(0) var<uniform> ghost: GhostParams;

// Vertex input
struct VertexInput {
    @location(0) coord3d: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) tex_id: i32,
};

// Vertex output / Fragment input
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) tex_id: i32,
    @location(2) world_pos: vec3<f32>,
    @location(3) viewport_fade: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = transforms.m_transform * transforms1.m_transform1 * vec4<f32>(in.coord3d, 1.0);
    out.tex_id = in.tex_id;
    out.uv = in.uv;
    out.world_pos = in.coord3d;

    // Viewport fade: smooth circular falloff at edges
    let dx = in.coord3d.x - light.camera_focus.x;
    let dy = in.coord3d.y - light.camera_focus.y;
    let dist = sqrt(dx * dx + dy * dy);
    let fade_start = light.viewport_radius * 1.3;
    let fade_end = light.viewport_radius * 1.5;
    out.viewport_fade = clamp(1.0 - (dist - fade_start) / (fade_end - fade_start), 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.viewport_fade < 0.01) {
        discard;
    }

    // Compute face normal from screen-space derivatives
    let world_normal = normalize(cross(dpdx(in.world_pos), dpdy(in.world_pos)));
    let ndotl = max(dot(world_normal, light.sun_dir), 0.0);
    let brightness = light.ambient + (1.0 - light.ambient) * ndotl;

    // Shadow mapping
    let shadow_world = transforms1.m_transform1 * vec4<f32>(in.world_pos, 1.0);
    let light_pos = shadow_light_mvp * shadow_world;
    let shadow_uv = light_pos.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    let shadow = textureSampleCompare(shadow_map, shadow_samp, shadow_uv, light_pos.z - 0.005);
    let shadow_factor = 0.3 + 0.7 * shadow;

    if (in.tex_id < 0 || in.tex_id > 255) {
        return vec4<f32>(vec3<f32>(0.6) * brightness * shadow_factor * in.viewport_fade * ghost.tint, ghost.alpha);
    }

    let row = in.tex_id / 8;
    let column = in.tex_id % 8;
    let hor_k = 1.0 / 8.0;
    let ver_k = 1.0 / 32.0;
    let u = hor_k * f32(column) + hor_k * in.uv.x;
    let v = ver_k * f32(row) + ver_k * in.uv.y;

    let color = textureSample(texture_main, texture_sampler, vec2<f32>(u, v));
    if (color.w > 0.0) {
        discard;
    }
    return vec4<f32>(color.rgb * brightness * shadow_factor * in.viewport_fade * ghost.tint, ghost.alpha);
}
