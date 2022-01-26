mod common;

use crate::common::{Example, InstanceInfo};
use bytemuck::Zeroable;
use glam::Mat4;
use odc::{Odc, RenderInfo, StaticMesh};

struct InstancesExample;

impl Example for InstancesExample {
    fn init(&mut self, renderer: &Odc) {
        let (vertex_data, index_data) = common::triangle_mesh();
        renderer.write_vertices(vertex_data, 0);
        renderer.write_indices(index_data, 0);

        let instances = get_instances();
        renderer.write_instances(&instances, 0);
    }

    fn update(&mut self, _renderer: &Odc) {}

    fn draw_info(&self) -> (RenderInfo, Vec<StaticMesh>) {
        let identity = Mat4::IDENTITY.to_cols_array_2d();
        let info = RenderInfo {
            world: identity,
            view_proj: identity,
        };

        let draw = StaticMesh {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..256,
        };

        (info, vec![draw])
    }
}

fn get_instances() -> [InstanceInfo; 256] {
    let mut instances = [InstanceInfo::zeroed(); 256];

    for i in 0..16 {
        for j in 0..16 {
            instances[i * 16 + j] = create_instance(i, j);
        }
    }

    instances
}

fn create_instance(x: usize, y: usize) -> InstanceInfo {
    let step = 2.0 / 16.0;
    let x = (x as f32 - 7.5) * step;
    let y = (y as f32 - 7.5) * step;
    let pos = glam::vec3(x, y, 0.0);
    let size = step / 2.5;
    let scale = glam::vec3(size, size, size);
    let transform = glam::Mat4::from_scale_rotation_translation(scale, glam::Quat::IDENTITY, pos)
        .to_cols_array_2d();
    InstanceInfo { transform }
}

fn main() {
    common::run_example(InstancesExample)
}
