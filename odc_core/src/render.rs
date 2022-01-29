use crate::structure as st;
use std::collections::HashMap;

pub struct RenderData {
    buffers: HashMap<String, Buffer>,
    textures: HashMap<String, Texture>,
    bind_groups: HashMap<String, BindGroup>,
    render_pipelines: HashMap<String, RenderPipeline>,
    passes: HashMap<String, Pass>,
}

impl RenderData {
    pub fn from_structure(device: &wgpu::Device, render: &st::Render) -> Self {
        let factory = HandlesFactory { device, render };

        let buffers = render
            .buffers
            .iter()
            .map(|(name, buf)| (name.clone(), factory.create_buffer(name, buf)))
            .collect();



        Self { buffers }
    }
}

struct HandlesFactory<'a> {
    device: &'a wgpu::Device,
    render: &'a st::Render,
}

impl<'a> HandlesFactory<'a> {
    pub fn create_buffer(&self, name: &str, buffer: &st::Buffer) -> Buffer {
        let mut usage = Buffer::find_usages(name, self.render);
        let descriptor = wgpu::BufferDescriptor {
            label: Some(name),
            size: buffer.size,
            usage,
            mapped_at_creation: false,
        };
        Buffer::new(self.device.create_buffer(&descriptor))
    }
}

struct Buffer(wgpu::Buffer);

impl Buffer {
    pub fn new(handle: wgpu::Buffer) -> Self {
        Self(handle)
    }

    pub fn find_usages(name: &str, render: &st::Render) -> wgpu::BufferUsages {
        let is_uniform = render.is_uniform_buffer(name);
        let is_storage = render.is_storage_buffer(name);
        let is_vertex = render.is_vertex_buffer(name);
        let is_index = render.is_index_buffer(name);

        let usages = wgpu::BufferUsages::COPY_DST;
        if is_uniform {
            usages |= wgpu::BufferUsages::UNIFORM;
        }
        if is_storage {
            usages |= wgpu::BufferUsages::STORAGE;
        }
        if is_vertex {
            usages |= wgpu::BufferUsages::VERTEX;
        }
        if is_index {
            usages |= wgpu::BufferUsages::INDEX;
        }
        usages
    }
}

type Texture = wgpu::Texture;
type BindGroup = wgpu::BindGroup;
type RenderPipeline = wgpu::RenderPipeline;

struct Pass;