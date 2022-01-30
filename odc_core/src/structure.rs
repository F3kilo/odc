use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Render {
    pub pass_tree: HashMap<String, Vec<String>>,
    pub passes: HashMap<String, Pass>,
    pub pipelines: HashMap<String, Pipeline>,
    pub bind_groups: HashMap<String, BindGroup>,
    pub textures: HashMap<String, Texture>,
    pub buffers: HashMap<String, Buffer>,
}

impl Render {
    pub fn has_buffer_binding(&self, name: &str, typ: BufferType) -> bool {
        self.bind_groups
            .iter()
            .any(|(_, bg)| bg.has_buffer_with_type(name, typ))
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

    pub fn bind_group_layout(&self, bind_group: &BindGroup) -> BindGroupLayout {
        let mut bindings = Vec::with_capacity(bind_group.bindings.len());
        for (stage, binding) in &bind_group.bindings {
            let binding_info = self.get_binding_info(binding);
            bindings.push((*stage, binding_info));
        }
        BindGroupLayout { bindings }
    }

    fn get_binding_info(&self, binding: &Binding) -> BindingInfo {
        match binding {
            Binding::Buffer(buffer_binding) => BindingInfo::Buffer(buffer_binding.typ),
            Binding::Texture(texture_binding) => {
                let texture = self.textures[&texture_binding.texture];
                BindingInfo::Texture(texture.typ, texture_binding.filterable)
            }
            Binding::Sampler(sampler_binding) => BindingInfo::Sampler(sampler_binding.typ),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Pass {
    pub pipelines: Vec<String>,
    pub attachments: Vec<Attachment>,
}

impl Pass {
    pub fn has_texture_attachment(&self, name: &str) -> bool {
        self.attachments.iter().any(|attachment| {
            return name == &attachment.texture;
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Attachment {
    pub texture: String,
    pub size: Size2d,
    pub offset: Size2d,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DepthOps {
    texture: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Shader {
    pub uri: Uri,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Uri {
    File(PathBuf),
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
    pub offset: u64,
    pub item: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct InputItem {
    pub typ: InputItemType,
    pub bytes: u8,
    pub count: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputType {
    PerVertex,
    PerInstance,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum InputItemType {
    Float,
    Signed,
    Unsigned,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BindGroupLayout {
    pub bindings: Vec<(ShaderStage, BindingInfo)>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BindingInfo {
    Buffer(BufferType),
    Texture(TextureType, Filterable),
    Sampler(SamplerType),
}

pub type Filterable = bool;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BindGroup {
    pub bindings: Vec<(ShaderStage, Binding)>,
}

impl BindGroup {
    pub fn has_buffer_with_type(&self, name: &str, in_typ: BufferType) -> bool {
        self.bindings.iter().any(|binding| {
            if let Binding::Buffer(BufferBinding { typ, buffer, .. }) = binding.1 {
                return name == buffer && typ == in_typ;
            }
            false
        })
    }

    pub fn has_texture(&self, name: &str) -> bool {
        self.bindings.iter().any(|binding| {
            if let Binding::Texture(TextureBinding { texture, .. }) = binding.1 {
                return name == texture;
            }
            false
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShaderStage {
    Vertex,
    Fragment,
    Both,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Binding {
    Buffer(BufferBinding),
    Texture(TextureBinding),
    Sampler(SamplerBinding),
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TextureBinding {
    pub texture: String,
    pub size: Size2d,
    pub offset: Size2d,
    pub filterable: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SamplerBinding {
    pub typ: SamplerType,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SamplerType {
    Color(Filterable),
    Depth,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Buffer {
    pub size: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
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
