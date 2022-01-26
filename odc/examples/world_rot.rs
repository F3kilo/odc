mod common;

use crate::common::{InstanceInfo, Example};
use glam::{Mat4, Vec3};
use odc::{Odc, RenderInfo, StaticMesh, Transform};
use std::f32::consts::PI;
use std::time::Instant;

struct WorldRot(Rotation);

impl Example for WorldRot {
    fn init(&mut self, renderer: &Odc) {
        let (vertex_data, index_data) = common::triangle_mesh();
        renderer.write_vertices(vertex_data, 0);
        renderer.write_indices(index_data, 0);

        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        let instance = InstanceInfo {
            transform: ident_transform,
        };
        renderer.write_instances(&[instance], 0);
    }

    fn update(&mut self, _renderer: &Odc) {}

    fn draw_info(&self) -> (RenderInfo, Vec<StaticMesh>) {
        let view_proj = Mat4::IDENTITY.to_cols_array_2d();
        let info = RenderInfo {
            world: self.0.transform(),
            view_proj,
        };

        let draw = StaticMesh {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };
        
        (info, vec!(draw))
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
        let angle = (2.0 * PI * elapsed) % (2.0 * PI);
        let rotation = Mat4::from_rotation_z(angle);
        let scale = Mat4::from_scale(Vec3::new(0.5, 0.5, 0.5));
        (rotation * scale).to_cols_array_2d()
    }
}

fn main() {
    let rotation = Rotation::default();
    common::run_example(WorldRot(rotation))
}