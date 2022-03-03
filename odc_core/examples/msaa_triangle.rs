mod common;

use crate::common::DrawDataStorage;
use common::{mesh, Example};
use glam::Mat4;
use odc_core::mdl::Size2d;
use odc_core::{mdl::RenderModel, BufferType, DrawData, OdcCore};

struct MsaaTriangle;

impl Example for MsaaTriangle {
    fn render_model() -> RenderModel {
        common::models::msaa_color_mesh::msaa_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![(2, "color".into(), Size2d { x: 800, y: 600 })]
    }

    fn init(&mut self, renderer: &mut OdcCore) {
        let (vertex_data, index_data) = mesh::triangle_mesh();
        renderer.write_buffer(BufferType::Vertex, vertex_data, 0);
        renderer.write_buffer(BufferType::Index, index_data, 0);

        let ident = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_buffer(BufferType::Uniform, &[ident, ident], 0);
        renderer.write_buffer(BufferType::Instance, &[ident], 0);
    }

    fn update(&mut self, _renderer: &mut OdcCore) {}

    fn draw_data(&self) -> Vec<DrawDataStorage> {
        let draw = DrawData {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };

        vec![DrawDataStorage {
            pass: 0,
            pipeline: 0,
            data: vec![draw],
        }]
    }
}

fn main() {
    common::run_example(MsaaTriangle)
}
