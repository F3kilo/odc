use std::num::NonZeroU8;

pub struct Sampler {
    pub handle: wgpu::Sampler,
    pub info: SamplerInfo,
}

pub struct SamplerInfo {
    pub mode: wgpu::FilterMode,
    pub anisotropy: Option<NonZeroU8>,
    pub compare: Option<wgpu::CompareFunction>,
}
