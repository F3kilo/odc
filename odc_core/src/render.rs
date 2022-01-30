use crate::structure as st;
use std::collections::HashMap;

pub struct RenderData {
    buffers: HashMap<String, Buffer>,
    textures: HashMap<String, Texture>,
    bind_groups: HashMap<String, BindGroup>,
    render_pipelines: HashMap<String, RenderPipeline>,
    passes: HashMap<String, Pass>,
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

        Self { buffers, textures }
    }
}

struct HandlesFactory<'a> {
    device: &'a wgpu::Device,
    render: &'a st::Render,
}

impl<'a> HandlesFactory<'a> {
    pub fn create_buffer(&self, name: &str, buffer: &st::Buffer) -> Buffer {
        let mut usage = Buffer::find_usages(name, self.render);
        let descriptor = wgpu::BufferDescriptor {
            label: Some(name),
            size: buffer.size,
            usage,
            mapped_at_creation: false,
        };
        Buffer::new(self.device.create_buffer(&descriptor))
    }

    pub fn create_texture(&self, name: &str, texture: &st::Texture) -> Texture {
        let mut usage = Texture::find_usages(name, self.render);

        let size = wgpu::Extent3d {
            width: texture.size.x as _,
            height: texture.size.y as _,
            depth_or_array_layers: 1,
        };

        let descriptor = wgpu::TextureDescriptor {
            label: Some(name),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Texture::find_format(texture.typ),
            usage,
        };
        Texture::new(self.device.create_texture(&descriptor))
    }
}

struct Buffer(wgpu::Buffer);

impl Buffer {
    pub fn new(handle: wgpu::Buffer) -> Self {
        Self(handle)
    }

    pub fn find_usages(name: &str, render: &st::Render) -> wgpu::BufferUsages {
        let is_uniform = render.has_buffer_binding(name, st::BufferType::Uniform);
        let is_storage = render.has_buffer_binding(name, st::BufferType::Storage);
        let is_vertex = render.has_input_buffer(name);
        let is_index = render.has_index_buffer(name);

        let usages = wgpu::BufferUsages::COPY_DST;
        if is_uniform {
            usages |= wgpu::BufferUsages::UNIFORM;
        }
        if is_storage {
            usages |= wgpu::BufferUsages::STORAGE;
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
    pub fn new(handle: wgpu::Texture) -> Self {
        Self(handle)
    }

    pub fn find_usages(name: &str, render: &st::Render) -> wgpu::TextureUsages {
        let is_attachment = render.has_texture_attachment(name);
        let is_binding = render.has_texture_binding(name);

        let usages = wgpu::TextureUsages::empty();
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
    		st::TextureType::Color {texel, texel_count} => Self::find_color_format(texel, texel_count),
    		st::TextureType::Depth => wgpu::TextureFormat::Depth32Float,
    	}
    }

    pub fn find_color_format(texel: st::TexelType, texel_count: st::TexelCount) -> wgpu::TextureFormat {
    	use st::TexelType as Texel;
    	use st::BytesPerFloatTexel as FloatBytes;
    	use st::BytesPerIntTexel as IntBytes;
    	use st::BytesPerNormTexel as NormBytes;
    	use st::TexelCount as Count;
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
    		(Texel::Int(IntBytes::One), Count::One) => Format::R8Sint,
    		(Texel::Int(IntBytes::One), Count::Two) => Format::Rg8Sint,
    		(Texel::Int(IntBytes::One), Count::Four) => Format::Rgba8Sint,

    		// 16-bit int
    		(Texel::Int(IntBytes::Two), Count::One) => Format::R16Sint,
    		(Texel::Int(IntBytes::Two), Count::Two) => Format::Rg16Sint,
    		(Texel::Int(IntBytes::Two), Count::Four) => Format::Rgba16Sint,

    		// 32-bit int
    		(Texel::Int(IntBytes::Four), Count::One) => Format::R32Sint,
    		(Texel::Int(IntBytes::Four), Count::Two) => Format::Rg32Sint,
    		(Texel::Int(IntBytes::Four), Count::Four) => Format::Rgba32Sint,

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
}

type BindGroup = wgpu::BindGroup;
type RenderPipeline = wgpu::RenderPipeline;

struct Pass;
