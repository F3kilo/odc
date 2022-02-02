use crate::model as mdl;
use crate::res::Resources;
use std::collections::HashMap;
use core::num::NonZeroU64;


pub struct BindGroups {
    bind_groups: HashMap<String, BindGroup>
}

impl BindGroups {
    pub fn new(device: &wgpu::Device, model: &mdl::RenderModel, resources: &Resources) -> Self {
        let factory = HandlesFactory { device, model };

        let bind_groups = model
            .bind_groups
            .iter()
            .map(|(name, item)| {
                let bind_group = factory.create_bind_group(name, item, resources);
                (name.clone(), bind_group)
            })
            .collect();
            Self {bind_groups}
    }

    pub fn raw_layout(&self, name: &str) -> &wgpu::BindGroupLayout {
        &self.bind_groups[name].layout
    }

    pub fn bind<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, bind_group: &str, index: u32) {
        let bind_group = &self.bind_groups[bind_group].handle;
        pass.set_bind_group(index, bind_group, &[]);
    }
}

struct BindGroup {
    layout: wgpu::BindGroupLayout,
    handle: wgpu::BindGroup,
}

impl BindGroup {
    pub fn new(layout: wgpu::BindGroupLayout, handle: wgpu::BindGroup) -> Self {
        Self { layout, handle }
    }

    pub fn layout_entry_visibility(stages: mdl::ShaderStages) -> wgpu::ShaderStages {
        match stages {
            mdl::ShaderStages::Vertex => wgpu::ShaderStages::VERTEX,
            mdl::ShaderStages::Fragment => wgpu::ShaderStages::FRAGMENT,
            mdl::ShaderStages::Both => wgpu::ShaderStages::VERTEX_FRAGMENT,
        }
    }

    pub fn texture_sample_type(
        texture_type: mdl::TextureType,
        filterable: bool,
    ) -> wgpu::TextureSampleType {
        match texture_type {
            mdl::TextureType::Color { texel, .. } => match texel {
                mdl::TexelType::Float(_) | mdl::TexelType::Snorm(_) | mdl::TexelType::Unorm(_) => {
                    wgpu::TextureSampleType::Float { filterable }
                }
                mdl::TexelType::Sint(_) => wgpu::TextureSampleType::Sint,
                mdl::TexelType::Uint(_) => wgpu::TextureSampleType::Uint,
            },
            mdl::TextureType::Depth => wgpu::TextureSampleType::Depth,
        }
    }

    pub fn sampler_layout_entry(
        binding: &mdl::Binding<mdl::SamplerInfo>,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: binding.index,
            visibility: Self::layout_entry_visibility(binding.shader_stages),
            ty: wgpu::BindingType::Sampler(Self::sampler_binding_type(binding.info.sampler_type)),
            count: None,
        }
    }

    pub fn sampler_binding_type(sampler_type: mdl::SamplerType) -> wgpu::SamplerBindingType {
        match sampler_type {
            mdl::SamplerType::Filter => wgpu::SamplerBindingType::Filtering,
            mdl::SamplerType::NonFilter => wgpu::SamplerBindingType::NonFiltering,
            mdl::SamplerType::Depth => wgpu::SamplerBindingType::Comparison,
        }
    }

    pub fn uniform_entry<'a>(
        binding: &mdl::Binding<mdl::UniformInfo>,
        buffer: &'a wgpu::Buffer,
    ) -> wgpu::BindGroupEntry<'a> {
        let buffer_binding = wgpu::BufferBinding {
            buffer,
            offset: binding.info.offset,
            size: NonZeroU64::new(binding.info.size),
        };

        wgpu::BindGroupEntry {
            binding: binding.index,
            resource: wgpu::BindingResource::Buffer(buffer_binding),
        }
    }

    pub fn texture_entry<'a>(
        binding: &mdl::Binding<mdl::TextureInfo>,
        texture_view: &'a wgpu::TextureView,
    ) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: binding.index,
            resource: wgpu::BindingResource::TextureView(texture_view),
        }
    }

    pub fn sampler_entry<'a>(
        binding: &mdl::Binding<mdl::SamplerInfo>,
        sampler: &'a wgpu::Sampler,
    ) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: binding.index,
            resource: wgpu::BindingResource::Sampler(sampler),
        }
    }
}


struct HandlesFactory<'a> {
    device: &'a wgpu::Device,
    model: &'a mdl::RenderModel,
}

impl<'a> HandlesFactory<'a> {
    pub fn create_bind_group(
        &self,
        name: &str,
        info: &mdl::BindGroup,
        resources: &Resources,
    ) -> BindGroup {
        let layout = self.create_bind_group_layout(name, info);

        let views: HashMap<_, _> = info
            .textures
            .iter()
            .map(|binding| {
                let texture_view = resources.texture_view(&binding.info.texture);
                (&binding.info.texture, texture_view)
            })
            .collect();

        let mut entries = Vec::with_capacity(info.bindings_count());
        entries.extend(info.uniforms.iter().map(|binding| {
            let buffer = &resources.raw_buffer(&binding.info.buffer);
            BindGroup::uniform_entry(binding, buffer)
        }));
        entries.extend(
            info.textures
                .iter()
                .map(|binding| BindGroup::texture_entry(binding, &views[&binding.info.texture])),
        );
        entries.extend(info.samplers.iter().map(|binding| {
            let sampler = &resources.raw_sampler(&binding.info.sampler_type);
            BindGroup::sampler_entry(binding, sampler)
        }));

        let descriptor = wgpu::BindGroupDescriptor {
            label: Some(name),
            layout: &layout,
            entries: &entries,
        };

        let bind_group = self.device.create_bind_group(&descriptor);

        BindGroup::new(layout, bind_group)
    }

    pub fn create_bind_group_layout(
        &self,
        name: &str,
        info: &mdl::BindGroup,
    ) -> wgpu::BindGroupLayout {
        let mut entries = Vec::with_capacity(info.bindings_count());
        entries.extend(self.uniform_entries(&info.uniforms));
        entries.extend(self.texture_entries(&info.textures));
        entries.extend(info.samplers.iter().map(BindGroup::sampler_layout_entry));

        let descriptor = wgpu::BindGroupLayoutDescriptor {
            label: Some(name),
            entries: &entries,
        };
        self.device.create_bind_group_layout(&descriptor)
    }

    fn uniform_entries<'b>(
        &self,
        bindings: &'b [mdl::Binding<mdl::UniformInfo>],
    ) -> impl Iterator<Item = wgpu::BindGroupLayoutEntry> + 'b {
        bindings.iter().map(|binding| {
            let size = NonZeroU64::new(binding.info.size);
            let ty = wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: size,
            };

            let visibility = BindGroup::layout_entry_visibility(binding.shader_stages);

            wgpu::BindGroupLayoutEntry {
                binding: binding.index,
                visibility,
                ty,
                count: None,
            }
        })
    }

    fn texture_entries<'b>(
        &self,
        bindings: &'b [mdl::Binding<mdl::TextureInfo>],
    ) -> impl Iterator<Item = wgpu::BindGroupLayoutEntry> + 'b
    where
        'a: 'b,
    {
        bindings.iter().map(|binding| {
            let filterable = binding.info.filterable;
            let texture = self.model.textures[&binding.info.texture];
            let sample_type = BindGroup::texture_sample_type(texture.typ, filterable);

            let ty = wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            };

            let visibility = BindGroup::layout_entry_visibility(binding.shader_stages);

            wgpu::BindGroupLayoutEntry {
                binding: binding.index,
                visibility,
                ty,
                count: None,
            }
        })
    }
}