use wgpu::{Adapter, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, Buffer, BufferDescriptor, Device, DeviceDescriptor, FragmentState, Instance, LoadOp, Operations, PipelineLayout, PipelineLayoutDescriptor, Queue, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule, ShaderModuleDescriptor, ShaderSource, Surface, TextureView, VertexBufferLayout, VertexState};
use std::borrow::Cow;
use crate::{BindGroupLayoutEntry, BindingType, BufferAddress, BufferBindingType, BufferSize, BufferUsages, Color, IndexFormat, Limits, PresentMode, ShaderStages, StaticMesh, SurfaceConfiguration, TextureFormat, TextureUsages, Vertex, VertexAttribute, VertexFormat, VertexStepMode, WindowSize};

pub struct GraphicsDevice {
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl GraphicsDevice {
    pub fn new(instance: &Instance, surface: Option<&Surface>) -> Self {
        let adapter = Self::request_adapter(instance, surface);
        let (device, queue) = Self::request_device(&adapter);
        Self {
            adapter,
            device,
            queue,
        }
    }

    pub fn create_gpu_buffer(&self, size: BufferAddress, usage: BufferUsages) -> Buffer {
        let descriptor = BufferDescriptor {
            label: None,
            size,
            usage,
            mapped_at_creation: false,
        };
        self.device.create_buffer(&descriptor)
    }

    pub fn create_bind_group_layout(
        &self,
        uniform_size: BufferSize,
        storage_size: BufferSize,
    ) -> BindGroupLayout {
        let uniform_entry = BindGroupLayoutEntry {
            binding: 0,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(uniform_size),
            },
            count: None,
            visibility: ShaderStages::VERTEX,
        };

        let storage_entry = BindGroupLayoutEntry {
            binding: 1,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: Some(storage_size),
            },
            count: None,
            visibility: ShaderStages::VERTEX,
        };

        let descriptor = BindGroupLayoutDescriptor {
            label: None,
            entries: &[uniform_entry, storage_entry],
        };
        self.device.create_bind_group_layout(&descriptor)
    }

    pub fn create_binding(
        &self,
        uniform: &Buffer,
        storage: &Buffer,
        layout: &BindGroupLayout,
    ) -> BindGroup {
        let entries = [
            BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: storage.as_entire_binding(),
            },
        ];

        let descriptor = BindGroupDescriptor {
            label: None,
            layout,
            entries: &entries,
        };
        self.device.create_bind_group(&descriptor)
    }

    pub fn create_shader(&self) -> ShaderModule {
        let shader_src = Cow::Borrowed(include_str!("shader.wgsl"));
        let source = ShaderSource::Wgsl(shader_src);
        let descriptor = ShaderModuleDescriptor {
            label: None,
            source,
        };
        self.device.create_shader_module(&descriptor)
    }

    pub fn create_pipeline_layout(
        &self,
        bind_group_layouts: &[&BindGroupLayout],
    ) -> PipelineLayout {
        let descriptor = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts,
            push_constant_ranges: &[],
        };
        self.device.create_pipeline_layout(&descriptor)
    }

    pub fn preferred_surface_format(&self, surface: &Surface) -> Option<TextureFormat> {
        surface.get_preferred_format(&self.adapter)
    }

    pub fn create_pipeline(
        &self,
        layout: &PipelineLayout,
        shader: &ShaderModule,
        swapchain_format: TextureFormat,
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

        let formats = [swapchain_format.into()];
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

        self.device.create_render_pipeline(&descriptor)
    }

    pub fn configure_surface(&self, surface: &Surface, size: WindowSize, format: TextureFormat) {
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.0,
            height: size.1,
            present_mode: PresentMode::Mailbox,
        };

        surface.configure(&self.device, &config);
    }

    pub fn write_buffer(&self, buffer: &Buffer, offset: BufferAddress, data: &[u8]) {
        self.queue.write_buffer(buffer, offset, data);
    }

    pub fn render<'a>(
        &self,
        view: &TextureView,
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
        binding: &BindGroup,
        pipeline: &RenderPipeline,
        draws: impl Iterator<Item = &'a StaticMesh>,
    ) {
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
            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.set_bind_group(0, binding, &[]);
            for draw in draws {
                render_pass.draw_indexed(
                    draw.indices.clone(),
                    draw.base_vertex,
                    draw.instances.clone(),
                );
            }
        }
        self.queue.submit(Some(encoder.finish()));
    }

    fn request_adapter(instance: &Instance, surface: Option<&Surface>) -> Adapter {
        let options = RequestAdapterOptions {
            compatible_surface: surface,
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
}
