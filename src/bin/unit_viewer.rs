//! Unit Animation Viewer
//!
//! Renders animated unit sprites using the VSTART/VFRA/VELE animation chain.
//! Shows all person subtypes (Brave, Warrior, Preacher, Spy, Firewarrior, Shaman)
//! in all animation states with composite multi-sprite frames.
//!
//! Controls:
//!   Space       - Pause/Resume
//!   Up/Down     - Animation speed
//!   Left/Right  - Frame step (when paused)
//!   Tab/N/P     - Cycle animation (skips static poses)
//!   +/-         - Jump ~10 animations forward/backward
//!   1-4         - Quick jump (15=Brave, 20=Shaman, 21=BraveWalk, 26=ShamanWalk)
//!   T           - Cycle tribe (0-3)
//!   U           - Cycle unit features overlay
//!   Escape      - Quit

use std::path::{Path, PathBuf};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, Command};

use pop3::render::gpu::buffer::GpuBuffer;
use pop3::render::gpu::context::GpuContext;
use pop3::render::gpu::pipeline::create_pipeline;
use pop3::render::gpu::texture::GpuTexture;

use pop3::data::animation::{
    build_tribe_atlas, compute_global_bbox, discover_unit_combos, AnimationSequence,
    AnimationsData, NUM_TRIBES,
};
use pop3::data::psfb::ContainerPSFB;
use pop3::data::types::BinDeserializer;

/******************************************************************************/
// Bitmap font (8x8, ASCII 32..127)
/******************************************************************************/

const FONT_CHAR_W: u32 = 8;
const FONT_CHAR_H: u32 = 8;
const FONT_COLS: u32 = 16;
const FONT_ROWS: u32 = 6;
const FONT_TEX_W: u32 = FONT_COLS * FONT_CHAR_W; // 128
const FONT_TEX_H: u32 = FONT_ROWS * FONT_CHAR_H; // 48

#[rustfmt::skip]
const FONT_8X8: [u64; 96] = [
    0x0000000000000000, // 32 ' '
    0x183C3C1818001800, // 33 '!'
    0x3636000000000000, // 34 '"'
    0x36367F367F363600, // 35 '#'
    0x0C3E031E301F0C00, // 36 '$'
    0x006333180C666300, // 37 '%'
    0x1C361C6E3B336E00, // 38 '&'
    0x0606030000000000, // 39 '''
    0x180C0606060C1800, // 40 '('
    0x060C1818180C0600, // 41 ')'
    0x00663CFF3C660000, // 42 '*'
    0x000C0C3F0C0C0000, // 43 '+'
    0x00000000000C0C06, // 44 ','
    0x0000003F00000000, // 45 '-'
    0x00000000000C0C00, // 46 '.'
    0x6030180C06030100, // 47 '/'
    0x3E63737B6F673E00, // 48 '0'
    0x0C0E0C0C0C0C3F00, // 49 '1'
    0x1E33301C06333F00, // 50 '2'
    0x1E33301C30331E00, // 51 '3'
    0x383C36337F307800, // 52 '4'
    0x3F031F3030331E00, // 53 '5'
    0x1C06031F33331E00, // 54 '6'
    0x3F33301806060600, // 55 '7'
    0x1E33331E33331E00, // 56 '8'
    0x1E33333E30180E00, // 57 '9'
    0x000C0C00000C0C00, // 58 ':'
    0x000C0C00000C0C06, // 59 ';'
    0x180C060306180000, // 60 '<' (corrected)
    0x00003F00003F0000, // 61 '='
    0x060C183018060000, // 62 '>' (corrected)
    0x1E3330180C000C00, // 63 '?'
    0x3E637B7B7B031E00, // 64 '@'
    0x0C1E33333F333300, // 65 'A'
    0x3F66663E66663F00, // 66 'B'
    0x3C66030303663C00, // 67 'C'
    0x1F36666666361F00, // 68 'D'
    0x7F46161E16467F00, // 69 'E'
    0x7F46161E16060F00, // 70 'F'
    0x3C66030373663C00, // 71 'G'
    0x3333333F33333300, // 72 'H'
    0x1E0C0C0C0C0C1E00, // 73 'I'
    0x7830303033331E00, // 74 'J'
    0x6766361E36666700, // 75 'K'
    0x0F06060646667F00, // 76 'L'
    0x63777F7F6B636300, // 77 'M'
    0x63676F7B73636300, // 78 'N'
    0x1C36636363361C00, // 79 'O'
    0x3F66663E06060F00, // 80 'P'
    0x1E3333333B1E3800, // 81 'Q'
    0x3F66663E36666700, // 82 'R'
    0x1E33070E38331E00, // 83 'S'
    0x3F2D0C0C0C0C1E00, // 84 'T'
    0x3333333333333F00, // 85 'U'
    0x33333333331E0C00, // 86 'V'
    0x6363636B7F776300, // 87 'W'
    0x6363361C1C366300, // 88 'X'
    0x3333331E0C0C1E00, // 89 'Y'
    0x7F6331184C667F00, // 90 'Z'
    0x1E06060606061E00, // 91 '['
    0x03060C1830604000, // 92 '\'
    0x1E18181818181E00, // 93 ']'
    0x081C366300000000, // 94 '^'
    0x00000000000000FF, // 95 '_'
    0x0C0C180000000000, // 96 '`'
    0x00001E303E336E00, // 97 'a'
    0x0706063E66663B00, // 98 'b'
    0x00001E3303331E00, // 99 'c'
    0x3830303E33336E00, // 100 'd'
    0x00001E333F031E00, // 101 'e'
    0x1C36060F06060F00, // 102 'f'
    0x00006E33333E301F, // 103 'g'
    0x0706366E66666700, // 104 'h'
    0x0C000E0C0C0C1E00, // 105 'i'
    0x300030303033331E, // 106 'j'
    0x070666361E366700, // 107 'k'
    0x0E0C0C0C0C0C1E00, // 108 'l'
    0x0000337F7F6B6300, // 109 'm'
    0x00001F3333333300, // 110 'n'
    0x00001E3333331E00, // 111 'o'
    0x00003B66663E060F, // 112 'p'
    0x00006E33333E3078, // 113 'q'
    0x00003B6E66060F00, // 114 'r'
    0x00003E031E301F00, // 115 's'
    0x080C3E0C0C2C1800, // 116 't'
    0x0000333333336E00, // 117 'u'
    0x00003333331E0C00, // 118 'v'
    0x0000636B7F7F3600, // 119 'w'
    0x000063361C366300, // 120 'x'
    0x00003333333E301F, // 121 'y'
    0x00003F190C263F00, // 122 'z'
    0x380C0C070C0C3800, // 123 '{'
    0x1818180018181800, // 124 '|'
    0x070C0C380C0C0700, // 125 '}'
    0x6E3B000000000000, // 126 '~'
    0x0000000000000000, // 127 DEL (blank)
];

fn build_font_texture() -> Vec<u8> {
    let mut data = vec![0u8; (FONT_TEX_W * FONT_TEX_H) as usize];
    for idx in 0..96usize {
        let col = idx % FONT_COLS as usize;
        let row = idx / FONT_COLS as usize;
        let bits = FONT_8X8[idx];
        for y in 0..8 {
            let byte = ((bits >> (56 - y * 8)) & 0xFF) as u8;
            for x in 0..8 {
                if byte & (1 << x) != 0 {
                    let px = col * 8 + x;
                    let py = row * 8 + y;
                    data[py * FONT_TEX_W as usize + px] = 255;
                }
            }
        }
    }
    data
}

/******************************************************************************/
// Constants
/******************************************************************************/

const STORED_DIRECTIONS: usize = 5;
const NUM_DIRECTIONS: usize = 8;
const DIRS_PER_ANIM: usize = 8;
const DEFAULT_SPEED: f32 = 0.1;
const DEFAULT_ANIM: usize = 15; // Brave Idle (from g_PersonAnimationTable)

const TRIBE_NAMES: &[&str] = &["Blue", "Red", "Yellow", "Green"];

/// Returns (source_direction, is_mirrored) for a display direction 0-7
fn get_source_direction(dir: usize) -> (usize, bool) {
    match dir {
        0 => (0, false),
        1 => (1, false),
        2 => (2, false),
        3 => (3, false),
        4 => (4, false),
        5 => (3, true),
        6 => (2, true),
        7 => (1, true),
        _ => (0, false),
    }
}

/// A static pose has ≤1 frame in direction 0 (single-frame self-loop in VFRA).
fn is_static_pose(sequences: &[AnimationSequence], anim_index: usize) -> bool {
    let base = anim_index * DIRS_PER_ANIM;
    if base >= sequences.len() {
        return true;
    }
    sequences[base].frames.len() <= 1
}

/// Find the next non-static animation in direction `delta` (+1 or -1).
fn find_next_animated(
    sequences: &[AnimationSequence],
    from: usize,
    delta: i32,
    total: usize,
) -> usize {
    let mut idx = from;
    for _ in 0..total {
        idx = (idx as i32 + delta).rem_euclid(total as i32) as usize;
        if !is_static_pose(sequences, idx) {
            return idx;
        }
    }
    from
}

/******************************************************************************/
// Palette loading
/******************************************************************************/

fn load_palette(path: &Path) -> Option<Vec<[u8; 4]>> {
    let data = std::fs::read(path).ok()?;
    if data.len() < 1024 {
        return None;
    }
    let mut palette = Vec::with_capacity(256);
    for i in 0..256 {
        let off = i * 4;
        palette.push([data[off], data[off + 1], data[off + 2], 255]);
    }
    Some(palette)
}

/******************************************************************************/
// Composite frame atlas building
/******************************************************************************/

struct SpriteAtlas {
    frames_per_dir: u32,
}

/******************************************************************************/
// Uniform data
/******************************************************************************/

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteUniforms {
    projection: [[f32; 4]; 4],
    uv_offset: [f32; 2],
    uv_scale: [f32; 2],
    mirror: [f32; 4],
}

fn ortho_projection(width: f32, height: f32) -> [[f32; 4]; 4] {
    let hw = width / 2.0;
    let hh = height / 2.0;
    [
        [1.0 / hw, 0.0, 0.0, 0.0],
        [0.0, 1.0 / hh, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

/******************************************************************************/
// Vertex data
/******************************************************************************/

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteVertex {
    position: [f32; 2],
    uv: [f32; 2],
}

fn make_quad(cx: f32, cy: f32, hw: f32, hh: f32) -> [SpriteVertex; 6] {
    [
        SpriteVertex {
            position: [cx - hw, cy - hh],
            uv: [0.0, 1.0],
        },
        SpriteVertex {
            position: [cx + hw, cy - hh],
            uv: [1.0, 1.0],
        },
        SpriteVertex {
            position: [cx + hw, cy + hh],
            uv: [1.0, 0.0],
        },
        SpriteVertex {
            position: [cx - hw, cy - hh],
            uv: [0.0, 1.0],
        },
        SpriteVertex {
            position: [cx + hw, cy + hh],
            uv: [1.0, 0.0],
        },
        SpriteVertex {
            position: [cx - hw, cy + hh],
            uv: [0.0, 0.0],
        },
    ]
}

/******************************************************************************/
// Application
/******************************************************************************/

struct TextRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    font_texture: GpuTexture,
    sampler: wgpu::Sampler,
}

impl TextRenderer {
    fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let font_data = build_font_texture();
        // R8Unorm for single-channel font
        let font_texture = GpuTexture::new_2d(
            device,
            queue,
            FONT_TEX_W,
            FONT_TEX_H,
            wgpu::TextureFormat::R8Unorm,
            &font_data,
            "font_texture",
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("font_sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("text_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let shader_source = include_str!("../../shaders/text.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("text_shader"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("text_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            immediate_size: 0,
        });

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("text_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        TextRenderer {
            pipeline,
            bind_group_layout,
            font_texture,
            sampler,
        }
    }

    /// Render text lines at pixel position (x, y) from top-left.
    /// Returns vertex buffer + bind group + vertex count.
    fn prepare_text(
        &self,
        device: &wgpu::Device,
        screen_w: f32,
        screen_h: f32,
        lines: &[(&str, [f32; 4])],
        start_x: f32,
        start_y: f32,
        scale: f32,
    ) -> (GpuBuffer, wgpu::BindGroup, GpuBuffer, u32) {
        let char_w = FONT_CHAR_W as f32 * scale;
        let char_h = FONT_CHAR_H as f32 * scale;
        let uv_char_w = FONT_CHAR_W as f32 / FONT_TEX_W as f32;
        let uv_char_h = FONT_CHAR_H as f32 / FONT_TEX_H as f32;

        let mut vertices: Vec<SpriteVertex> = Vec::new();

        let mut cursor_y = start_y;
        for (text, _color) in lines {
            let mut cursor_x = start_x;
            for ch in text.chars() {
                let idx = (ch as u32).wrapping_sub(32) as usize;
                if idx >= 96 {
                    cursor_x += char_w;
                    continue;
                }

                let col = idx % FONT_COLS as usize;
                let row = idx / FONT_COLS as usize;
                let u0 = col as f32 * uv_char_w;
                let v1 = row as f32 * uv_char_h;
                let u1 = u0 + uv_char_w;
                let v0 = v1 + uv_char_h;

                // Convert pixel coords to NDC: [-1, 1]
                let x0 = (cursor_x / screen_w) * 2.0 - 1.0;
                let x1 = ((cursor_x + char_w) / screen_w) * 2.0 - 1.0;
                let y0 = 1.0 - (cursor_y / screen_h) * 2.0;
                let y1 = 1.0 - ((cursor_y + char_h) / screen_h) * 2.0;

                vertices.push(SpriteVertex {
                    position: [x0, y1],
                    uv: [u0, v0],
                });
                vertices.push(SpriteVertex {
                    position: [x1, y1],
                    uv: [u1, v0],
                });
                vertices.push(SpriteVertex {
                    position: [x1, y0],
                    uv: [u1, v1],
                });
                vertices.push(SpriteVertex {
                    position: [x0, y1],
                    uv: [u0, v0],
                });
                vertices.push(SpriteVertex {
                    position: [x1, y0],
                    uv: [u1, v1],
                });
                vertices.push(SpriteVertex {
                    position: [x0, y0],
                    uv: [u0, v1],
                });

                cursor_x += char_w;
            }
            cursor_y += char_h + 2.0 * scale;
        }

        let vertex_count = vertices.len() as u32;
        let vb = GpuBuffer::new_vertex(device, bytemuck::cast_slice(&vertices), "text_vb");

        // Use the color of the first line (or white) for the uniform.
        // We use identity projection since we directly output NDC coords.
        let color = if lines.is_empty() {
            [1.0, 1.0, 1.0, 1.0]
        } else {
            lines[0].1
        };
        let uniforms = SpriteUniforms {
            projection: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            uv_offset: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            mirror: color,
        };

        let ub = GpuBuffer::new_uniform_init(device, bytemuck::bytes_of(&uniforms), "text_ub");

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("text_bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: ub.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.font_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        });

        (vb, bind_group, ub, vertex_count)
    }
}

struct App {
    window: Option<Arc<Window>>,
    state: Option<ViewerState>,
    container: ContainerPSFB,
    palette: Vec<[u8; 4]>,
    sequences: Vec<AnimationSequence>,
    initial_anim: usize,
    initial_tribe: u8,
    global_bbox: (i32, i32, i32, i32),
}

struct ViewerState {
    gpu: GpuContext,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: Vec<wgpu::BindGroup>,
    vertex_buffers: Vec<GpuBuffer>,
    uniform_buffers: Vec<GpuBuffer>,
    sampler: wgpu::Sampler,
    sprite_atlas: GpuTexture,

    atlas: SpriteAtlas,

    // Text overlay
    text_renderer: TextRenderer,
    info_text: String,

    // Anim state
    current_anim: usize,
    total_anims: usize,
    current_tribe: u8,
    unit_combos: Vec<(u16, u16)>,
    current_combo_idx: Option<usize>,

    // Animation playback
    current_frame: u32,
    anim_timer: f32,
    anim_speed: f32,
    paused: bool,
    last_instant: std::time::Instant,
}

impl App {
    fn new(
        container: ContainerPSFB,
        palette: Vec<[u8; 4]>,
        sequences: Vec<AnimationSequence>,
        initial_anim: usize,
        initial_tribe: u8,
        global_bbox: (i32, i32, i32, i32),
    ) -> Self {
        Self {
            window: None,
            state: None,
            container,
            palette,
            sequences,
            initial_anim,
            initial_tribe,
            global_bbox,
        }
    }
}

fn format_anim_info(
    anim_index: usize,
    total_anims: usize,
    tribe: u8,
    combo_idx: Option<usize>,
    combos: &[(u16, u16)],
    frames: u32,
) -> String {
    let tribe_name = TRIBE_NAMES.get(tribe as usize).unwrap_or(&"?");
    let combo_str = match combo_idx {
        Some(i) => {
            let (l, h) = combos[i];
            format!("Combo {}/{} (L{},H{})", i + 1, combos.len(), l, h)
        }
        None if combos.is_empty() => "Base".to_string(),
        None => format!("Base (0/{})", combos.len()),
    };
    format!(
        "Anim {}/{} | Tribe {} ({}) | {} | {} frames",
        anim_index, total_anims, tribe, tribe_name, combo_str, frames
    )
}

impl ViewerState {
    fn rebuild_atlas(
        &mut self,
        sequences: &[AnimationSequence],
        container: &ContainerPSFB,
        palette: &[[u8; 4]],
        anim_index: usize,
        tribe: u8,
        combo_idx: Option<usize>,
        global_bbox: (i32, i32, i32, i32),
    ) {
        let base = anim_index * DIRS_PER_ANIM;

        // Discover available unit combos
        let combos = discover_unit_combos(sequences, base);
        // Clamp combo index to new animation's combo list
        let combo_idx = combo_idx.and_then(|i| if i < combos.len() { Some(i) } else { None });
        let unit_combo = combo_idx.and_then(|i| combos.get(i).copied());

        // Use the shared build_tribe_atlas with explicit combo override and global bbox
        let combo_override = Some(unit_combo);
        let vstart_base = anim_index * DIRS_PER_ANIM;
        if let Some((atlas_w, atlas_h, rgba, _fw, _fh, max_frames, _max_y)) = build_tribe_atlas(
            sequences,
            container,
            palette,
            vstart_base,
            combo_override,
            Some(global_bbox),
        ) {
            let atlas = SpriteAtlas {
                frames_per_dir: max_frames,
            };
            self.sprite_atlas = GpuTexture::new_2d(
                &self.gpu.device,
                &self.gpu.queue,
                atlas_w,
                atlas_h,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                &rgba,
                "sprite_atlas",
            );

            // Rebuild bind groups with new texture
            self.bind_groups.clear();
            for dir in 0..NUM_DIRECTIONS {
                let bg = self
                    .gpu
                    .device
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&format!("sprite_bg_{}", dir)),
                        layout: &self.bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.uniform_buffers[dir].buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::TextureView(
                                    &self.sprite_atlas.view,
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: wgpu::BindingResource::Sampler(&self.sampler),
                            },
                        ],
                    });
                self.bind_groups.push(bg);
            }

            // Vertex buffers use fixed-size quads (built once in resumed()),
            // so no rebuild needed here.

            self.info_text = format_anim_info(
                anim_index,
                self.total_anims,
                tribe,
                combo_idx,
                &combos,
                atlas.frames_per_dir,
            );
            println!("{}", self.info_text);

            self.atlas = atlas;
            self.current_frame = 0;
            self.current_anim = anim_index;
            self.current_tribe = tribe;
            self.unit_combos = combos;
            self.current_combo_idx = combo_idx;
        } else {
            println!("No animation data for anim index {}", anim_index);
        }
    }

    fn render(&mut self) {
        let output = match self.gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let proj = ortho_projection(
            self.gpu.size.width as f32 / 1.5,
            self.gpu.size.height as f32 / 1.5,
        );

        let fpd = self.atlas.frames_per_dir;
        let uv_scale_x = 1.0 / fpd as f32;
        let total_rows = (NUM_TRIBES * STORED_DIRECTIONS) as f32;
        let uv_scale_y = 1.0 / total_rows;
        let frame_uv_x = self.current_frame as f32 / fpd as f32;

        for dir in 0..NUM_DIRECTIONS {
            let (source_dir, mirrored) = get_source_direction(dir);
            let frame_uv_y =
                (self.current_tribe as usize * STORED_DIRECTIONS + source_dir) as f32 / total_rows;

            let uniforms = SpriteUniforms {
                projection: proj,
                uv_offset: [frame_uv_x, frame_uv_y],
                uv_scale: [uv_scale_x, uv_scale_y],
                mirror: [if mirrored { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
            };

            self.gpu.queue.write_buffer(
                &self.uniform_buffers[dir].buffer,
                0,
                bytemuck::bytes_of(&uniforms),
            );
        }

        // Prepare text overlay
        let screen_w = self.gpu.size.width as f32;
        let screen_h = self.gpu.size.height as f32;

        let white = [1.0f32, 1.0, 1.0, 1.0];
        let yellow = [1.0f32, 1.0, 0.3, 1.0];

        let help_lines: Vec<(&str, [f32; 4])> = vec![
            ("Unit Animation Viewer", white),
            ("", white),
            ("Space:      Pause/Resume", white),
            ("N/P/Tab:    Change anim", white),
            ("+/-:        Jump 10 anims", white),
            ("1-4:        Quick anims", white),
            ("T:          Cycle tribe", white),
            ("U:          Cycle unit features", white),
            ("Up/Down:    Speed", white),
            ("Left/Right: Frame step (paused)", white),
            ("Escape:     Quit", white),
        ];

        let (help_vb, help_bg, _help_ub, help_vc) = self.text_renderer.prepare_text(
            &self.gpu.device,
            screen_w,
            screen_h,
            &help_lines,
            8.0,
            8.0,
            1.5,
        );

        let frame_info = format!("Frame {}/{}", self.current_frame + 1, fpd);
        let info_lines: Vec<(&str, [f32; 4])> =
            vec![(&self.info_text, yellow), (&frame_info, yellow)];

        let (info_vb, info_bg, _info_ub, info_vc) = self.text_renderer.prepare_text(
            &self.gpu.device,
            screen_w,
            screen_h,
            &info_lines,
            screen_w - 500.0,
            8.0,
            1.5,
        );

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("sprite_encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sprite_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.05,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            // Draw sprites
            pass.set_pipeline(&self.pipeline);
            for dir in 0..NUM_DIRECTIONS {
                pass.set_bind_group(0, &self.bind_groups[dir], &[]);
                pass.set_vertex_buffer(0, self.vertex_buffers[dir].buffer.slice(..));
                pass.draw(0..6, 0..1);
            }

            // Draw text overlay
            pass.set_pipeline(&self.text_renderer.pipeline);

            if help_vc > 0 {
                pass.set_bind_group(0, &help_bg, &[]);
                pass.set_vertex_buffer(0, help_vb.buffer.slice(..));
                pass.draw(0..help_vc, 0..1);
            }

            if info_vc > 0 {
                pass.set_bind_group(0, &info_bg, &[]);
                pass.set_vertex_buffer(0, info_vb.buffer.slice(..));
                pass.draw(0..info_vc, 0..1);
            }
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn update(&mut self) {
        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_instant).as_secs_f32();
        self.last_instant = now;

        if !self.paused {
            self.anim_timer += dt;
            if self.anim_timer >= self.anim_speed {
                self.anim_timer -= self.anim_speed;
                self.current_frame = (self.current_frame + 1) % self.atlas.frames_per_dir.max(1);
            }
        }
    }

    fn handle_key(
        &mut self,
        key: KeyCode,
        sequences: &[AnimationSequence],
        container: &ContainerPSFB,
        palette: &[[u8; 4]],
        global_bbox: (i32, i32, i32, i32),
    ) {
        match key {
            KeyCode::Space => {
                self.paused = !self.paused;
                println!(
                    "Animation {}",
                    if self.paused { "paused" } else { "running" }
                );
            }
            KeyCode::ArrowUp => {
                self.anim_speed = (self.anim_speed - 0.02).max(0.02);
                println!("Speed: {:.2}s/frame", self.anim_speed);
            }
            KeyCode::ArrowDown => {
                self.anim_speed = (self.anim_speed + 0.02).min(0.5);
                println!("Speed: {:.2}s/frame", self.anim_speed);
            }
            KeyCode::ArrowRight if self.paused => {
                self.current_frame = (self.current_frame + 1) % self.atlas.frames_per_dir.max(1);
                println!("Frame {}/{}", self.current_frame, self.atlas.frames_per_dir);
            }
            KeyCode::ArrowLeft if self.paused => {
                let fpd = self.atlas.frames_per_dir.max(1);
                self.current_frame = if self.current_frame == 0 {
                    fpd - 1
                } else {
                    self.current_frame - 1
                };
                println!("Frame {}/{}", self.current_frame, self.atlas.frames_per_dir);
            }
            // Cycle animation forward (skip static poses)
            KeyCode::Tab | KeyCode::KeyN => {
                let next = find_next_animated(sequences, self.current_anim, 1, self.total_anims);
                self.rebuild_atlas(
                    sequences,
                    container,
                    palette,
                    next,
                    self.current_tribe,
                    self.current_combo_idx,
                    global_bbox,
                );
            }
            // Cycle animation backward (skip static poses)
            KeyCode::KeyP => {
                let prev = find_next_animated(sequences, self.current_anim, -1, self.total_anims);
                self.rebuild_atlas(
                    sequences,
                    container,
                    palette,
                    prev,
                    self.current_tribe,
                    self.current_combo_idx,
                    global_bbox,
                );
            }
            // Jump ~10 animations forward (skip static poses)
            KeyCode::Equal => {
                let target = (self.current_anim + 10) % self.total_anims;
                let next = find_next_animated(sequences, target.max(1) - 1, 1, self.total_anims);
                self.rebuild_atlas(
                    sequences,
                    container,
                    palette,
                    next,
                    self.current_tribe,
                    self.current_combo_idx,
                    global_bbox,
                );
            }
            // Jump ~10 animations backward (skip static poses)
            KeyCode::Minus => {
                let target = (self.current_anim + self.total_anims - 10) % self.total_anims;
                let prev = find_next_animated(sequences, target + 1, -1, self.total_anims);
                self.rebuild_atlas(
                    sequences,
                    container,
                    palette,
                    prev,
                    self.current_tribe,
                    self.current_combo_idx,
                    global_bbox,
                );
            }
            // Cycle tribe (UV-only, no atlas rebuild needed since atlas has all 4 tribes)
            KeyCode::KeyT => {
                self.current_tribe = (self.current_tribe + 1) % 4;
                let tribe_name = TRIBE_NAMES.get(self.current_tribe as usize).unwrap_or(&"?");
                println!("Tribe: {} ({})", self.current_tribe, tribe_name);
                self.info_text = format_anim_info(
                    self.current_anim,
                    self.total_anims,
                    self.current_tribe,
                    self.current_combo_idx,
                    &self.unit_combos,
                    self.atlas.frames_per_dir,
                );
            }
            // Cycle unit features overlay
            KeyCode::KeyU => {
                let next_combo = if self.unit_combos.is_empty() {
                    None
                } else {
                    match self.current_combo_idx {
                        None => Some(0),
                        Some(i) if i + 1 >= self.unit_combos.len() => None,
                        Some(i) => Some(i + 1),
                    }
                };
                self.rebuild_atlas(
                    sequences,
                    container,
                    palette,
                    self.current_anim,
                    self.current_tribe,
                    next_combo,
                    global_bbox,
                );
            }
            // Quick-jump to known animations
            KeyCode::Digit1 => {
                self.rebuild_atlas(
                    sequences,
                    container,
                    palette,
                    15,
                    self.current_tribe,
                    self.current_combo_idx,
                    global_bbox,
                );
            }
            KeyCode::Digit2 => {
                self.rebuild_atlas(
                    sequences,
                    container,
                    palette,
                    20,
                    self.current_tribe,
                    self.current_combo_idx,
                    global_bbox,
                );
            }
            KeyCode::Digit3 => {
                self.rebuild_atlas(
                    sequences,
                    container,
                    palette,
                    21,
                    self.current_tribe,
                    self.current_combo_idx,
                    global_bbox,
                );
            }
            KeyCode::Digit4 => {
                self.rebuild_atlas(
                    sequences,
                    container,
                    palette,
                    26,
                    self.current_tribe,
                    self.current_combo_idx,
                    global_bbox,
                );
            }
            _ => {}
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Unit Animation Viewer")
                        .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        // Build initial atlas
        let initial_anim = self.initial_anim;
        let initial_tribe = self.initial_tribe;
        let total_anims = self.sequences.len() / DIRS_PER_ANIM;

        let initial_vstart = initial_anim * DIRS_PER_ANIM;
        let (atlas_w, atlas_h, rgba, _fw, _fh, max_frames, _max_y) = build_tribe_atlas(
            &self.sequences,
            &self.container,
            &self.palette,
            initial_vstart,
            Some(None),
            Some(self.global_bbox),
        )
        .expect("Failed to build initial animation atlas");
        let atlas = SpriteAtlas {
            frames_per_dir: max_frames,
        };

        let base = initial_anim * DIRS_PER_ANIM;
        let initial_combos = discover_unit_combos(&self.sequences, base);

        let sprite_atlas = GpuTexture::new_2d(
            device,
            &gpu.queue,
            atlas_w,
            atlas_h,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &rgba,
            "sprite_atlas",
        );

        let sampler = GpuTexture::create_sampler(device, true);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite_bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let mut uniform_buffers = Vec::new();
        let mut bind_groups = Vec::new();
        for dir in 0..NUM_DIRECTIONS {
            let ub = GpuBuffer::new_uniform(
                device,
                std::mem::size_of::<SpriteUniforms>() as u64,
                &format!("sprite_uniforms_{}", dir),
            );
            let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(&format!("sprite_bg_{}", dir)),
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: ub.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&sprite_atlas.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });
            uniform_buffers.push(ub);
            bind_groups.push(bg);
        }

        let vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SpriteVertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 8,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        };

        let shader_source = include_str!("../../shaders/sprite.wgsl");
        let pipeline = create_pipeline(
            device,
            shader_source,
            &[vertex_layout],
            &[&bind_group_layout],
            gpu.surface_format(),
            false,
            wgpu::PrimitiveTopology::TriangleList,
            "sprite_pipeline",
        );

        // Fixed-size quads (matching bevy_demo5's fixed Rectangle::new(100, 100))
        let hw = 130.0; // 260×340 default quad size
        let hh = 170.0;
        let radius = 200.0;

        let mut vertex_buffers = Vec::new();
        for dir in 0..NUM_DIRECTIONS {
            let angle = dir as f32 * std::f32::consts::TAU / NUM_DIRECTIONS as f32;
            let cx = angle.sin() * radius;
            let cy = angle.cos() * radius;
            let quad = make_quad(cx, cy, hw, hh);
            let buf = GpuBuffer::new_vertex(
                device,
                bytemuck::cast_slice(&quad),
                &format!("quad_{}", dir),
            );
            vertex_buffers.push(buf);
        }

        let info_text = format_anim_info(
            initial_anim,
            total_anims,
            initial_tribe,
            None,
            &initial_combos,
            atlas.frames_per_dir,
        );
        println!("{}", info_text);

        let text_renderer = TextRenderer::new(device, &gpu.queue, gpu.surface_format());

        self.state = Some(ViewerState {
            gpu,
            pipeline,
            bind_group_layout,
            bind_groups,
            vertex_buffers,
            uniform_buffers,
            sampler,
            sprite_atlas,
            atlas,
            text_renderer,
            info_text,
            current_anim: initial_anim,
            total_anims,
            current_tribe: initial_tribe,
            unit_combos: initial_combos,
            current_combo_idx: None,
            current_frame: 0,
            anim_timer: 0.0,
            anim_speed: DEFAULT_SPEED,
            paused: false,
            last_instant: std::time::Instant::now(),
        });
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _wid: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if let Some(state) = &mut self.state {
                    state.gpu.resize(size);
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        if key == KeyCode::Escape {
                            event_loop.exit();
                            return;
                        }
                        if let Some(state) = &mut self.state {
                            state.handle_key(
                                key,
                                &self.sequences,
                                &self.container,
                                &self.palette,
                                self.global_bbox,
                            );
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
                    state.update();
                    state.render();
                }
            }
            _ => {}
        }

        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/******************************************************************************/
// CLI & main
/******************************************************************************/

fn cli() -> Command {
    Command::new("unit-viewer")
        .about("Unit animation viewer for Populous: The Beginning")
        .arg(
            Arg::new("base")
                .long("base")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Path to game data directory"),
        )
        .arg(
            Arg::new("anim")
                .long("anim")
                .value_parser(clap::value_parser!(usize))
                .default_value("15")
                .help("Start with global animation index N (15=Brave Idle, 20=Shaman Idle)"),
        )
        .arg(
            Arg::new("tribe")
                .long("tribe")
                .value_parser(clap::value_parser!(u8))
                .default_value("0")
                .help("Start with tribe N (0-3)"),
        )
}

fn main() {
    let matches = cli().get_matches();

    let base = matches
        .get_one::<PathBuf>("base")
        .cloned()
        .unwrap_or_else(|| {
            PathBuf::from("/Users/adriencandiotti/Library/Containers/com.isaacmarovitz.Whisky/Bottles/74820C9D-5F8C-4BFE-B5DB-90E1DE818D3F/drive_c/GOG Games/Populous - The Beginning")
        });

    let initial_anim = matches
        .get_one::<usize>("anim")
        .copied()
        .unwrap_or(DEFAULT_ANIM);
    let initial_tribe = matches.get_one::<u8>("tribe").copied().unwrap_or(0);

    let data_dir = base.join("data");

    // Load palette
    let palette =
        load_palette(&data_dir.join("pal0-0.dat")).expect("Failed to load palette from pal0-0.dat");

    // Load sprite container
    let sprite_path = data_dir.join("HSPR0-0.DAT");
    let container = ContainerPSFB::from_file(&sprite_path).expect("Failed to load HSPR0-0.DAT");
    println!("Loaded {} sprites from HSPR0-0.DAT", container.len());

    // Load animation data
    let anims_data = AnimationsData::from_path(&data_dir);
    println!(
        "Animation data: {} vele, {} vfra, {} vstart",
        anims_data.vele.len(),
        anims_data.vfra.len(),
        anims_data.vstart.len()
    );

    let sequences = AnimationSequence::from_data(&anims_data);
    println!(
        "Loaded {} animation sequences ({} animations)",
        sequences.len(),
        sequences.len() / 8
    );

    // Compute global bounding box across ALL animations for consistent sizing
    let global_bbox = compute_global_bbox(&sequences, &container);
    let (gx0, gy0, gx1, gy1) = global_bbox;
    println!(
        "Global bbox: {}x{} (x: {}..{}, y: {}..{})",
        gx1 - gx0,
        gy1 - gy0,
        gx0,
        gx1,
        gy0,
        gy1
    );

    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", "info")
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    println!("\nUnit Animation Viewer");
    println!("---------------------");
    println!("Space:       Pause/Resume");
    println!("N/P/Tab:     Change anim (skips static)");
    println!("+/-:         Jump ~10 anims");
    println!("1-4:         Quick anims (Brave/Shaman/BraveWalk/ShamanWalk)");
    println!("T:           Cycle tribe color (Blue/Red/Yellow/Green)");
    println!("U:           Cycle unit features");
    println!("Up/Down:     Speed");
    println!("Left/Right:  Frame step (paused)");
    println!("Escape:      Quit");
    println!();

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(
        container,
        palette,
        sequences,
        initial_anim,
        initial_tribe,
        global_bbox,
    );
    event_loop.run_app(&mut app).unwrap();
}
