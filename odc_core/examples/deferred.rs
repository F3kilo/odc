mod common;

use crate::common::{DrawDataTree, Example, mesh, models};
use glam::Mat4;
use odc_core::mdl::Size2d;
use odc_core::{DrawData, mdl::RenderModel, OdcCore, Pass, Stage};
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
        models::color_mesh::color_mesh_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![(0, "color".into(), Size2d { x: 800, y: 600 })]
    }

    fn init(&mut self, renderer: &OdcCore) {
        let (vertex_data, index_data) = mesh::rectangle_mesh();
        renderer.write_index(index_data, 0);
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
        vec![vec![Pass {
            index: 0,
            pipelines: vec![0],
        }]]
    }

    fn draw_data(&self) -> DrawDataTree {
        let draw = DrawData {
            indices: 0..6,
            base_vertex: 0,
            instances: 0..1,
        };

        DrawDataTree(vec![vec![vec![draw]]])
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
