use crate::gdevice::GfxDevice;
use crate::mdl_parse::ModelParser;
use crate::pipelines::PipelinesFactory;
use crate::res::{BindGroupFactory, BindGroups, Buffers, ResourceFactory, Resources, TextureInfo};
pub use crate::window::WindowInfo;
use crate::window::WindowSource;
use bytemuck::Pod;
use pipelines::Pipelines;
use raw_window_handle::HasRawWindowHandle;
use std::collections::{HashMap, HashSet};
use std::mem;
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
    resources: Resources,
    bind_groups: BindGroups,
    pipelines: Pipelines,
    model: mdl::RenderModel,
    windows: HashMap<String, Window>,
    texture_windows: HashMap<usize, HashSet<String>>,
}

impl OdcCore {
    pub fn new(model: mdl::RenderModel) -> Self {
        let instance = Instance::new(Backends::all());
        let device = GfxDevice::new(&instance, None);
        let parser = ModelParser::new(&model);
        let resources = Self::create_resources(&device.device, &parser);
        let bind_groups = Self::create_bind_groups(&device.device, &parser, &resources);
        let pipelines = Self::create_pipelines(&device.device, &parser, &bind_groups);

        Self {
            instance,
            device,
            resources,
            bind_groups,
            pipelines,
            model,
            windows: Default::default(),
            texture_windows: Default::default(),
        }
    }

    pub fn with_window_support(model: mdl::RenderModel, window: &impl HasRawWindowHandle) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let device = GfxDevice::new(&instance, Some(&surface));
        let parser = ModelParser::new(&model);
        let resources = Self::create_resources(&device.device, &parser);
        let bind_groups = Self::create_bind_groups(&device.device, &parser, &resources);
        let pipelines = Self::create_pipelines(&device.device, &parser, &bind_groups);

        Self {
            instance,
            device,
            resources,
            bind_groups,
            pipelines,
            model,
            windows: Default::default(),
            texture_windows: Default::default(),
        }
    }

    /// # Safety
    /// Handle `window` MUST stay valid until `remove_window` call with same `source_texture_id`.
    pub unsafe fn add_window<Handle>(
        &mut self,
        source_index: usize,
        window_info: WindowInfo<Handle>,
    ) where
        Handle: HasRawWindowHandle,
    {
        let surface = self.instance.create_surface(&window_info.handle);
        let swapchain = Swapchain::new(surface, &self.device.adapter);
        swapchain.resize(&self.device.device, window_info.size);

        let source_texture = &self.resources.textures[source_index];
        let texture_view = source_texture.create_view();
        let format = source_texture.info.format;
        let source = WindowSource {
            texture_view,
            format,
        };

        let window = Window::new(&self.device.device, swapchain, source);

        self.windows.insert(window_info.name.to_string(), window);
        self.texture_windows
            .entry(source_index)
            .or_default()
            .insert(window_info.name.to_string());
    }

    pub fn remove_window(&mut self, name: &str) {
        self.windows.remove(name);
        for window_set in self.texture_windows.values_mut() {
            window_set.remove(name);
        }
    }

    pub fn resize_window(&mut self, window_name: &str, size: mdl::Size2d) {
        if size.is_zero() {
            return;
        }
        self.windows[window_name].resize(&self.device.device, size)
    }

    pub fn resize_attachments(&mut self, attachment: usize, size: mdl::Size2d) {
        if size.is_zero() {
            return;
        }

        let to_resize = self.model.connected_attachments(attachment);
        let factory = ResourceFactory::new(&self.device.device);

        let mut broken_bind_groups = HashSet::new();
        for texture_index in to_resize {
            broken_bind_groups.extend(self.model.bind_groups.iter().enumerate().filter_map(
                |(i, bg)| {
                    if bg.has_texture(texture_index) {
                        Some(i)
                    } else {
                        None
                    }
                },
            ));

            let size = wgpu::Extent3d {
                width: size.x as _,
                height: size.y as _,
                depth_or_array_layers: 1,
            };

            let info = TextureInfo {
                size,
                ..self.resources.textures[texture_index].info
            };
            self.resources.textures[texture_index] = factory.create_texture(info);
            if let Some(windows) = self.texture_windows.get(&texture_index) {
                for window in windows.iter() {
                    let source_view = self.resources.textures[texture_index].create_view();
                    let window = self.windows.get_mut(window).unwrap();
                    window.refresh_bind_group(&self.device.device, &source_view);
                }
            }
        }

        let factory = BindGroupFactory::new(&self.device.device, &self.resources);
        for bind_group_index in broken_bind_groups {
            factory.refresh_bind_group(&mut self.bind_groups.0[bind_group_index]);
        }
    }

    pub fn write_index(&self, data: &[u32], offset: u64) {
        let buffer = &self.resources.buffers.index.handle;
        self.write_buffer(buffer, data, offset)
    }

    pub fn write_vertex<T: Pod>(&self, data: &[T], offset: u64) {
        let buffer = &self.resources.buffers.vertex.handle;
        self.write_buffer(buffer, data, offset)
    }

    pub fn write_instance<T: Pod>(&self, data: &[T], offset: u64) {
        let buffer = &self.resources.buffers.instance.handle;
        self.write_buffer(buffer, data, offset)
    }

    pub fn write_uniform<T: Pod>(&self, data: &[T], offset: u64) {
        let buffer = &self.resources.buffers.uniform.handle;
        self.write_buffer(buffer, data, offset)
    }

    fn write_buffer<T: Pod>(&self, buffer: &wgpu::Buffer, data: &[T], offset: u64) {
        let data = bytemuck::cast_slice(data);
        let offset = mem::size_of::<T>() as u64 * offset;
        self.device.queue.write_buffer(buffer, offset, data);
    }

    pub fn draw(&self, data: &impl DrawDataSource, stages: &[Stage]) {
        let mut encoder = self
            .device
            .device
            .create_command_encoder(&Default::default());

        for stage in stages {
            for pass_index in stage.iter() {
                self.draw_pass(&mut encoder, data, *pass_index)
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

    fn draw_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        data: &impl DrawDataSource,
        pass_index: PassIndex,
    ) {
        let pass_info = &self.model.passes[pass_index];

        let color_views: Vec<_> = pass_info
            .color_attachments
            .iter()
            .map(|attachment| self.resources.textures[attachment.texture].create_view())
            .collect();

        let attachments_iter = color_views.iter().zip(pass_info.color_attachments.iter());
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

        let depth_view = pass_info
            .depth_attachment
            .as_ref()
            .map(|attachment| self.resources.textures[attachment.texture].create_view());
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
        let mut render_pass = encoder.begin_render_pass(&descriptor);

        let index_buffer = &self.resources.buffers.index.handle;
        let vertex_buffer = &self.resources.buffers.vertex.handle;
        let instance_buffer = &self.resources.buffers.instance.handle;
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, instance_buffer.slice(..));

        for pipeline in &pass_info.pipelines {
            self.draw_pipeline(&mut render_pass, pass_index, *pipeline, data);
        }
    }

    fn draw_pipeline<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        pass_index: usize,
        pipeline_index: usize,
        data: &impl DrawDataSource,
    ) {
        pass.set_pipeline(&self.pipelines.render[pipeline_index].handle);
        let pipeline_info = &self.model.pipelines[pipeline_index];

        for (i, bind_group) in pipeline_info.bind_groups.iter().enumerate() {
            let bind_groups = &self.bind_groups.0;
            pass.set_bind_group(i as _, &bind_groups[*bind_group].handle, &[]);
        }

        let draws = data.draw_data(pass_index, pipeline_index);
        for draw in draws.iter() {
            pass.draw_indexed(
                draw.indices.clone(),
                draw.base_vertex,
                draw.instances.clone(),
            );
        }
    }

    fn create_resources(device: &wgpu::Device, parser: &ModelParser) -> Resources {
        let factory = ResourceFactory::new(device);

        let index_info = parser.index_info();
        let index = factory.create_buffer(index_info);

        let vertex_info = parser.vertex_info();
        let vertex = factory.create_buffer(vertex_info);

        let instance_info = parser.instance_info();
        let instance = factory.create_buffer(instance_info);

        let uniform_info = parser.uniform_info();
        let uniform = factory.create_buffer(uniform_info);

        let buffers = Buffers {
            index,
            vertex,
            instance,
            uniform,
        };

        let textures = parser
            .textures_info()
            .map(|info| factory.create_texture(info))
            .collect();
        let samplers = parser
            .samplers_info()
            .map(|info| factory.create_sampler(info))
            .collect();

        Resources {
            buffers,
            textures,
            samplers,
        }
    }

    fn create_bind_groups(
        device: &wgpu::Device,
        parser: &ModelParser,
        resources: &Resources,
    ) -> BindGroups {
        let factory = BindGroupFactory::new(device, resources);
        let storage = parser
            .bind_groups_info()
            .map(|info| factory.create_bind_group(info))
            .collect();
        BindGroups::new(storage)
    }

    fn create_pipelines(
        device: &wgpu::Device,
        parser: &ModelParser,
        bind_groups: &BindGroups,
    ) -> Pipelines {
        let factory = PipelinesFactory::new(device, bind_groups);

        let render = parser
            .render_pipelines_info()
            .map(|info| factory.create_render_pipeline(info))
            .collect();

        Pipelines { render }
    }
}

#[repr(C)]
#[derive(Debug, Clone)]
pub struct DrawData {
    pub indices: Range<u32>,
    pub base_vertex: i32,
    pub instances: Range<u32>,
}

pub trait DrawDataSource {
    fn draw_data(&self, pass: usize, pipeline: usize) -> &[DrawData];
}

pub type Stage = Vec<PassIndex>;
pub type PassIndex = usize;
