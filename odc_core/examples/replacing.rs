mod common;

use crate::common::DrawDataStorage;
use common::{mesh, Example};
use glam::{Mat4, Quat};
use image::{EncodableLayout, ImageFormat};
use odc_core::mdl::Size2d;
use odc_core::{mdl, mdl::RenderModel, BufferType, DrawData, OdcCore, TextureData, TextureWrite};
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct Replacing {
    texture_timer: Timer,
    uniform_timer: Timer,
}

impl Example for Replacing {
    fn render_model() -> RenderModel {
        common::models::sprites::sprites_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![(0, "color".into(), Size2d { x: 800, y: 600 })]
    }

    fn init(&mut self, renderer: &mut OdcCore) {
        let (vertex_data, index_data) = mesh::sprite_mesh();
        renderer.write_buffer(BufferType::Vertex, vertex_data, 0);
        renderer.write_buffer(BufferType::Index, index_data, 0);

        renderer.insert_stock_texture(1, "sprite".into(), None);
        renderer.insert_stock_buffer(BufferType::Uniform, "uniform".into(), None);

        let instance_transform = Mat4::from_scale_rotation_translation(
            (0.5, 0.5, 1.0).into(),
            Quat::IDENTITY,
            (0.0, 0.0, 0.0).into(),
        )
        .to_cols_array_2d();
        let planet_uv_offset_scale = [0.0f32, 0.0, 0.5, 1.0];

        let mut instance_data: Vec<[f32; 4]> = Vec::new();
        instance_data.extend_from_slice(&instance_transform);
        instance_data.push(planet_uv_offset_scale);
        renderer.write_buffer(BufferType::Instance, &instance_data, 0);

        let ident = Mat4::IDENTITY.to_cols_array_2d();
        let world = Mat4::from_translation(glam::vec3(-0.25, 0.0, 0.0)).to_cols_array_2d();
        renderer.write_buffer(BufferType::Uniform, &[world, ident], 0);

        let world = Mat4::from_translation(glam::vec3(0.25, 0.0, 0.0)).to_cols_array_2d();
        renderer.write_stock_buffer("uniform", &[world, ident], 0);

        write_images(renderer);
    }

    fn update(&mut self, renderer: &mut OdcCore) {
        if self.uniform_timer.ready() {
            renderer.swap_stock_buffer("uniform")
        }

        if self.texture_timer.ready() {
            renderer.swap_stock_texture("sprite")
        }
    }

    fn draw_data(&self) -> Vec<DrawDataStorage> {
        let draw = DrawData {
            indices: 0..6,
            base_vertex: 0,
            instances: 0..1,
        };

        vec![DrawDataStorage {
            pass: 0,
            pipeline: 0,
            data: vec![draw],
        }]
    }
}

impl Default for Replacing {
    fn default() -> Self {
        Self {
            texture_timer: Timer::new(2000),
            uniform_timer: Timer::new(1000),
        }
    }
}

fn write_images(renderer: &OdcCore) {
    let data = load_image("odc_core/examples/data/planet.png");

    let size = Size2d::from((128, 128)).into();
    let write = TextureWrite {
        size,
        offset: mdl::Origin3d::ZERO,
        mip_level: 0,
    };

    let data = TextureData {
        data: &data,
        bytes_per_row: 128 * 4,
        rows_per_layer: 0,
    };

    renderer.write_texture(1, write, data);

    let data = load_image("odc_core/examples/data/black_hole.png");

    let write = TextureWrite {
        size,
        offset: mdl::Origin3d::ZERO,
        mip_level: 0,
    };

    let data = TextureData {
        bytes_per_row: 128 * 4,
        data: &data,
        rows_per_layer: 0,
    };

    renderer.write_stock_texture("sprite", write, data);
}

fn load_image<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let file = fs::File::open(path).unwrap();
    let buffered = BufReader::new(file);
    let image = image::load(buffered, ImageFormat::Png).unwrap();
    let data = image.into_rgba8().as_bytes().to_vec();
    data
}

struct Timer(Arc<AtomicBool>);

impl Timer {
    pub fn new(millis: u64) -> Self {
        let signal = Arc::new(AtomicBool::default());
        let cloned_signal = signal.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(Duration::from_millis(millis));
            cloned_signal.store(true, Ordering::SeqCst)
        });

        Self(signal)
    }

    pub fn ready(&self) -> bool {
        self.0.fetch_and(false, Ordering::SeqCst)
    }
}

fn main() {
    common::run_example(Replacing::default())
}
