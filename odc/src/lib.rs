use bytemuck::{Pod, Zeroable};
use config::{Config, ResourceConfig};
use gdevice::GfxDevice;
use instances::Instances;
use pipeline::ColorMeshPipeline;
use raw_window_handle::HasRawWindowHandle;
use std::collections::HashMap;
use std::mem;
use std::ops::Range;
use swapchain::Swapchain;
use uniform::Uniform;
use wgpu::{
    Backends, Buffer, BufferUsages, Color, CommandEncoder, Instance, LoadOp, Operations,
    RenderPass, RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, TextureView, IndexFormat
};

pub mod config;
mod gdevice;
mod instances;
mod pipeline;
mod swapchain;
mod uniform;

pub struct Odc {
    swapchain: Option<Swapchain>,
    device: GfxDevice,
    buffers: HashMap<u64, Buffer>,
    instances: Instances,
    uniform: Uniform,
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
                } // ResourceConfig::UniformBuffer(size) => {
                  //     let usages = BufferUsages::COPY_DST | BufferUsages::UNIFORM;
                  //     device.create_gpu_buffer(size, usages)
                  // }
            };
        }

        let instances = Instances::new(&device);
        let uniform = Uniform::new(&device);

        let pipeline = ColorMeshPipeline::new(
            &device,
            &instances,
            &uniform,
            swapchain.as_ref().unwrap().format,
        );

        Self {
            swapchain,
            device,
            buffers,
            instances,
            uniform,
            pipeline,
        }
    }

    pub fn write_instances<I: Pod>(&mut self, instances: &[I], offset: u64) {
        let byte_offset = mem::size_of::<I>() as u64 * offset;
        let instance_data = bytemuck::cast_slice(instances);
        self.device
            .queue
            .write_buffer(&self.instances.buffer, byte_offset, instance_data)
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
        self.device.queue.write_buffer(buffer, byte_offset, data_bytes);
    }

    pub fn render(&self, info: &RenderInfo, draws: Draws) {
        let swapchain = self.swapchain.as_ref().unwrap();

        self.update_uniform(info);

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

    fn draw_colored_geometry<'a>(&'a self, pass: &mut RenderPass<'a>, draws: Draws) {
        pass.set_pipeline(&self.pipeline.pipeline);
        pass.set_vertex_buffer(0, self.buffers[&0].slice(..));
        pass.set_index_buffer(self.buffers[&1].slice(..), IndexFormat::Uint32);
        pass.set_bind_group(0, &self.instances.bind_group, &[]);
        pass.set_bind_group(1, &self.uniform.bind_group, &[]);
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
