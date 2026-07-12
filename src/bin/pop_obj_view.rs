use std::path::{Path, PathBuf};
use std::sync::Arc;

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowAttributes};

use clap::{Arg, ArgAction, Command};

use cgmath::Vector3;

use pop3::render::camera::*;
use pop3::render::model::MeshModel;
use pop3::render::tex_model::TexModel;

use pop3::data::bl320::make_bl320_texture_rgba;
use pop3::data::level::{GlobeTextureParams, LevelPaths};
use pop3::data::objects::{mk_pop_object, Object3D};

use pop3::render::envelop::*;
use pop3::render::gpu::buffer::GpuBuffer;
use pop3::render::gpu::context::GpuContext;
use pop3::render::gpu::pipeline::create_pipeline;
use pop3::render::gpu::texture::GpuTexture;

/******************************************************************************/

fn mk_pop_envelope(device: &wgpu::Device, object: &Object3D) -> ModelEnvelop<TexModel> {
    let model = mk_pop_object(object);
    let m = vec![(RenderType::Triangles, model)];
    let mut e = ModelEnvelop::<TexModel>::new(device, m);
    if let Some(m) = e.get(0) {
        m.location[1] = -0.5;
        m.scale = (object.coord_scale() / 300.0) * 0.5;
    }
    e
}

fn mk_empty_envelope(device: &wgpu::Device) -> ModelEnvelop<TexModel> {
    let model: TexModel = MeshModel::new();
    let m = vec![(RenderType::Triangles, model)];
    ModelEnvelop::<TexModel>::new(device, m)
}

fn obj_title(obj_num: usize, total: usize, objects_3d: &[Option<Object3D>]) -> String {
    match objects_3d.get(obj_num) {
        Some(Some(o)) => format!(
            "pop-obj-view [{}/{}] faces={} scale={}",
            obj_num,
            total,
            o.iter_face().count(),
            o.coord_scale() as u32
        ),
        _ => format!("pop-obj-view [{}/{}] (empty)", obj_num, total),
    }
}

/******************************************************************************/

fn cli() -> Command {
    let args = [
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
        Arg::new("debug")
            .long("debug")
            .action(ArgAction::SetTrue)
            .help("Enable debug printing"),
        Arg::new("obj_num")
            .long("obj_num")
            .action(ArgAction::Set)
            .value_name("OBJ")
            .value_parser(clap::value_parser!(u16).range(0..16000))
            .help("Obj file index (0-based)"),
        Arg::new("bank")
            .long("bank")
            .action(ArgAction::Set)
            .value_name("BANK")
            .value_parser(clap::builder::StringValueParser::new())
            .help("OBJS bank number (0-7, default 0)"),
    ];
    Command::new("pop-obj-view")
        .about("POP3 object viewer")
        .args(&args)
}

struct App {
    window: Option<Arc<Window>>,
    gpu: Option<GpuContext>,
    pipeline: Option<wgpu::RenderPipeline>,
    mvp_buffer: Option<GpuBuffer>,
    model_transform_buffer: Option<GpuBuffer>,
    mvp_bind_group: Option<wgpu::BindGroup>,
    texture_bind_group: Option<wgpu::BindGroup>,
    shadow_bind_group: Option<wgpu::BindGroup>,
    pop_obj: Option<ModelEnvelop<TexModel>>,
    camera: Camera,
    screen: Screen,
    objects_3d: Vec<Option<Object3D>>,
    obj_num: usize,
    scale: f32,
    scale_origin: f32,
    do_render: bool,
    // Init data
    base: PathBuf,
    landtype: String,
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
            pipeline: None,
            mvp_buffer: None,
            model_transform_buffer: None,
            mvp_bind_group: None,
            texture_bind_group: None,
            shadow_bind_group: None,
            pop_obj: None,
            camera,
            screen: Screen {
                width: 800,
                height: 600,
            },
            objects_3d,
            obj_num: init_obj_num.unwrap_or(0) as usize,
            scale: 1.0,
            scale_origin: 1.0,
            do_render: true,
            base,
            landtype,
        }
    }

    fn render(&mut self) {
        let gpu = self.gpu.as_ref().unwrap();
        let pipeline = self.pipeline.as_ref().unwrap();
        let mvp_buffer = self.mvp_buffer.as_ref().unwrap();
        let model_transform_buffer = self.model_transform_buffer.as_ref().unwrap();
        let mvp_bind_group = self.mvp_bind_group.as_ref().unwrap();
        let texture_bind_group = self.texture_bind_group.as_ref().unwrap();
        let shadow_bind_group = self.shadow_bind_group.as_ref().unwrap();
        let pop_obj = self.pop_obj.as_ref().unwrap();

        // Update MVP
        let mvp = MVP::new(&self.screen, &self.camera, Vector3::new(0.0, 0.0, 0.0));
        let mvp_m = mvp.projection * mvp.view * mvp.transform;
        let mvp_raw: TransformRaw = mvp_m.into();
        mvp_buffer.update(&gpu.queue, 0, bytemuck::bytes_of(&mvp_raw));

        // Update model transform
        pop_obj.write_transform(&gpu.queue, &model_transform_buffer.buffer, 0);

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
                label: Some("render_encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
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

            render_pass.set_pipeline(pipeline);
            render_pass.set_bind_group(0, mvp_bind_group, &[]);
            render_pass.set_bind_group(1, texture_bind_group, &[]);
            render_pass.set_bind_group(2, shadow_bind_group, &[]);
            pop_obj.draw(&mut render_pass);
        }

        gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let title = obj_title(self.obj_num, self.objects_3d.len(), &self.objects_3d);
        let window = Arc::new(
            event_loop
                .create_window(WindowAttributes::default().with_title(&title))
                .unwrap(),
        );
        self.window = Some(window.clone());

        let gpu = pollster::block_on(GpuContext::new(window));
        let device = &gpu.device;

        // Load texture
        let (level_paths, params) = {
            let data_dir = self.base.join("data");
            let paths = LevelPaths::from_base(&data_dir, &self.landtype);
            let params = GlobeTextureParams::from_level(&paths);
            (paths, params)
        };
        let (width, height, mut bl320_tex) =
            make_bl320_texture_rgba(&level_paths.bl320, &params.palette);

        // Mark transparent pixels (palette index 0) with alpha=255 so the shader
        // can discard them via `if (color.w > 0.0) { discard; }`.
        // Palette entry 0 is the key/transparent color: its RGB = (pal[0], pal[1], pal[2]).
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
        let sampler = GpuTexture::create_sampler(device, false);

        // MVP bind group (group 0): m_transform + m_transform1 + lighting
        let mvp_buffer = GpuBuffer::new_uniform(device, 64, "mvp_buffer");
        let model_transform_buffer = GpuBuffer::new_uniform(device, 64, "model_transform_buffer");

        // LightParams: sun_dir(3f) + ambient(1f) + camera_focus(2f) + viewport_radius(1f) + game_tick(1f)
        let sun_len = (0.5_f32 * 0.5 + 1.0 * 1.0 + 0.7 * 0.7).sqrt();
        let light_data: [f32; 8] = [
            0.5 / sun_len,
            1.0 / sun_len,
            0.7 / sun_len,
            0.4,
            0.0,
            0.0,
            100.0,
            0.0,
        ];
        let lighting_buffer =
            GpuBuffer::new_uniform_init(device, bytemuck::bytes_of(&light_data), "lighting_buffer");

        let mvp_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("mvp_bind_group_layout"),
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

        let mvp_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mvp_bind_group"),
            layout: &mvp_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: mvp_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: model_transform_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: lighting_buffer.buffer.as_entire_binding(),
                },
            ],
        });

        // Texture bind group (group 1): texture + sampler
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
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

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&bl320_gpu_tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        // Shadow bind group (group 2): dummy 1x1 depth texture + comparison sampler + identity MVP
        let shadow_depth_tex = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("dummy_shadow_depth"),
            size: wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let shadow_depth_view =
            shadow_depth_tex.create_view(&wgpu::TextureViewDescriptor::default());
        let shadow_comparison_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow_comparison_sampler"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        // Identity matrix — no shadow transform
        let identity_mat: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ];
        let shadow_mvp_buffer = GpuBuffer::new_uniform_init(
            device,
            bytemuck::bytes_of(&identity_mat),
            "shadow_mvp_buffer",
        );

        let shadow_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("shadow_bind_group_layout"),
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

        let shadow_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadow_bind_group"),
            layout: &shadow_bind_group_layout,
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
                    resource: shadow_mvp_buffer.buffer.as_entire_binding(),
                },
            ],
        });

        // Create pipeline
        let shader_source = include_str!("../../shaders/objects_tex.wgsl");
        let vertex_layouts = TexModel::vertex_buffer_layouts();
        let pipeline = create_pipeline(
            device,
            shader_source,
            &vertex_layouts,
            &[
                &mvp_bind_group_layout,
                &texture_bind_group_layout,
                &shadow_bind_group_layout,
            ],
            gpu.surface_format(),
            true,
            wgpu::PrimitiveTopology::TriangleList,
            "objects_tex_pipeline",
        );

        // Create model
        if self.obj_num >= self.objects_3d.len() {
            log::error!(
                "Object number is too big {:?} >= {:?}",
                self.obj_num,
                self.objects_3d.len()
            );
            event_loop.exit();
            return;
        }

        let pop_obj = match &self.objects_3d[self.obj_num] {
            Some(obj) => {
                let mut e = mk_pop_envelope(device, obj);
                self.scale_origin = e.get(0).map(|m| m.scale).unwrap();
                e
            }
            None => {
                self.scale_origin = 1.0;
                mk_empty_envelope(device)
            }
        };
        eprintln!(
            "{}",
            obj_title(self.obj_num, self.objects_3d.len(), &self.objects_3d)
        );

        self.pipeline = Some(pipeline);
        self.mvp_buffer = Some(mvp_buffer);
        self.model_transform_buffer = Some(model_transform_buffer);
        self.mvp_bind_group = Some(mvp_bind_group);
        self.texture_bind_group = Some(texture_bind_group);
        self.shadow_bind_group = Some(shadow_bind_group);
        self.pop_obj = Some(pop_obj);
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
                            KeyCode::ArrowUp => {
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.angles[0] += 5.0;
                                }
                            }
                            KeyCode::ArrowDown => {
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.angles[0] -= 5.0;
                                }
                            }
                            KeyCode::ArrowLeft => {
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.angles[1] += 5.0;
                                }
                            }
                            KeyCode::ArrowRight => {
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.angles[1] -= 5.0;
                                }
                            }
                            KeyCode::KeyL => {
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.location[0] += 0.1;
                                }
                            }
                            KeyCode::KeyH => {
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.location[0] -= 0.1;
                                }
                            }
                            KeyCode::KeyJ => {
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.location[1] += 0.1;
                                }
                            }
                            KeyCode::KeyK => {
                                if let Some(m) = self.pop_obj.as_mut().and_then(|o| o.get(0)) {
                                    m.location[1] -= 0.1;
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
                            KeyCode::KeyV => {
                                // Previous non-empty object
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
                                // Next non-empty object
                                let mut i = self.obj_num;
                                while i + 1 < self.objects_3d.len() {
                                    i += 1;
                                    if self.objects_3d[i].is_some() {
                                        self.switch_object(i);
                                        break;
                                    }
                                }
                            }
                            KeyCode::KeyR => {
                                self.scale = 1.0;
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

impl App {
    fn switch_object(&mut self, new_obj: usize) {
        let (l, a) = self
            .pop_obj
            .as_mut()
            .and_then(|o| o.get(0))
            .map(|m| (m.location, m.angles))
            .unwrap_or((Vector3::new(0.0, -0.5, 0.0), Vector3::new(0.0, 0.0, 0.0)));

        let device = &self.gpu.as_ref().unwrap().device;
        let pop_obj = match &self.objects_3d[new_obj] {
            Some(obj) => {
                let mut e = mk_pop_envelope(device, obj);
                if let Some(m) = e.get(0) {
                    m.location = l;
                    m.angles = a;
                    self.scale_origin = m.scale;
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

        let title = obj_title(self.obj_num, self.objects_3d.len(), &self.objects_3d);
        eprintln!("{}", title);
        if let Some(window) = &self.window {
            window.set_title(&title);
        }
    }
}

fn main() {
    let matches = cli().get_matches();

    let base = {
        let base = matches.get_one("base").cloned();
        base.unwrap_or_else(|| Path::new("/opt/sandbox/pop").to_path_buf())
    };
    let landtype = matches
        .get_one("landtype")
        .cloned()
        .unwrap_or_else(|| "1".to_string());
    let bank = matches
        .get_one("bank")
        .cloned()
        .unwrap_or_else(|| "0".to_string());
    let debug = matches.get_flag("debug");
    let obj_num: Option<u16> = matches.get_one("obj_num").copied();

    let log_level: &str = if debug { "debug" } else { "info" };
    let env = env_logger::Env::default()
        .filter_or("F_LOG_LEVEL", log_level)
        .write_style_or("F_LOG_STYLE", "always");
    env_logger::init_from_env(env);

    let objects_3d = Object3D::from_file_all(&base, &bank);
    eprintln!(
        "Loaded {} objects from OBJS0-{}.DAT ({} with faces)",
        objects_3d.len(),
        bank,
        objects_3d.iter().filter(|o| o.is_some()).count()
    );

    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new(base, landtype, obj_num, objects_3d);
    event_loop.run_app(&mut app).unwrap();
}
