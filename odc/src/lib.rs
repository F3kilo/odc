use bytemuck::{Pod, Zeroable};
use raw_window_handle::HasRawWindowHandle;
use std::borrow::Cow;
use std::mem;
use std::num::NonZeroU64;
use wgpu::{
    Adapter, Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress,
    BufferBindingType, BufferDescriptor, BufferUsages, Color, CommandBuffer, Device,
    DeviceDescriptor, FragmentState, Instance, Limits, LoadOp, Operations, PipelineLayout,
    PipelineLayoutDescriptor, PresentMode, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, RequestAdapterOptions, ShaderModule,
    ShaderModuleDescriptor, ShaderSource, ShaderStages, Surface, SurfaceConfiguration,
    TextureFormat, TextureUsages, TextureView, VertexState,
};

pub struct TriangleRenderer {
    surface: Surface,
    swapchain_format: TextureFormat,
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
    uniform: Buffer,
    uniform_binding: BindGroup,
}

impl TriangleRenderer {
    pub fn new(window: &impl HasRawWindowHandle, size: WindowSize) -> Self {
        let instance = wgpu::Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = Self::request_adapter(&instance, &surface);
        let (device, queue) = Self::request_device(&adapter);
        let uniform = Self::create_uniform_buffer(&device);
        let uniform_layout = Self::create_uniform_layout(&device);
        let uniform_binding = Self::create_uniform_binding(&device, &uniform, &uniform_layout);

        let shader = Self::create_shader(&device);
        let pipeline_layout = Self::create_pipeline_layout(&device, &uniform_layout);
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
            pipeline,
            uniform,
            uniform_binding,
        }
    }

    pub fn render_triangle(&self, info: &RenderInfo) {
        let info_bytes = bytemuck::bytes_of(info);
        self.queue.write_buffer(&self.uniform, 0, info_bytes);
        let frame = self.surface.get_current_texture().unwrap();
        let view = frame.texture.create_view(&Default::default());
        let cmd_buffer = self.prepare_cmd_buffer(&view);
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
        let limits = Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits());
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

    fn create_pipeline_layout(device: &Device, uniform_layout: &BindGroupLayout) -> PipelineLayout {
        let layouts = [uniform_layout];
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
        let vertex = VertexState {
            module: shader,
            entry_point: "vs_main",
            buffers: &[],
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

    fn create_uniform_buffer(device: &Device) -> Buffer {
        let descriptor = BufferDescriptor {
            label: None,
            size: mem::size_of::<RenderInfo>() as BufferAddress,
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

    fn prepare_cmd_buffer(&self, view: &TextureView) -> CommandBuffer {
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
            render_pass.set_bind_group(0, &self.uniform_binding, &[]);
            render_pass.draw(0..3, 0..1);
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

pub type Transform = [[f32; 4]; 4];
