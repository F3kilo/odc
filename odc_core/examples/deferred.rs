mod common;

use crate::common::{mesh, models, DrawDataTree, Example};
use glam::Mat4;
use odc_core::mdl::Size2d;
use odc_core::{mdl::RenderModel, DrawData, OdcCore, Stage};
use std::f32::consts::PI;
use std::time::Instant;
use vp_cam::{Camera, CameraBuilder};

fn main() {
    let rotation = Rotation::default();
    let camera = create_camera();
    common::run_example(InstancesExample(camera, rotation))
}

struct InstancesExample(Camera, Rotation);

impl Example for InstancesExample {
    fn render_model() -> RenderModel {
        models::deferred::deferred_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![
            (0, "position".into(), Size2d { x: 800, y: 600 }),
            (1, "albedo".into(), Size2d { x: 800, y: 600 }),
            (2, "light".into(), Size2d { x: 800, y: 600 }),
        ]
    }

    fn init(&mut self, renderer: &OdcCore) {
        let triangle_indices = [0, 1, 2];
        renderer.write_index(&triangle_indices, 0);

        let (vertex_data, index_data) = mesh::rectangle_mesh();
        renderer.write_index(index_data, 3);
        renderer.write_vertex(vertex_data, 0);

        let instance = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_instance(&[instance], 0);
    }

    fn update(&mut self, renderer: &OdcCore) {
        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        let world = ident_transform;
        let view_proj = self.0.view_proj_transform();
        renderer.write_uniform(&[world, view_proj], 0);
    }

    fn draw_stages(&self) -> Vec<Stage> {
        vec![vec![0, 1]]
    }

    fn draw_data(&self) -> DrawDataTree {
        let rect = DrawData {
            indices: 3..9,
            base_vertex: 0,
            instances: 0..1,
        };

        let tri = DrawData {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };

        let deferred_pipeline_draws = vec![rect];
        let deferred_pass_draws = vec![deferred_pipeline_draws];

        let light_pipeline_draws = vec![tri];
        let light_pass_draws = vec![vec![], light_pipeline_draws];

        DrawDataTree(vec![deferred_pass_draws, light_pass_draws])
    }
}

fn create_camera() -> Camera {
    let pos = [0.0, 0.0, -2.0];
    let target = [0.0; 3];
    let up = [0.0, 1.0, 0.0];
    CameraBuilder::default()
        .look_at(pos, target, up)
        // .orthographic(-5.0, 5.0, -5.0, 5.0, -5.0, 5.0)
        .perspective(PI / 2.0, 4.0 / 3.0, 0.1, Some(10.0))
        .build()
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
    pub fn angle(&self) -> f32 {
        let elapsed = (Instant::now() - self.start).as_secs_f32();
        let secs_per_cycle = 4.0;
        ((2.0 * PI * elapsed) / secs_per_cycle) % (2.0 * PI)
    }
}
