use bytemuck::{Pod, Zeroable};
use gdevice::GfxDevice;
use mesh_buf::MeshBuffers;
use raw_window_handle::HasRawWindowHandle;
use std::mem;
use std::num::NonZeroU64;
use std::ops::Range;
use wgpu::{
    Backends, BindGroup, Buffer, BufferSize, BufferUsages, Instance, RenderPipeline, Surface,
    SurfaceError, TextureFormat,
};

mod gdevice;
mod mesh_buf;

pub struct TriangleRenderer {
    surface: Surface,
    swapchain_format: TextureFormat,
    device: GfxDevice,
    mesh_buffers: MeshBuffers,
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
        let device = GfxDevice::new(&instance, Some(&surface));

        let mesh_buffers =
            MeshBuffers::new(&device, Self::VERTEX_BUFFER_SIZE, Self::INDEX_BUFFER_SIZE);

        let uniform_size = Self::uniform_size();
        let uniform = device.create_gpu_buffer(
            uniform_size.get(),
            BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        );
        let storage = device.create_gpu_buffer(
            Self::storage_size(),
            BufferUsages::COPY_DST | BufferUsages::STORAGE,
        );

        let storage_min_size =
            BufferSize::new(InstanceInfo::size() as _).expect("unexpected zero size instance");
        let global_binding_layout = device.create_bind_group_layout(uniform_size, storage_min_size);

        let binding = device.create_binding(&uniform, &storage, &global_binding_layout);

        let shader = device.create_shader();
        let pipeline_layout = device.create_pipeline_layout(&[&global_binding_layout]);
        let swapchain_format = device.preferred_surface_format(&surface).unwrap();
        let pipeline = device.create_pipeline(&pipeline_layout, &shader, swapchain_format);

        device.configure_surface(&surface, size, swapchain_format);

        Self {
            surface,
            swapchain_format,
            device,
            mesh_buffers,
            pipeline,
            uniform,
            storage,
            binding,
        }
    }

    pub fn write_instances(&mut self, instances: &[InstanceInfo], offset: u64) {
        let instance_data = bytemuck::cast_slice(instances);
        self.device
            .write_buffer(&self.storage, offset, instance_data)
    }

    pub fn write_mesh(&mut self, mesh: &Mesh, vertex_offset: u64, index_offset: u64) {
        self.mesh_buffers
            .write_vertices(&self.device, &mesh.vertices, vertex_offset);
        self.mesh_buffers
            .write_indices(&self.device, &mesh.indices, index_offset);
    }

    pub fn render<'a>(&'a self, info: &RenderInfo, draws: impl Iterator<Item = &'a StaticMesh>) {
        let render_data = bytemuck::bytes_of(info);
        self.device.write_buffer(&self.uniform, 0, render_data);
        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(SurfaceError::Outdated) => return,
            e => e.unwrap(),
        };
        let view = frame.texture.create_view(&Default::default());
        self.device.render(
            &view,
            &self.mesh_buffers,
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

        self.device
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
