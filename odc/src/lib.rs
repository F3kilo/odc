use raw_window_handle::HasRawWindowHandle;
use std::borrow::Cow;
use wgpu::{
    Adapter, Backends, Color, CommandBuffer, Device, DeviceDescriptor, FragmentState, Instance,
    Limits, LoadOp, Operations, PipelineLayout, PresentMode, Queue,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, ShaderModule, ShaderModuleDescriptor, ShaderSource, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages, TextureView, VertexState,
};

pub struct TriangleRenderer {
    surface: Surface,
    swapchain_format: TextureFormat,
    device: Device,
    queue: Queue,
    pipeline: RenderPipeline,
}

impl TriangleRenderer {
    pub fn new(window: &impl HasRawWindowHandle, size: WindowSize) -> Self {
        let instance = wgpu::Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        let adapter = Self::request_adapter(&instance, &surface);
        let (device, queue) = Self::request_device(&adapter);
        let shader = Self::create_shader(&device);
        let pipeline_layout = device.create_pipeline_layout(&Default::default());
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
        }
    }

    pub fn render_triangle(&self) {
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
            render_pass.draw(0..3, 0..1);
        }
        encoder.finish()
    }
}

pub struct WindowSize(pub u32, pub u32);
