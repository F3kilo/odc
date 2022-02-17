mod common;

use crate::common::DrawDataStorage;
use common::{mesh, Example};
use glam::{Mat4, Quat};
use image::{EncodableLayout, ImageFormat};
use odc_core::mdl::Size2d;
use odc_core::{mdl::RenderModel, DrawData, OdcCore, TextureData, TextureWrite};
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
        renderer.write_vertex(vertex_data, 0);
        renderer.write_index(index_data, 0);

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
        renderer.write_uniform(&[ident, ident], 0);
        renderer.write_instance(&instance_data, 0);

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

    let write = TextureWrite {
        size: (128, 128).into(),
        offset: (0, 0).into(),
        index: 2,
    };

    let data = TextureData {
        bytes_per_row: 128 * 4,
        data: &data,
    };

    renderer.write_texture(write, data);

    let data = load_image("odc_core/examples/data/black_hole.png");

    let write = TextureWrite {
        size: (128, 128).into(),
        offset: (128, 0).into(),
        index: 2,
    };

    let data = TextureData {
        bytes_per_row: 128 * 4,
        data: &data,
    };

    renderer.write_texture(write, data);
}

fn load_image<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let file = fs::File::open(path).unwrap();
    let buffered = BufReader::new(file);
    let image = image::load(buffered, ImageFormat::Png).unwrap();
    let data = image.into_rgba8().as_bytes().to_vec();
    println!("Image data len: {}", data.len());
    data
}

fn main() {
    common::run_example(Sprite)
}