use std::collections::HashSet;
use std::path::PathBuf;
pub use wgpu::{BlendComponent, BlendFactor, BlendOperation, BlendState, Extent3d, Origin3d};

#[derive(Debug, Clone)]
pub struct RenderModel {
    pub passes: Vec<Pass>,
    pub pipelines: Vec<RenderPipeline>,
    pub bind_groups: Vec<BindGroup>,
    pub textures: Vec<Texture>,
    pub samplers: Vec<Sampler>,
    pub buffers: Buffers,
}

#[derive(Debug, Copy, Clone)]
pub struct Buffers {
    pub index: u64,
    pub vertex: u64,
    pub instance: u64,
    pub uniform: u64,
}

impl RenderModel {
    pub fn has_texture_attachment(&self, index: usize) -> bool {
        self.passes
            .iter()
            .any(|pass| pass.has_texture_attachment(index))
    }

    pub fn has_texture_binding(&self, index: usize) -> bool {
        self.bind_groups
            .iter()
            .any(|bind_group| bind_group.has_texture(index))
    }

    pub fn connected_attachments(&self, index: usize) -> impl Iterator<Item = usize> {
        let mut connected = HashSet::with_capacity(16);
        connected.insert(index);
        let mut prev_len = 0;

        loop {
            for index in connected.clone() {
                for pass in self.passes.iter() {
                    if pass.has_texture_attachment(index) {
                        let color_iter = pass.color_attachments.iter().map(|att| att.texture);
                        let depth_iter = pass.depth_attachment.iter().map(|att| att.texture);
                        connected.extend(color_iter.chain(depth_iter));
                    }
                }
            }

            if connected.len() == prev_len {
                break;
            }
            prev_len = connected.len();
        }

        connected.into_iter()
    }
}

#[derive(Debug, Clone)]
pub struct Pass {
    pub pipelines: Vec<usize>,
    pub color_attachments: Vec<Attachment>,
    pub depth_attachment: Option<DepthAttachment>,
}

impl Pass {
    pub fn has_texture_attachment(&self, index: usize) -> bool {
        let color_attachment = self
            .color_attachments
            .iter()
            .any(|attachment| attachment.texture == index);

        let depth_attachment = self
            .depth_attachment
            .iter()
            .any(|attachment| attachment.texture == index);
        color_attachment || depth_attachment
    }
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub texture: usize,
    pub clear: Option<[f64; 4]>,
    pub store: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DepthAttachment {
    pub texture: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RenderPipeline {
    pub input: Option<PipelineInpit>,
    pub bind_groups: Vec<usize>,
    pub shader: Shader,
    pub blend: Vec<Option<BlendState>>,
    pub depth: Option<DepthOps>,
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
pub struct PipelineInpit {
    pub vertex: InputInfo,
    pub instance: InputInfo,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct InputInfo {
    pub attributes: Vec<InputAttribute>,
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

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct BindGroup {
    pub uniform: Option<Binding<UniformInfo>>,
    pub textures: Vec<Binding<TextureInfo>>,
    pub samplers: Vec<Binding<SamplerInfo>>,
}

impl BindGroup {
    pub fn has_texture(&self, index: usize) -> bool {
        self.textures
            .iter()
            .any(|binding| binding.info.texture == index)
    }

    pub fn has_sampler(&self, index: usize) -> bool {
        self.samplers
            .iter()
            .any(|binding| binding.info.sampler == index)
    }

    pub fn bindings_count(&self) -> usize {
        self.textures.len()
            + self.samplers.len()
            + self.uniform.as_ref().map(|_| 1).unwrap_or_default()
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
    pub size: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TextureInfo {
    pub texture: usize,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SamplerInfo {
    pub sampler: usize,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Size2d {
    pub x: u32,
    pub y: u32,
}

impl From<(u32, u32)> for Size2d {
    fn from((x, y): (u32, u32)) -> Self {
        Self { x, y }
    }
}

impl From<u32> for Size2d {
    fn from(size: u32) -> Self {
        Self { x: size, y: size }
    }
}

impl From<Size2d> for wgpu::Extent3d {
    fn from(s: Size2d) -> Self {
        Extent3d {
            width: s.x as _,
            height: s.y as _,
            depth_or_array_layers: 1,
        }
    }
}

impl Size2d {
    pub fn is_zero(&self) -> bool {
        self.x * self.y == 0
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Texture {
    pub typ: TextureType,
    pub size: Extent3d,
    pub mip_levels: u32,
    pub sample_count: u32,
    pub window_source: bool,
    pub writable: bool,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TextureType {
    Color {
        texel: TexelType,
        texel_count: TexelCount,
    },
    Srgb,
    Depth,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TexelType {
    Float(BytesPerFloatTexel),
    Sint(BytesPerIntTexel),
    Uint(BytesPerIntTexel),
    Snorm,
    Unorm,
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
