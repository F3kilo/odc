mod common;
mod models;

use common::Example;
use glam::Mat4;
use odc_core::{mdl::RenderModel, DrawData, OdcCore, StagePass, StagePasses};

struct Triangle;

impl Example for Triangle {
    fn render_model() -> RenderModel {
        models::color_mesh_model()
    }

    fn init(&mut self, renderer: &OdcCore) {
        let (vertex_data, index_data) = common::triangle_mesh();
        renderer.write_vertex(vertex_data, 0);
        renderer.write_index(index_data, 0);

        let ident = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_uniform(&[ident, ident], 0);
        renderer.write_instance(&[ident], 0);
    }

    fn update(&mut self, _renderer: &OdcCore) {}

    fn draw_info(&self) -> Vec<StagePasses> {
        let draw = DrawData {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };

        vec![&[StagePass {
            index: 0,
            pipelines: &[&[draw]],
        }]]
    }
}

fn main() {
    common::run_example(Triangle)
}
