use std::path::PathBuf;
use std::collections::HashMap;

pub struct RenderStructure {
	pub pass_tree: HashMap<String, Vec<String>>,
	pub passes: HashMap<String, Pass>,
	pub pipelines: HashMap<String, Pipeline>,
	pub bind_groups: HashMap<String, BindGroup>,
	pub textures: HashMap<String, Texture>,
	pub buffers: HashMap<String, Buffer>,
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
	pub bind_groups: Vec<String>,
	pub shader: Shader,
	pub depth: Option<DepthOps>,
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

pub enum Binding {
	Buffer(BufferBinding),
	Texture(TextureBinding),
	Sampler(SamplerBinding),
}

pub struct BufferBinding {
	pub buffer: String,
	pub size: u64,
	pub offset: u64,
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
	pub buffer_type: BufferType,
}

pub enum BufferType {
	Uniform,
	Storage,
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
	Color{
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