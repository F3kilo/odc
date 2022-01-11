use bytemuck::{Pod, Zeroable};
use raw_window_handle::HasRawWindowHandle;
use std::borrow::Cow;
use std::mem;
use std::num::NonZeroU64;
use std::ops::Range;
use wgpu::{
    Adapter, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferBindingType,
    BufferDescriptor, BufferUsages, Color, CommandBuffer, Device, DeviceDescriptor, FragmentState,
    IndexFormat, Instance, Limits, LoadOp, Operations, PipelineLayout, PipelineLayoutDescriptor,
    PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
    TextureView, VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

pub struct TriangleRenderer {
    surface: Surface,
    swapchain_format: TextureFormat,
    device: Device,
    queue: Queue,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    pipeline: RenderPipeline,
    uniform: Buffer,
    uniform_binding: BindGroup,
    storage: Buffer,
    storage_binding: BindGroup,
}

impl TriangleRenderer {
    pub const MAX_INSTANCE_COUNT: usize = 2usize.pow(16);
    pub const VERTEX_BUFFER_SIZE: u64 = 2u64.pow(24);
    pub const INDEX_BUFFER_SIZE: u64 = 2u64.pow(22);

    pub fn new(window: &impl HasRawWindowHandle, size: WindowSize) -> Self {
        let instance = wgpu::Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = Self::request_adapter(&instance, &surface);
        let (device, queue) = Self::request_device(&adapter);

        let vertex_buffer = Self::create_vertex_buffer(&device);
        let index_buffer = Self::create_index_buffer(&device);

        let uniform = Self::create_uniform_buffer(&device);
        let uniform_layout = Self::create_uniform_layout(&device);
        let uniform_binding = Self::create_uniform_binding(&device, &uniform, &uniform_layout);

        let storage = Self::create_storage_buffer(&device);
        let storage_layout = Self::create_storage_layout(&device);
        let storage_binding = Self::create_storage_binding(&device, &storage, &storage_layout);

        let shader = Self::create_shader(&device);
        let pipeline_layout =
            Self::create_pipeline_layout(&device, &uniform_layout, &storage_layout);
        let swapchain_format = surface.get_preferred_format(&adapter).unwrap();
        let pipeline = Self::create_pipeline(&device, &pipeline_layout, &shader, &swapchain_format);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.0,
            height: size.1,
            present_mode: PresentMode::Mailbox,
        };

        surface.configure(&device, &config);
        Self {
            surface,
            swapchain_format,
            device,
            queue,
            vertex_buffer,
            index_buffer,
            pipeline,
            uniform,
            uniform_binding,
            storage,
            storage_binding,
        }
    }

    pub fn write_instances(&mut self, instances: &[InstanceInfo], offset: u64) {
        let instance_data = bytemuck::cast_slice(instances);
        self.queue
            .write_buffer(&self.storage, offset, instance_data);
    }

    pub fn write_mesh(&mut self, mesh: &Mesh, vertex_offset: u64, index_offset: u64) {
        let vertex_data = bytemuck::cast_slice(&mesh.vertices);
        self.queue
            .write_buffer(&self.vertex_buffer, vertex_offset, vertex_data);

        let index_data = bytemuck::cast_slice(&mesh.indices);
        self.queue
            .write_buffer(&self.index_buffer, index_offset, index_data);
    }

    pub fn render_triangle<'a>(
        &self,
        info: &RenderInfo,
        draws: impl Iterator<Item = &'a StaticMesh>,
    ) {
        let info_bytes = bytemuck::bytes_of(info);
        self.queue.write_buffer(&self.uniform, 0, info_bytes);
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let cmd_buffer = self.prepare_cmd_buffer(&view, draws);
        self.queue.submit(Some(cmd_buffer));
        frame.present();
    }

    pub fn resize(&mut self, size: WindowSize) {
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: self.swapchain_format,
            width: size.0,
            height: size.1,
            present_mode: PresentMode::Mailbox,
        };
        self.surface.configure(&self.device, &config);
    }

    fn request_adapter(instance: &Instance, surface: &Surface) -> Adapter {
        let options = RequestAdapterOptions {
            compatible_surface: Some(surface),
            ..Default::default()
        };
        let adapter_fut = instance.request_adapter(&options);
        pollster::block_on(adapter_fut).unwrap()
    }

    fn request_device(adapter: &Adapter) -> (Device, Queue) {
        let limits = Limits::downlevel_defaults().using_resolution(adapter.limits());
        let descriptor = DeviceDescriptor {
            limits,
            ..Default::default()
        };
        let device_fut = adapter.request_device(&descriptor, None);
        pollster::block_on(device_fut).unwrap()
    }

    fn create_shader(device: &Device) -> ShaderModule {
        let shader_src = Cow::Borrowed(include_str!("shader.wgsl"));
        let source = ShaderSource::Wgsl(shader_src);
        let descriptor = ShaderModuleDescriptor {
            label: None,
            source,
        };
        device.create_shader_module(&descriptor)
    }

    fn uniform_size() -> NonZeroU64 {
        NonZeroU64::new(mem::size_of::<RenderInfo>() as _).expect("Zero sized uniform")
    }

    fn create_uniform_layout(device: &Device) -> BindGroupLayout {
        let uniform_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(Self::uniform_size()),
            },
            count: None,
            visibility: ShaderStages::VERTEX,
        };
        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[uniform_entry],
        };
        device.create_bind_group_layout(&descriptor)
    }

    fn create_storage_layout(device: &Device) -> BindGroupLayout {
        let alignment = device.limits().min_storage_buffer_offset_alignment;
        let min_size = Self::aligned_storage_size(alignment as _);

        let storage_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: true,
                min_binding_size: Some(min_size),
            },
            count: None,
            visibility: ShaderStages::VERTEX,
        };
        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[storage_entry],
        };
        device.create_bind_group_layout(&descriptor)
    }

    fn create_pipeline_layout(
        device: &Device,
        uniform_layout: &BindGroupLayout,
        storage_layout: &BindGroupLayout,
    ) -> PipelineLayout {
        let layouts = [uniform_layout, storage_layout];
        let descriptor = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        };
        device.create_pipeline_layout(&descriptor)
    }

    fn create_pipeline(
        device: &Device,
        layout: &PipelineLayout,
        shader: &ShaderModule,
        swapchain_format: &TextureFormat,
    ) -> RenderPipeline {
        let attributes = [
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: Vertex::position_offset() as _,
                shader_location: 0,
            },
            VertexAttribute {
                format: VertexFormat::Float32x4,
                offset: Vertex::color_offset() as _,
                shader_location: 1,
            },
        ];

        let vertex_layout = VertexBufferLayout {
            array_stride: Vertex::size() as _,
            attributes: &attributes,
            step_mode: VertexStepMode::Vertex,
        };

        let vertex = VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[vertex_layout],
        };

        let formats = [(*swapchain_format).into()];
        let fragment = Some(FragmentState {
            module: shader,
            entry_point: "fs_main",
            targets: &formats,
        });

        let descriptor = RenderPipelineDescriptor {
            label: None,
            layout: Some(layout),
            vertex,
            fragment,
            primitive: Default::default(),
            multisample: Default::default(),
            depth_stencil: None,
            multiview: None,
        };

        device.create_render_pipeline(&descriptor)
    }

    fn create_vertex_buffer(device: &Device) -> Buffer {
        let descriptor = BufferDescriptor {
            label: None,
            size: Self::VERTEX_BUFFER_SIZE,
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
            mapped_at_creation: false,
        };
        device.create_buffer(&descriptor)
    }

    fn create_index_buffer(device: &Device) -> Buffer {
        let descriptor = BufferDescriptor {
            label: None,
            size: Self::INDEX_BUFFER_SIZE,
            usage: BufferUsages::COPY_DST | BufferUsages::INDEX,
            mapped_at_creation: false,
        };
        device.create_buffer(&descriptor)
    }

    fn aligned_storage_size(alignment: usize) -> NonZeroU64 {
        let size = mem::size_of::<InstanceInfo>();
        let rem = size % alignment;
        let result = if rem == 0 {
            size
        } else {
            size + alignment - rem
        };
        NonZeroU64::new(result as _).expect("Unexpected zero size storage buffer item")
    }

    fn create_storage_buffer(device: &Device) -> Buffer {
        let alignment = device.limits().min_storage_buffer_offset_alignment;
        let aligned_size = Self::aligned_storage_size(alignment as _).get();
        let size = aligned_size * Self::MAX_INSTANCE_COUNT as u64;
        let descriptor = BufferDescriptor {
            label: None,
            size,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            mapped_at_creation: false,
        };
        device.create_buffer(&descriptor)
    }

    fn create_uniform_buffer(device: &Device) -> Buffer {
        let descriptor = BufferDescriptor {
            label: None,
            size: Self::uniform_size().get() as _,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        };
        device.create_buffer(&descriptor)
    }

    fn create_uniform_binding(
        device: &Device,
        uniform: &Buffer,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let entries = [BindGroupEntry {
            binding: 0,
            resource: uniform.as_entire_binding(),
        }];

        let descriptor = BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        };
        device.create_bind_group(&descriptor)
    }

    fn create_storage_binding(
        device: &Device,
        storage: &Buffer,
        layout: &BindGroupLayout,
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
        device.create_bind_group(&descriptor)
    }

    fn prepare_cmd_buffer<'a>(
        &self,
        view: &TextureView,
        draws: impl Iterator<Item = &'a StaticMesh>,
    ) -> CommandBuffer {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let attachment = RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true,
            },
        };
        let attachments = [attachment];
        let color_attachments = &attachments;
        let render_pass_descriptor = RenderPassDescriptor {
            color_attachments,
            ..Default::default()
        };
        {
            let mut render_pass = encoder.begin_render_pass(&render_pass_descriptor);
            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.set_bind_group(0, &self.uniform_binding, &[]);
            render_pass.set_bind_group(1, &self.storage_binding, &[0]);
            for draw in draws {
                render_pass.draw_indexed(
                    draw.indices.clone(),
                    draw.base_vertex,
                    draw.instances.clone(),
                );
            }
        }
        encoder.finish()
    }
}

pub struct WindowSize(pub u32, pub u32);

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
