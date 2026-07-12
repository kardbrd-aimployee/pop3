//! Sky Texture Viewer
//!
//! Standalone viewer for Populous: The Beginning sky textures.
//! Cycles through all sky variants (0-9, a-z) with their matched palettes.
//!
//! Controls:
//!   N/P         - Next/Previous sky variant
//!   Q/E         - Scroll horizontally
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

/******************************************************************************/
// Sky data
/******************************************************************************/

const KEYS: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "6", "7", "8", "9", "a", "b", "c", "d", "e", "f", "g", "h", "i",
    "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
];

struct SkyVariant {
    key: String,
    sky_path: PathBuf,
    pal_path: PathBuf,
}

struct SkyImage {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

fn discover_sky_variants(data_dir: &Path) -> Vec<SkyVariant> {
    let mut variants = Vec::new();
    for key in KEYS {
        let sky_path = data_dir.join(format!("sky0-{key}.dat"));
        let pal_path = data_dir.join(format!("pal0-{key}.dat"));
        if sky_path.exists() && pal_path.exists() {
            variants.push(SkyVariant {
                key: key.to_string(),
                sky_path,
                pal_path,
            });
        }
    }
    variants
}

fn load_sky_rgba(variant: &SkyVariant) -> SkyImage {
    let sky_raw = std::fs::read(&variant.sky_path).expect("Failed to read sky file");
    let pal = std::fs::read(&variant.pal_path).expect("Failed to read palette file");

    // Always 512x512 — the game processes exactly 0x40000 bytes then adds 0x70
    let width = 512usize;
    let height = 512usize;
    let pixel_count = width * height;

    let indices = &sky_raw[..pixel_count.min(sky_raw.len())];
    let min_idx = indices.iter().copied().min().unwrap_or(0);
    let max_idx = indices.iter().copied().max().unwrap_or(0);

    // After +0x70: show the actual palette indices the game would use
    let remapped_min = min_idx.wrapping_add(0x70);
    let remapped_max = max_idx.wrapping_add(0x70);
    println!("sky0-{}.dat: {} bytes (512x512), raw range [{}-{}], after +0x70: [{}-{}] (0x{:02x}-0x{:02x})",
        variant.key, sky_raw.len(), min_idx, max_idx,
        remapped_min, remapped_max, remapped_min, remapped_max);

    // Print palette colors at the remapped range
    if pal.len() >= 256 * 4 {
        let start = remapped_min as usize;
        let end = (remapped_max as usize + 1).min(256);
        print!("  palette [0x{:02x}..0x{:02x}]: ", start, end);
        for i in start..end {
            let off = i * 4;
            print!("#{:02x}{:02x}{:02x} ", pal[off], pal[off + 1], pal[off + 2]);
        }
        println!();
    }

    // Game adds 0x70 to every sky byte, then uses result as direct palette index
    // (the interp table from FUN_004dc3f0 is for mode 2's flat gradient, not texture mode)
    let mut rgba = vec![0u8; pixel_count * 4];
    for (i, &idx) in indices.iter().enumerate() {
        let pal_idx = idx.wrapping_add(0x70) as usize;
        let off = pal_idx * 4;
        if off + 2 < pal.len() {
            rgba[i * 4] = pal[off];
            rgba[i * 4 + 1] = pal[off + 1];
            rgba[i * 4 + 2] = pal[off + 2];
            rgba[i * 4 + 3] = 255;
        }
    }
    SkyImage {
        rgba,
        width: width as u32,
        height: height as u32,
    }
}

/******************************************************************************/
// Application
/******************************************************************************/

struct App {
    window: Option<Arc<Window>>,
    state: Option<ViewerState>,
    variants: Vec<SkyVariant>,
    current_index: usize,
}

struct ViewerState {
    gpu: GpuContext,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    sky_texture: GpuTexture,
    sampler: wgpu::Sampler,
    uniform_buffer: GpuBuffer,
    yaw_offset: f32,
}

impl App {
    fn new(variants: Vec<SkyVariant>) -> Self {
        Self {
            window: None,
            state: None,
            variants,
            current_index: 0,
        }
    }

    fn load_sky(&mut self, index: usize) {
        self.current_index = index;
        let variant = &self.variants[index];
        let sky = load_sky_rgba(variant);

        if let Some(state) = &mut self.state {
            // Update texture with correct dimensions
            state.sky_texture = GpuTexture::new_2d(
                &state.gpu.device,
                &state.gpu.queue,
                sky.width,
                sky.height,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                &sky.rgba,
                "sky_texture",
            );

            // Rebuild bind group with new texture
            state.bind_group = state
                .gpu
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("sky_bind_group"),
                    layout: &state.bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: state.uniform_buffer.buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&state.sky_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&state.sampler),
                        },
                    ],
                });

            state.yaw_offset = 0.0;
        }

        // Update window title
        if let Some(window) = &self.window {
            window.set_title(&format!(
                "Sky Viewer — sky0-{}.dat [{}/{}]",
                variant.key,
                index + 1,
                self.variants.len()
            ));
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
                        .with_title("Sky Viewer")
                        .with_inner_size(winit::dpi::LogicalSize::new(800, 600)),
                )
                .unwrap(),
        );
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        // Load initial sky
        let variant = &self.variants[self.current_index];
        let sky = load_sky_rgba(variant);

        let sky_texture = GpuTexture::new_2d(
            device,
            &gpu.queue,
            sky.width,
            sky.height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            &sky.rgba,
            "sky_texture",
        );

        let sampler = GpuTexture::create_sampler(device, false);

        // Uniform buffer for yaw offset (16 bytes for alignment)
        let uniform_buffer = GpuBuffer::new_uniform(device, 16, "sky_uniform");
        gpu.queue.write_buffer(
            &uniform_buffer.buffer,
            0,
            bytemuck::bytes_of(&[0.0f32, 0.0f32, 0.0f32, 0.0f32]),
        );

        // Bind group layout matching sky.wgsl
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky_bgl"),
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&sky_texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Pipeline — no vertex buffers (fullscreen triangle), no depth
        let shader_source = include_str!("../../shaders/sky.wgsl");
        let pipeline = create_pipeline(
            device,
            shader_source,
            &[],
            &[&bind_group_layout],
            gpu.surface_format(),
            false,
            wgpu::PrimitiveTopology::TriangleList,
            "sky_pipeline",
        );

        if let Some(window) = &self.window {
            window.set_title(&format!(
                "Sky Viewer — sky0-{}.dat [1/{}]",
                self.variants[0].key,
                self.variants.len()
            ));
        }

        self.state = Some(ViewerState {
            gpu,
            pipeline,
            bind_group_layout,
            bind_group,
            sky_texture,
            sampler,
            uniform_buffer,
            yaw_offset: 0.0,
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
                        match key {
                            KeyCode::Escape => {
                                event_loop.exit();
                                return;
                            }
                            KeyCode::KeyN | KeyCode::ArrowRight => {
                                let next = (self.current_index + 1) % self.variants.len();
                                self.load_sky(next);
                            }
                            KeyCode::KeyP | KeyCode::ArrowLeft => {
                                let prev = if self.current_index == 0 {
                                    self.variants.len() - 1
                                } else {
                                    self.current_index - 1
                                };
                                self.load_sky(prev);
                            }
                            KeyCode::KeyQ => {
                                if let Some(state) = &mut self.state {
                                    state.yaw_offset -= 0.05;
                                }
                            }
                            KeyCode::KeyE => {
                                if let Some(state) = &mut self.state {
                                    state.yaw_offset += 0.05;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if let Some(state) = &mut self.state {
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

impl ViewerState {
    fn render(&mut self) {
        // Update yaw uniform
        self.gpu.queue.write_buffer(
            &self.uniform_buffer.buffer,
            0,
            bytemuck::bytes_of(&[self.yaw_offset, 0.0f32, 0.0f32, 0.0f32]),
        );

        let output = match self.gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("sky_encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sky_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

/******************************************************************************/
// CLI & main
/******************************************************************************/

fn cli() -> Command {
    Command::new("sky-viewer")
        .about("Sky texture viewer for Populous: The Beginning")
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
    let variants = discover_sky_variants(&data_dir);

    if variants.is_empty() {
        eprintln!("No sky variants found in {:?}", data_dir);
        eprintln!("Expected files like sky0-0.dat + pal0-0.dat");
        std::process::exit(1);
    }

    println!(
        "Found {} sky variants: {}",
        variants.len(),
        variants
            .iter()
            .map(|v| v.key.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    );

    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", "info")
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(variants);
    event_loop.run_app(&mut app).unwrap();
}
