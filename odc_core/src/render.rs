use crate::{GfxDevice, RenderStructure};
use structure::Pass;

pub struct RenderData {
	buffers: HashMap<String, Buffer>,
	textures: HashMap<String, Texture>,
	bind_groups: HashMap<String, BindGroup>,
	pipelines: HashMap<String, Pipeline>,
	passes: HashMap<String, Pass>
}

impl RenderData {
	pub fn from_structure(device: &GfxDevice, structure: &RenderStructure) -> Self {
		todo!()
	}
}