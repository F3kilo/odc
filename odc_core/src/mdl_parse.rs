use crate::mdl;
use crate::mdl::{BufferRole, RenderModel};
use crate::res::BufferInfo;

pub struct ModelParser<'a> {
    model: &'a mdl::RenderModel,
}

impl<'a> ModelParser<'a> {
    pub fn new(model: &RenderModel) -> Self {
        Self {
            model
        }
    }

    pub fn get_buffers(&self) -> impl Iterator<Item = BufferInfo> + 'a {
        self.model.buffers.iter().map(|(name, val)| {
            let mut usage = wgpu::BufferUsages::COPY_DST;
            usage |= match val.role {
                BufferRole::Index => wgpu::BufferUsages::INDEX,
                BufferRole::Input => wgpu::BufferUsages::VERTEX,
                BufferRole::Uniform => wgpu::BufferUsages::UNIFORM,
            };

            BufferInfo {
                name: name.to_string(),
                size: val.size,
                usage,
            }
        })
    }


}
