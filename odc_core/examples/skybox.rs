mod common;

use crate::common::DrawDataStorage;
use common::{mesh, Example};
use glam::{Mat3, Mat4};
use image::{EncodableLayout, ImageFormat};
use odc_core::mdl::{Extent3d, Size2d};
use odc_core::{mdl, mdl::RenderModel, DrawData, OdcCore, TextureData, TextureWrite, BufferType};
use std::f32::consts::PI;
use std::fs;
use std::io::BufReader;
use std::path::Path;
use std::time::Instant;
use vp_cam::{Camera, CameraBuilder};

struct Skybox(CameraRotation);

impl Example for Skybox {
    fn render_model() -> RenderModel {
        common::models::skybox::skybox_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![(0, "color".into(), Size2d { x: 800, y: 600 })]
    }

    fn init(&mut self, renderer: &mut OdcCore) {
        let (vertex_data, index_data) = mesh::skybox_mesh();
        renderer.write_buffer(BufferType::Vertex, vertex_data, 0);
        renderer.write_buffer(BufferType::Index, index_data, 0);

        write_skybox(renderer);
    }

    fn update(&mut self, renderer: &mut OdcCore) {
        let angle = self.0.angle();
        let camera = create_camera(angle);
        let ident = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_buffer(BufferType::Uniform, &[ident, camera.view_proj_transform()], 0);
    }

    fn draw_data(&self) -> Vec<DrawDataStorage> {
        let draw = DrawData {
            indices: 0..36,
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

fn write_skybox(renderer: &OdcCore) {
    let data = load_image("odc_core/examples/data/skybox/name");

    let size = Extent3d {
        width: 256,
        height: 256,
        depth_or_array_layers: 6,
    };
    let write = TextureWrite {
        size,
        offset: mdl::Origin3d::ZERO,
        mip_level: 0,
    };

    let data = TextureData {
        data: &data,
        bytes_per_row: size.width * 4,
        rows_per_layer: size.height,
    };

    renderer.write_texture(1, write, data)
}

fn load_image<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let row_texels = 256;
    let layer_texels = row_texels * 256;
    let layer_bytes = layer_texels * 4;
    let mut data = Vec::with_capacity(layer_bytes * 6);

    let names = ["posx", "negx", "posy", "negy", "posz", "negz"];

    for name in names {
        let mut path = path.as_ref().with_file_name(name);
        path.set_extension("jpg");
        let file = fs::File::open(path).unwrap();
        let buffered = BufReader::new(file);
        let image = image::load(buffered, ImageFormat::Jpeg).unwrap();
        data.extend_from_slice(image.into_rgba8().as_bytes());
    }

    assert_eq!(data.len(), layer_bytes * 6);

    data
}

struct CameraRotation {
    start: Instant,
}

impl Default for CameraRotation {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl CameraRotation {
    pub fn angle(&self) -> f32 {
        let elapsed = (Instant::now() - self.start).as_secs_f32();
        let secs_per_cycle = 10.0;
        ((2.0 * PI * elapsed) / secs_per_cycle) % (2.0 * PI)
    }
}

fn create_camera(angle: f32) -> Camera {
    let pos = [0.0, 0.0, 0.0];
    let rot = Mat3::from_rotation_y(angle);
    let target = rot * glam::vec3(0.0, 0.0, 1.0);
    let up = [0.0, 1.0, 0.0];
    CameraBuilder::default()
        .look_at(pos, target.to_array(), up)
        // .orthographic(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0)
        .perspective(PI / 2.0, 4.0 / 3.0, 0.1, Some(5.0))
        .build()
}

fn main() {
    common::run_example(Skybox(CameraRotation::default()))
}
