mod common;

use crate::common::Example;
use glam::Mat4;
use odc::{DrawData, Odc};

struct InstancesExample;

impl Example for InstancesExample {
    fn init(&mut self, renderer: &mut Odc) {
        let material = renderer.create_material(&common::color_mesh_material_data().as_info());
        renderer.insert_material(0, material);

        let (vertex_data, index_data) = common::triangle_mesh();
        renderer.write_vertices(vertex_data, 0);
        renderer.write_indices(index_data, 0);

        let instances = get_instances();
        renderer.write_instances(&[instances], 0);
    }

    fn update(&mut self, renderer: &Odc) {
        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        let world = ident_transform;
        let view_proj = ident_transform;
        renderer.write_uniform(&[world, view_proj], 0);
    }

    fn draw_info(&self) -> Vec<(u64, Vec<DrawData>)> {
        let draw = DrawData {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..256,
        };

        vec![(0, vec![draw])]
    }
}

fn get_instances() -> [Transform; 256] {
    let mut instances = [Transform::default(); 256];

    for i in 0..16 {
        for j in 0..16 {
            instances[i * 16 + j] = create_instance(i, j);
        }
    }

    instances
}

fn create_instance(x: usize, y: usize) -> Transform {
    let step = 2.0 / 16.0;
    let x = (x as f32 - 7.5) * step;
    let y = (y as f32 - 7.5) * step;
    let pos = glam::vec3(x, y, 0.0);
    let size = step / 2.5;
    let scale = glam::vec3(size, size, size);
    glam::Mat4::from_scale_rotation_translation(scale, glam::Quat::IDENTITY, pos).to_cols_array_2d()
}

type Transform = [[f32; 4]; 4];

fn main() {
    common::run_example(InstancesExample)
}
