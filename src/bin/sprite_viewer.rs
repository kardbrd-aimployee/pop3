//! Sprite Animation Viewer
//!
//! A wgpu+winit port of the Bevy sprite_demo. Shows all 8 animation directions
//! in a circle with keyboard controls for animation playback.
//!
//! Controls:
//!   Space       - Pause/Resume
//!   Up/Down     - Animation speed
//!   Left/Right  - Frame step (when paused)
//!   Tab/N/P     - Switch character
//!   Q/E         - Rotate camera

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

use pop3::data::psfb::ContainerPSFB;
use pop3::data::types::BinDeserializer;

/******************************************************************************/
// Animation definitions (ported from bevy_demo5/src/animation_data.rs)
/******************************************************************************/

struct AnimationDef {
    sprite_start: u16,
    frames_per_dir: u8,
    name: &'static str,
}

const STORED_DIRECTIONS: usize = 5;
const NUM_DIRECTIONS: usize = 8;
const DEFAULT_SPEED: f32 = 0.08;

const CHARACTER_ANIMATIONS: &[AnimationDef] = &[
    AnimationDef {
        sprite_start: 7578,
        frames_per_dir: 8,
        name: "Shaman Idle 1",
    },
    AnimationDef {
        sprite_start: 7618,
        frames_per_dir: 8,
        name: "Shaman Idle 2",
    },
    AnimationDef {
        sprite_start: 7658,
        frames_per_dir: 8,
        name: "Shaman Idle 3",
    },
    AnimationDef {
        sprite_start: 7698,
        frames_per_dir: 8,
        name: "Shaman Idle 4",
    },
    AnimationDef {
        sprite_start: 7738,
        frames_per_dir: 3,
        name: "Animation 5",
    },
    AnimationDef {
        sprite_start: 7753,
        frames_per_dir: 3,
        name: "Animation 6",
    },
    AnimationDef {
        sprite_start: 7768,
        frames_per_dir: 3,
        name: "Animation 7",
    },
    AnimationDef {
        sprite_start: 7783,
        frames_per_dir: 3,
        name: "Animation 8",
    },
    AnimationDef {
        sprite_start: 7798,
        frames_per_dir: 3,
        name: "Animation 9",
    },
    AnimationDef {
        sprite_start: 7813,
        frames_per_dir: 3,
        name: "Animation 10",
    },
];

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
// Atlas building
/******************************************************************************/

struct SpriteAtlas {
    rgba: Vec<u8>,
    atlas_width: u32,
    atlas_height: u32,
    frame_width: u32,
    frame_height: u32,
    frames_per_dir: u32,
}

/// Build a texture atlas for a character animation.
/// Layout: rows = stored directions (0..5), cols = frames (0..fpd)
/// Each cell is padded to (max_width x max_height).
fn build_atlas(
    container: &ContainerPSFB,
    palette: &[[u8; 4]],
    anim: &AnimationDef,
) -> Option<SpriteAtlas> {
    let fpd = anim.frames_per_dir as usize;

    // First pass: find max dimensions
    let mut max_w: u16 = 0;
    let mut max_h: u16 = 0;
    for dir in 0..STORED_DIRECTIONS {
        for f in 0..fpd {
            let idx = anim.sprite_start as usize + dir * fpd + f;
            if let Some(info) = container.get_info(idx) {
                max_w = max_w.max(info.width);
                max_h = max_h.max(info.height);
            }
        }
    }
    if max_w == 0 || max_h == 0 {
        return None;
    }

    let fw = max_w as u32;
    let fh = max_h as u32;
    let atlas_w = fw * fpd as u32;
    let atlas_h = fh * STORED_DIRECTIONS as u32;
    let mut rgba = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    // Second pass: decode sprites into atlas
    for dir in 0..STORED_DIRECTIONS {
        for f in 0..fpd {
            let idx = anim.sprite_start as usize + dir * fpd + f;
            if let Some(image) = container.get_image(idx) {
                let info = container.get_info(idx).unwrap();
                let sw = info.width as u32;
                let sh = info.height as u32;
                // Center the sprite in its cell
                let ox = (fw - sw) / 2;
                let oy = (fh - sh) / 2;
                let cell_x = f as u32 * fw;
                let cell_y = dir as u32 * fh;

                for y in 0..sh {
                    for x in 0..sw {
                        let src = image.data[(y * sw + x) as usize];
                        let dst_x = cell_x + ox + x;
                        let dst_y = cell_y + oy + y;
                        let dst_off = ((dst_y * atlas_w + dst_x) * 4) as usize;
                        if src == 0 {
                            // Transparent
                            rgba[dst_off] = 0;
                            rgba[dst_off + 1] = 0;
                            rgba[dst_off + 2] = 0;
                            rgba[dst_off + 3] = 0;
                        } else {
                            let c = palette.get(src as usize).unwrap_or(&[255, 0, 255, 255]);
                            rgba[dst_off] = c[0];
                            rgba[dst_off + 1] = c[1];
                            rgba[dst_off + 2] = c[2];
                            rgba[dst_off + 3] = 255;
                        }
                    }
                }
            }
        }
    }

    Some(SpriteAtlas {
        rgba,
        atlas_width: atlas_w,
        atlas_height: atlas_h,
        frame_width: fw,
        frame_height: fh,
        frames_per_dir: fpd as u32,
    })
}

/******************************************************************************/
// Uniform data
/******************************************************************************/

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SpriteUniforms {
    projection: [[f32; 4]; 4], // offset 0, size 64
    uv_offset: [f32; 2],       // offset 64, size 8
    uv_scale: [f32; 2],        // offset 72, size 8
    mirror: [f32; 4],          // offset 80, size 16 (vec4 in WGSL)
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

/// Create a quad centered at (cx, cy) with given half-sizes
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

struct App {
    window: Option<Arc<Window>>,
    state: Option<ViewerState>,
    container: ContainerPSFB,
    palette: Vec<[u8; 4]>,
}

struct ViewerState {
    gpu: GpuContext,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_groups: Vec<wgpu::BindGroup>, // One per direction
    vertex_buffers: Vec<GpuBuffer>,    // One per direction
    uniform_buffers: Vec<GpuBuffer>,   // One per direction
    sampler: wgpu::Sampler,
    sprite_atlas: GpuTexture,

    // Atlas info
    atlas: SpriteAtlas,

    // Animation state
    current_character: usize,
    current_frame: u32,
    anim_timer: f32,
    anim_speed: f32,
    paused: bool,
    camera_angle: f32,
    last_instant: std::time::Instant,
}

impl App {
    fn new(container: ContainerPSFB, palette: Vec<[u8; 4]>) -> Self {
        Self {
            window: None,
            state: None,
            container,
            palette,
        }
    }
}

impl ViewerState {
    fn rebuild_atlas(&mut self, container: &ContainerPSFB, palette: &[[u8; 4]], char_idx: usize) {
        let anim = &CHARACTER_ANIMATIONS[char_idx];
        if let Some(atlas) = build_atlas(container, palette, anim) {
            self.sprite_atlas = GpuTexture::new_2d(
                &self.gpu.device,
                &self.gpu.queue,
                atlas.atlas_width,
                atlas.atlas_height,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                &atlas.rgba,
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

            // Rebuild vertex buffers with new aspect ratio
            let sprite_scale = 60.0;
            let aspect = atlas.frame_width as f32 / atlas.frame_height as f32;
            let hw = sprite_scale * aspect;
            let hh = sprite_scale;
            let radius = 200.0;

            self.vertex_buffers.clear();
            for dir in 0..NUM_DIRECTIONS {
                let angle = dir as f32 * std::f32::consts::TAU / NUM_DIRECTIONS as f32;
                let cx = angle.sin() * radius;
                let cy = -angle.cos() * radius;
                let quad = make_quad(cx, cy, hw, hh);
                let buf = GpuBuffer::new_vertex(
                    &self.gpu.device,
                    bytemuck::cast_slice(&quad),
                    &format!("quad_{}", dir),
                );
                self.vertex_buffers.push(buf);
            }

            self.atlas = atlas;
            self.current_frame = 0;
            self.current_character = char_idx;
            println!(
                "Loaded: {} ({} frames/dir, atlas {}x{})",
                anim.name, anim.frames_per_dir, self.atlas.atlas_width, self.atlas.atlas_height
            );
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
            self.gpu.size.width as f32 / 2.0,
            self.gpu.size.height as f32 / 2.0,
        );

        let fpd = self.atlas.frames_per_dir;
        let uv_scale_x = 1.0 / fpd as f32;
        let uv_scale_y = 1.0 / STORED_DIRECTIONS as f32;
        let frame_uv_x = self.current_frame as f32 / fpd as f32;

        let ca = self.camera_angle.cos();
        let sa = self.camera_angle.sin();
        let rot_proj = [
            [
                proj[0][0] * ca + proj[1][0] * sa,
                proj[0][1] * ca + proj[1][1] * sa,
                0.0,
                0.0,
            ],
            [
                proj[1][0] * ca - proj[0][0] * sa,
                proj[1][1] * ca - proj[0][1] * sa,
                0.0,
                0.0,
            ],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        // Write all uniform buffers BEFORE starting the render pass
        for dir in 0..NUM_DIRECTIONS {
            let (source_dir, mirrored) = get_source_direction(dir);
            let frame_uv_y = source_dir as f32 / STORED_DIRECTIONS as f32;

            let uniforms = SpriteUniforms {
                projection: rot_proj,
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

            pass.set_pipeline(&self.pipeline);

            for dir in 0..NUM_DIRECTIONS {
                pass.set_bind_group(0, &self.bind_groups[dir], &[]);
                pass.set_vertex_buffer(0, self.vertex_buffers[dir].buffer.slice(..));
                pass.draw(0..6, 0..1);
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
                self.current_frame = (self.current_frame + 1) % self.atlas.frames_per_dir;
            }
        }
    }

    fn handle_key(&mut self, key: KeyCode, container: &ContainerPSFB, palette: &[[u8; 4]]) {
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
                self.current_frame = (self.current_frame + 1) % self.atlas.frames_per_dir;
                println!("Frame {}/{}", self.current_frame, self.atlas.frames_per_dir);
            }
            KeyCode::ArrowLeft if self.paused => {
                self.current_frame = if self.current_frame == 0 {
                    self.atlas.frames_per_dir - 1
                } else {
                    self.current_frame - 1
                };
                println!("Frame {}/{}", self.current_frame, self.atlas.frames_per_dir);
            }
            KeyCode::Tab | KeyCode::KeyN => {
                let next = (self.current_character + 1) % CHARACTER_ANIMATIONS.len();
                self.rebuild_atlas(container, palette, next);
            }
            KeyCode::KeyP => {
                let prev = if self.current_character == 0 {
                    CHARACTER_ANIMATIONS.len() - 1
                } else {
                    self.current_character - 1
                };
                self.rebuild_atlas(container, palette, prev);
            }
            KeyCode::KeyQ => {
                self.camera_angle += 0.1;
            }
            KeyCode::KeyE => {
                self.camera_angle -= 0.1;
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
                        .with_title("Sprite Viewer")
                        .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        // Build initial atlas
        let anim = &CHARACTER_ANIMATIONS[0];
        let atlas = build_atlas(&self.container, &self.palette, anim)
            .expect("Failed to build initial sprite atlas");

        // Create GPU texture for atlas
        let sprite_atlas = GpuTexture::new_2d(
            device,
            &gpu.queue,
            atlas.atlas_width,
            atlas.atlas_height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &atlas.rgba,
            "sprite_atlas",
        );

        let sampler = GpuTexture::create_sampler(device, true);

        // Bind group layout
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

        // One uniform buffer + bind group per direction
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

        // Vertex layout
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

        // Pipeline
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

        // Create vertex buffers for 8 directions in a circle
        let sprite_scale = 60.0;
        let aspect = atlas.frame_width as f32 / atlas.frame_height as f32;
        let hw = sprite_scale * aspect;
        let hh = sprite_scale;
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

        println!(
            "Loaded: {} ({} frames/dir, atlas {}x{})",
            anim.name, anim.frames_per_dir, atlas.atlas_width, atlas.atlas_height
        );

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
            current_character: 0,
            current_frame: 0,
            anim_timer: 0.0,
            anim_speed: DEFAULT_SPEED,
            paused: false,
            camera_angle: 0.0,
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
                            state.handle_key(key, &self.container, &self.palette);
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

        // Request continuous redraw for animation
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }
}

/******************************************************************************/
// CLI & main
/******************************************************************************/

fn cli() -> Command {
    Command::new("sprite-viewer")
        .about("Sprite animation viewer for Populous: The Beginning")
        .arg(
            Arg::new("base")
                .long("base")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Path to game data directory"),
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

    let data_dir = base.join("data");

    // Load palette
    let palette =
        load_palette(&data_dir.join("pal0-0.dat")).expect("Failed to load palette from pal0-0.dat");

    // Load sprite container
    let sprite_path = data_dir.join("HSPR0-0.DAT");
    let container = ContainerPSFB::from_file(&sprite_path).expect("Failed to load HSPR0-0.DAT");

    println!("Loaded {} sprites from HSPR0-0.DAT", container.len());

    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", "info")
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(container, palette);
    event_loop.run_app(&mut app).unwrap();
}
