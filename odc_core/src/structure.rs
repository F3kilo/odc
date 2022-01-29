use std::collections::HashMap;
use std::path::PathBuf;

pub struct Render {
    pub pass_tree: HashMap<String, Vec<String>>,
    pub passes: HashMap<String, Pass>,
    pub pipelines: HashMap<String, Pipeline>,
    pub bind_groups: HashMap<String, BindGroup>,
    pub textures: HashMap<String, Texture>,
    pub buffers: HashMap<String, Buffer>,
}

impl Render {
    pub fn is_uniform_buffer(&self, name: &str) -> bool {
        for (_, bg) in &self.bind_groups {
            if bg.has_buffer_with_type(name, BufferType::Uniform) {
                return true;
            }
        }
        false
    }

    pub fn is_storage_buffer(&self, name: &str) -> bool {
        for (_, bg) in &self.bind_groups {
            if bg.has_buffer_with_type(name, BufferType::Storage) {
                return true;
            }
        }
        false
    }

    pub fn is_vertex_buffer(&self, name: &str) -> bool {
        for (_, pipeline) in &self.pipelines {
            if pipeline.has_input_buffer(name) {
                return true;
            }
        }
        false
    }

    pub fn is_index_buffer(&self, name: &str) -> bool {
        for (_, pipeline) in &self.pipelines {
            if pipeline.has_index_buffer(name) {
                return true;
            }
        }
        false
    }
}

pub struct Pass {
    pub pipelines: Vec<String>,
    pub attachments: Vec<Attachment>,
}

pub struct Attachment {
    pub texture: String,
    pub size: Size2d,
    pub offset: Size2d,
}

pub struct Pipeline {
    pub input_buffers: Vec<InputBuffer>,
    pub index_buffer: String,
    pub bind_groups: Vec<String>,
    pub shader: Shader,
    pub depth: Option<DepthOps>,
}

impl Pipeline {
    pub fn has_input_buffer(&self, name: &str) -> bool {
        self.input_buffers
            .iter()
            .any(|in_buf| &in_buf.buffer == name)
    }

    pub fn has_index_buffer(&self, name: &str) -> bool {
        &self.index_buffer == name
    }
}

pub struct DepthOps {
    texture: String,
}

pub struct Shader {
    pub uri: Uri,
}

pub enum Uri {
    File(PathBuf),
}

pub struct InputBuffer {
    pub buffer: String,
    pub attributes: Vec<InputAttribute>,
    pub input_type: InputType,
    pub stride: u64,
}

pub struct InputAttribute {
    pub offset: u64,
    pub item: u64,
}

pub struct InputItem {
    pub typ: InputItemType,
    pub bytes: u8,
    pub count: u8,
}

pub enum InputType {
    PerVertex,
    PerInstance,
}

pub enum InputItemType {
    Float,
    Signed,
    Unsigned,
}

pub struct BindGroup {
    pub bindings: Vec<Binding>,
}

impl BindGroup {
    pub fn has_buffer_with_type(&self, name: &str, in_typ: BufferType) -> bool {
        self.bindings.iter().any(|binding| {
            if let Binding::Buffer(BufferBinding { typ, buffer, .. }) = binding {
                return name == buffer && *typ == in_typ;
            }
            false
        })
    }
}

pub enum Binding {
    Buffer(BufferBinding),
    Texture(TextureBinding),
    Sampler(SamplerBinding),
}

pub struct BufferBinding {
    pub buffer: String,
    pub typ: BufferType,
    pub size: u64,
    pub offset: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BufferType {
    Uniform,
    Storage,
}

pub struct TextureBinding {
    pub texture: String,
    pub size: Size2d,
    pub offset: Size2d,
}

pub struct SamplerBinding {
    pub sampler_type: SamplerType,
}

pub enum SamplerType {
    Color,
    Depth,
}

pub struct Buffer {
    pub size: u64,
}

pub struct Size2d {
    pub x: u64,
    pub y: u64,
}

pub struct Texture {
    pub typ: TextureType,
    pub size: Size2d,
}

pub enum TextureType {
    Color {
        texel: TexelType,
        texel_count: TexelCount,
    },
    Depth,
}

pub enum TexelType {
    Float(BytesPerFloatTexel),
    Int(BytesPerIntTexel),
}

pub enum BytesPerFloatTexel {
    Two,
    Four,
}

pub enum BytesPerIntTexel {
    One,
    Two,
    Four,
}

pub enum TexelCount {
    One,
    Two,
    Four,
}
