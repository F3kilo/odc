use crate::res::Storage;
use crate::Resources;
use std::num::NonZeroU64;

pub struct BindGroups(Storage<String, BindGroup>);

pub struct BindGroup {
    pub handle: wgpu::BindGroup,
    pub layout: wgpu::BindGroupLayout,
    pub info: BindGroupInfo,
}

pub struct BindGroupInfo {
    pub name: String,
    pub uniforms: Vec<Binding<UniformBindingInfo>>,
    pub textures: Vec<Binding<TextureBindingInfo>>,
    pub samplers: Vec<Binding<SamplerBindingInfo>>,
}

impl BindGroupInfo {
    pub fn bindings_count(&self) -> usize {
        self.uniforms.len() + self.textures.len() + self.samplers.len()
    }
}

struct Binding<BindingInfo> {
    pub index: u32,
    pub id: String,
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
            view_dimension: wgpu::TextureViewDimension::D2,
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
    pub format: wgpu::TextureFormat,
}

pub struct SamplerBindingInfo {
    pub typ: wgpu::SamplerBindingType,
}

pub struct BindGroupFactory<'a> {
    device: &'a wgpu::Device,
    resources: &'a Resources<String>,
}

impl<'a> BindGroupFactory<'a> {
    pub fn new(device: &'a wgpu::Device, resources: &'a Resources<String>) -> Self {
        Self { device, resources }
    }

    pub fn create_bind_group(&self, name: String, info: BindGroupInfo) -> BindGroup {
        let layout = self.create_bind_group_layout(&info);

        let mut entries = Vec::with_capacity(info.bindings_count());
        entries.extend(info.uniforms.iter().map(|b| {
            let buffer = self.resources.buffers.get(&b.id);
            b.entry(&buffer.handle)
        }));

        let views: Vec<_> = info
            .textures
            .iter()
            .map(|b| self.resources.textures.get(&b.id).create_view())
            .collect();
        entries.extend(
            info.textures
                .iter()
                .zip(views.iter())
                .map(|(b, view)| b.entry(view)),
        );

        entries.extend(info.samplers.iter().map(|b| {
            let sampler = self.resources.samplers.get(&b.id);
            b.entry(&sampler.handle)
        }));

        let descriptor = wgpu::BindGroupDescriptor {
            label: Some(&name),
            layout: &layout,
            entries: &entries,
        };

        let handle = self.device.create_bind_group(&descriptor);
        BindGroup {
            handle,
            layout,
            info,
        }
    }

    fn create_bind_group_layout(&self, info: &BindGroupInfo) -> wgpu::BindGroupLayout {
        let mut entries = Vec::with_capacity(info.bindings_count());
        entries.extend(info.uniforms.iter().map(|b| b.layout_entry()));
        entries.extend(info.textures.iter().map(|b| b.layout_entry()));
        entries.extend(info.samplers.iter().map(|b| b.layout_entry()));

        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some(&info.name),
            entries: &entries,
        };
        self.device.create_bind_group_layout(&descriptor)
    }
}
