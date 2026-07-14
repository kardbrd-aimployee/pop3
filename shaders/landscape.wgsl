// landscape.wgsl — GPU texture generation landscape shader.
// Replaces landscape.vert + landscape.frag.

// ---------- Shared types ----------

struct LandscapeParams {
    level_shift: vec4<i32>,
    height_scale: f32,
    step: f32,
    width: i32,
    sunlight: vec4<f32>,
    wat_offset: i32,
    curvature_scale: f32,
    camera_focus: vec2<f32>,
    viewport_radius: f32,
};

struct Transforms {
    m_transform: mat4x4<f32>,
};

struct Transforms1 {
    m_transform1: mat4x4<f32>,
};

// ---------- Bindings ----------

// Group 0: Uniforms
@group(0) @binding(0) var<uniform> transforms: Transforms;
@group(0) @binding(1) var<uniform> transforms1: Transforms1;
@group(0) @binding(2) var<uniform> params: LandscapeParams;

// Group 1: Storage buffers
@group(1) @binding(0) var<storage, read> heights: array<u32>;
@group(1) @binding(1) var<storage, read> watdisp: array<u32>;
@group(1) @binding(2) var<storage, read> palette: array<u32>;
@group(1) @binding(3) var<storage, read> disp: array<i32>;
@group(1) @binding(4) var<storage, read> bigf: array<u32>;
@group(1) @binding(5) var<storage, read> sla: array<u32>;

// Group 2: Shadow map
@group(2) @binding(0) var shadow_map: texture_depth_2d;
@group(2) @binding(1) var shadow_samp: sampler_comparison;
@group(2) @binding(2) var<uniform> shadow_light_mvp: mat4x4<f32>;

// ---------- Vertex ----------

struct VertexInput {
    @location(0) coord_in: vec2<u32>,
    @builtin(vertex_index) vertex_index: u32,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) coord3d_out: vec3<f32>,
    @location(1) height_out: f32,
    @location(2) brightness: f32,
    @location(3) @interpolate(flat) primitive_id: u32,
    @location(4) viewport_fade: f32,
    @location(5) curved_pos: vec3<f32>,
};

fn wat_height(x: u32, y: u32) -> u32 {
    let x_wat = x * 2u;
    let y_wat = y * 2u;
    let index = (y_wat * 256u + x) * 8u;
    let wat_offset = u32(params.wat_offset) & 0xffu;
    let index1 = (index + wat_offset * 0x101u) & 0xffffu;
    let index2 = (index + 0x4cu - wat_offset * 0x101u) & 0xffffu;
    return (watdisp[index1] + watdisp[index2]) / 8u;
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let coord3d = vec3<f32>(f32(in.coord_in.x) * params.step, f32(in.coord_in.y) * params.step, 0.0);

    let w = u32(params.width);
    let x = (in.coord_in.x + u32(params.level_shift.x)) % w;
    let y = (in.coord_in.y + u32(params.level_shift.y)) % w;
    let index = y * w + x;
    var height = heights[index];

    out.height_out = f32(height);

    if (params.wat_offset > -1 && height == 0u) {
        height = wat_height(x, y);
    }

    let coordf = vec3<f32>(coord3d.x, coord3d.y, f32(height) * params.height_scale);

    // Curvature: pull Z down by distance² from camera focus (planet illusion)
    let dx = coordf.x - params.camera_focus.x;
    let dy = coordf.y - params.camera_focus.y;
    let dist_sq = dx * dx + dy * dy;
    let curvature_offset = dist_sq * params.curvature_scale;
    let curved = vec3<f32>(coordf.x, coordf.y, coordf.z - curvature_offset);

    // Viewport fade: smooth circular falloff at edges
    let dist = sqrt(dist_sq);
    let fade_start = params.viewport_radius * 0.85;
    let fade_end = params.viewport_radius;
    var vp_fade = 1.0;
    if (dist > fade_end) {
        vp_fade = 0.0;
    } else if (dist > fade_start) {
        vp_fade = 1.0 - (dist - fade_start) / (fade_end - fade_start);
    }
    out.viewport_fade = vp_fade;

    let coord = transforms.m_transform * transforms1.m_transform1 * vec4<f32>(curved, 1.0);
    out.position = coord;
    out.coord3d_out = vec3<f32>(coord3d.xy, coordf.z);
    out.curved_pos = curved;

    // Brightness calculation
    let index1 = ((in.coord_in.y + u32(params.level_shift.y) + 1u) % w) * w + ((in.coord_in.x + u32(params.level_shift.x)) % w);
    let index2 = ((in.coord_in.y + u32(params.level_shift.y)) % w) * w + ((in.coord_in.x + u32(params.level_shift.x) + 1u) % w);
    let ch = i32(heights[index]);
    let br_i = i32(params.sunlight.z) + i32(params.sunlight.y) * (i32(heights[index1]) - ch) - i32(params.sunlight.x) * (ch - i32(heights[index2]));
    let br_f = f32(br_i) / f32(0x15e) + f32(0x80);
    out.brightness = clamp(br_f, 0.0, 255.0);

    out.primitive_id = in.vertex_index / 3u;

    return out;
}

// ---------- Fragment ----------

fn srgb_to_linear(c: f32) -> f32 {
    return pow(c, 2.2);
}

fn mk_tex(val: u32) -> vec3<f32> {
    let packed = palette[val % 128u];
    let r = f32((packed >> 0u) & 0xffu) / 255.0;
    let g = f32((packed >> 8u) & 0xffu) / 255.0;
    let b = f32((packed >> 16u) & 0xffu) / 255.0;
    return vec3<f32>(r, g, b);
}

fn mk_height(z: f32) -> u32 {
    let height = u32(z);
    if (height > 0u) {
        return min(height + 0x96u, 0x400u);
    }
    if (z > 0.0) {
        let h = u32(z * f32(0x4b));
        return min(h + 0x4bu, 0x400u);
    }
    return min(height + 0x4bu, 0x400u);
}

fn get_wat_color(z: f32, z_current: f32) -> vec3<f32> {
    if (z <= 1.0 && z_current > 0.0) {
        let c = -((z_current / params.height_scale) / 512.0) / 1.0;
        return vec3<f32>(c, c, c);
    }
    return vec3<f32>(0.0, 0.0, 0.0);
}

fn get_disp(x: i32, y: i32) -> i32 {
    let sx = params.level_shift.x * 32;
    let sy = params.level_shift.y * 32;
    let dx = ((x + sx) % 256) * 256;
    let dy = (y + sy) % 256;
    return disp[dx + dy];
}

fn get_disp_2(x: i32, y: i32) -> i32 {
    let sx = params.level_shift.x * 32;
    let sy = params.level_shift.y * 32;
    let ly = (y + sy) % 32;
    var dx: i32;
    if (ly == 31) {
        dx = 0;
    } else {
        dx = 1;
    }
    let x1 = ((x + dx + sx) % 256) * 256;
    let y1 = (y + 1 + sy) % 256;
    return disp[x1 + y1];
}

struct ShoreSample {
    coverage: f32,
    land_offset: vec2<f32>,
};

fn wrap_cell(value: i32) -> u32 {
    let width = params.width;
    return u32(((value % width) + width) % width);
}

fn is_land(x: i32, y: i32) -> f32 {
    let index = wrap_cell(y) * u32(params.width) + wrap_cell(x);
    if (heights[index] > 0u) {
        return 1.0;
    }
    return 0.0;
}

fn shore_hash(pixel: vec2<f32>) -> f32 {
    return fract(sin(dot(pixel, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

fn sample_shore(coord: vec2<f32>) -> ShoreSample {
    let grid = coord / params.step
        + vec2<f32>(f32(params.level_shift.x), f32(params.level_shift.y));
    let cell = vec2<i32>(floor(grid));
    let local = fract(grid);

    let land00 = is_land(cell.x, cell.y);
    let land10 = is_land(cell.x + 1, cell.y);
    let land01 = is_land(cell.x, cell.y + 1);
    let land11 = is_land(cell.x + 1, cell.y + 1);
    let land_count = land00 + land10 + land01 + land11;

    if (land_count < 0.5 || land_count > 3.5) {
        return ShoreSample(0.0, vec2<f32>(0.0));
    }

    let top = mix(land00, land10, local.x);
    let bottom = mix(land01, land11, local.x);
    let coverage = smoothstep(0.08, 0.92, mix(top, bottom, local.y));
    let centroid = (
        vec2<f32>(0.0, 0.0) * land00
        + vec2<f32>(1.0, 0.0) * land10
        + vec2<f32>(0.0, 1.0) * land01
        + vec2<f32>(1.0, 1.0) * land11
    ) / land_count;
    let toward_land = centroid - local;
    var land_offset = vec2<f32>(0.0);
    if (length(toward_land) > 0.001) {
        land_offset = normalize(toward_land);
    }

    // Work at the original 32 texture pixels per landscape cell. The thresholded
    // noise preserves the game's stippled land/water transition without obscuring
    // the animated water surface below it.
    let pixel = floor(grid * 32.0);
    let pattern = select(0.0, 1.0, shore_hash(pixel) < coverage);
    return ShoreSample(pattern, land_offset);
}

fn land_tex_static(coord: vec3<f32>, height_in: f32, brightness: f32) -> vec3<f32> {
    let height = mk_height(height_in);

    let disp_val = get_disp(i32(coord.x), i32(coord.y) + 32);
    let disp_val_2 = get_disp_2(i32(coord.x), i32(coord.y) + 32);

    var disp_param = i32((f32(disp_val_2) - f32(disp_val)) / 4.0) + i32(brightness);
    disp_param = clamp(disp_param, 0, 255);

    let sla_val = sla[height];
    var static_component = i32(sla_val) * disp_val;
    var static_component_u = u32(static_component);
    static_component_u = static_component_u & 0xfffffc03u;
    static_component = i32(static_component_u);
    static_component = static_component >> 2u;

    let height_component = i32(height * 256u) & 0x7fffff00;
    let index = static_component + height_component + disp_param;

    let bigf_index = min(bigf[index], 128u);
    return mk_tex(bigf_index);
}

fn land_tex(coord: vec3<f32>, height_in: f32, brightness: f32) -> vec3<f32> {
    let res_color = land_tex_static(coord, height_in, brightness);
    let wat_color = get_wat_color(height_in, coord.z);
    return res_color + wat_color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.viewport_fade < 0.01) {
        discard;
    }

    let coordi = vec3<f32>(
        in.coord3d_out.x / 8.0 * 4096.0,
        in.coord3d_out.y / 8.0 * 4096.0,
        in.coord3d_out.z,
    );
    var c = land_tex(coordi, in.height_out, in.brightness);

    let shore = sample_shore(in.coord3d_out.xy);
    if (shore.coverage > 0.0) {
        var coast_coordi = coordi;
        coast_coordi.x += shore.land_offset.x * 20.0;
        coast_coordi.y += shore.land_offset.y * 20.0;
        let coast = land_tex_static(coast_coordi, max(in.height_out, 1.0), in.brightness);
        let low_ground = 1.0 - smoothstep(0.0, 20.0, in.height_out);
        c = mix(c, coast, shore.coverage * low_ground * 0.88);
    }

    // Shadow mapping
    let shadow_world = transforms1.m_transform1 * vec4<f32>(in.curved_pos, 1.0);
    let light_pos = shadow_light_mvp * shadow_world;
    let shadow_uv = light_pos.xy * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    let shadow = textureSampleCompare(shadow_map, shadow_samp, shadow_uv, light_pos.z - 0.005);
    let shadow_factor = 0.3 + 0.7 * shadow;

    return vec4<f32>(pow(c, vec3<f32>(2.2)) * shadow_factor * in.viewport_fade, 0.0);
}
