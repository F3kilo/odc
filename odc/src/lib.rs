use bytemuck::{Pod, Zeroable};
use gdevice::GfxDevice;
use mesh_buf::MeshBuffers;
use raw_window_handle::HasRawWindowHandle;
use renderer::BasicRenderer;
use std::mem;
use std::ops::Range;
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress,
    BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, Instance, PresentMode,
    ShaderStages, Surface, SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages,
};

mod gdevice;
mod mesh_buf;
mod renderer;

pub struct TriangleRenderer {
    surface: Surface,
    swapchain_format: TextureFormat,
    device: GfxDevice,
    mesh_buffers: MeshBuffers,
    instances: Buffer,
    instances_binding: BindGroup,
    renderer: BasicRenderer,
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

        let instances = create_gpu_buffer(
            &device,
            Self::storage_size(),
            BufferUsages::COPY_DST | BufferUsages::STORAGE,
        );

        let instances_binding_layout = Self::create_instances_binding_layout(&device);
        let instances_binding =
            Self::create_instances_binding(&device, &instances_binding_layout, &instances);

        let swapchain_format = surface
            .get_preferred_format(&device.adapter)
            .expect("can't find suit surface format");

        let renderer = BasicRenderer::new(&device, swapchain_format, &instances_binding_layout);

        configure_surface(&device, &surface, size, swapchain_format);

        Self {
            surface,
            swapchain_format,
            device,
            mesh_buffers,
            instances,
            instances_binding,
            renderer,
        }
    }

    pub fn write_instances(&mut self, instances: &[InstanceInfo], offset: u64) {
        let instance_data = bytemuck::cast_slice(instances);
        self.device
            .queue
            .write_buffer(&self.instances, offset, instance_data)
    }

    pub fn write_mesh(&mut self, mesh: &Mesh, vertex_offset: u64, index_offset: u64) {
        self.mesh_buffers
            .write_vertices(&self.device, &mesh.vertices, vertex_offset);
        self.mesh_buffers
            .write_indices(&self.device, &mesh.indices, index_offset);
    }

    pub fn render<'a>(&'a self, info: &RenderInfo, draws: impl Iterator<Item = &'a StaticMesh>) {
        self.renderer.update(&self.device, info);

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(SurfaceError::Outdated) => return,
            e => e.unwrap(),
        };
        let view = frame.texture.create_view(&Default::default());

        let encoder = self
            .device
            .device
            .create_command_encoder(&Default::default());

        let cmd_buf = self.renderer.render(
            encoder,
            &self.mesh_buffers,
            &self.instances_binding,
            &view,
            draws,
        );

        self.device.queue.submit(Some(cmd_buf));
        frame.present();
    }

    pub fn resize(&mut self, size: WindowSize) {
        if size.is_zero_square() {
            return;
        }

        configure_surface(&self.device, &self.surface, size, self.swapchain_format)
    }

    fn storage_size() -> u64 {
        InstanceInfo::size() as u64 * Self::MAX_INSTANCE_COUNT as u64
    }

    fn create_instances_binding_layout(device: &GfxDevice) -> BindGroupLayout {
        let storage_min_size =
            BufferSize::new(InstanceInfo::size() as _).expect("unexpected zero size instance");

        let storage_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(storage_min_size),
            },
            count: None,
            visibility: ShaderStages::VERTEX,
        };

        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[storage_entry],
        };
        device.device.create_bind_group_layout(&descriptor)
    }

    pub fn create_instances_binding(
        device: &GfxDevice,
        layout: &BindGroupLayout,
        storage: &Buffer,
    ) -> BindGroup {
        let entries = [BindGroupEntry {
            binding: 0,
            resource: storage.as_entire_binding(),
        }];

        let descriptor = BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        };
        device.device.create_bind_group(&descriptor)
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

pub fn configure_surface(
    device: &GfxDevice,
    surface: &Surface,
    size: WindowSize,
    format: TextureFormat,
) {
    let config = SurfaceConfiguration {
        usage: TextureUsages::RENDER_ATTACHMENT,
        format,
        width: size.0,
        height: size.1,
        present_mode: PresentMode::Mailbox,
    };

    surface.configure(&device.device, &config);
}

pub fn create_gpu_buffer(device: &GfxDevice, size: BufferAddress, usage: BufferUsages) -> Buffer {
    let descriptor = BufferDescriptor {
        label: None,
        size,
        usage,
        mapped_at_creation: false,
    };
    device.device.create_buffer(&descriptor)
}
