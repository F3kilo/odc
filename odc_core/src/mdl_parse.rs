use crate::mdl;
use crate::mdl::AddressMode;
use crate::pipelines::{
    InputBufferLayout, RenderPipelineInfo, RenderPipelineInput, RenderShaderInfo,
};
use crate::res::{
    BindGroupInfo, Binding, BufferInfo, SamplerBindingInfo, SamplerInfo, TextureBindingInfo,
    TextureInfo, UniformBindingInfo,
};
use std::fs;
use std::num::NonZeroU8;

pub struct ModelParser<'a> {
    model: &'a mdl::RenderModel,
}

impl<'a> ModelParser<'a> {
    pub fn new(model: &'a mdl::RenderModel) -> Self {
        Self { model }
    }

    pub fn index_info(&self) -> BufferInfo {
        BufferInfo {
            size: self.model.buffers.index,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        }
    }

    pub fn vertex_info(&self) -> BufferInfo {
        BufferInfo {
            size: self.model.buffers.vertex,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }
    }

    pub fn instance_info(&self) -> BufferInfo {
        BufferInfo {
            size: self.model.buffers.instance,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        }
    }

    pub fn uniform_info(&self) -> BufferInfo {
        BufferInfo {
            size: self.model.buffers.index,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    }

    pub fn textures_info(&self) -> impl Iterator<Item = TextureInfo> + 'a {
        let model = self.model;
        model.textures.iter().enumerate().map(|(i, texture_model)| {
            let mut usages = wgpu::TextureUsages::empty();
            if model.has_texture_binding(i) || texture_model.window_source {
                usages |= wgpu::TextureUsages::TEXTURE_BINDING;
            }

            if model.has_texture_attachment(i) {
                usages |= wgpu::TextureUsages::RENDER_ATTACHMENT;
            }

            if texture_model.writable {
                usages |= wgpu::TextureUsages::COPY_DST;
            }

            let sample_count = if texture_model.multisampled { 4 } else { 1 };

            TextureInfo {
                format: Self::parse_texture_format(texture_model.typ),
                size: texture_model.size,
                mip_levels: texture_model.mip_levels,
                sample_count,
                usages,
            }
        })
    }

    pub fn samplers_info(&self) -> impl Iterator<Item = SamplerInfo> + 'a {
        self.model.samplers.iter().map(|sampler_model| SamplerInfo {
            mode: Self::parse_filter_mode(sampler_model.typ),
            compare: Self::parse_comparison(sampler_model.typ),
            anisotropy: Self::parse_anisotropy(sampler_model.typ),
            u_address: Self::parse_address_mode(sampler_model.u_address),
            v_address: Self::parse_address_mode(sampler_model.v_address),
            w_address: Self::parse_address_mode(sampler_model.w_address),
        })
    }

    pub fn parse_filter_mode(sampler_model: mdl::SamplerType) -> wgpu::FilterMode {
        match sampler_model {
            mdl::SamplerType::NonFilter => wgpu::FilterMode::Nearest,
            mdl::SamplerType::Filter(_) => wgpu::FilterMode::Linear,
            mdl::SamplerType::Comparison(_) => wgpu::FilterMode::Nearest,
        }
    }

    pub fn parse_comparison(sampler_model: mdl::SamplerType) -> Option<wgpu::CompareFunction> {
        if let mdl::SamplerType::Comparison(mode) = sampler_model {
            let mode = match mode {
                mdl::CompareMode::Never => wgpu::CompareFunction::Never,
                mdl::CompareMode::Less => wgpu::CompareFunction::Less,
                mdl::CompareMode::Equal => wgpu::CompareFunction::Equal,
                mdl::CompareMode::LessEqual => wgpu::CompareFunction::LessEqual,
                mdl::CompareMode::Greater => wgpu::CompareFunction::Greater,
                mdl::CompareMode::NotEqual => wgpu::CompareFunction::NotEqual,
                mdl::CompareMode::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
                mdl::CompareMode::Always => wgpu::CompareFunction::Always,
            };

            return Some(mode);
        }
        None
    }

    pub fn parse_anisotropy(sampler_model: mdl::SamplerType) -> Option<NonZeroU8> {
        if let mdl::SamplerType::Filter(mdl::FilterMode::Anisotropic(level)) = sampler_model {
            let level = match level {
                mdl::AnisotropyLevel::One => NonZeroU8::new(1),
                mdl::AnisotropyLevel::Two => NonZeroU8::new(2),
                mdl::AnisotropyLevel::Four => NonZeroU8::new(4),
                mdl::AnisotropyLevel::Eight => NonZeroU8::new(8),
                mdl::AnisotropyLevel::Sixteen => NonZeroU8::new(16),
            };

            return level;
        }
        None
    }

    pub fn parse_address_mode(mode: mdl::AddressMode) -> wgpu::AddressMode {
        match mode {
            AddressMode::Edge => wgpu::AddressMode::ClampToEdge,
            AddressMode::Repeat => wgpu::AddressMode::Repeat,
            AddressMode::Mirror => wgpu::AddressMode::MirrorRepeat,
        }
    }

    pub fn bind_groups_info(&self) -> impl Iterator<Item = BindGroupInfo> + 'a {
        let model = self.model;
        model.bind_groups.iter().map(|bg| {
            let uniform = bg.uniform.as_ref().map(|uniform_model| Binding {
                index: uniform_model.index,
                visibility: Self::parse_visibility(uniform_model.shader_stages),
                info: UniformBindingInfo {
                    size: uniform_model.info.size,
                    offset: uniform_model.info.offset,
                },
            });

            let textures = bg
                .textures
                .iter()
                .map(|texture_model| Binding {
                    index: texture_model.index,
                    visibility: Self::parse_visibility(texture_model.shader_stages),
                    info: TextureBindingInfo {
                        format: Self::parse_texture_format(
                            model.textures[texture_model.info.texture].typ,
                        ),
                        texture_index: texture_model.info.texture,
                        dimension: texture_model.info.dimension,
                    },
                })
                .collect();

            let samplers = bg
                .samplers
                .iter()
                .map(|sampler_model| {
                    let sampler_index = sampler_model.info.sampler;
                    let sampler = model.samplers[sampler_index];
                    Binding {
                        index: sampler_model.index,
                        visibility: Self::parse_visibility(sampler_model.shader_stages),
                        info: SamplerBindingInfo {
                            sampler_index,
                            typ: Self::parse_sampler_type(sampler.typ),
                        },
                    }
                })
                .collect();

            BindGroupInfo {
                uniform,
                textures,
                samplers,
            }
        })
    }

    pub fn render_pipelines_info(&self) -> impl Iterator<Item = RenderPipelineInfo> + 'a {
        let model = self.model;
        model.pipelines.iter().enumerate().map(|(i, info)| {
            let source = fs::read_to_string(&info.shader.path).expect("shader file not found");
            let shader = RenderShaderInfo {
                source,
                vs_main: info.shader.vs_main.clone(),
                fs_main: info.shader.fs_main.clone(),
            };

            let sample_count = if info.multisampled { 4 } else { 1 };

            RenderPipelineInfo {
                shader,
                input: Self::input_buffers_info(info),
                bind_groups: info.bind_groups.clone(),
                depth_test: info.depth.is_some(),
                color_targets: Self::pipeline_color_targets(model, i),
                sample_count,
            }
        })
    }

    fn pipeline_color_targets(
        model: &mdl::RenderModel,
        pipeline_index: usize,
    ) -> Vec<wgpu::ColorTargetState> {
        for pass in model.passes.iter() {
            if pass.pipelines.iter().any(|p| *p == pipeline_index) {
                return pass
                    .color_attachments
                    .iter()
                    .zip(model.pipelines[pipeline_index].blend.iter().copied())
                    .map(|(attachment, blend)| {
                        let texture_type = model.textures[attachment.texture].typ;
                        let format = Self::parse_texture_format(texture_type);
                        wgpu::ColorTargetState {
                            format,
                            blend,
                            write_mask: Default::default(),
                        }
                    })
                    .collect();
            }
        }
        panic!("pipeline {} is not used in any pass", pipeline_index);
    }

    fn input_buffers_info(pipeline: &mdl::RenderPipeline) -> Option<RenderPipelineInput> {
        pipeline.input.as_ref().map(|input| {
            let vertex = Self::input_buffer_layout(&input.vertex, wgpu::VertexStepMode::Vertex);
            let instance =
                Self::input_buffer_layout(&input.instance, wgpu::VertexStepMode::Instance);
            RenderPipelineInput { vertex, instance }
        })
    }

    fn input_buffer_layout(
        input_buffer: &mdl::InputInfo,
        step_mode: wgpu::VertexStepMode,
    ) -> InputBufferLayout {
        let attributes = input_buffer
            .attributes
            .iter()
            .map(Self::wgpu_input_attributes)
            .collect();

        InputBufferLayout {
            stride: input_buffer.stride,
            step_mode,
            attributes,
        }
    }

    fn wgpu_input_attributes(attribute: &mdl::InputAttribute) -> wgpu::VertexAttribute {
        let format = match attribute.item {
            mdl::InputItem::Float16x2 => wgpu::VertexFormat::Float16x2,
            mdl::InputItem::Float16x4 => wgpu::VertexFormat::Float16x4,
            mdl::InputItem::Float32 => wgpu::VertexFormat::Float32,
            mdl::InputItem::Float32x2 => wgpu::VertexFormat::Float32x2,
            mdl::InputItem::Float32x3 => wgpu::VertexFormat::Float32x3,
            mdl::InputItem::Float32x4 => wgpu::VertexFormat::Float32x4,
            mdl::InputItem::Sint16x2 => wgpu::VertexFormat::Sint16x2,
            mdl::InputItem::Sint16x4 => wgpu::VertexFormat::Sint16x4,
            mdl::InputItem::Sint32 => wgpu::VertexFormat::Sint32,
            mdl::InputItem::Sint32x2 => wgpu::VertexFormat::Sint32x2,
            mdl::InputItem::Sint32x3 => wgpu::VertexFormat::Sint32x3,
            mdl::InputItem::Sint32x4 => wgpu::VertexFormat::Sint32x4,
            mdl::InputItem::Sint8x2 => wgpu::VertexFormat::Sint8x2,
            mdl::InputItem::Sint8x4 => wgpu::VertexFormat::Sint8x4,
            mdl::InputItem::Snorm16x2 => wgpu::VertexFormat::Snorm16x2,
            mdl::InputItem::Snorm16x4 => wgpu::VertexFormat::Snorm16x4,
            mdl::InputItem::Snorm8x2 => wgpu::VertexFormat::Snorm8x2,
            mdl::InputItem::Snorm8x4 => wgpu::VertexFormat::Snorm8x4,
            mdl::InputItem::Uint16x2 => wgpu::VertexFormat::Uint16x2,
            mdl::InputItem::Uint16x4 => wgpu::VertexFormat::Uint16x4,
            mdl::InputItem::Uint32 => wgpu::VertexFormat::Uint32,
            mdl::InputItem::Uint32x2 => wgpu::VertexFormat::Uint32x2,
            mdl::InputItem::Uint32x3 => wgpu::VertexFormat::Uint32x3,
            mdl::InputItem::Uint32x4 => wgpu::VertexFormat::Uint32x4,
            mdl::InputItem::Uint8x2 => wgpu::VertexFormat::Uint8x2,
            mdl::InputItem::Uint8x4 => wgpu::VertexFormat::Uint8x4,
            mdl::InputItem::Unorm16x2 => wgpu::VertexFormat::Unorm16x2,
            mdl::InputItem::Unorm16x4 => wgpu::VertexFormat::Unorm16x4,
            mdl::InputItem::Unorm8x2 => wgpu::VertexFormat::Unorm8x2,
            mdl::InputItem::Unorm8x4 => wgpu::VertexFormat::Unorm8x4,
        };

        wgpu::VertexAttribute {
            format,
            offset: attribute.offset,
            shader_location: attribute.location,
        }
    }

    fn parse_visibility(model: mdl::ShaderStages) -> wgpu::ShaderStages {
        match model {
            mdl::ShaderStages::Vertex => wgpu::ShaderStages::VERTEX,
            mdl::ShaderStages::Fragment => wgpu::ShaderStages::FRAGMENT,
            mdl::ShaderStages::Both => wgpu::ShaderStages::VERTEX_FRAGMENT,
        }
    }

    fn parse_sampler_type(model: mdl::SamplerType) -> wgpu::SamplerBindingType {
        match model {
            mdl::SamplerType::NonFilter => wgpu::SamplerBindingType::NonFiltering,
            mdl::SamplerType::Filter(_) => wgpu::SamplerBindingType::Filtering,
            mdl::SamplerType::Comparison(_) => wgpu::SamplerBindingType::Comparison,
        }
    }

    fn parse_texture_format(typ: mdl::TextureType) -> wgpu::TextureFormat {
        match typ {
            mdl::TextureType::Color { texel, texel_count } => {
                Self::parse_color_format(texel, texel_count)
            }
            mdl::TextureType::Srgb => wgpu::TextureFormat::Rgba8UnormSrgb,
            mdl::TextureType::Depth => wgpu::TextureFormat::Depth32Float,
        }
    }

    fn parse_color_format(
        texel: mdl::TexelType,
        texel_count: mdl::TexelCount,
    ) -> wgpu::TextureFormat {
        use mdl::BytesPerFloatTexel as FloatBytes;
        use mdl::BytesPerIntTexel as IntBytes;
        use mdl::TexelCount as Count;
        use mdl::TexelType as Texel;
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
            (Texel::Snorm, Count::One) => Format::R8Snorm,
            (Texel::Snorm, Count::Two) => Format::Rg8Snorm,
            (Texel::Snorm, Count::Four) => Format::Rgba8Snorm,

            // 8-bit unorm
            (Texel::Unorm, Count::One) => Format::R8Unorm,
            (Texel::Unorm, Count::Two) => Format::Rg8Unorm,
            (Texel::Unorm, Count::Four) => Format::Rgba8Unorm,
        }
    }
}
