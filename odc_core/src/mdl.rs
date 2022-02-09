use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RenderModel {
    pub stages: Stages,
    pub passes: HashMap<String, Pass>,
    pub pipelines: HashMap<String, RenderPipeline>,
    pub bind_groups: HashMap<String, BindGroup>,
    pub textures: HashMap<String, Texture>,
    pub buffers: HashMap<String, Buffer>,
    pub samplers: HashMap<String, Sampler>,
}

impl RenderModel {
    pub fn has_uniform_binding(&self, name: &str) -> bool {
        self.bind_groups.iter().any(|(_, bg)| bg.has_uniform(name))
    }

    pub fn has_texture_binding(&self, name: &str) -> bool {
        let in_bind_group = self.bind_groups.iter().any(|(_, bg)| bg.has_texture(name));
        let in_window = self.textures[name].window_source;
        in_bind_group | in_window
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

    pub fn has_sampler(&self, name: &str) -> bool {
        self.bind_groups.iter().any(|(_, bg)| bg.has_sampler(name))
    }

    pub fn connected_attachments<'a>(&'a self, name: &'a str) -> HashSet<&'a str> {
        let mut connected = HashSet::with_capacity(16);
        connected.insert(name);
        let mut prev_len = 0;

        loop {
            for pass in self.passes.values() {
                if pass.has_texture_attachment(name) {
                    let color_iter = pass
                        .color_attachments
                        .iter()
                        .map(|att| att.texture.as_str());
                    let depth_iter = pass.depth_attachment.iter().map(|att| att.texture.as_str());
                    connected.extend(color_iter.chain(depth_iter));
                }
            }

            if connected.len() == prev_len {
                break;
            }
            prev_len = connected.len();
        }

        connected
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
            .any(|attachment| attachment.texture == name);

        let depth_attachment = self
            .depth_attachment
            .iter()
            .any(|attachment| attachment.texture == name);
        color_attachment || depth_attachment
    }
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub texture: String,
    pub clear: Option<[f64; 4]>,
    pub store: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DepthAttachment {
    pub texture: String,
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

    pub fn has_sampler(&self, name: &str) -> bool {
        self.samplers
            .iter()
            .any(|binding| binding.info.sampler == name)
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
    pub filterable: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SamplerInfo {
    pub sampler: String,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Buffer {
    pub size: u64,
    pub role: BufferRole,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BufferRole {
    Index,
    Input,
    Uniform,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Size2d {
    pub x: u64,
    pub y: u64,
}

impl Size2d {
    pub fn is_zero(&self) -> bool {
        self.x * self.y == 0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Texture {
    pub typ: TextureType,
    pub size: Size2d,
    pub window_source: bool,
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Sampler {
    NonFilter,
    Filter(FilterMode),
    Comparison(CompareMode),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FilterMode {
    Linear,
    Anisotropic(AnisotropyLevel),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AnisotropyLevel {
    One,
    Two,
    Four,
    Eight,
    Sixteen,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum CompareMode {
    Never,
    Less,
    Equal,
    LessEqual,
    Greater,
    NotEqual,
    GreaterEqual,
    Always,
}
