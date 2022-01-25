use std::mem;
use bytemuck::{Pod, Zeroable};
use gdevice::GfxDevice;
use gbuf::GBuffer;
use instances::Instances;
use mesh_buf::MeshBuffers;
use pipeline::ColorMeshPipeline;
use raw_window_handle::HasRawWindowHandle;
use std::ops::Range;
use swapchain::Swapchain;
use uniform::Uniform;
use wgpu::{
    Backends, Color, CommandEncoder, Instance, LoadOp, Operations, RenderPass,
    RenderPassColorAttachment, RenderPassDescriptor, SurfaceError, TextureView,
};
use gbuf::pipeline::GBufferPipeline;

mod gdevice;
mod instances;
mod mesh_buf;
mod pipeline;
mod swapchain;
mod uniform;
mod gbuf;

pub struct Odc {
    swapchain: Swapchain,
    device: GfxDevice,
    mesh_buffers: MeshBuffers,
    instances: Instances,
    uniform: Uniform,
    gbuffer: GBuffer,
    // pipeline: ColorMeshPipeline,
    gbuf_pipeline: GBufferPipeline,
}

impl Odc {
    pub fn new(window: &impl HasRawWindowHandle, size: WindowSize) -> Self {
        let instance = Instance::new(Backends::all());

        let surface = unsafe { instance.create_surface(&window) };
        let device = GfxDevice::new(&instance, Some(&surface));
        let swapchain = Swapchain::new(&device, surface);
        swapchain.resize(&device, size);

        let mesh_buffers = MeshBuffers::new(&device);

        let instances = Instances::new(&device);
        let uniform = Uniform::new(&device);

        let gbuffer = GBuffer::new(&device, size);

        // let pipeline = ColorMeshPipeline::new(&device, &instances, &uniform);
        let gbuf_pipeline = GBufferPipeline::new(&device, swapchain.format);

        Self {
            swapchain,
            device,
            mesh_buffers,
            instances,
            uniform,
            gbuffer,
            // pipeline,
            gbuf_pipeline,
        }
    }

    pub fn write_instances<I: Pod>(&mut self, instances: &[I], offset: u64) {
        let byte_offset = mem::size_of::<I>() as u64 * offset;
        let instance_data = bytemuck::cast_slice(instances);
        self.device
            .queue
            .write_buffer(&self.instances.buffer, byte_offset, instance_data)
    }

    pub fn write_vertices<V: Pod>(&mut self, vertices: &[V], offset: u64) {
        let byte_offset = mem::size_of::<V>() as u64 * offset;
        let data = bytemuck::cast_slice(vertices);
        self.mesh_buffers.write_vertices(&self.device, data, byte_offset);
    }

    pub fn write_indices(&mut self, indices: &[u32], offset: u64) {
        self.mesh_buffers.write_indices(&self.device, indices, offset);
    }

    pub fn render(&self, info: &RenderInfo, draws: Draws) {
        self.update_uniform(info);

        let frame = match self.swapchain.surface.get_current_texture() {
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
        pass.set_pipeline(&self.gbuf_pipeline.pipeline);
        pass.draw(0..3, 0..1);
        // self.mesh_buffers.bind(pass);
        // pass.set_bind_group(0, &self.instances.bind_group, &[]);
        // pass.set_bind_group(1, &self.uniform.bind_group, &[]);
        // for draw in draws.static_mesh {
        //     pass.draw_indexed(
        //         draw.indices.clone(),
        //         draw.base_vertex,
        //         draw.instances.clone(),
        //     );
        // }
    }

    pub fn resize(&mut self, size: WindowSize) {
        if size.is_zero_square() {
            return;
        }

        self.swapchain.resize(&self.device, size)
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
