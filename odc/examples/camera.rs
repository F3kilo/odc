mod common;

use crate::common::{Example, InstanceInfo};
use glam::Mat4;
use odc::{Odc, RenderInfo, StaticMesh};
use std::f32::consts::PI;
use std::time::Instant;
use vp_cam::{Camera, CameraBuilder, Vec3};

struct CameraExample(Camera, CameraMovement);

impl Example for CameraExample {
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

    fn update(&mut self, _renderer: &Odc) {
        self.0.set_position(self.1.cam_position());
    }

    fn draw_info(&self) -> (RenderInfo, Vec<StaticMesh>) {
        let world = Mat4::IDENTITY.to_cols_array_2d();

        let view_proj = self.0.view_proj_transform();
        let info = RenderInfo { world, view_proj };

        let draw = StaticMesh {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };

        (info, vec![draw])
    }
}

fn create_camera() -> Camera {
    let pos = [0.0, 0.0, -CameraMovement::RADIUS];
    let target = [0.0; 3];
    let up = [0.0, 1.0, 0.0];
    CameraBuilder::default()
        .look_at(pos, target, up)
        // .orthographic(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0)
        .perspective(PI / 2.0, 4.0 / 3.0, 0.1, Some(10.0))
        .build()
}

struct CameraMovement {
    start: Instant,
}

impl Default for CameraMovement {
    fn default() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl CameraMovement {
    pub const RADIUS: f32 = 4.0;

    pub fn cam_position(&self) -> Vec3 {
        let elapsed = (Instant::now() - self.start).as_secs_f32();
        let secs_per_cycle = 4.0;
        let angle = ((2.0 * PI * elapsed) / secs_per_cycle) % (2.0 * PI);

        let x = Self::RADIUS * angle.sin();
        let z = Self::RADIUS * -angle.cos();
        [x, 0.0, z]
    }
}

fn main() {
    let rotation = CameraMovement::default();
    let camera = create_camera();
    common::run_example(CameraExample(camera, rotation))
}
