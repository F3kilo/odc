mod common;
mod models;

use crate::common::{DrawDataTree, Example};
use glam::Mat4;
use odc_core::mdl::Size2d;
use odc_core::{mdl::RenderModel, DrawData, OdcCore, Pass, Stage};
use std::f32::consts::PI;
use std::time::Instant;
use vp_cam::{Camera, CameraBuilder, Vec3};

fn main() {
    let rotation = CameraMovement::default();
    let camera = create_camera();
    common::run_example(InstancesExample(camera, rotation))
}

struct InstancesExample(Camera, CameraMovement);

impl Example for InstancesExample {
    fn render_model() -> RenderModel {
        models::color_mesh_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![(0, "color".into(), Size2d { x: 800, y: 600 })]
    }

    fn init(&mut self, renderer: &OdcCore) {
        let (vertex_data, index_data) = common::triangle_mesh();
        renderer.write_index(index_data, 0);
        renderer.write_vertex(vertex_data, 0);

        let instances = get_instances();
        renderer.write_instance(&[instances], 0);
    }

    fn update(&mut self, renderer: &OdcCore) {
        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        let world = ident_transform;
        self.0.set_position(self.1.cam_position());
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
            indices: 0..3,
            base_vertex: 0,
            instances: 0..256,
        };

        DrawDataTree(vec![vec![vec![draw]]])
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
    pub const RADIUS: f32 = 1.0;

    pub fn cam_position(&self) -> Vec3 {
        let elapsed = (Instant::now() - self.start).as_secs_f32();
        let secs_per_cycle = 4.0;
        let angle = ((2.0 * PI * elapsed) / secs_per_cycle) % (2.0 * PI);

        let x = Self::RADIUS * angle.sin();
        let y = Self::RADIUS * -angle.cos();
        [x, y, -1.0]
    }
}
