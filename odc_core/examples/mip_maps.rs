mod common;

use crate::common::DrawDataStorage;
use common::{mesh, Example};
use glam::{Mat4, Quat};
use odc_core::mdl::Size2d;
use odc_core::{mdl, mdl::RenderModel, DrawData, OdcCore, TextureData, TextureWrite, BufferType};
use std::f32::consts::PI;
use vp_cam::{Camera, CameraBuilder};

struct MipMaps;

impl Example for MipMaps {
    fn render_model() -> RenderModel {
        common::models::mip_map::mip_map_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![(0, "color".into(), Size2d { x: 800, y: 600 })]
    }

    fn init(&mut self, renderer: &mut OdcCore) {
        let (vertex_data, index_data) = mesh::sprite_mesh();
        renderer.write_buffer(BufferType::Vertex, vertex_data, 0);
        renderer.write_buffer(BufferType::Index, index_data, 0);

        let ident = Mat4::IDENTITY.to_cols_array_2d();
        let camera = create_camera();
        renderer.write_buffer(BufferType::Uniform, &[ident, camera.view_proj_transform()], 0);

        let instance_data = instance_data();
        renderer.write_buffer(BufferType::Instance, &instance_data, 0);

        write_image(renderer);
    }

    fn update(&mut self, _renderer: &mut OdcCore) {}

    fn draw_data(&self) -> Vec<DrawDataStorage> {
        let draw = DrawData {
            indices: 0..6,
            base_vertex: 0,
            instances: 0..7,
        };

        vec![DrawDataStorage {
            pass: 0,
            pipeline: 0,
            data: vec![draw],
        }]
    }
}

fn write_image(renderer: &OdcCore) {
    let width = 256u32;
    let mip_levels = 32 - width.leading_zeros();

    for level in 0..mip_levels {
        let size = Size2d::from(width >> level);
        let data = image_data(size, level);

        let layout = TextureData {
            data: &data,
            bytes_per_row: size.x * 4,
            rows_per_layer: 0,
        };

        let write = TextureWrite {
            offset: mdl::Origin3d::ZERO,
            mip_level: level,
            size: size.into(),
        };

        renderer.write_texture(1, write, layout);
    }
}

fn image_data(size: Size2d, level: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((4 * size.x * size.y) as _);

    let texel = match level {
        0 => [255, 0, 0, 255],
        1 => [0, 255, 0, 255],
        2 => [0, 0, 255, 255],
        3 => [255, 255, 0, 255],
        4 => [255, 0, 255, 255],
        5 => [0, 255, 255, 255],
        6 => [255, 255, 255, 255],
        _ => [128, 128, 128, 128],
    };

    for _row in 0..size.y {
        for _texel in 0..size.x {
            data.extend_from_slice(&texel)
        }
    }
    data
}

fn create_camera() -> Camera {
    let pos = [0.0, 0.0, -1.0];
    let target = [0.0; 3];
    let up = [0.0, 1.0, 0.0];
    CameraBuilder::default()
        .look_at(pos, target, up)
        // .orthographic(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0)
        .perspective(PI / 2.0, 4.0 / 3.0, 0.1, Some(10.0))
        .build()
}

fn instance_data() -> Vec<u8> {
    let mut data = Vec::with_capacity(6 * 5 * 4 * 4);

    let uv_offset_scale = [0.0f32, 0.0, 1.0, 1.0];

    for i in 0..7 {
        let scale = 1.0 / 2.0f32.powi(i);
        let scale = glam::vec3(scale, scale, 1.0);
        let translation = glam::vec3(0.0, 0.0, 0.0);
        let transform = Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, translation);
        data.extend_from_slice(bytemuck::cast_slice(&transform.to_cols_array_2d()));
        data.extend_from_slice(bytemuck::cast_slice(&uv_offset_scale));
    }

    data
}

fn main() {
    common::run_example(MipMaps)
}
