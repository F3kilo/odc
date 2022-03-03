use crate::{BufferType, Resources};
use std::num::NonZeroU64;
use wgpu::TextureView;

pub struct BindGroups(pub Vec<BindGroup>);

impl BindGroups {
    pub fn new(storage: Vec<BindGroup>) -> Self {
        Self(storage)
    }
}

pub struct BindGroup {
    pub handle: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
    pub info: BindGroupInfo,
}

pub struct BindGroupInfo {
    pub uniform: Option<Binding<UniformBindingInfo>>,
    pub textures: Vec<Binding<TextureBindingInfo>>,
    pub samplers: Vec<Binding<SamplerBindingInfo>>,
}

impl BindGroupInfo {
    pub fn bindings_count(&self) -> usize {
        self.textures.len()
            + self.samplers.len()
            + self.uniform.as_ref().map(|_| 1).unwrap_or_default()
    }
}

pub struct Binding<BindingInfo> {
    pub index: u32,
    pub visibility: wgpu::ShaderStages,
    pub info: BindingInfo,
}

impl Binding<UniformBindingInfo> {
    pub fn layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        let size = NonZeroU64::new(self.info.size);
        let ty = wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: size,
        };

        wgpu::BindGroupLayoutEntry {
            binding: self.index,
            visibility: self.visibility,
            ty,
            count: None,
        }
    }

    pub fn entry<'a>(&self, buffer: &'a wgpu::Buffer) -> wgpu::BindGroupEntry<'a> {
        let size = NonZeroU64::new(self.info.size);
        wgpu::BindGroupEntry {
            binding: self.index,
            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                size,
                offset: self.info.offset,
            }),
        }
    }
}

impl Binding<TextureBindingInfo> {
    pub fn layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        let sample_type = self.info.format.describe().sample_type;
        let ty = wgpu::BindingType::Texture {
            sample_type,
            view_dimension: self.info.dimension,
            multisampled: false,
        };

        wgpu::BindGroupLayoutEntry {
            binding: self.index,
            visibility: self.visibility,
            ty,
            count: None,
        }
    }

    pub fn entry<'a>(&self, view: &'a wgpu::TextureView) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: self.index,
            resource: wgpu::BindingResource::TextureView(view),
        }
    }
}

impl Binding<SamplerBindingInfo> {
    pub fn layout_entry(&self) -> wgpu::BindGroupLayoutEntry {
        let sampler_type = self.info.typ;
        let ty = wgpu::BindingType::Sampler(sampler_type);

        wgpu::BindGroupLayoutEntry {
            binding: self.index,
            visibility: self.visibility,
            ty,
            count: None,
        }
    }

    pub fn entry<'a>(&self, sampler: &'a wgpu::Sampler) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: self.index,
            resource: wgpu::BindingResource::Sampler(sampler),
        }
    }
}

pub struct UniformBindingInfo {
    pub size: u64,
    pub offset: u64,
}

pub struct TextureBindingInfo {
    pub texture_index: usize,
    pub format: wgpu::TextureFormat,
    pub dimension: wgpu::TextureViewDimension,
}

pub struct SamplerBindingInfo {
    pub sampler_index: usize,
    pub typ: wgpu::SamplerBindingType,
}

pub struct BindGroupFactory<'a> {
    device: &'a wgpu::Device,
    resources: &'a Resources,
}

impl<'a> BindGroupFactory<'a> {
    pub fn new(device: &'a wgpu::Device, resources: &'a Resources) -> Self {
        Self { device, resources }
    }

    pub fn refresh_bind_group(&self, bind_group: &mut BindGroup) {
        let info = &bind_group.info;

        let views = self.texture_views(&info.textures);
        let entries = self.collect_entries(info, views.iter());
        let handle = self.create_raw_handle(&bind_group.layout, &entries);

        bind_group.handle = handle;
    }

    pub fn create_bind_group(&self, info: BindGroupInfo) -> BindGroup {
        let layout = self.create_bind_group_layout(&info);

        let views = self.texture_views(&info.textures);
        let entries = self.collect_entries(&info, views.iter());
        let handle = self.create_raw_handle(&layout, &entries);

        BindGroup {
            handle,
            layout,
            info,
        }
    }

    fn create_raw_handle(
        &self,
        layout: &wgpu::BindGroupLayout,
        entries: &[wgpu::BindGroupEntry],
    ) -> wgpu::BindGroup {
        let descriptor = wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries,
        };

        self.device.create_bind_group(&descriptor)
    }

    fn collect_entries<'b, ViewsIter>(
        &self,
        info: &BindGroupInfo,
        views: ViewsIter,
    ) -> Vec<wgpu::BindGroupEntry<'b>>
    where
        'a: 'b,
        ViewsIter: Iterator<Item = &'b TextureView>,
    {
        let mut entries = Vec::with_capacity(info.bindings_count());
        entries.extend(info.uniform.iter().map(|b| {
            let buffer = self.resources.buffers.get(BufferType::Uniform);
            b.entry(&buffer.handle)
        }));

        entries.extend(
            info.textures
                .iter()
                .zip(views)
                .map(|(b, view)| b.entry(view)),
        );

        entries.extend(info.samplers.iter().map(|b| {
            let sampler = &self.resources.samplers[b.info.sampler_index];
            b.entry(&sampler.handle)
        }));
        entries
    }

    fn texture_views(&self, textures: &[Binding<TextureBindingInfo>]) -> Vec<wgpu::TextureView> {
        textures
            .iter()
            .map(|b| {
                let index = b.info.texture_index;
                let texture = &self.resources.textures[index];
                let dimension = Some(b.info.dimension);
                texture.create_view(dimension)
            })
            .collect()
    }

    fn create_bind_group_layout(&self, info: &BindGroupInfo) -> wgpu::BindGroupLayout {
        let mut entries = Vec::with_capacity(info.bindings_count());
        entries.extend(info.uniform.iter().map(|b| b.layout_entry()));
        entries.extend(info.textures.iter().map(|b| b.layout_entry()));
        entries.extend(info.samplers.iter().map(|b| b.layout_entry()));

        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &entries,
        };
        self.device.create_bind_group_layout(&descriptor)
    }
}
