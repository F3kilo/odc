use std::num::NonZeroU8;
use crate::model as mdl;
use std::collections::HashMap;

pub struct Resources {
    buffers: HashMap<String, Buffer>,
    textures: HashMap<String, Texture>,
    samplers: HashMap<String, Sampler>,
}

impl Resources {
    pub fn new(device: &wgpu::Device, model: &mdl::RenderModel) -> Self {
        let factory = HandlesFactory { device, model };

        let buffers = model
            .buffers
            .iter()
            .map(|(name, item)| (name.clone(), factory.create_buffer(name, item)))
            .collect();

        let textures = model
            .textures
            .iter()
            .map(|(name, item)| (name.clone(), factory.create_texture(name, item)))
            .collect();

        let samplers = model
            .samplers
            .iter()
            .map(|(name, item)| (name.clone(), factory.create_sampler(name, *item)))
            .collect();

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

    pub fn raw_sampler(&self, name: &str) -> &wgpu::Sampler {
        &self.samplers[name].0
    }

    pub fn texture_format(&self, id: &str) -> wgpu::TextureFormat {
        self.textures[id].format
    }

    pub fn texture_format_by_type(typ: mdl::TextureType) -> wgpu::TextureFormat {
        Texture::find_format(typ)
    }

    pub fn sampler_binding_type_from_format(
        format: wgpu::TextureFormat,
    ) -> wgpu::SamplerBindingType {
        match format.describe().sample_type {
            wgpu::TextureSampleType::Float { filterable: false } => {
                wgpu::SamplerBindingType::NonFiltering
            }
            wgpu::TextureSampleType::Depth => wgpu::SamplerBindingType::NonFiltering,
            _ => wgpu::SamplerBindingType::Filtering,
        }
    }

    pub fn bind_input_buffer<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        buffer: &str,
        index: u32,
    ) {
        let buffer = &self.buffers[buffer].0;
        pass.set_vertex_buffer(index, buffer.slice(..));
    }

    pub fn bind_index_buffer<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, buffer: &str) {
        let buffer = &self.buffers[buffer].0;
        pass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint32);
    }

    pub fn write_buffer(&self, queue: &wgpu::Queue, id: &str, data: &[u8], offset: u64) {
        let buffer = &self.buffers[id].0;
        queue.write_buffer(buffer, offset, data);
    }

    pub fn resize_texture(
        &mut self,
        device: &wgpu::Device,
        texture_id: &str,
        new_size: mdl::Size2d,
    ) {
        let texture = &self.textures[texture_id];

        let size = wgpu::Extent3d {
            width: new_size.x as _,
            height: new_size.y as _,
            depth_or_array_layers: 1,
        };

        let format = texture.format;
        let usage = texture.usages;

        let descriptor = wgpu::TextureDescriptor {
            label: Some(texture_id),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        };
        let raw_texture = device.create_texture(&descriptor);
        let new_texture = Texture::new(raw_texture, usage, format);
        self.textures.insert(texture_id.into(), new_texture);
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

struct Texture {
    handle: wgpu::Texture,
    usages: wgpu::TextureUsages,
    format: wgpu::TextureFormat,
}

impl Texture {
    pub fn new(
        handle: wgpu::Texture,
        usages: wgpu::TextureUsages,
        format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            handle,
            usages,
            format,
        }
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
        self.handle.create_view(&Default::default())
    }
}

struct Sampler(wgpu::Sampler);

impl Sampler {
    pub fn new(handle: wgpu::Sampler) -> Self {
        Self(handle)
    }

    pub fn sampler_info(info: mdl::Sampler) -> (wgpu::FilterMode, Option<NonZeroU8>, Option<wgpu::CompareFunction>) {
        match info {
            mdl::Sampler::NonFilter => (wgpu::FilterMode::Nearest, None, None),
            mdl::Sampler::Filter(mode) => (wgpu::FilterMode::Linear, Self::anisotropy_mode(mode), None),
            mdl::Sampler::Comparison(mode) => (wgpu::FilterMode::Nearest, None, Some(Self::compare_mode(mode))),
        }
    }

    pub fn compare_mode(mode: mdl::CompareMode) -> wgpu::CompareFunction {
        match mode {
            mdl::CompareMode::Never => wgpu::CompareFunction::Never,
            mdl::CompareMode::Less => wgpu::CompareFunction::Less,
            mdl::CompareMode::Equal => wgpu::CompareFunction::Equal,
            mdl::CompareMode::LessEqual => wgpu::CompareFunction::LessEqual,
            mdl::CompareMode::Greater => wgpu::CompareFunction::Greater,
            mdl::CompareMode::NotEqual => wgpu::CompareFunction::NotEqual,
            mdl::CompareMode::GreaterEqual => wgpu::CompareFunction::GreaterEqual,
            mdl::CompareMode::Always => wgpu::CompareFunction::Always,
        }
    }

    fn anisotropy_mode(mode: mdl::FilterMode) -> Option<NonZeroU8> {
        let level = match mode {
            mdl::FilterMode::Linear => return None,
            mdl::FilterMode::Anisotropic(level) => level,
        };

        match level {
            mdl::AnisotropyLevel::One => Some(NonZeroU8::new(1).unwrap()),
            mdl::AnisotropyLevel::Two => Some(NonZeroU8::new(2).unwrap()),
            mdl::AnisotropyLevel::Four => Some(NonZeroU8::new(4).unwrap()),
            mdl::AnisotropyLevel::Eight => Some(NonZeroU8::new(8).unwrap()),
            mdl::AnisotropyLevel::Sixteen => Some( NonZeroU8::new(16).unwrap()),
        }
    }
}

struct HandlesFactory<'a> {
    device: &'a wgpu::Device,
    model: &'a mdl::RenderModel,
}

impl<'a> HandlesFactory<'a> {
    pub fn create_buffer(&self, name: &str, info: &mdl::Buffer) -> Buffer {
        let usage = Buffer::find_usages(name, self.model);
        let descriptor = wgpu::BufferDescriptor {
            label: Some(name),
            size: info.size,
            usage,
            mapped_at_creation: false,
        };
        Buffer::new(self.device.create_buffer(&descriptor))
    }

    pub fn create_texture(&self, name: &str, info: &mdl::Texture) -> Texture {
        let usage = Texture::find_usages(name, self.model);

        let size = wgpu::Extent3d {
            width: info.size.x as _,
            height: info.size.y as _,
            depth_or_array_layers: 1,
        };

        let format = Texture::find_format(info.typ);

        let descriptor = wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        };

        Texture::new(self.device.create_texture(&descriptor), usage, format)
    }

    pub fn create_sampler(&self, name: &str, info: mdl::Sampler) -> Sampler {
        
                let (filter, anisotropy, compare) = Sampler::sampler_info(info);

                let descriptor = wgpu::SamplerDescriptor {
                    label: Some(name),
                    mag_filter: filter,
                    min_filter: filter,
                    mipmap_filter: filter,
                    anisotropy_clamp: anisotropy,
                    compare,
                    ..Default::default()
                };
                let sampler = self.device.create_sampler(&descriptor);
                Sampler::new(sampler)
    }
}
