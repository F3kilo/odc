use crate::gdevice::GfxDevice;
use crate::mdl_parse::ModelParser;
use crate::pipelines::PipelinesFactory;
use crate::res::{BindGroupFactory, BindGroups, Buffers, ResourceFactory, Resources, TextureInfo};
use bytemuck::Pod;
use pipelines::Pipelines;
use raw_window_handle::HasRawWindowHandle;
pub use res::BufferType;
use std::collections::{HashMap, HashSet};
use std::mem;
use std::num::NonZeroU32;
use std::ops::Range;
use swapchain::Swapchain;
use wgpu::{Backends, Instance};
use window::Window;
pub use window::WindowInfo;
use window::WindowSource;

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
        let texture_view = source_texture.create_view(None);
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
                    let texture = &self.resources.textures[texture_index];
                    let source_view = texture.create_view(None);
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

    pub fn write_buffer<T: Pod>(&self, typ: BufferType, data: &[T], offset: u64) {
        let buffer = self.resources.buffers.get(typ);
        let data = bytemuck::cast_slice(data);
        let offset = mem::size_of::<T>() as u64 * offset;
        self.device.queue.write_buffer(&buffer.handle, offset, data);
    }

    pub fn write_texture(&self, write: TextureWrite, data: TextureData) {
        let tex = &self.resources.textures[write.index];
        self.write_texture_inner(&tex.handle, write, data)
    }

    pub fn write_stock_texture(&self, name: &str, write: TextureWrite, data: TextureData) {
        let id = self.resources.stock.texture_id(name);
        if id != write.index {
            panic!("Try to write to texture with wrong id");
        }
        self.write_texture_inner(&tex.handle, write, data)
    }

    fn write_texture_inner(&self, texture: &wgpu::Texture, write: TextureWrite, data: TextureData) {
        let texture_copy = wgpu::ImageCopyTexture {
            texture,
            aspect: wgpu::TextureAspect::All,
            mip_level: write.mip_level,
            origin: write.offset,
        };

        let bytes_per_row = NonZeroU32::new(data.bytes_per_row);
        let rows_per_image = NonZeroU32::new(data.rows_per_layer);
        let layout = wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row,
            rows_per_image,
        };

        self.device
            .queue
            .write_texture(texture_copy, data.data, layout, write.size);
    }

    pub fn insert_stock_buffer(&mut self, typ: BufferType, name: String, size: Option<u64>) {
        self.resources
            .insert_stock_buffer(&self.device.device, typ, name, size);
    }

    pub fn swap_stock_buffer(&mut self, name: &str) {
        self.resources.swap_stock_buffer(name);
        if self.resources.stock.buffer_type(name) == BufferType::Uniform {
            let factory = BindGroupFactory::new(&self.device.device, &self.resources);
            for bind_group in self.model.uniform_bind_groups() {
                let bind_group = &mut self.bind_groups.0[bind_group];
                factory.refresh_bind_group(bind_group)
            }
        }
    }

    pub fn remove_stock_buffer(&mut self, name: &str) {
        self.resources.remove_stock_buffer(name);
    }

    pub fn insert_stock_texture(&mut self, id: usize, name: String, size: Option<mdl::Extent3d>) {
        if self.model.has_texture_attachment(id) && size.is_some() {
            panic!("Can't create stock texture, which used as attachment")
        }
        self.resources
            .insert_stock_texture(&self.device.device, id, name, size);
    }

    pub fn swap_stock_texture(&mut self, name: &str) {
        self.resources.swap_stock_texture(name);
        let id = self.resources.stock.texture_id(name);
        let factory = BindGroupFactory::new(&self.device.device, &self.resources);
        for bind_group_index in self.model.texture_bind_groups(id) {
            factory.refresh_bind_group(&mut self.bind_groups.0[bind_group_index]);
        }
    }

    pub fn remove_stock_texture(&mut self, name: &str) {
        self.resources.remove_stock_texture(name);
    }

    pub fn draw<'a, RenderIter>(&'a self, steps: RenderIter)
    where
        RenderIter: Iterator<Item = RenderStep<'a>>,
    {
        let mut encoder = self
            .device
            .device
            .create_command_encoder(&Default::default());

        for step in steps {
            self.draw_pass(&mut encoder, step)
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

    fn draw_pass(&self, encoder: &mut wgpu::CommandEncoder, step: RenderStep) {
        let color_views = self.pass_targets(step.pass);
        let color_attachments = self.pass_color_attachments(step.pass, color_views.iter());

        let depth_view = self.pass_depth_view(step.pass);
        let depth_attachment = depth_view
            .as_ref()
            .map(|view| self.pass_depth_attachment(view));

        let descriptor = wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &color_attachments,
            depth_stencil_attachment: depth_attachment,
        };
        let mut render_pass = encoder.begin_render_pass(&descriptor);
        self.resources.buffers.bind_buffers(&mut render_pass);

        self.draw_pipeline(&mut render_pass, step);
    }

    fn pass_targets(&self, pass: usize) -> Vec<PassTargets> {
        let pass_info = &self.model.passes[pass];

        pass_info
            .color_attachments
            .iter()
            .map(|attachment| {
                let texture = &self.resources.textures[attachment.texture];
                let color = texture.create_view(None);
                let resolve = attachment.resolve.map(|index| {
                    let texture = &self.resources.textures[index];
                    texture.create_view(None)
                });
                PassTargets { color, resolve }
            })
            .collect()
    }

    fn pass_color_attachments<'a, ViewsIter>(
        &self,
        pass: usize,
        views: ViewsIter,
    ) -> Vec<wgpu::RenderPassColorAttachment<'a>>
    where
        ViewsIter: Iterator<Item = &'a PassTargets>,
    {
        let pass_info = &self.model.passes[pass];

        let attachments_iter = views.zip(pass_info.color_attachments.iter());
        attachments_iter
            .map(|(target, info)| {
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
                    view: &target.color,
                    resolve_target: target.resolve.as_ref(),
                    ops: wgpu::Operations {
                        load,
                        store: info.store,
                    },
                }
            })
            .collect()
    }

    fn pass_depth_view(&self, pass: usize) -> Option<wgpu::TextureView> {
        let pass_info = &self.model.passes[pass];
        pass_info.depth_attachment.as_ref().map(|attachment| {
            let texture = &self.resources.textures[attachment.texture];
            texture.create_view(None)
        })
    }

    fn pass_depth_attachment<'a>(
        &self,
        view: &'a wgpu::TextureView,
    ) -> wgpu::RenderPassDepthStencilAttachment<'a> {
        wgpu::RenderPassDepthStencilAttachment {
            view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: true,
            }),
            stencil_ops: None,
        }
    }

    fn draw_pipeline<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, step: RenderStep) {
        pass.set_pipeline(&self.pipelines.render[step.pipeline].handle);
        let pipeline_info = &self.model.pipelines[step.pipeline];

        for (i, bind_group) in pipeline_info.bind_groups.iter().enumerate() {
            let bind_groups = &self.bind_groups.0;
            pass.set_bind_group(i as _, &bind_groups[*bind_group].handle, &[]);
        }

        for draw in step.data.iter() {
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

        let buffers = Buffers::new(index, vertex, instance, uniform);

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
            stock: Default::default(),
        }
    }

    // fn create_stock_resource(&mut self, resource_type: ResourceType, name: String) {
    //     let factory = ResourceFactory::new(&self.device.device);
    //
    //     match resource_type {
    //         ResourceType::IndexBuffer => {
    //             let info = self.resources.buffers.index.info.clone();
    //
    //         }
    //         ResourceType::VertexBuffer => {}
    //         ResourceType::InstanceBuffer => {}
    //         ResourceType::UniformBuffer => {}
    //         ResourceType::Texture(_) => {}
    //     }
    // }

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

#[derive(Debug, Copy, Clone)]
pub struct RenderStep<'a> {
    pub pass: usize,
    pub pipeline: usize,
    pub data: &'a [DrawData],
}

#[derive(Debug, Copy, Clone)]
pub struct TextureWrite {
    pub index: usize,
    pub mip_level: u32,
    pub offset: mdl::Origin3d,
    pub size: mdl::Extent3d,
}

#[derive(Debug, Copy, Clone)]
pub struct TextureData<'a> {
    pub data: &'a [u8],
    pub bytes_per_row: u32,
    pub rows_per_layer: u32,
}

struct PassTargets {
    pub color: wgpu::TextureView,
    pub resolve: Option<wgpu::TextureView>,
}
