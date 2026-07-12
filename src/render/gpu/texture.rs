pub struct GpuTexture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub size: wgpu::Extent3d,
}

impl GpuTexture {
    pub fn new_2d(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
        data: &[u8],
        label: &str,
    ) -> Self {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let bytes_per_pixel = format.block_copy_size(None).unwrap_or(4);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_pixel * width),
                rows_per_image: Some(height),
            },
            size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        GpuTexture {
            texture,
            view,
            size,
        }
    }

    /// Create a 2D texture with height=1, replacing OpenGL 1D textures
    pub fn new_2d_height1(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        format: wgpu::TextureFormat,
        data: &[u8],
        label: &str,
    ) -> Self {
        Self::new_2d(device, queue, width, 1, format, data, label)
    }

    pub fn update(&self, queue: &wgpu::Queue, data: &[u8]) {
        let bytes_per_pixel = self.texture.format().block_copy_size(None).unwrap_or(4);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_pixel * self.size.width),
                rows_per_image: Some(self.size.height),
            },
            self.size,
        );
    }

    pub fn create_sampler(device: &wgpu::Device, nearest: bool) -> wgpu::Sampler {
        let filter = if nearest {
            wgpu::FilterMode::Nearest
        } else {
            wgpu::FilterMode::Linear
        };
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture_sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        })
    }
}
