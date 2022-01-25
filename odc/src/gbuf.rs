use wgpu::{TextureFormat, Extent3d, TextureDescriptor, TextureUsages, TextureDimension, Texture};
use crate::{WindowSize};
use std::time::Instant;
use crate::gdevice::GfxDevice;

pub struct GBuffer {
	pub position: Texture,
	pub albedo: Texture,
	pub depth: Texture,
}

impl GBuffer {
	pub const POSITION_FORMAT: TextureFormat = TextureFormat::Rg32Float;
	pub const ALBEDO_FORMAT: TextureFormat = TextureFormat::Rg32Float;
	pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

	pub fn new(device: &GfxDevice, size: WindowSize) -> Self {
		let position = Self::create_position_texture(device, size);
		let albedo = Self::create_albedo_texture(device, size);
		let depth = Self::create_depth_texture(device, size);

		Self {
			position,
			albedo,
			depth,
		}
	}

	fn create_position_texture(device: &GfxDevice, size: WindowSize) -> Texture {
		let size = Extent3d {
			width: size.0,
			height: size.1,
			depth_or_array_layers: 1,
		};

		let descriptor = TextureDescriptor {
			label: None,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format: Self::POSITION_FORMAT,
			usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
		};

		device.device.create_texture(&descriptor)
	}

	fn create_albedo_texture(device: &GfxDevice, size: WindowSize) -> Texture {
		let size = Extent3d {
			width: size.0,
			height: size.1,
			depth_or_array_layers: 1,
		};

		let descriptor = TextureDescriptor {
			label: None,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format: Self::ALBEDO_FORMAT,
			usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
		};

		device.device.create_texture(&descriptor)
	}

	fn create_depth_texture(device: &GfxDevice, size: WindowSize) -> Texture {
		let size = Extent3d {
			width: size.0,
			height: size.1,
			depth_or_array_layers: 1,
		};

		let descriptor = TextureDescriptor {
			label: None,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: TextureDimension::D2,
			format: Self::DEPTH_FORMAT,
			usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
		};

		device.device.create_texture(&descriptor)
	}
}