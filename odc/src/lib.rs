use bytemuck::{Pod, Zeroable};
use config::{Config, ResourceConfig};
use gdevice::GfxDevice;
use pipeline::ColorMeshPipeline;
use raw_window_handle::HasRawWindowHandle;
use std::collections::HashMap;
use std::mem;
use std::ops::Range;
use swapchain::Swapchain;
use wgpu::{
    Backends, Buffer, BufferUsages, Color, CommandEncoder, IndexFormat, Instance, LoadOp,
    Operations, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, SurfaceError,
    TextureView,
};

pub mod config;
mod gdevice;
mod pipeline;
mod swapchain;

pub struct Odc {
    swapchain: Option<Swapchain>,
    device: GfxDevice,
    buffers: HashMap<u64, Buffer>,
    pipeline: ColorMeshPipeline,
}

impl Odc {
    pub fn new<Window: HasRawWindowHandle>(config: &Config<Window>) -> Self {
        let instance = Instance::new(Backends::all());

        let (device, swapchain) = if let Some(window) = &config.window {
            let surface = unsafe { instance.create_surface(&window.handle) };
            let device = GfxDevice::new(&instance, Some(&surface));
            let swapchain = Swapchain::new(&device, surface);
            swapchain.resize(&device, window.size);
            (device, Some(swapchain))
        } else {
            let device = GfxDevice::new(&instance, None);
            (device, None)
        };

        let mut buffers = HashMap::new();

        for (id, res) in &config.resources {
            match res {
                ResourceConfig::VertexBuffer(size) => {
                    let usages = BufferUsages::COPY_DST | BufferUsages::VERTEX;
                    let buffer = device.create_gpu_buffer(*size, usages);
                    buffers.insert(*id, buffer);
                }
                ResourceConfig::IndexBuffer(size) => {
                    let usages = BufferUsages::COPY_DST | BufferUsages::INDEX;
                    let buffer = device.create_gpu_buffer(*size, usages);
                    buffers.insert(*id, buffer);
                }
                ResourceConfig::UniformBuffer(size) => {
                    let usages = BufferUsages::COPY_DST | BufferUsages::UNIFORM;
                    let buffer = device.create_gpu_buffer(*size, usages);
                    buffers.insert(*id, buffer);
                }
            };
        }

        let pipeline =
            ColorMeshPipeline::new(&device, swapchain.as_ref().unwrap().format, &buffers[&3]);

        Self {
            swapchain,
            device,
            buffers,
            pipeline,
        }
    }

    pub fn write_buffer<T: Pod>(&self, buffer_id: &u64, items: &[T], offset: u64) {
        let buffer = if let Some(buf) = self.buffers.get(buffer_id) {
            buf
        } else {
            log::error!("try to write to unexistent buffer: {buffer_id}");
            return;
        };
        let byte_offset = mem::size_of::<T>() as u64 * offset;
        let data_bytes = bytemuck::cast_slice(items);
        self.device
            .queue
            .write_buffer(buffer, byte_offset, data_bytes);
    }

    pub fn render(&self, draws: Draws) {
        let swapchain = self.swapchain.as_ref().unwrap();

        let frame = match swapchain.surface.get_current_texture() {
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

    fn draw_colored_geometry<'a>(&'a self, pass: &mut RenderPass<'a>, draws: Draws) {
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_vertex_buffer(0, self.buffers[&0].slice(..));
        pass.set_vertex_buffer(1, self.buffers[&1].slice(..));
        pass.set_index_buffer(self.buffers[&2].slice(..), IndexFormat::Uint32);
        pass.set_bind_group(0, &self.pipeline.uniform_bind_group, &[]);
        for draw in draws.static_mesh {
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

        self.swapchain.as_ref().unwrap().resize(&self.device, size)
    }
}

#[derive(Debug, Default, Copy, Clone)]
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

pub type Transform = [[f32; 4]; 4];

pub struct Draws<'a> {
    pub static_mesh: &'a [StaticMesh],
}
