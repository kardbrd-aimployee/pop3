pub fn create_pipeline(
    device: &wgpu::Device,
    shader_source: &str,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    surface_format: wgpu::TextureFormat,
    depth: bool,
    topology: wgpu::PrimitiveTopology,
    label: &str,
) -> wgpu::RenderPipeline {
    create_pipeline_blended(
        device,
        shader_source,
        vertex_layouts,
        bind_group_layouts,
        surface_format,
        depth,
        topology,
        label,
        wgpu::BlendState::REPLACE,
    )
}

pub fn create_pipeline_blended(
    device: &wgpu::Device,
    shader_source: &str,
    vertex_layouts: &[wgpu::VertexBufferLayout],
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    surface_format: wgpu::TextureFormat,
    depth: bool,
    topology: wgpu::PrimitiveTopology,
    label: &str,
    blend: wgpu::BlendState,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(label),
        source: wgpu::ShaderSource::Wgsl(shader_source.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("{}_layout", label)),
        bind_group_layouts,
        immediate_size: 0,
    });

    let depth_stencil = if depth {
        Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        })
    } else {
        None
    };

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: vertex_layouts,
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(blend),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview_mask: None,
        cache: None,
    })
}
