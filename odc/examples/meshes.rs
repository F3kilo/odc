mod common;

use crate::common::{Example, InstanceInfo};
use glam::{Mat4, Quat, Vec3};
use odc::{Odc, RenderInfo, StaticMesh};

struct InstancesExample;

impl Example for InstancesExample {
    fn init(&mut self, renderer: &Odc) {
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

        let left_instance = InstanceInfo {
            transform: left_transform,
        };
        let right_instance = InstanceInfo {
            transform: right_transform,
        };

        let instances = [left_instance, right_instance];
        renderer.write_instances(&instances, 0);
    }

    fn update(&mut self, _renderer: &Odc) {}

    fn draw_info(&self) -> (RenderInfo, Vec<StaticMesh>) {
        let identity = Mat4::IDENTITY.to_cols_array_2d();
        let info = RenderInfo {
            world: identity,
            view_proj: identity,
        };

        let draw_triangle = StaticMesh {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };

        let draw_rectangle = StaticMesh {
            indices: 3..9,
            base_vertex: 3,
            instances: 1..2,
        };

        (info, vec![draw_triangle, draw_rectangle])
    }
}

fn main() {
    common::run_example(InstancesExample)
}
