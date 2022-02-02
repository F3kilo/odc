use bytemuck::Pod;
use crate::gdevice::GfxDevice;
use bind::BindGroups;
use model as mdl;
use model::Size2d;
use pipelines::Pipelines;
use raw_window_handle::HasRawWindowHandle;
use res::Resources;
use std::ops::Range;
use swapchain::Swapchain;
use wgpu::{Backends, Instance};

mod bind;
mod gdevice;
pub mod model;
mod pipelines;
mod res;
mod swapchain;

pub struct OdcCore {
    device: GfxDevice,
    swapchain: Swapchain,
    resources: Resources,
    bind_groups: BindGroups,
    pipelines: Pipelines,
    model: mdl::RenderModel,
}

impl OdcCore {
    pub fn with_window_support(model: mdl::RenderModel, window: &WindowInfo) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window.handle) };
        let device = GfxDevice::new(&instance, Some(&surface));
        let swapchain = Swapchain::new(&device, surface);
        let resources = Resources::new(&device.device, &model);
        let bind_groups = BindGroups::new(&device.device, &model, &resources);
        let pipelines = Pipelines::new(&device.device, &model, &bind_groups, swapchain.format);

        swapchain.resize(&device, window.size);

        Self {
            device,
            swapchain,
            resources,
            bind_groups,
            pipelines,
            model,
        }
    }

    pub fn write_buffer<T: Pod>(&self, id: &str, data: &[T], offset: u64) {
        let data = bytemuck::cast_slice(data);
        self.resources.write_buffer(&self.device.queue, id, data, offset);
    }

    pub fn draw(&self, data: &[DrawData], ranges: &[Range<usize>]) {
        let mut data_per_pipeline = ranges.iter().cloned().map(|r| &data[r]);

        let frame = match self.swapchain.surface.get_current_texture() {
            Ok(f) => f,
            Err(wgpu::SurfaceError::Outdated) => return,
            e => e.unwrap(),
        };

        let mut encoder = self
            .device
            .device
            .create_command_encoder(&Default::default());

        for pass_group in &self.model.stages.0 {
            for pass in &pass_group.0 {
                let pass_info = &self.model.passes[pass];
                self.draw_pass(
                    pass_info,
                    &mut encoder,
                    &mut data_per_pipeline,
                    &frame.texture,
                )
            }
        }

        self.device.queue.submit([encoder.finish()]);
        frame.present();
    }

    fn draw_pass<'a>(
        &self,
        info: &mdl::Pass,
        encoder: &mut wgpu::CommandEncoder,
        data: &mut impl Iterator<Item = &'a [DrawData]>,
        window_texture: &wgpu::Texture,
    ) {
        let color_views = self.color_tagets_views(&info.color_attachments, window_texture);
        let attachments_iter = color_views.iter().zip(info.color_attachments.iter());
        let color_attachments: Vec<_> = attachments_iter
            .map(|(view, _info)| wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN), // todo: from info
                    store: true,
                },
            })
            .collect();
        // todo: depth attachment
        let descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: None,
        };
        let mut pass = encoder.begin_render_pass(&descriptor);

        for pipeline in &info.pipelines {
            let data = data.next().unwrap();
            self.draw_pipeline(&mut pass, pipeline, data);
        }
    }

    fn draw_pipeline<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, pipeline: &str, draw_data: &[DrawData]) {
        self.pipelines.bind(pass, pipeline);
        let pipeline_info = &self.model.pipelines[pipeline];

        self.resources
            .bind_index_buffer(pass, &pipeline_info.index_buffer);

        for (i, input_buffer) in pipeline_info.input_buffers.iter().enumerate() {
            self.resources.bind_input_buffer(pass, &input_buffer.buffer, i as _);
        }

        for (i, bind_group) in pipeline_info.bind_groups.iter().enumerate() {
            self.bind_groups.bind(pass, bind_group, i as _);
        }

        for draw in draw_data {
            pass.draw_indexed(
                draw.indices.clone(),
                draw.base_vertex,
                draw.instances.clone(),
            );
        }
    }

    fn color_tagets_views(
        &self,
        attachments: &[mdl::Attachment],
        window_texture: &wgpu::Texture,
    ) -> Vec<wgpu::TextureView> {
        attachments
            .iter()
            .map(|attachment| match &attachment.target {
                mdl::AttachmentTarget::Texture(texture) => self.resources.texture_view(texture),
                mdl::AttachmentTarget::Window => window_texture.create_view(&Default::default()),
            })
            .collect()
    }
}

pub struct WindowInfo<'a> {
    pub handle: &'a dyn HasRawWindowHandle,
    pub size: Size2d,
}

#[derive(Debug)]
pub struct DrawData {
    pub indices: Range<u32>,
    pub base_vertex: i32,
    pub instances: Range<u32>,
}
