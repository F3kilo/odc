use crate::structure as st;
use core::num::NonZeroU64;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;

pub struct RenderData {
    resources: Resources,
    bind_groups: HashMap<String, BindGroup>,
    render_pipelines: HashMap<String, RenderPipeline>,
}

impl RenderData {
    pub fn from_structure(device: &wgpu::Device, render: &st::Render) -> Self {
        let factory = HandlesFactory { device, render };

        let buffers = render
            .buffers
            .iter()
            .map(|(name, item)| (name.clone(), factory.create_buffer(name, item)))
            .collect();

        let textures = render
            .textures
            .iter()
            .map(|(name, item)| (name.clone(), factory.create_texture(name, item)))
            .collect();

        let samplers = factory.create_samplers();

        let resources = Resources {
            buffers,
            textures,
            samplers,
        };

        let bind_groups = render
            .bind_groups
            .iter()
            .map(|(name, item)| {
                let bind_group = factory.create_bind_group(name, item, &resources);
                (name.clone(), bind_group)
            })
            .collect();

        let render_pipelines = render
            .pipelines
            .iter()
            .map(|(name, item)| {
                let pipeline = factory.create_pipeline(name, item, &bind_groups);
                (name.clone(), pipeline)
            })
            .collect();

        Self {
            resources,
            bind_groups,
            render_pipelines,
        }
    }
}

struct HandlesFactory<'a> {
    device: &'a wgpu::Device,
    render: &'a st::Render,
}

impl<'a> HandlesFactory<'a> {
    pub fn create_buffer(&self, name: &str, info: &st::Buffer) -> Buffer {
        let usage = Buffer::find_usages(name, self.render);
        let descriptor = wgpu::BufferDescriptor {
            label: Some(name),
            size: info.size,
            usage,
            mapped_at_creation: false,
        };
        Buffer::new(self.device.create_buffer(&descriptor))
    }

    pub fn create_texture(&self, name: &str, info: &st::Texture) -> Texture {
        let usage = Texture::find_usages(name, self.render);

        let size = wgpu::Extent3d {
            width: info.size.x as _,
            height: info.size.y as _,
            depth_or_array_layers: 1,
        };

        let descriptor = wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Texture::find_format(info.typ),
            usage,
        };
        Texture::new(self.device.create_texture(&descriptor))
    }

    pub fn create_samplers(&self) -> HashMap<st::SamplerType, Sampler> {
        let sampler_types = [
            st::SamplerType::Filter,
            st::SamplerType::NonFilter,
            st::SamplerType::Depth,
        ];
        sampler_types
            .iter()
            .filter(|typ| self.render.has_sampler(**typ))
            .map(|typ| {
                let filter_mode = Sampler::filter_mode(*typ);
                let compare = Sampler::compare(*typ);
                let descriptor = wgpu::SamplerDescriptor {
                    mag_filter: filter_mode,
                    min_filter: filter_mode,
                    mipmap_filter: filter_mode,
                    compare,
                    ..Default::default()
                };
                let sampler = self.device.create_sampler(&descriptor);
                (*typ, Sampler::new(sampler))
            })
            .collect()
    }

    pub fn create_bind_group(
        &self,
        name: &str,
        info: &st::BindGroup,
        resources: &Resources,
    ) -> BindGroup {
        let layout = self.create_bind_group_layout(name, info);

        let views: HashMap<_, _> = info
            .textures
            .iter()
            .map(|binding| {
                let texture = &resources.textures[&binding.info.texture];
                (&binding.info.texture, texture.create_view())
            })
            .collect();

        let mut entries = Vec::with_capacity(info.bindings_count());
        entries.extend(info.uniforms.iter().map(|binding| {
            let buffer = &resources.buffers[&binding.info.buffer];
            BindGroup::uniform_entry(binding, buffer)
        }));
        entries.extend(
            info.textures
                .iter()
                .map(|binding| BindGroup::texture_entry(binding, &views[&binding.info.texture])),
        );
        entries.extend(info.samplers.iter().map(|binding| {
            let sampler = &resources.samplers[&binding.info.sampler_type];
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
        info: &st::BindGroup,
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

    pub fn create_pipeline(
        &self,
        name: &str,
        info: &st::RenderPipeline,
        bind_groups: &HashMap<String, BindGroup>,
    ) -> RenderPipeline {
        let shader_data = fs::read_to_string(&info.shader.path).expect("shader file not found");
        let shader_src = Cow::Owned(shader_data);
        let source = wgpu::ShaderSource::Wgsl(shader_src);
        let descriptor = wgpu::ShaderModuleDescriptor {
            label: None,
            source,
        };
        let shader_module = self.device.create_shader_module(&descriptor);

        let input_buffers_data = RenderPipeline::input_buffers_data(&info.input_buffers);
        let input_buffers = RenderPipeline::wgpu_vertex_buffer_layouts(&input_buffers_data);

        let vertex = wgpu::VertexState {
            module: &shader_module,
            entry_point: &info.shader.vs_main,
            buffers: &input_buffers,
        };

        let layout = self.create_pipeline_layout(name, bind_groups);

        let primitive = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Front),
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        };

        let depth_stencil = info.depth.as_ref().map(|_| wgpu::DepthStencilState {
            format: Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let color_targets = self.pipeline_targets(name);

        let fragment = Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: &info.shader.fs_main,
            targets: &color_targets,
        });

        let descriptor = wgpu::RenderPipelineDescriptor {
            label: Some(name),
            layout: Some(&layout),
            vertex,
            primitive,
            depth_stencil,
            multisample: Default::default(),
            fragment,
            multiview: None,
        };

        RenderPipeline::new(self.device.create_render_pipeline(&descriptor))
    }

    pub fn create_pipeline_layout(
        &self,
        name: &str,
        bind_groups: &HashMap<String, BindGroup>,
    ) -> wgpu::PipelineLayout {
        let layouts: Vec<_> = bind_groups.values().map(|bg| &bg.layout).collect();

        let descriptor = wgpu::PipelineLayoutDescriptor {
            label: Some(name),
            bind_group_layouts: &layouts,
            push_constant_ranges: &[],
        };

        self.device.create_pipeline_layout(&descriptor)
    }

    fn pipeline_targets(&self, pipeline: &str) -> Vec<wgpu::ColorTargetState> {
        for pass in self.render.passes.values() {
            if pass.pipelines.iter().any(|p| p == pipeline) {
                return pass
                    .attachments
                    .iter()
                    .map(|attachment| {
                        let texture = &self.render.textures[&attachment.texture];
                        wgpu::ColorTargetState {
                            format: Texture::find_format(texture.typ),
                            blend: None,
                            write_mask: Default::default(),
                        }
                    })
                    .collect();
            }
        }
        panic!("pipeline {} is not used in any pass", pipeline);
    }

    fn uniform_entries<'b>(
        &self,
        bindings: &'b [st::Binding<st::UniformInfo>],
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
        bindings: &'b [st::Binding<st::TextureInfo>],
    ) -> impl Iterator<Item = wgpu::BindGroupLayoutEntry> + 'b
    where
        'a: 'b,
    {
        bindings.iter().map(|binding| {
            let filterable = binding.info.filterable;
            let texture = self.render.textures[&binding.info.texture];
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

struct Buffer(wgpu::Buffer);

impl Buffer {
    pub fn new(handle: wgpu::Buffer) -> Self {
        Self(handle)
    }

    pub fn find_usages(name: &str, render: &st::Render) -> wgpu::BufferUsages {
        let is_uniform = render.has_uniform_binding(name);
        // let is_storage = render.has_buffer_binding(name, st::BufferType::Storage);
        let is_vertex = render.has_input_buffer(name);
        let is_index = render.has_index_buffer(name);

        let mut usages = wgpu::BufferUsages::COPY_DST;
        if is_uniform {
            usages |= wgpu::BufferUsages::UNIFORM;
        }
        if is_vertex {
            usages |= wgpu::BufferUsages::VERTEX;
        }
        if is_index {
            usages |= wgpu::BufferUsages::INDEX;
        }
        usages
    }
}

struct Texture(wgpu::Texture);

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(handle: wgpu::Texture) -> Self {
        Self(handle)
    }

    pub fn find_usages(name: &str, render: &st::Render) -> wgpu::TextureUsages {
        let is_attachment = render.has_texture_attachment(name);
        let is_binding = render.has_texture_binding(name);

        let mut usages = wgpu::TextureUsages::empty();
        if is_attachment {
            usages |= wgpu::TextureUsages::RENDER_ATTACHMENT;
        }
        if is_binding {
            usages |= wgpu::TextureUsages::TEXTURE_BINDING;
        }
        usages
    }

    pub fn find_format(typ: st::TextureType) -> wgpu::TextureFormat {
        match typ {
            st::TextureType::Color { texel, texel_count } => {
                Self::find_color_format(texel, texel_count)
            }
            st::TextureType::Depth => wgpu::TextureFormat::Depth32Float,
        }
    }

    pub fn find_color_format(
        texel: st::TexelType,
        texel_count: st::TexelCount,
    ) -> wgpu::TextureFormat {
        use st::BytesPerFloatTexel as FloatBytes;
        use st::BytesPerIntTexel as IntBytes;
        use st::BytesPerNormTexel as NormBytes;
        use st::TexelCount as Count;
        use st::TexelType as Texel;
        use wgpu::TextureFormat as Format;

        match (texel, texel_count) {
            // 16-bit float
            (Texel::Float(FloatBytes::Two), Count::One) => Format::R16Float,
            (Texel::Float(FloatBytes::Two), Count::Two) => Format::Rg16Float,
            (Texel::Float(FloatBytes::Two), Count::Four) => Format::Rgba16Float,

            // 32-bit float
            (Texel::Float(FloatBytes::Four), Count::One) => Format::R32Float,
            (Texel::Float(FloatBytes::Four), Count::Two) => Format::Rg32Float,
            (Texel::Float(FloatBytes::Four), Count::Four) => Format::Rgba32Float,

            // 8-bit int
            (Texel::Sint(IntBytes::One), Count::One) => Format::R8Sint,
            (Texel::Sint(IntBytes::One), Count::Two) => Format::Rg8Sint,
            (Texel::Sint(IntBytes::One), Count::Four) => Format::Rgba8Sint,

            // 16-bit int
            (Texel::Sint(IntBytes::Two), Count::One) => Format::R16Sint,
            (Texel::Sint(IntBytes::Two), Count::Two) => Format::Rg16Sint,
            (Texel::Sint(IntBytes::Two), Count::Four) => Format::Rgba16Sint,

            // 32-bit int
            (Texel::Sint(IntBytes::Four), Count::One) => Format::R32Sint,
            (Texel::Sint(IntBytes::Four), Count::Two) => Format::Rg32Sint,
            (Texel::Sint(IntBytes::Four), Count::Four) => Format::Rgba32Sint,

            // 8-bit uint
            (Texel::Uint(IntBytes::One), Count::One) => Format::R8Uint,
            (Texel::Uint(IntBytes::One), Count::Two) => Format::Rg8Uint,
            (Texel::Uint(IntBytes::One), Count::Four) => Format::Rgba8Uint,

            // 16-bit uint
            (Texel::Uint(IntBytes::Two), Count::One) => Format::R16Uint,
            (Texel::Uint(IntBytes::Two), Count::Two) => Format::Rg16Uint,
            (Texel::Uint(IntBytes::Two), Count::Four) => Format::Rgba16Uint,

            // 32-bit uint
            (Texel::Uint(IntBytes::Four), Count::One) => Format::R32Uint,
            (Texel::Uint(IntBytes::Four), Count::Two) => Format::Rg32Uint,
            (Texel::Uint(IntBytes::Four), Count::Four) => Format::Rgba32Uint,

            // 8-bit snorm
            (Texel::Snorm(NormBytes::One), Count::One) => Format::R8Snorm,
            (Texel::Snorm(NormBytes::One), Count::Two) => Format::Rg8Snorm,
            (Texel::Snorm(NormBytes::One), Count::Four) => Format::Rgba8Snorm,

            // 16-bit snorm
            (Texel::Snorm(NormBytes::Two), Count::One) => Format::R16Snorm,
            (Texel::Snorm(NormBytes::Two), Count::Two) => Format::Rg16Snorm,
            (Texel::Snorm(NormBytes::Two), Count::Four) => Format::Rgba16Snorm,

            // 8-bit unorm
            (Texel::Unorm(NormBytes::One), Count::One) => Format::R8Unorm,
            (Texel::Unorm(NormBytes::One), Count::Two) => Format::Rg8Unorm,
            (Texel::Unorm(NormBytes::One), Count::Four) => Format::Rgba8Unorm,

            // 16-bit unorm
            (Texel::Unorm(NormBytes::Two), Count::One) => Format::R16Unorm,
            (Texel::Unorm(NormBytes::Two), Count::Two) => Format::Rg16Unorm,
            (Texel::Unorm(NormBytes::Two), Count::Four) => Format::Rgba16Unorm,
        }
    }

    pub fn create_view(&self) -> wgpu::TextureView {
        self.0.create_view(&Default::default())
    }
}

struct Sampler(wgpu::Sampler);

impl Sampler {
    pub fn new(handle: wgpu::Sampler) -> Self {
        Self(handle)
    }

    pub fn filter_mode(sampler_type: st::SamplerType) -> wgpu::FilterMode {
        match sampler_type {
            st::SamplerType::Filter => wgpu::FilterMode::Linear,
            st::SamplerType::NonFilter => wgpu::FilterMode::Nearest,
            st::SamplerType::Depth => wgpu::FilterMode::Nearest,
        }
    }

    pub fn compare(sampler_type: st::SamplerType) -> Option<wgpu::CompareFunction> {
        match sampler_type {
            st::SamplerType::Filter | st::SamplerType::NonFilter => None,
            st::SamplerType::Depth => Some(wgpu::CompareFunction::Less),
        }
    }

    pub fn binding_type(sampler_type: st::SamplerType) -> wgpu::SamplerBindingType {
        match sampler_type {
            st::SamplerType::Filter => wgpu::SamplerBindingType::Filtering,
            st::SamplerType::NonFilter => wgpu::SamplerBindingType::NonFiltering,
            st::SamplerType::Depth => wgpu::SamplerBindingType::Comparison,
        }
    }
}

struct Resources {
    pub buffers: HashMap<String, Buffer>,
    pub textures: HashMap<String, Texture>,
    pub samplers: HashMap<st::SamplerType, Sampler>,
}

struct BindGroup {
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl BindGroup {
    pub fn new(layout: wgpu::BindGroupLayout, bind_group: wgpu::BindGroup) -> Self {
        Self { layout, bind_group }
    }

    pub fn layout_entry_visibility(stages: st::ShaderStages) -> wgpu::ShaderStages {
        match stages {
            st::ShaderStages::Vertex => wgpu::ShaderStages::VERTEX,
            st::ShaderStages::Fragment => wgpu::ShaderStages::FRAGMENT,
            st::ShaderStages::Both => wgpu::ShaderStages::VERTEX_FRAGMENT,
        }
    }

    pub fn texture_sample_type(
        texture_type: st::TextureType,
        filterable: bool,
    ) -> wgpu::TextureSampleType {
        match texture_type {
            st::TextureType::Color { texel, .. } => match texel {
                st::TexelType::Float(_) | st::TexelType::Snorm(_) | st::TexelType::Unorm(_) => {
                    wgpu::TextureSampleType::Float { filterable }
                }
                st::TexelType::Sint(_) => wgpu::TextureSampleType::Sint,
                st::TexelType::Uint(_) => wgpu::TextureSampleType::Uint,
            },
            st::TextureType::Depth => wgpu::TextureSampleType::Depth,
        }
    }

    pub fn sampler_layout_entry(
        binding: &st::Binding<st::SamplerInfo>,
    ) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: binding.index,
            visibility: Self::layout_entry_visibility(binding.shader_stages),
            ty: wgpu::BindingType::Sampler(Sampler::binding_type(binding.info.sampler_type)),
            count: None,
        }
    }

    pub fn uniform_entry<'a>(
        binding: &st::Binding<st::UniformInfo>,
        buffer: &'a Buffer,
    ) -> wgpu::BindGroupEntry<'a> {
        let buffer_binding = wgpu::BufferBinding {
            buffer: &buffer.0,
            offset: binding.info.offset,
            size: NonZeroU64::new(binding.info.size),
        };

        wgpu::BindGroupEntry {
            binding: binding.index,
            resource: wgpu::BindingResource::Buffer(buffer_binding),
        }
    }

    pub fn texture_entry<'a>(
        binding: &st::Binding<st::TextureInfo>,
        texture_view: &'a wgpu::TextureView,
    ) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: binding.index,
            resource: wgpu::BindingResource::TextureView(texture_view),
        }
    }

    pub fn sampler_entry<'a>(
        binding: &st::Binding<st::SamplerInfo>,
        sampler: &'a Sampler,
    ) -> wgpu::BindGroupEntry<'a> {
        wgpu::BindGroupEntry {
            binding: binding.index,
            resource: wgpu::BindingResource::Sampler(&sampler.0),
        }
    }

    // pub fn layout_entry(
    //     index: u32,
    //     stage: st::ShaderStage,
    //     binding_info: &st::BindingInfo,
    // ) -> wgpu::BindGroupLayoutEntry {
    //     let binding_type = match binding_info {
    //         st::BindingInfo::Buffer(buffer_type) => Self::buffer_binding_type(*buffer_type),
    //         st::BindingInfo::Texture(texture_type, filterable) => {
    //             Self::texture_binding_type(*texture_type, *filterable)
    //         }
    //         st::BindingInfo::Sampler(sampler_type) => Self::sampler_binding_type(*sampler_type),
    //     };

    //     let visibility = match stage {
    //         st::ShaderStage::Vertex => wgpu::ShaderStages::VERTEX,
    //         st::ShaderStage::Fragment => wgpu::ShaderStages::FRAGMENT,
    //         st::ShaderStage::Both => wgpu::ShaderStages::VERTEX_FRAGMENT,
    //     };

    //     wgpu::BindGroupLayoutEntry {
    //         binding: index,
    //         visibility,
    //         ty: binding_type,
    //         count: None,
    //     }
    // }

    // fn buffer_binding_type(buffer_type: st::BufferType) -> wgpu::BindingType {
    //     let ty = match buffer_type {
    //         st::BufferType::Uniform => wgpu::BufferBindingType::Uniform,
    //         st::BufferType::Storage => wgpu::BufferBindingType::Storage { read_only: true },
    //     };
    //     wgpu::BindingType::Buffer {
    //         ty,
    //         has_dynamic_offset: false,
    //         min_binding_size: None,
    //     }
    // }

    // fn texture_binding_type(texture_type: st::TextureType, filterable: bool) -> wgpu::BindingType {
    //     let sample_type = match texture_type {
    //         st::TextureType::Color { texel, .. } => match texel {
    //             st::TexelType::Float(_) | st::TexelType::Snorm(_) | st::TexelType::Unorm(_) => {
    //                 wgpu::TextureSampleType::Float { filterable }
    //             }
    //             st::TexelType::Sint(_) => wgpu::TextureSampleType::Sint,
    //             st::TexelType::Uint(_) => wgpu::TextureSampleType::Uint,
    //         },
    //         st::TextureType::Depth => wgpu::TextureSampleType::Depth,
    //     };
    //     wgpu::BindingType::Texture {
    //         sample_type,
    //         view_dimension: wgpu::TextureViewDimension::D2,
    //         multisampled: false,
    //     }
    // }

    // fn sampler_binding_type(sampler_type: st::SamplerType) -> wgpu::BindingType {
    //     match sampler_type {
    //         st::SamplerType::Color(true) => {
    //             wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering)
    //         }
    //         st::SamplerType::Color(false) => {
    //             wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering)
    //         }
    //         st::SamplerType::Color(true) => {
    //             wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison)
    //         }
    //     }
    // }
}

struct RenderPipeline(wgpu::RenderPipeline);

impl RenderPipeline {
    pub fn new(pipeline: wgpu::RenderPipeline) -> Self {
        Self(pipeline)
    }

    pub fn input_buffers_data(input_buffers: &[st::InputBuffer]) -> Vec<InputBufferLayout> {
        input_buffers.iter().map(InputBufferLayout::new).collect()
    }

    pub fn wgpu_vertex_buffer_layouts(data: &[InputBufferLayout]) -> Vec<wgpu::VertexBufferLayout> {
        data.iter()
            .map(|layout| wgpu::VertexBufferLayout {
                array_stride: layout.stride,
                step_mode: layout.step_mode,
                attributes: &layout.attributes,
            })
            .collect()
    }
}

struct InputBufferLayout {
    pub stride: u64,
    pub step_mode: wgpu::VertexStepMode,
    pub attributes: Vec<wgpu::VertexAttribute>,
}

impl InputBufferLayout {
    pub fn new(input_buffer: &st::InputBuffer) -> Self {
        let step_mode = match input_buffer.input_type {
            st::InputType::PerVertex => wgpu::VertexStepMode::Vertex,
            st::InputType::PerInstance => wgpu::VertexStepMode::Instance,
        };

        let attributes = input_buffer
            .attributes
            .iter()
            .map(Self::wgpu_input_attributes)
            .collect();

        Self {
            stride: input_buffer.stride,
            step_mode,
            attributes,
        }
    }

    fn wgpu_input_attributes(attribute: &st::InputAttribute) -> wgpu::VertexAttribute {
        let format = match attribute.item {
            st::InputItem::Float16x2 => wgpu::VertexFormat::Float16x2,
            st::InputItem::Float16x4 => wgpu::VertexFormat::Float16x4,
            st::InputItem::Float32 => wgpu::VertexFormat::Float32,
            st::InputItem::Float32x2 => wgpu::VertexFormat::Float32x2,
            st::InputItem::Float32x3 => wgpu::VertexFormat::Float32x3,
            st::InputItem::Float32x4 => wgpu::VertexFormat::Float32x4,
            st::InputItem::Sint16x2 => wgpu::VertexFormat::Sint16x2,
            st::InputItem::Sint16x4 => wgpu::VertexFormat::Sint16x4,
            st::InputItem::Sint32 => wgpu::VertexFormat::Sint32,
            st::InputItem::Sint32x2 => wgpu::VertexFormat::Sint32x2,
            st::InputItem::Sint32x3 => wgpu::VertexFormat::Sint32x3,
            st::InputItem::Sint32x4 => wgpu::VertexFormat::Sint32x4,
            st::InputItem::Sint8x2 => wgpu::VertexFormat::Sint8x2,
            st::InputItem::Sint8x4 => wgpu::VertexFormat::Sint8x4,
            st::InputItem::Snorm16x2 => wgpu::VertexFormat::Snorm16x2,
            st::InputItem::Snorm16x4 => wgpu::VertexFormat::Snorm16x4,
            st::InputItem::Snorm8x2 => wgpu::VertexFormat::Snorm8x2,
            st::InputItem::Snorm8x4 => wgpu::VertexFormat::Snorm8x4,
            st::InputItem::Uint16x2 => wgpu::VertexFormat::Uint16x2,
            st::InputItem::Uint16x4 => wgpu::VertexFormat::Uint16x4,
            st::InputItem::Uint32 => wgpu::VertexFormat::Uint32,
            st::InputItem::Uint32x2 => wgpu::VertexFormat::Uint32x2,
            st::InputItem::Uint32x3 => wgpu::VertexFormat::Uint32x3,
            st::InputItem::Uint32x4 => wgpu::VertexFormat::Uint32x4,
            st::InputItem::Uint8x2 => wgpu::VertexFormat::Uint8x2,
            st::InputItem::Uint8x4 => wgpu::VertexFormat::Uint8x4,
            st::InputItem::Unorm16x2 => wgpu::VertexFormat::Unorm16x2,
            st::InputItem::Unorm16x4 => wgpu::VertexFormat::Unorm16x4,
            st::InputItem::Unorm8x2 => wgpu::VertexFormat::Unorm8x2,
            st::InputItem::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
        };

        wgpu::VertexAttribute {
            format,
            offset: attribute.offset,
            shader_location: attribute.location,
        }
    }
}

struct Pass;
