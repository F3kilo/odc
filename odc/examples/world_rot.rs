mod common;

use crate::common::Example;
use glam::{Mat4, Vec3};
use odc::{DrawData, Odc, Transform};
use std::f32::consts::PI;
use std::time::Instant;

struct WorldRot(Rotation);

impl Example for WorldRot {
    fn init(&mut self, renderer: &mut Odc) {
        let material = renderer.create_material(&common::color_mesh_material_data().as_info());
        renderer.insert_material(0, material);

        let (vertex_data, index_data) = common::triangle_mesh();
        renderer.write_vertices(vertex_data, 0);
        renderer.write_indices(index_data, 0);

        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_instances(&[ident_transform], 0);
    }

    fn update(&mut self, renderer: &Odc) {
        let world = self.0.transform();
        let view_proj = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_uniform(&[world, view_proj], 0);
    }

    fn draw_info(&self) -> Vec<(u64, Vec<DrawData>)> {
        let draw = DrawData {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };

        vec![(0, vec![draw])]
    }
}

struct Rotation {
    start: Instant,
}

impl Default for Rotation {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Rotation {
    pub fn transform(&self) -> Transform {
        let elapsed = (Instant::now() - self.start).as_secs_f32();
        let secs_per_cycle = 4.0;
        let angle = ((2.0 * PI * elapsed) / secs_per_cycle) % (2.0 * PI);

        let rotation = Mat4::from_rotation_z(angle);
        let scale = Mat4::from_scale(Vec3::new(0.5, 0.5, 0.5));
        (rotation * scale).to_cols_array_2d()
    }
}

fn main() {
    let rotation = Rotation::default();
    common::run_example(WorldRot(rotation))
}
