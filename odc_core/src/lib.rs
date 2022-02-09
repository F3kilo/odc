use crate::gdevice::GfxDevice;
use crate::mdl_parse::ModelParser;
use crate::res::{BindGroupFactory, BindGroups, ResourceFactory, Resources};
pub use crate::window::WindowInfo;
use crate::window::WindowSource;
use bytemuck::Pod;
use pipelines::Pipelines;
use raw_window_handle::HasRawWindowHandle;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Range;
use swapchain::Swapchain;
use wgpu::{Backends, Instance};
use window::Window;

mod gdevice;
pub mod mdl;
mod mdl_parse;
mod pipelines;
mod res;
mod swapchain;
mod window;

pub struct OdcCore {
    instance: wgpu::Instance,
    device: GfxDevice,
    resources: Resources<String>,
    bind_groups: BindGroups,
    pipelines: Pipelines,
    model: mdl::RenderModel,
    windows: HashMap<String, Window>,
}

impl OdcCore {
    pub fn with_window_support(model: mdl::RenderModel, window: &impl HasRawWindowHandle) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let device = GfxDevice::new(&instance, Some(&surface));
        let parser = ModelParser::new(&model);
        let resources = Self::create_resources(&device.device, &parser);
        let bind_groups = Self::create_bind_groups();
        let pipelines = Pipelines::new(&device.device, &model, &bind_groups);

        Self {
            instance,
            device,
            resources,
            bind_groups,
            pipelines,
            model,
            windows: Default::default(),
        }
    }

    /// # Safety
    /// Handle `window` MUST stay valid until `remove_window` call with same `source_texture_id`.
    pub unsafe fn add_window<Handle>(
        &mut self,
        source_texture: &str,
        window_info: WindowInfo<Handle>,
    ) where
        Handle: HasRawWindowHandle,
    {
        let surface = self.instance.create_surface(&window_info.handle);
        let swapchain = Swapchain::new(surface, &self.device.adapter);
        swapchain.resize(&self.device.device, window_info.size);

        let texture_view = self.resources.texture_view(source_texture);
        let format = self.resources.texture_format(source_texture);
        let source = WindowSource {
            texture_view,
            format,
        };

        let window = Window::new(&self.device.device, swapchain, source, source_texture);

        self.windows.insert(window_info.name.to_string(), window);
    }

    pub fn remove_window(&mut self, source_texture_id: &str) {
        self.windows.remove(source_texture_id);
    }

    pub fn resize_window(&mut self, source_texture_id: &str, size: mdl::Size2d) {
        if size.is_zero() {
            return;
        }
        self.windows[source_texture_id].resize(&self.device.device, size)
    }

    pub fn resize_attachments(&mut self, attachment_id: &str, size: mdl::Size2d) {
        if size.is_zero() {
            return;
        }

        let to_resize = self.model.connected_attachments(attachment_id);
        for texture_id in to_resize {
            self.resources
                .resize_texture(&self.device.device, texture_id, size);

            if let Entry::Occupied(mut entry) = self.windows.entry(texture_id.into()) {
                let window = entry.get_mut();
                window.refresh_bind_group(&self.device.device, &self.resources, texture_id);
            }
        }
    }

    pub fn write_buffer<T: Pod>(&self, id: &str, data: &[T], offset: u64) {
        let data = bytemuck::cast_slice(data);
        self.resources
            .write_buffer(&self.device.queue, id, data, offset);
    }

    pub fn draw<DataRanges>(&self, data: &[DrawData], ranges: DataRanges)
    where
        DataRanges: Iterator<Item = Range<usize>>,
    {
        let mut data_per_pipeline = ranges.map(|r| &data[r]);

        let mut encoder = self
            .device
            .device
            .create_command_encoder(&Default::default());

        for pass_group in &self.model.stages.0 {
            for pass in &pass_group.0 {
                let pass_info = &self.model.passes[pass];
                self.draw_pass(pass_info, &mut encoder, &mut data_per_pipeline)
            }
        }

        let window_frames: Vec<_> = self
            .windows
            .values()
            .filter_map(|window| window.render(&mut encoder))
            .collect();

        self.device.queue.submit([encoder.finish()]);
        for frame in window_frames {
            frame.present();
        }
    }

    fn draw_pass<'a>(
        &self,
        info: &mdl::Pass,
        encoder: &mut wgpu::CommandEncoder,
        data: &mut impl Iterator<Item = &'a [DrawData]>,
    ) {
        let color_views = self.color_tagets_views(&info.color_attachments);
        let attachments_iter = color_views.iter().zip(info.color_attachments.iter());
        let color_attachments: Vec<_> = attachments_iter
            .map(|(view, info)| {
                let load = match info.clear {
                    Some(color) => wgpu::LoadOp::Clear(wgpu::Color {
                        r: color[0],
                        g: color[1],
                        b: color[2],
                        a: color[3],
                    }),
                    None => wgpu::LoadOp::Load,
                };

                wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load,
                        store: info.store,
                    },
                }
            })
            .collect();

        let depth_view = info
            .depth_attachment
            .as_ref()
            .map(|attachment| self.resources.texture_view(&attachment.texture));
        let depth_attachment =
            depth_view
                .as_ref()
                .map(|view| wgpu::RenderPassDepthStencilAttachment {
                    view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                });

        let descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: depth_attachment,
        };
        let mut pass = encoder.begin_render_pass(&descriptor);

        for pipeline in &info.pipelines {
            let data = data.next().unwrap();
            self.draw_pipeline(&mut pass, pipeline, data);
        }
    }

    fn draw_pipeline<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        pipeline: &str,
        draw_data: &[DrawData],
    ) {
        self.pipelines.bind(pass, pipeline);
        let pipeline_info = &self.model.pipelines[pipeline];

        self.resources
            .bind_index_buffer(pass, &pipeline_info.index_buffer);

        for (i, input_buffer) in pipeline_info.input_buffers.iter().enumerate() {
            self.resources
                .bind_input_buffer(pass, &input_buffer.buffer, i as _);
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

    fn color_tagets_views(&self, attachments: &[mdl::Attachment]) -> Vec<wgpu::TextureView> {
        attachments
            .iter()
            .map(|attachment| self.resources.texture_view(&attachment.texture))
            .collect()
    }

    fn create_resources(device: &wgpu::Device, parser: &ModelParser) -> Resources<String> {
        let factory = ResourceFactory::new(device);
        let buffers = parser
            .get_buffers()
            .map(|info| (info.name.clone(), factory.create_buffer(info)))
            .collect();

        Resources {
            buffers,
            textures: Default::default(),
            samplers: Default::default(),
        }
    }

    fn create_bind_groups(device: &wgpu::Device, parser: &ModelParser, resources: &Resources<String>) -> BindGroups {
        let factory = BindGroupFactory::new(device, resources);
        for bind_group in parser.bind_groups {

        }
    }
}

#[derive(Debug)]
pub struct DrawData {
    pub indices: Range<u32>,
    pub base_vertex: i32,
    pub instances: Range<u32>,
}
