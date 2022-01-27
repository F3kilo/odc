mod common;

use crate::common::Example;
use glam::{Mat4, Quat, Vec3};
use odc::{DrawData, Odc};

struct InstancesExample;

impl Example for InstancesExample {
    fn init(&mut self, renderer: &mut Odc) {
        let material = renderer.create_material(&common::color_mesh_material_data().as_info());
        renderer.insert_material(0, material);
        let material = renderer.create_material(&common::blue_mesh_material_data().as_info());
        renderer.insert_material(1, material);

        let (vertex_data, index_data) = common::triangle_mesh();
        renderer.write_vertices(vertex_data, 0);
        renderer.write_indices(index_data, 0);

        let vertex_offset = vertex_data.len();
        let index_offset = index_data.len();
        let (vertex_data, index_data) = common::rectangle_mesh();
        renderer.write_vertices(vertex_data, vertex_offset as _);
        renderer.write_indices(index_data, index_offset as _);

        let scale = Vec3::new(0.4, 0.4, 0.4);
        let left = Vec3::new(-0.5, 0.0, 0.0);
        let right = Vec3::new(0.5, 0.0, 0.0);
        let left_transform =
            Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, left).to_cols_array_2d();
        let right_transform =
            Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, right).to_cols_array_2d();
        renderer.write_instances(&[left_transform, right_transform], 0);

        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        let world = ident_transform;
        let view_proj = ident_transform;
        renderer.write_uniform(&[world, view_proj], 0);
    }

    fn update(&mut self, _renderer: &Odc) {}

    fn draw_info(&self) -> Vec<(u64, Vec<DrawData>)> {
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

        vec![(0, vec![draw_triangle]), (1, vec![draw_rectangle])]
    }
}

fn main() {
    common::run_example(InstancesExample)
}
