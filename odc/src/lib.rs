use bytemuck::{Pod, Zeroable};
use gdevice::GfxDevice;
use mesh_buf::MeshBuffers;
use raw_window_handle::HasRawWindowHandle;
use std::borrow::Cow;
use std::mem;
use std::num::NonZeroU64;
use std::ops::Range;
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress,
    BufferBindingType, BufferDescriptor, BufferSize, BufferUsages, Color, CommandEncoder,
    FragmentState, Instance, LoadOp, Operations, PipelineLayout, PipelineLayoutDescriptor,
    PresentMode, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    Surface, SurfaceConfiguration, SurfaceError, TextureFormat, TextureUsages, TextureView,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexState, VertexStepMode,
};

mod gdevice;
mod mesh_buf;

pub struct OdcCore {
    surface: Surface,
    swapchain_format: TextureFormat,
    device: GfxDevice,
    mesh_buffers: MeshBuffers,
    instances: Instances,
    uniform: Uniform,
    pipeline: RenderPipeline,
}

impl OdcCore {
    pub fn new(window: &impl HasRawWindowHandle, size: WindowSize) -> Self {
        let instance = Instance::new(Backends::all());

        let surface = unsafe { instance.create_surface(&window) };
        let device = GfxDevice::new(&instance, Some(&surface));
        let swapchain_format = surface
            .get_preferred_format(&device.adapter)
            .expect("can't find suit surface format");

        let mesh_buffers = MeshBuffers::new(&device);

        let instances = Instances::new(&device);
        let uniform = Uniform::new(&device);

        let pipeline_layout =
            Self::create_pipeline_layout(&device, &instances.layout, &uniform.layout);
        let pipeline = Self::create_pipeline(&device, &pipeline_layout, swapchain_format);

        configure_surface(&device, &surface, size, swapchain_format);

        Self {
            surface,
            swapchain_format,
            device,
            mesh_buffers,
            instances,
            uniform,
            pipeline,
        }
    }

    pub fn write_instances(&mut self, instances: &[InstanceInfo], offset: u64) {
        let instance_data = bytemuck::cast_slice(instances);
        self.device
            .queue
            .write_buffer(&self.instances.buffer, offset, instance_data)
    }

    pub fn write_mesh(&mut self, mesh: &Mesh, vertex_offset: u64, index_offset: u64) {
        self.mesh_buffers
            .write_vertices(&self.device, &mesh.vertices, vertex_offset);
        self.mesh_buffers
            .write_indices(&self.device, &mesh.indices, index_offset);
    }

    pub fn render<'b>(&self, info: &'b RenderInfo, draws: impl Iterator<Item = &'b StaticMesh>) {
        self.update_uniform(info);

        let frame = match self.surface.get_current_texture() {
            Ok(f) => f,
            Err(SurfaceError::Outdated) => return,
            e => e.unwrap(),
        };
        let view = frame.texture.create_view(&Default::default());

        let mut encoder = self
            .device
            .device
            .create_command_encoder(&Default::default());
        {
            let mut render_pass = self.begin_render_pass(&mut encoder, &view);
            self.draw_colored_geometry(&mut render_pass, draws);
        }
        let cmd_buf = encoder.finish();

        self.device.queue.submit(Some(cmd_buf));
        frame.present();
    }

    fn update_uniform(&self, info: &RenderInfo) {
        let render_data = bytemuck::bytes_of(info);
        self.device
            .queue
            .write_buffer(&self.uniform.buffer, 0, render_data);
    }

    fn begin_render_pass<'a>(
        &self,
        encoder: &'a mut CommandEncoder,
        view: &'a TextureView,
    ) -> RenderPass<'a> {
        let attachment = RenderPassColorAttachment {
            view,
            resolve_target: None,
            ops: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true,
            },
        };
        let attachments = [attachment];
        let render_pass_descriptor = RenderPassDescriptor {
            color_attachments: &attachments,
            ..Default::default()
        };
        encoder.begin_render_pass(&render_pass_descriptor)
    }

    fn draw_colored_geometry<'a, 'b>(
        &'a self,
        pass: &mut RenderPass<'a>,
        draws: impl Iterator<Item = &'b StaticMesh>,
    ) {
        pass.set_pipeline(&self.pipeline);
        self.mesh_buffers.bind(pass);
        pass.set_bind_group(0, &self.instances.bind_group, &[]);
        pass.set_bind_group(1, &self.uniform.bind_group, &[]);
        for draw in draws {
            pass.draw_indexed(
                draw.indices.clone(),
                draw.base_vertex,
                draw.instances.clone(),
            );
        }
    }

    pub fn resize(&mut self, size: WindowSize) {
        if size.is_zero_square() {
            return;
        }

        configure_surface(&self.device, &self.surface, size, self.swapchain_format)
    }

    fn create_shader(device: &GfxDevice) -> ShaderModule {
        let shader_src = Cow::Borrowed(include_str!("shader.wgsl"));
        let source = ShaderSource::Wgsl(shader_src);
        let descriptor = ShaderModuleDescriptor {
            label: None,
            source,
        };
        device.device.create_shader_module(&descriptor)
    }

    fn create_pipeline_layout(
        device: &GfxDevice,
        instances_layout: &BindGroupLayout,
        uniform_layout: &BindGroupLayout,
    ) -> PipelineLayout {
        let layouts = [instances_layout, uniform_layout];
        let descriptor = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        };
        device.device.create_pipeline_layout(&descriptor)
    }

    fn create_pipeline(
        device: &GfxDevice,
        layout: &PipelineLayout,
        output_format: TextureFormat,
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

        let shader = Self::create_shader(device);

        let vertex = VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[vertex_layout],
        };

        let formats = [output_format.into()];
        let fragment = Some(FragmentState {
            module: &shader,
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

        device.device.create_render_pipeline(&descriptor)
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

pub struct Instances {
    pub buffer: Buffer,
    pub layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl Instances {
    pub fn new(device: &GfxDevice) -> Self {
        let usage = BufferUsages::COPY_DST | BufferUsages::STORAGE;
        let buffer = create_gpu_buffer(device, Self::instances_size(), usage);
        let layout = Self::create_layout(device);
        let bind_group = Self::create_bind_group(device, &layout, &buffer);

        Self {
            buffer,
            layout,
            bind_group,
        }
    }

    pub const MAX_INSTANCE_COUNT: usize = 2usize.pow(16);

    fn instances_size() -> u64 {
        InstanceInfo::size() as u64 * Self::MAX_INSTANCE_COUNT as u64
    }

    fn create_layout(device: &GfxDevice) -> BindGroupLayout {
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

    pub fn create_bind_group(
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

struct Uniform {
    pub buffer: Buffer,
    pub layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl Uniform {
    pub fn new(device: &GfxDevice) -> Self {
        let usage = BufferUsages::COPY_DST | BufferUsages::UNIFORM;
        let buffer = crate::create_gpu_buffer(device, Self::buffer_size().get(), usage);
        let layout = Self::create_bind_group_layout(device);
        let bind_group = Self::create_bind_group(device, &buffer, &layout);

        Self {
            buffer,
            layout,
            bind_group,
        }
    }

    fn buffer_size() -> BufferSize {
        NonZeroU64::new(mem::size_of::<RenderInfo>() as _).expect("Zero sized uniform")
    }

    fn create_bind_group_layout(device: &GfxDevice) -> BindGroupLayout {
        let uniform_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(Self::buffer_size()),
            },
            count: None,
            visibility: ShaderStages::VERTEX,
        };

        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[uniform_entry],
        };
        device.device.create_bind_group_layout(&descriptor)
    }

    fn create_bind_group(
        device: &GfxDevice,
        buffer: &Buffer,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let entries = [BindGroupEntry {
            binding: 0,
            resource: buffer.as_entire_binding(),
        }];

        let descriptor = BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        };
        device.device.create_bind_group(&descriptor)
    }
}
