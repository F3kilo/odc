use bytemuck::{Pod, Zeroable};
use raw_window_handle::HasRawWindowHandle;
use std::mem;
use std::num::NonZeroU64;
use std::ops::Range;
use wgpu::{
    Backends, BindGroup, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress,
    BufferBindingType, BufferSize, BufferUsages, Color, IndexFormat, Instance, Limits, PresentMode,
    RenderPipeline, ShaderStages, Surface, SurfaceConfiguration,
    SurfaceError, TextureFormat, TextureUsages, VertexAttribute,
    VertexFormat, VertexStepMode,
};
use gdevice::GraphicsDevice;

mod gdevice;

pub struct TriangleRenderer {
    surface: Surface,
    swapchain_format: TextureFormat,
    graphics_device: GraphicsDevice,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    pipeline: RenderPipeline,
    uniform: Buffer,
    storage: Buffer,
    binding: BindGroup,
}

impl TriangleRenderer {
    pub const MAX_INSTANCE_COUNT: usize = 2usize.pow(16);
    pub const VERTEX_BUFFER_SIZE: u64 = 2u64.pow(24);
    pub const INDEX_BUFFER_SIZE: u64 = 2u64.pow(22);

    pub fn new(window: &impl HasRawWindowHandle, size: WindowSize) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let graphics_device = GraphicsDevice::new(&instance, Some(&surface));

        let vertex_buffer = graphics_device.create_gpu_buffer(
            Self::VERTEX_BUFFER_SIZE,
            BufferUsages::COPY_DST | BufferUsages::VERTEX,
        );
        let index_buffer = graphics_device.create_gpu_buffer(
            Self::INDEX_BUFFER_SIZE,
            BufferUsages::COPY_DST | BufferUsages::INDEX,
        );

        let unioform_size = Self::uniform_size();
        let uniform = graphics_device.create_gpu_buffer(
            unioform_size.get(),
            BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        );
        let storage = graphics_device.create_gpu_buffer(
            Self::storage_size(),
            BufferUsages::COPY_DST | BufferUsages::STORAGE,
        );

        let storage_min_size =
            BufferSize::new(InstanceInfo::size() as _).expect("unexpected zero size instance");
        let global_binding_layout =
            graphics_device.create_bind_group_layout(unioform_size, storage_min_size);

        let binding = graphics_device.create_binding(&uniform, &storage, &global_binding_layout);

        let shader = graphics_device.create_shader();
        let pipeline_layout = graphics_device.create_pipeline_layout(&[&global_binding_layout]);
        let swapchain_format = graphics_device.preferred_surface_format(&surface).unwrap();
        let pipeline = graphics_device.create_pipeline(&pipeline_layout, &shader, swapchain_format);

        graphics_device.configure_surface(&surface, size, swapchain_format);

        Self {
            surface,
            swapchain_format,
            graphics_device,
            vertex_buffer,
            index_buffer,
            pipeline,
            uniform,
            storage,
            binding,
        }
    }

    pub fn write_instances(&mut self, instances: &[InstanceInfo], offset: u64) {
        let instance_data = bytemuck::cast_slice(instances);
        self.graphics_device
            .write_buffer(&self.storage, offset, instance_data)
    }

    pub fn write_mesh(&mut self, mesh: &Mesh, vertex_offset: u64, index_offset: u64) {
        let data = bytemuck::cast_slice(&mesh.vertices);
        let offset = vertex_offset * Vertex::size() as u64;
        self.graphics_device
            .write_buffer(&self.vertex_buffer, offset, data);

        let data = bytemuck::cast_slice(&mesh.indices);
        let offset = index_offset * mem::size_of::<u32>() as u64;
        self.graphics_device
            .write_buffer(&self.index_buffer, offset, data);
    }

    pub fn render<'a>(&self, info: &RenderInfo, draws: impl Iterator<Item = &'a StaticMesh>) {
        let render_data = bytemuck::bytes_of(info);
        self.graphics_device
            .write_buffer(&self.uniform, 0, render_data);
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(SurfaceError::Outdated) => return,
            e => e.unwrap(),
        };
        let view = frame.texture.create_view(&Default::default());
        self.graphics_device.render(
            &view,
            &self.vertex_buffer,
            &self.index_buffer,
            &self.binding,
            &self.pipeline,
            draws,
        );
        frame.present();
    }

    pub fn resize(&mut self, size: WindowSize) {
        if size.is_zero_square() {
            return;
        }

        self.graphics_device
            .configure_surface(&self.surface, size, self.swapchain_format)
    }

    fn uniform_size() -> BufferSize {
        NonZeroU64::new(mem::size_of::<RenderInfo>() as _).expect("Zero sized uniform")
    }

    fn storage_size() -> u64 {
        InstanceInfo::size() as u64 * Self::MAX_INSTANCE_COUNT as u64
    }
}

pub struct WindowSize(pub u32, pub u32);

impl WindowSize {
    pub fn is_zero_square(&self) -> bool {
        self.0 * self.1 == 0
    }
}

#[derive(Copy, Clone)]
pub struct RenderInfo {
    pub world: Transform,
    pub view_proj: Transform,
}

unsafe impl Zeroable for RenderInfo {}
unsafe impl Pod for RenderInfo {}

pub struct StaticMesh {
    pub indices: Range<u32>,
    pub base_vertex: i32,
    pub instances: Range<u32>,
}

#[derive(Copy, Clone)]
pub struct InstanceInfo {
    pub transform: Transform,
}

impl InstanceInfo {
    pub const fn size() -> usize {
        mem::size_of::<Self>()
    }
}

unsafe impl Zeroable for InstanceInfo {}
unsafe impl Pod for InstanceInfo {}

pub type Transform = [[f32; 4]; 4];

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 4],
    pub color: [f32; 4],
}

impl Vertex {
    pub const fn size() -> usize {
        mem::size_of::<Self>()
    }

    pub const fn position_offset() -> usize {
        0
    }

    pub const fn color_offset() -> usize {
        mem::size_of::<[f32; 4]>()
    }
}

unsafe impl Zeroable for Vertex {}
unsafe impl Pod for Vertex {}
