use std::path::{Path, PathBuf};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, ArgAction, Command};

use cgmath::{InnerSpace, Matrix4, Point3, Vector2, Vector3};

use pop3::data::types::BinDeserializer;
use pop3::render::camera::*;
use pop3::render::envelop::*;
use pop3::render::model::{MeshModel, VertexModel};
use pop3::render::tex_model::{TexModel, TexVertex};

use pop3::data::animation::{
    build_tribe_atlas, AnimationSequence, AnimationsData, UNIT_IDLE_ANIMS,
};
use pop3::data::bl320::make_bl320_texture_rgba;
use pop3::data::level::{GlobeTextureParams, LevelPaths};
use pop3::data::objects::{mk_pop_object, Object3D};
use pop3::data::psfb::ContainerPSFB;

use pop3::render::gpu::buffer::GpuBuffer;
use pop3::render::gpu::context::GpuContext;
use pop3::render::gpu::pipeline::create_pipeline;
use pop3::render::gpu::texture::GpuTexture;

/******************************************************************************/

const SHADOW_MAP_SIZE: u32 = 2048;

fn mk_pop_envelope(device: &wgpu::Device, object: &Object3D) -> ModelEnvelop<TexModel> {
    let model = mk_pop_object(object);
    let m = vec![(RenderType::Triangles, model)];
    let mut e = ModelEnvelop::<TexModel>::new(device, m);
    if let Some(m) = e.get(0) {
        m.location[2] = -0.5; // base sits on ground plane (z = -0.5)
        m.angles[0] = 90.0; // rotate Y-up model to Z-up world
        m.scale = (object.coord_scale() / 300.0) * 0.5;
    }
    e
}

fn mk_empty_envelope(device: &wgpu::Device) -> ModelEnvelop<TexModel> {
    let model: TexModel = MeshModel::new();
    let m = vec![(RenderType::Triangles, model)];
    ModelEnvelop::<TexModel>::new(device, m)
}

/// Build ground plane vertices: flat quad at z = -0.5
fn ground_vertices() -> Vec<[f32; 3]> {
    let z = -0.5_f32;
    let s = 3.0_f32;
    vec![
        [-s, -s, z],
        [s, -s, z],
        [s, s, z],
        [-s, -s, z],
        [s, s, z],
        [-s, s, z],
    ]
}

/// Build a camera-facing billboard quad for a sprite at a fixed world position.
/// Returns a ModelEnvelop using TexVertex format (same as buildings).
fn build_sprite_billboard(
    device: &wgpu::Device,
    camera: &Camera,
    pos: Vector3<f32>,
    sprite_h: f32,
    aspect: f32,
    _frame_w: u32,
    _frame_h: u32,
    frames_per_dir: u32,
) -> ModelEnvelop<TexModel> {
    let half_w = sprite_h * aspect / 2.0;

    // Billboard stands upright (Z-up) and faces the camera horizontally.
    let az = (camera.angle_z as f32).to_radians();
    let up = Vector3::new(0.0_f32, 0.0, 1.0);
    let right = Vector3::new(az.cos(), -az.sin(), 0.0);

    // UV: single frame (frame 0, direction 0, tribe 0)
    let u_left = 0.0_f32;
    let u_right = 1.0 / frames_per_dir as f32;
    let v_top = 0.0_f32;
    let v_bottom = 1.0 / 20.0; // 4 tribes * 5 stored directions = 20 rows

    let bl = pos - right * half_w;
    let br = pos + right * half_w;
    let tl = bl + up * sprite_h;
    let tr = br + up * sprite_h;

    let v = |p: Vector3<f32>, u: f32, v: f32| -> TexVertex {
        TexVertex {
            coord: p,
            uv: Vector2::new(u, v),
            tex_id: 0,
        }
    };

    let mut model: TexModel = MeshModel::new();
    model.push_vertex(v(bl, u_left, v_bottom));
    model.push_vertex(v(br, u_right, v_bottom));
    model.push_vertex(v(tr, u_right, v_top));
    model.push_vertex(v(bl, u_left, v_bottom));
    model.push_vertex(v(tr, u_right, v_top));
    model.push_vertex(v(tl, u_left, v_top));

    let m = vec![(RenderType::Triangles, model)];
    ModelEnvelop::<TexModel>::new(device, m)
}

/// Correction matrix: cgmath uses OpenGL z range [-1, 1], wgpu expects [0, 1].
/// Maps z_out = z_in * 0.5 + 0.5.
#[rustfmt::skip]
const OPENGL_TO_WGPU: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

fn compute_light_mvp(sun_azimuth: f32, sun_elevation: f32) -> Matrix4<f32> {
    let dir = Vector3::new(
        sun_azimuth.cos() * sun_elevation.cos(),
        sun_azimuth.sin() * sun_elevation.cos(),
        -sun_elevation.sin(),
    )
    .normalize();
    let center = Point3::new(0.0_f32, 0.0, -0.25);
    let eye = Point3::new(
        center.x - dir.x * 10.0,
        center.y - dir.y * 10.0,
        center.z - dir.z * 10.0,
    );
    let up = if dir.z.abs() > 0.99 {
        Vector3::new(0.0_f32, 1.0, 0.0)
    } else {
        Vector3::new(0.0_f32, 0.0, 1.0)
    };
    let light_view = Matrix4::look_at_rh(eye, center, up);
    let light_proj = cgmath::ortho(-4.0_f32, 4.0, -4.0, 4.0, 0.1, 25.0);
    OPENGL_TO_WGPU * light_proj * light_view
}

fn compute_sun_dir(sun_azimuth: f32, sun_elevation: f32) -> [f32; 4] {
    let dir = Vector3::new(
        sun_azimuth.cos() * sun_elevation.cos(),
        sun_azimuth.sin() * sun_elevation.cos(),
        -sun_elevation.sin(),
    )
    .normalize();
    [dir.x, dir.y, dir.z, 0.0]
}

/******************************************************************************/

fn cli() -> Command {
    Command::new("shadow-demo")
        .about("Shadow mapping demo")
        .args(&[
            Arg::new("base")
                .long("base")
                .action(ArgAction::Set)
                .value_name("BASE_PATH")
                .value_parser(clap::value_parser!(PathBuf))
                .help("Path to POP3 directory"),
            Arg::new("landtype")
                .long("landtype")
                .action(ArgAction::Set)
                .value_name("LAND_TYPE")
                .value_parser(clap::builder::StringValueParser::new())
                .help("Override land type"),
            Arg::new("obj_num")
                .long("obj_num")
                .action(ArgAction::Set)
                .value_name("OBJ")
                .value_parser(clap::value_parser!(u16).range(0..16000))
                .help("Object index (0-based)"),
        ])
}

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,
    screen: Screen,
    camera: Camera,
    do_render: bool,
    // Init data
    base: PathBuf,
    landtype: String,
    objects_3d: Vec<Option<Object3D>>,
    obj_num: usize,
    scale: f32,
    scale_origin: f32,
    // Sun
    sun_azimuth: f32,
    sun_elevation: f32,
    // Building model
    pop_obj: Option<ModelEnvelop<TexModel>>,
    // Buffers
    camera_mvp_buffer: Option<GpuBuffer>,
    model_transform_buffer: Option<GpuBuffer>,
    light_mvp_buffer: Option<GpuBuffer>,
    sun_dir_buffer: Option<GpuBuffer>,
    // Shadow pass
    shadow_depth_view: Option<wgpu::TextureView>,
    shadow_depth_pipeline: Option<wgpu::RenderPipeline>,
    shadow_pass_group0: Option<wgpu::BindGroup>,
    shadow_pass_group1: Option<wgpu::BindGroup>,
    // Main pass - ground
    ground_pipeline: Option<wgpu::RenderPipeline>,
    ground_vertex_buffer: Option<GpuBuffer>,
    ground_vertex_count: u32,
    ground_group0: Option<wgpu::BindGroup>,
    shadow_recv_group: Option<wgpu::BindGroup>,
    // Main pass - object
    object_pipeline: Option<wgpu::RenderPipeline>,
    object_group0: Option<wgpu::BindGroup>,
    object_group1: Option<wgpu::BindGroup>,
    // Sprite
    sprite_model: Option<ModelEnvelop<TexModel>>, // vertical billboard for main pass
    sprite_transform_buffer: Option<GpuBuffer>,
    sprite_pipeline: Option<wgpu::RenderPipeline>,
    sprite_group0: Option<wgpu::BindGroup>,
    sprite_group1: Option<wgpu::BindGroup>,
    sprite_frame_w: u32,
    sprite_frame_h: u32,
    sprite_frames_per_dir: u32,
}

impl App {
    fn new(
        base: PathBuf,
        landtype: String,
        init_obj_num: Option<u16>,
        objects_3d: Vec<Option<Object3D>>,
    ) -> Self {
        let mut camera = Camera::new();
        camera.angle_x = -75;
        camera.angle_z = 60;
        App {
            window: None,
            gpu: None,
            screen: Screen {
                width: 800,
                height: 600,
            },
            camera,
            do_render: true,
            base,
            landtype,
            objects_3d,
            obj_num: init_obj_num.unwrap_or(0) as usize,
            scale: 1.0,
            scale_origin: 1.0,
            sun_azimuth: 0.8,
            sun_elevation: 0.6,
            pop_obj: None,
            camera_mvp_buffer: None,
            model_transform_buffer: None,
            light_mvp_buffer: None,
            sun_dir_buffer: None,
            shadow_depth_view: None,
            shadow_depth_pipeline: None,
            shadow_pass_group0: None,
            shadow_pass_group1: None,
            ground_pipeline: None,
            ground_vertex_buffer: None,
            ground_vertex_count: 0,
            ground_group0: None,
            shadow_recv_group: None,
            object_pipeline: None,
            object_group0: None,
            object_group1: None,
            sprite_model: None,
            sprite_transform_buffer: None,
            sprite_pipeline: None,
            sprite_group0: None,
            sprite_group1: None,
            sprite_frame_w: 0,
            sprite_frame_h: 0,
            sprite_frames_per_dir: 0,
        }
    }

    fn render(&mut self) {
        let gpu = self.gpu.as_ref().unwrap();
        let pop_obj = self.pop_obj.as_ref().unwrap();

        // Update camera MVP
        let mvp = MVP::new(&self.screen, &self.camera, Vector3::new(0.0, 0.0, 0.0));
        let camera_m = mvp.projection * mvp.view * mvp.transform;
        let camera_raw: TransformRaw = camera_m.into();
        self.camera_mvp_buffer.as_ref().unwrap().update(
            &gpu.queue,
            0,
            bytemuck::bytes_of(&camera_raw),
        );

        // Update model transform
        pop_obj.write_transform(
            &gpu.queue,
            &self.model_transform_buffer.as_ref().unwrap().buffer,
            0,
        );

        // Update sprite transform (identity — billboard is already in world coords)
        if let Some(ref sprite_buf) = self.sprite_transform_buffer {
            let identity: TransformRaw = Matrix4::from_scale(1.0_f32).into();
            sprite_buf.update(&gpu.queue, 0, bytemuck::bytes_of(&identity));
        }

        // Update light MVP
        let light_mvp = compute_light_mvp(self.sun_azimuth, self.sun_elevation);
        let light_raw: TransformRaw = light_mvp.into();
        self.light_mvp_buffer.as_ref().unwrap().update(
            &gpu.queue,
            0,
            bytemuck::bytes_of(&light_raw),
        );

        // Update sun direction
        let sun_dir = compute_sun_dir(self.sun_azimuth, self.sun_elevation);
        self.sun_dir_buffer
            .as_ref()
            .unwrap()
            .update(&gpu.queue, 0, bytemuck::bytes_of(&sun_dir));

        let output = match gpu.surface.get_current_texture() {
            Ok(t) => t,
            Err(wgpu::SurfaceError::Lost) => return,
            Err(wgpu::SurfaceError::OutOfMemory) => panic!("Out of GPU memory"),
            Err(e) => {
                log::error!("Surface error: {:?}", e);
                return;
            }
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("shadow_demo_encoder"),
            });

        // Pass 1: Shadow depth
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow_depth_pass"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.shadow_depth_view.as_ref().unwrap(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(self.shadow_depth_pipeline.as_ref().unwrap());
            pass.set_bind_group(0, self.shadow_pass_group0.as_ref().unwrap(), &[]);
            pass.set_bind_group(1, self.shadow_pass_group1.as_ref().unwrap(), &[]);
            pop_obj.draw(&mut pass);
        }

        // Pass 2: Main (ground + building)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.3,
                            g: 0.4,
                            b: 0.6,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &gpu.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });

            // Draw ground
            pass.set_pipeline(self.ground_pipeline.as_ref().unwrap());
            pass.set_bind_group(0, self.ground_group0.as_ref().unwrap(), &[]);
            pass.set_bind_group(1, self.shadow_recv_group.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(
                0,
                self.ground_vertex_buffer.as_ref().unwrap().buffer.slice(..),
            );
            pass.draw(0..self.ground_vertex_count, 0..1);

            // Draw building
            pass.set_pipeline(self.object_pipeline.as_ref().unwrap());
            pass.set_bind_group(0, self.object_group0.as_ref().unwrap(), &[]);
            pass.set_bind_group(1, self.object_group1.as_ref().unwrap(), &[]);
            pop_obj.draw(&mut pass);

            // Draw sprite
            if let (Some(ref spl), Some(ref sg0), Some(ref sg1), Some(ref sm)) = (
                &self.sprite_pipeline,
                &self.sprite_group0,
                &self.sprite_group1,
                &self.sprite_model,
            ) {
                pass.set_pipeline(spl);
                pass.set_bind_group(0, sg0, &[]);
                pass.set_bind_group(1, sg1, &[]);
                sm.draw(&mut pass);
            }
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    fn switch_object(&mut self, new_obj: usize) {
        let device = &self.gpu.as_ref().unwrap().device;
        let pop_obj = match &self.objects_3d[new_obj] {
            Some(obj) => {
                let mut e = mk_pop_envelope(device, obj); // already sets angles[0]=90, location[2]=-0.5
                self.scale_origin = e.get(0).map(|m| m.scale).unwrap_or(1.0);
                if let Some(m) = e.get(0) {
                    m.scale = self.scale_origin * self.scale;
                }
                e
            }
            None => {
                self.scale_origin = 1.0;
                mk_empty_envelope(device)
            }
        };
        self.obj_num = new_obj;
        self.pop_obj = Some(pop_obj);
        eprintln!("Object {}/{}", self.obj_num, self.objects_3d.len());
        if let Some(window) = &self.window {
            window.set_title(&format!("shadow-demo [obj {}]", self.obj_num));
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
                .create_window(WindowAttributes::default().with_title("shadow-demo"))
                .unwrap(),
        );
        self.window = Some(window.clone());
        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        // --- Load BL320 texture ---
        let (level_paths, params) = {
            let data_dir = self.base.join("data");
            let paths = LevelPaths::from_base(&data_dir, &self.landtype);
            let params = GlobeTextureParams::from_level(&paths);
            (paths, params)
        };
        let (width, height, mut bl320_tex) =
            make_bl320_texture_rgba(&level_paths.bl320, &params.palette);
        let key_r = params.palette[0];
        let key_g = params.palette[1];
        let key_b = params.palette[2];
        for pixel in bl320_tex.chunks_exact_mut(4) {
            if pixel[0] == key_r && pixel[1] == key_g && pixel[2] == key_b && pixel[3] == 0 {
                pixel[3] = 255;
            }
        }
        let bl320_gpu_tex = GpuTexture::new_2d(
            device,
            &gpu.queue,
            width as u32,
            height as u32,
            wgpu::TextureFormat::Rgba8Unorm,
            &bl320_tex,
            "bl320_texture",
        );
        let tex_sampler = GpuTexture::create_sampler(device, false);

        // --- Uniform buffers ---
        let camera_mvp_buffer = GpuBuffer::new_uniform(device, 64, "camera_mvp");
        let model_transform_buffer = GpuBuffer::new_uniform(device, 64, "model_transform");
        let light_mvp_buffer = GpuBuffer::new_uniform(device, 64, "light_mvp");
        let sun_dir_buffer = GpuBuffer::new_uniform(device, 16, "sun_dir");

        // --- Shadow depth texture ---
        let shadow_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("shadow_depth_texture"),
            size: wgpu::Extent3d {
                width: SHADOW_MAP_SIZE,
                height: SHADOW_MAP_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_depth_view = shadow_tex.create_view(&wgpu::TextureViewDescriptor::default());

        let shadow_comparison_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_comparison_sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // --- Bind group layouts ---

        // Shadow pass group 0: light_mvp + model_transform (VERTEX)
        let shadow_group0_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow_group0_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Texture group: texture + sampler (FRAGMENT) — shared by shadow depth and object main pass
        let tex_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tex_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Ground group 0: camera_mvp (VERTEX)
        let ground_group0_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("ground_group0_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        // Shadow receive group: shadow_map + comparison_sampler + light_mvp (FRAGMENT)
        let shadow_recv_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow_recv_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Depth,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Object group 0: camera_mvp + model_transform + sun_dir (VERTEX_FRAGMENT)
        let object_group0_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("object_group0_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // --- Bind groups ---

        let shadow_pass_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_pass_group0"),
            layout: &shadow_group0_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: light_mvp_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: model_transform_buffer.buffer.as_entire_binding(),
                },
            ],
        });

        let shadow_pass_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_pass_group1"),
            layout: &tex_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bl320_gpu_tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex_sampler),
                },
            ],
        });

        let object_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("object_group1"),
            layout: &tex_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bl320_gpu_tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex_sampler),
                },
            ],
        });

        let ground_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ground_group0"),
            layout: &ground_group0_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_mvp_buffer.buffer.as_entire_binding(),
            }],
        });

        let shadow_recv_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_recv_group"),
            layout: &shadow_recv_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&shadow_depth_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&shadow_comparison_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: light_mvp_buffer.buffer.as_entire_binding(),
                },
            ],
        });

        let object_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("object_group0"),
            layout: &object_group0_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_mvp_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: model_transform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: sun_dir_buffer.buffer.as_entire_binding(),
                },
            ],
        });

        // --- Pipelines ---

        // Shadow depth pipeline (manual — no color targets)
        let shadow_depth_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow_depth_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../shaders/shadow_demo_depth.wgsl").into(),
            ),
        });
        let shadow_depth_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("shadow_depth_pipeline_layout"),
                bind_group_layouts: &[&shadow_group0_layout, &tex_group_layout],
                immediate_size: 0,
            });
        let shadow_depth_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("shadow_depth_pipeline"),
                layout: Some(&shadow_depth_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shadow_depth_shader,
                    entry_point: Some("vs_main"),
                    buffers: &TexModel::vertex_buffer_layouts(),
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shadow_depth_shader,
                    entry_point: Some("fs_main"),
                    targets: &[],
                    compilation_options: Default::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    cull_mode: None,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            });

        // Ground pipeline
        let ground_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: 12,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            }],
        };
        let ground_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("ground_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../shaders/shadow_demo_ground.wgsl").into(),
            ),
        });
        let ground_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("ground_pipeline_layout"),
                bind_group_layouts: &[&ground_group0_layout, &shadow_recv_layout],
                immediate_size: 0,
            });
        let ground_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ground_pipeline"),
            layout: Some(&ground_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &ground_shader,
                entry_point: Some("vs_main"),
                buffers: &[ground_vertex_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &ground_shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: gpu.surface_format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // Object pipeline (uses create_pipeline helper)
        let object_shader_source = include_str!("../../shaders/shadow_demo_object.wgsl");
        let tex_vertex_layouts = TexModel::vertex_buffer_layouts();
        let object_pipeline = create_pipeline(
            device,
            object_shader_source,
            &tex_vertex_layouts,
            &[&object_group0_layout, &tex_group_layout],
            gpu.surface_format(),
            true,
            wgpu::PrimitiveTopology::TriangleList,
            "shadow_demo_object_pipeline",
        );

        // --- Ground mesh ---
        let gv = ground_vertices();
        let ground_data: Vec<u8> = gv
            .iter()
            .flat_map(|v| bytemuck::bytes_of(v).to_vec())
            .collect();
        let ground_vertex_buffer = GpuBuffer::new_vertex(device, &ground_data, "ground_vertices");
        let ground_vertex_count = gv.len() as u32;

        // --- Building model ---
        let pop_obj = if self.obj_num < self.objects_3d.len() {
            match &self.objects_3d[self.obj_num] {
                Some(obj) => {
                    let mut e = mk_pop_envelope(device, obj);
                    self.scale_origin = e.get(0).map(|m| m.scale).unwrap_or(1.0);
                    e
                }
                None => {
                    self.scale_origin = 1.0;
                    mk_empty_envelope(device)
                }
            }
        } else {
            self.scale_origin = 1.0;
            mk_empty_envelope(device)
        };

        // --- Sprite atlas ---
        let sprite_transform_buffer = GpuBuffer::new_uniform(device, 64, "sprite_transform");

        let palette = {
            let raw = &params.palette;
            let mut pal = Vec::with_capacity(256);
            for i in 0..256 {
                let off = i * 4;
                if off + 3 < raw.len() {
                    pal.push([raw[off], raw[off + 1], raw[off + 2], 255]);
                } else {
                    pal.push([0, 0, 0, 255]);
                }
            }
            pal
        };
        let hspr_path = self.base.join("data").join("HSPR0-0.DAT");
        let container = ContainerPSFB::from_file(&hspr_path);
        let anim_data = AnimationsData::from_path(&self.base.join("data"));
        let sequences = AnimationSequence::from_data(&anim_data);

        // Use brave idle anim (index 15)
        let (_, anim_idx) = UNIT_IDLE_ANIMS[0];
        let sprite_atlas = container
            .as_ref()
            .and_then(|c| build_tribe_atlas(&sequences, c, &palette, anim_idx, Some(None), None));

        let (
            sprite_model,
            sprite_pipeline,
            sprite_group0,
            sprite_group1,
            sprite_fw,
            sprite_fh,
            sprite_fpd,
        ) = if let Some((atlas_w, atlas_h, rgba, fw, fh, fpd, _max_y)) = sprite_atlas {
            let sprite_tex = GpuTexture::new_2d(
                device,
                &gpu.queue,
                atlas_w,
                atlas_h,
                wgpu::TextureFormat::Rgba8UnormSrgb,
                &rgba,
                "sprite_atlas",
            );
            let sprite_sampler = GpuTexture::create_sampler(device, true);

            let aspect = if fh > 0 { fw as f32 / fh as f32 } else { 1.0 };
            let sprite_h = 0.35;
            let sprite_pos = Vector3::new(0.6, 0.3, -0.55); // next to building, sunk slightly into ground
            let sm = build_sprite_billboard(
                device,
                &self.camera,
                sprite_pos,
                sprite_h,
                aspect,
                fw,
                fh,
                fpd,
            );
            // Sprite main pass
            let spr_group0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sprite_group0"),
                layout: &object_group0_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: camera_mvp_buffer.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: sprite_transform_buffer.buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: sun_dir_buffer.buffer.as_entire_binding(),
                    },
                ],
            });
            let spr_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("sprite_group1"),
                layout: &tex_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&sprite_tex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sprite_sampler),
                    },
                ],
            });

            // Sprite main pass pipeline
            let sprite_shader_source = include_str!("../../shaders/shadow_demo_sprite.wgsl");
            let sprite_pl = create_pipeline(
                device,
                sprite_shader_source,
                &tex_vertex_layouts,
                &[&object_group0_layout, &tex_group_layout],
                gpu.surface_format(),
                true,
                wgpu::PrimitiveTopology::TriangleList,
                "sprite_pipeline",
            );

            (
                Some(sm),
                Some(sprite_pl),
                Some(spr_group0),
                Some(spr_group1),
                fw,
                fh,
                fpd,
            )
        } else {
            eprintln!("Warning: could not load sprite atlas");
            (None, None, None, None, 0, 0, 0)
        };

        // --- Store everything ---
        self.camera_mvp_buffer = Some(camera_mvp_buffer);
        self.model_transform_buffer = Some(model_transform_buffer);
        self.light_mvp_buffer = Some(light_mvp_buffer);
        self.sun_dir_buffer = Some(sun_dir_buffer);
        self.shadow_depth_view = Some(shadow_depth_view);
        self.shadow_depth_pipeline = Some(shadow_depth_pipeline);
        self.shadow_pass_group0 = Some(shadow_pass_group0);
        self.shadow_pass_group1 = Some(shadow_pass_group1);
        self.ground_pipeline = Some(ground_pipeline);
        self.ground_vertex_buffer = Some(ground_vertex_buffer);
        self.ground_vertex_count = ground_vertex_count;
        self.ground_group0 = Some(ground_group0);
        self.shadow_recv_group = Some(shadow_recv_group);
        self.object_pipeline = Some(object_pipeline);
        self.object_group0 = Some(object_group0);
        self.object_group1 = Some(object_group1);
        self.pop_obj = Some(pop_obj);
        self.sprite_model = sprite_model;
        self.sprite_transform_buffer = Some(sprite_transform_buffer);
        self.sprite_pipeline = sprite_pipeline;
        self.sprite_group0 = sprite_group0;
        self.sprite_group1 = sprite_group1;
        self.sprite_frame_w = sprite_fw;
        self.sprite_frame_h = sprite_fh;
        self.sprite_frames_per_dir = sprite_fpd;
        self.gpu = Some(gpu);
        self.do_render = true;
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::Resized(physical_size) => {
                self.screen.width = physical_size.width;
                self.screen.height = physical_size.height;
                if let Some(gpu) = self.gpu.as_mut() {
                    gpu.resize(physical_size);
                }
                self.do_render = true;
            }
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state == ElementState::Pressed {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        match key {
                            KeyCode::KeyQ => {
                                event_loop.exit();
                                return;
                            }
                            // Sun controls
                            KeyCode::ArrowLeft => {
                                self.sun_azimuth += 0.1;
                            }
                            KeyCode::ArrowRight => {
                                self.sun_azimuth -= 0.1;
                            }
                            KeyCode::ArrowUp => {
                                self.sun_elevation = (self.sun_elevation + 0.1).min(1.4);
                            }
                            KeyCode::ArrowDown => {
                                self.sun_elevation = (self.sun_elevation - 0.1).max(0.1);
                            }
                            // Camera controls
                            KeyCode::KeyA => {
                                self.camera.angle_z += 5;
                            }
                            KeyCode::KeyD => {
                                self.camera.angle_z -= 5;
                            }
                            KeyCode::KeyW => {
                                self.camera.angle_x = (self.camera.angle_x - 5).max(-89);
                            }
                            KeyCode::KeyS => {
                                self.camera.angle_x = (self.camera.angle_x + 5).min(-10);
                            }
                            // Object browsing
                            KeyCode::KeyV => {
                                let mut i = self.obj_num;
                                while i > 0 {
                                    i -= 1;
                                    if self.objects_3d[i].is_some() {
                                        self.switch_object(i);
                                        break;
                                    }
                                }
                            }
                            KeyCode::KeyB => {
                                let mut i = self.obj_num;
                                while i + 1 < self.objects_3d.len() {
                                    i += 1;
                                    if self.objects_3d[i].is_some() {
                                        self.switch_object(i);
                                        break;
                                    }
                                }
                            }
                            KeyCode::KeyN => {
                                self.scale -= self.scale * 0.1;
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.scale = self.scale_origin * self.scale;
                                }
                            }
                            KeyCode::KeyM => {
                                self.scale += self.scale * 0.1;
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.scale = self.scale_origin * self.scale;
                                }
                            }
                            _ => (),
                        }
                        self.do_render = true;
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.do_render && self.gpu.is_some() {
                    self.render();
                    self.do_render = false;
                }
            }
            _ => (),
        }
        if self.do_render {
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }
}

fn main() {
    let matches = cli().get_matches();

    let base = matches
        .get_one("base")
        .cloned()
        .unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf());
    let landtype = matches
        .get_one("landtype")
        .cloned()
        .unwrap_or_else(|| "1".to_string());
    let obj_num: Option<u16> = matches.get_one("obj_num").copied();

    env_logger::init_from_env(
        env_logger::Env::default()
            .filter_or("F_LOG_LEVEL", "info")
            .write_style_or("F_LOG_STYLE", "always"),
    );

    let objects_3d = Object3D::from_file_all(&base, "0");
    eprintln!(
        "Loaded {} objects ({} with faces)",
        objects_3d.len(),
        objects_3d.iter().filter(|o| o.is_some()).count()
    );

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(base, landtype, obj_num, objects_3d);
    event_loop.run_app(&mut app).unwrap();
}
