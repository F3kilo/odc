mod common;

use crate::common::{DrawDataStorage, Example};
use glam::{Mat4, Quat, Vec3};
use odc_core::mdl::Size2d;
use odc_core::{mdl::RenderModel, DrawData, OdcCore, BufferType};
use std::f32::consts::PI;
use vp_cam::{Camera, CameraBuilder};

fn main() {
    common::run_example(MeshesExample)
}

struct MeshesExample;

impl Example for MeshesExample {
    fn render_model() -> RenderModel {
        common::models::color_mesh::color_mesh_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![
            (0, "color".into(), Size2d { x: 800, y: 600 }),
            (1, "depth".into(), Size2d { x: 800, y: 600 }),
        ]
    }

    fn init(&mut self, renderer: &OdcCore) {
        let (vertex_data, index_data) = common::mesh::triangle_mesh();
        renderer.write_buffer(BufferType::Index, index_data, 0);
        renderer.write_buffer(BufferType::Vertex, vertex_data, 0);

        let vertex_offset = vertex_data.len();
        let index_offset = index_data.len();
        let (vertex_data, index_data) = common::mesh::rectangle_mesh();
        renderer.write_buffer(BufferType::Index, index_data, index_offset as _);
        renderer.write_buffer(BufferType::Vertex, vertex_data, vertex_offset as _);

        let scale = Vec3::new(0.5, 0.5, 0.5);
        let left = Vec3::new(-0.2, 0.0, 0.2);
        let rot_left = Quat::from_rotation_y(PI / 6.0);
        let right = Vec3::new(0.2, 0.0, 0.2);
        let rot_right = Quat::from_rotation_y(-PI / 6.0);
        let left_transform =
            Mat4::from_scale_rotation_translation(scale, rot_left, left).to_cols_array_2d();
        let right_transform =
            Mat4::from_scale_rotation_translation(scale, rot_right, right).to_cols_array_2d();
        renderer.write_buffer(BufferType::Instance, &[left_transform, right_transform], 0);

        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        let world = ident_transform;
        let camera = create_camera();
        let view_proj = camera.view_proj_transform();
        renderer.write_buffer(BufferType::Uniform, &[world, view_proj], 0);
    }

    fn update(&mut self, _renderer: &OdcCore) {}

    fn draw_data(&self) -> Vec<DrawDataStorage> {
        let draw_triangle = DrawData {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };

        let draw_rectangle = DrawData {
            indices: 3..9,
            base_vertex: 3,
            instances: 1..2,
        };

        vec![DrawDataStorage {
            pass: 0,
            pipeline: 0,
            data: vec![draw_triangle, draw_rectangle],
        }]
    }
}

fn create_camera() -> Camera {
    let pos = [0.0, -0.3, -1.0];
    let target = [0.0; 3];
    let up = [0.0, 1.0, 0.0];
    CameraBuilder::default()
        .look_at(pos, target, up)
        // .orthographic(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0)
        .perspective(PI / 2.0, 4.0 / 3.0, 0.1, Some(10.0))
        .build()
}
