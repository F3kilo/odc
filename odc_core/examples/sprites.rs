mod common;

use crate::common::DrawDataStorage;
use common::{mesh, Example};
use glam::{Mat4, Quat};
use image::{EncodableLayout, ImageFormat};
use odc_core::mdl::Size2d;
use odc_core::{mdl, mdl::RenderModel, DrawData, OdcCore, TextureData, TextureWrite, BufferType};
use std::fs;
use std::io::BufReader;
use std::path::Path;

struct Sprite;

impl Example for Sprite {
    fn render_model() -> RenderModel {
        common::models::sprites::sprites_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![(0, "color".into(), Size2d { x: 800, y: 600 })]
    }

    fn init(&mut self, renderer: &OdcCore) {
        let (vertex_data, index_data) = mesh::sprite_mesh();
        renderer.write_buffer(BufferType::Vertex, vertex_data, 0);
        renderer.write_buffer(BufferType::Index, index_data, 0);

        let planet_transform = Mat4::from_scale_rotation_translation(
            (0.5, 0.5, 1.0).into(),
            Quat::IDENTITY,
            (-0.25, -0.25, 0.0).into(),
        )
        .to_cols_array_2d();
        let black_hole_transform = Mat4::from_scale_rotation_translation(
            (0.5, 0.5, 1.0).into(),
            Quat::IDENTITY,
            (0.25, 0.25, 0.0).into(),
        )
        .to_cols_array_2d();
        let planet_uv_offset_scale = [0.0f32, 0.0, 0.5, 1.0];
        let black_hole_uv_offset_scale = [0.5f32, 0.0, 0.5, 1.0];

        let mut instance_data: Vec<[f32; 4]> = Vec::new();
        instance_data.extend(black_hole_transform.iter());
        instance_data.push(black_hole_uv_offset_scale);
        instance_data.extend(planet_transform.iter());
        instance_data.push(planet_uv_offset_scale);

        let ident = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_buffer(BufferType::Uniform, &[ident, ident], 0);
        renderer.write_buffer(BufferType::Instance, &instance_data, 0);

        write_images(renderer);
    }

    fn update(&mut self, _renderer: &OdcCore) {}

    fn draw_data(&self) -> Vec<DrawDataStorage> {
        let draw = DrawData {
            indices: 0..6,
            base_vertex: 0,
            instances: 0..2,
        };

        vec![DrawDataStorage {
            pass: 0,
            pipeline: 0,
            data: vec![draw],
        }]
    }
}

fn write_images(renderer: &OdcCore) {
    let data = load_image("odc_core/examples/data/planet.png");

    let size = Size2d::from((128, 128)).into();
    let write = TextureWrite {
        size,
        offset: mdl::Origin3d::ZERO,
        mip_level: 0,
        index: 1,
    };

    let data = TextureData {
        data: &data,
        bytes_per_row: 128 * 4,
        rows_per_layer: 0,
    };

    renderer.write_texture(write, data);

    let data = load_image("odc_core/examples/data/black_hole.png");

    let write = TextureWrite {
        size,
        offset: mdl::Origin3d { x: 128, y: 0, z: 0 },
        mip_level: 0,
        index: 1,
    };

    let data = TextureData {
        bytes_per_row: 128 * 4,
        data: &data,
        rows_per_layer: 0,
    };

    renderer.write_texture(write, data);
}

fn load_image<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let file = fs::File::open(path).unwrap();
    let buffered = BufReader::new(file);
    let image = image::load(buffered, ImageFormat::Png).unwrap();
    let data = image.into_rgba8().as_bytes().to_vec();
    data
}

fn main() {
    common::run_example(Sprite)
}
