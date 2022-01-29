use crate::material::{Material, MaterialFactory, MaterialInfo};
use bytemuck::Pod;
use gbuf::GBuffer;
use gdevice::GfxDevice;
use instances::Instances;
use mesh_buf::MeshBuffers;
use raw_window_handle::HasRawWindowHandle;
use std::collections::HashMap;
use std::mem;
use std::ops::Range;
use swapchain::Swapchain;
use uniform::Uniform;
use wgpu::{
    Backends, CommandEncoder, Instance, PipelineLayout, PipelineLayoutDescriptor, RenderPass,
    RenderPassDescriptor, SurfaceError,
};

mod gbuf;
mod gdevice;
mod instances;
pub mod material;
mod mesh_buf;
mod swapchain;
mod uniform;

pub struct Odc {
    swapchain: Swapchain,
    device: GfxDevice,
    mesh_buffers: MeshBuffers,
    instances: Instances,
    uniform: Uniform,
    gbuffer: GBuffer,
    material_pipeline_layout: PipelineLayout,
    materials: HashMap<u64, Material>,
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

        let gbuffer = GBuffer::new(&device, size, swapchain.format);

        let material_pipeline_layout = Self::create_material_pipeline_layout(&device, &uniform);

        Self {
            swapchain,
            device,
            mesh_buffers,
            instances,
            uniform,
            gbuffer,
            material_pipeline_layout,
            materials: Default::default(),
        }
    }

    pub fn create_material(&self, info: &MaterialInfo) -> Material {
        let factory = MaterialFactory {
            device: &self.device,
            uniform: &self.uniform,
            layout: &self.material_pipeline_layout,
        };

        factory.create_material(info)
    }

    pub fn insert_material(&mut self, id: u64, material: Material) -> Option<Material> {
        self.materials.insert(id, material)
    }

    pub fn write_instances<T: Pod>(&self, instances: &[T], offset: u64) {
        let byte_offset = mem::size_of::<T>() as u64 * offset;
        let byte_data = bytemuck::cast_slice(instances);
        self.instances.write(&self.device, byte_data, byte_offset);
    }

    pub fn write_uniform<T: Pod>(&self, data: &[T], offset: u64) {
        let byte_offset = mem::size_of::<T>() as u64 * offset;
        let byte_data = bytemuck::cast_slice(data);
        self.uniform.write(&self.device, byte_data, byte_offset);
    }

    pub fn write_vertices<V: Pod>(&self, vertices: &[V], offset: u64) {
        let byte_offset = mem::size_of::<V>() as u64 * offset;
        let byte_data = bytemuck::cast_slice(vertices);
        self.mesh_buffers
            .write_vertices(&self.device, byte_data, byte_offset);
    }

    pub fn write_indices(&self, indices: &[u8], offset: u64) {
        self.mesh_buffers
            .write_indices(&self.device, indices, offset);
    }

    pub fn render(&self, draws: &Draws) {
        let mut encoder = self
            .device
            .device
            .create_command_encoder(&Default::default());

        {
            let mut render_pass = self.begin_render_pass(&mut encoder);
            self.draw_geometry(&mut render_pass, draws);
        }

        let frame = match self.swapchain.surface.get_current_texture() {
            Ok(f) => f,
            Err(SurfaceError::Outdated) => return,
            e => e.unwrap(),
        };
        let view = frame.texture.create_view(&Default::default());
        self.gbuffer.render(&mut encoder, &view);
        let cmd_buf = encoder.finish();

        self.device.queue.submit(Some(cmd_buf));
        frame.present();
    }

    fn create_material_pipeline_layout(device: &GfxDevice, uniform: &Uniform) -> PipelineLayout {
        let uniform_bind_group_layout = uniform.get_bind_group_layout();

        let layouts = [uniform_bind_group_layout];
        let descriptor = PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        };
        device.device.create_pipeline_layout(&descriptor)
    }

    fn begin_render_pass<'a>(&'a self, encoder: &'a mut CommandEncoder) -> RenderPass<'a> {
        let color_attachments = self.gbuffer.get_color_attachments();
        let depth_attachment = self.gbuffer.get_depth_attachment();

        let render_pass_descriptor = RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: Some(depth_attachment),
        };
        encoder.begin_render_pass(&render_pass_descriptor)
    }

    fn draw_geometry<'a>(&'a self, pass: &mut RenderPass<'a>, draws: &Draws) {
        self.mesh_buffers.bind(pass);
        self.instances.bind(pass);

        for (mat_id, to_draw) in draws {
            let material = match self.materials.get(mat_id) {
                Some(m) => m,
                None => {
                    log::error!("no material found for draws with id={}", mat_id);
                    continue;
                }
            };

            material.draw(pass, to_draw);
        }
    }

    pub fn resize(&mut self, size: WindowSize) {
        if size.is_zero_square() {
            return;
        }

        self.swapchain.resize(&self.device, size);
        self.gbuffer.resize(&self.device, size);
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct WindowSize(pub u32, pub u32);

impl WindowSize {
    pub fn is_zero_square(&self) -> bool {
        self.0 * self.1 == 0
    }
}

pub struct DrawData {
    pub indices: Range<u32>,
    pub base_vertex: i32,
    pub instances: Range<u32>,
}

pub type Transform = [[f32; 4]; 4];

pub type Draws<'a> = HashMap<u64, &'a [DrawData]>;
