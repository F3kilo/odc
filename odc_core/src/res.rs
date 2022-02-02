use crate::model as mdl;
use std::collections::HashMap;

pub struct Resources {
    buffers: HashMap<String, Buffer>,
    textures: HashMap<String, Texture>,
    samplers: HashMap<mdl::SamplerType, Sampler>,
}

impl Resources {
    pub fn new(device: &wgpu::Device, render: &mdl::RenderModel) -> Self {
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

        Self {
            buffers,
            textures,
            samplers,
        }
    }

    pub fn raw_buffer(&self, name: &str) -> &wgpu::Buffer {
        &self.buffers[name].0
    }

    pub fn texture_view(&self, name: &str) -> wgpu::TextureView {
        self.textures[name].create_view()
    }

    pub fn raw_sampler(&self, typ: &mdl::SamplerType) -> &wgpu::Sampler {
        &self.samplers[typ].0
    }

    pub fn texture_format(typ: mdl::TextureType) -> wgpu::TextureFormat {
        Texture::find_format(typ)
    }
}

struct Buffer(wgpu::Buffer);

impl Buffer {
    pub fn new(handle: wgpu::Buffer) -> Self {
        Self(handle)
    }

    pub fn find_usages(name: &str, render: &mdl::RenderModel) -> wgpu::BufferUsages {
        let is_uniform = render.has_uniform_binding(name);
        // let is_storage = render.has_buffer_binding(name, mdl::BufferType::Storage);
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

    pub fn find_usages(name: &str, render: &mdl::RenderModel) -> wgpu::TextureUsages {
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

    pub fn find_format(typ: mdl::TextureType) -> wgpu::TextureFormat {
        match typ {
            mdl::TextureType::Color { texel, texel_count } => {
                Self::find_color_format(texel, texel_count)
            }
            mdl::TextureType::Depth => wgpu::TextureFormat::Depth32Float,
        }
    }

    pub fn find_color_format(
        texel: mdl::TexelType,
        texel_count: mdl::TexelCount,
    ) -> wgpu::TextureFormat {
        use mdl::BytesPerFloatTexel as FloatBytes;
        use mdl::BytesPerIntTexel as IntBytes;
        use mdl::BytesPerNormTexel as NormBytes;
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

    pub fn filter_mode(sampler_type: mdl::SamplerType) -> wgpu::FilterMode {
        match sampler_type {
            mdl::SamplerType::Filter => wgpu::FilterMode::Linear,
            mdl::SamplerType::NonFilter => wgpu::FilterMode::Nearest,
            mdl::SamplerType::Depth => wgpu::FilterMode::Nearest,
        }
    }

    pub fn compare(sampler_type: mdl::SamplerType) -> Option<wgpu::CompareFunction> {
        match sampler_type {
            mdl::SamplerType::Filter | mdl::SamplerType::NonFilter => None,
            mdl::SamplerType::Depth => Some(wgpu::CompareFunction::Less),
        }
    }

    pub fn binding_type(sampler_type: mdl::SamplerType) -> wgpu::SamplerBindingType {
        match sampler_type {
            mdl::SamplerType::Filter => wgpu::SamplerBindingType::Filtering,
            mdl::SamplerType::NonFilter => wgpu::SamplerBindingType::NonFiltering,
            mdl::SamplerType::Depth => wgpu::SamplerBindingType::Comparison,
        }
    }
}

struct HandlesFactory<'a> {
    device: &'a wgpu::Device,
    render: &'a mdl::RenderModel,
}

impl<'a> HandlesFactory<'a> {
    pub fn create_buffer(&self, name: &str, info: &mdl::Buffer) -> Buffer {
        let usage = Buffer::find_usages(name, self.render);
        let descriptor = wgpu::BufferDescriptor {
            label: Some(name),
            size: info.size,
            usage,
            mapped_at_creation: false,
        };
        Buffer::new(self.device.create_buffer(&descriptor))
    }

    pub fn create_texture(&self, name: &str, info: &mdl::Texture) -> Texture {
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

    pub fn create_samplers(&self) -> HashMap<mdl::SamplerType, Sampler> {
        let sampler_types = [
            mdl::SamplerType::Filter,
            mdl::SamplerType::NonFilter,
            mdl::SamplerType::Depth,
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
}
