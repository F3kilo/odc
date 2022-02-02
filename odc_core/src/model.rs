use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RenderModel {
    pub stages: Stages,
    pub passes: HashMap<String, Pass>,
    pub pipelines: HashMap<String, RenderPipeline>,
    pub bind_groups: HashMap<String, BindGroup>,
    pub textures: HashMap<String, Texture>,
    pub buffers: HashMap<String, Buffer>,
}

impl RenderModel {
    pub fn has_uniform_binding(&self, name: &str) -> bool {
        self.bind_groups.iter().any(|(_, bg)| bg.has_uniform(name))
    }

    pub fn has_texture_binding(&self, name: &str) -> bool {
        self.bind_groups.iter().any(|(_, bg)| bg.has_texture(name))
    }

    pub fn has_texture_attachment(&self, name: &str) -> bool {
        self.passes
            .iter()
            .any(|(_, pass)| pass.has_texture_attachment(name))
    }

    pub fn has_input_buffer(&self, name: &str) -> bool {
        self.pipelines
            .iter()
            .any(|(_, pipeline)| pipeline.has_input_buffer(name))
    }

    pub fn has_index_buffer(&self, name: &str) -> bool {
        self.pipelines
            .iter()
            .any(|(_, pipeline)| pipeline.has_index_buffer(name))
    }

    pub fn has_sampler(&self, sampler_type: SamplerType) -> bool {
        self.bind_groups
            .iter()
            .any(|(_, bg)| bg.has_sampler(sampler_type))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Stages(pub Vec<PassGroup>);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PassGroup(pub Vec<String>);

#[derive(Debug, Clone)]
pub struct Pass {
    pub pipelines: Vec<String>,
    pub color_attachments: Vec<Attachment>,
    pub depth_attachment: Option<DepthAttachment>,
}

impl Pass {
    pub fn has_texture_attachment(&self, name: &str) -> bool {
        let color_attachment = self
            .color_attachments
            .iter()
            .any(|attachment| attachment.has_texture(name));

        let depth_attachment = self.depth_attachment
            .as_ref()
            .map(|attachment| attachment.texture == name)
            .unwrap_or(false);
        color_attachment || depth_attachment
    }
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub target: AttachmentTarget,
    pub size: Size2d,
    pub offset: Size2d,
    pub clear: Option<[f64; 4]>,
    pub store: bool,
}

impl Attachment {
    pub fn has_texture(&self, name: &str) -> bool {
        if let AttachmentTarget::Texture(tex) = &self.target {
            return tex == name;
        }
        false
    }

    pub fn is_window(&self) -> bool {
        matches!(self.target, AttachmentTarget::Window)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AttachmentTarget {
    Window,
    Texture(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DepthAttachment {
    pub texture: String,
    pub size: Size2d,
    pub offset: Size2d,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenderPipeline {
    pub input_buffers: Vec<InputBuffer>,
    pub index_buffer: String,
    pub bind_groups: Vec<String>,
    pub shader: Shader,
    pub depth: Option<DepthOps>,
}

impl RenderPipeline {
    pub fn has_input_buffer(&self, name: &str) -> bool {
        self.input_buffers
            .iter()
            .any(|in_buf| in_buf.buffer == name)
    }

    pub fn has_index_buffer(&self, name: &str) -> bool {
        self.index_buffer == name
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct DepthOps {}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Shader {
    pub path: PathBuf,
    pub vs_main: String,
    pub fs_main: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InputBuffer {
    pub buffer: String,
    pub attributes: Vec<InputAttribute>,
    pub input_type: InputType,
    pub stride: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct InputAttribute {
    pub item: InputItem,
    pub offset: u64,
    pub location: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputItem {
    Float16x2,
    Float16x4,
    Float32,
    Float32x2,
    Float32x3,
    Float32x4,
    Sint16x2,
    Sint16x4,
    Sint32,
    Sint32x2,
    Sint32x3,
    Sint32x4,
    Sint8x2,
    Sint8x4,
    Snorm16x2,
    Snorm16x4,
    Snorm8x2,
    Snorm8x4,
    Uint16x2,
    Uint16x4,
    Uint32,
    Uint32x2,
    Uint32x3,
    Uint32x4,
    Uint8x2,
    Uint8x4,
    Unorm16x2,
    Unorm16x4,
    Unorm8x2,
    Unorm8x4,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputType {
    PerVertex,
    PerInstance,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct BindGroup {
    pub uniforms: Vec<Binding<UniformInfo>>,
    pub textures: Vec<Binding<TextureInfo>>,
    pub samplers: Vec<Binding<SamplerInfo>>,
}

impl BindGroup {
    pub fn has_uniform(&self, name: &str) -> bool {
        self.uniforms
            .iter()
            .any(|binding| binding.info.buffer == name)
    }

    pub fn has_texture(&self, name: &str) -> bool {
        self.textures
            .iter()
            .any(|binding| binding.info.texture == name)
    }

    pub fn has_sampler(&self, samplers_type: SamplerType) -> bool {
        self.samplers
            .iter()
            .any(|binding| binding.info.sampler_type == samplers_type)
    }

    pub fn bindings_count(&self) -> usize {
        self.uniforms.len() + self.textures.len() + self.samplers.len()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShaderStages {
    Vertex,
    Fragment,
    Both,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Binding<BindingInfo> {
    pub index: u32,
    pub shader_stages: ShaderStages,
    pub info: BindingInfo,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UniformInfo {
    pub buffer: String,
    pub size: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TextureInfo {
    pub texture: String,
    pub size: Size2d,
    pub offset: Size2d,
    pub filterable: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SamplerInfo {
    pub sampler_type: SamplerType,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum SamplerType {
    Filter,
    NonFilter,
    Depth,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Buffer {
    pub size: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Size2d {
    pub x: u64,
    pub y: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Texture {
    pub typ: TextureType,
    pub size: Size2d,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TextureType {
    Color {
        texel: TexelType,
        texel_count: TexelCount,
    },
    Depth,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TexelType {
    Float(BytesPerFloatTexel),
    Sint(BytesPerIntTexel),
    Uint(BytesPerIntTexel),
    Snorm(BytesPerNormTexel),
    Unorm(BytesPerNormTexel),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BytesPerFloatTexel {
    Two,
    Four,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BytesPerIntTexel {
    One,
    Two,
    Four,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BytesPerNormTexel {
    One,
    Two,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TexelCount {
    One,
    Two,
    Four,
}
