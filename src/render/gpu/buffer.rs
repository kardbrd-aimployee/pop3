use wgpu::util::DeviceExt;

pub struct GpuBuffer {
    pub buffer: wgpu::Buffer,
    pub size: u64,
}

impl GpuBuffer {
    pub fn new_vertex(device: &wgpu::Device, data: &[u8], label: &str) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        GpuBuffer {
            buffer,
            size: data.len() as u64,
        }
    }

    pub fn new_index(device: &wgpu::Device, data: &[u8], label: &str) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });
        GpuBuffer {
            buffer,
            size: data.len() as u64,
        }
    }

    pub fn new_uniform(device: &wgpu::Device, size: u64, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        GpuBuffer { buffer, size }
    }

    pub fn new_uniform_init(device: &wgpu::Device, data: &[u8], label: &str) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        GpuBuffer {
            buffer,
            size: data.len() as u64,
        }
    }

    pub fn new_storage(device: &wgpu::Device, data: &[u8], label: &str) -> Self {
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: data,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        GpuBuffer {
            buffer,
            size: data.len() as u64,
        }
    }

    pub fn update(&self, queue: &wgpu::Queue, offset: u64, data: &[u8]) {
        queue.write_buffer(&self.buffer, offset, data);
    }
}
