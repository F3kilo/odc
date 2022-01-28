mod common;

use std::f32::consts::PI;
use vp_cam::{Camera, CameraBuilder};
use crate::common::Example;
use glam::Mat4;
use odc::{DrawData, Odc};

struct MeshLoadExample(u32);

impl Example for MeshLoadExample {
    fn init(&mut self, renderer: &mut Odc) {
        let material = renderer.create_material(&common::mesh_material_data().as_info());
        renderer.insert_material(0, material);

        let (vertex_data, index_data) = common::load_gltf_mesh("examples/data/models/monkey.glb");
        renderer.write_vertices(&vertex_data, 0);
        renderer.write_indices(&index_data, 0);
        let index_count = index_data.len() / 4;
        self.0 = index_count as _;

        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_instances(&[ident_transform], 0);

        let world = ident_transform;


        let camera = create_camera();
        let view_proj = camera.view_proj_transform();
        renderer.write_uniform(&[world, view_proj], 0);
    }

    fn update(&mut self, _renderer: &Odc) {}

    fn draw_info(&self) -> Vec<(u64, Vec<DrawData>)> {
        let draw = DrawData {
            indices: 0..self.0,
            base_vertex: 0,
            instances: 0..1,
        };

        vec![(0, vec![draw])]
    }
}

fn create_camera() -> Camera {
    let pos = [0.0, 0.0, 3.0];
    let target = [0.0; 3];
    let up = [0.0, 1.0, 0.0];
    CameraBuilder::default()
        .look_at(pos, target, up)
        // .orthographic(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0)
        .perspective(PI / 2.0, 4.0 / 3.0, 0.1, Some(10.0))
        .build()
}

fn main() {
    common::run_example(MeshLoadExample(0))
}
