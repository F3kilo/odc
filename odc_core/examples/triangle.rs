mod common;
mod models;

use common::Example;
use odc_core::{OdcCore, model::RenderModel};

struct Triangle;

impl Example for Triangle {
    fn render_model() -> RenderModel {
        models::color_mesh_model()
    }

    fn init(&mut self, renderer: &OdcCore) {
        // let material = renderer.create_material(&common::color_mesh_material_data().as_info());
        // renderer.insert_material(0, material);

        // let (vertex_data, index_data) = common::triangle_mesh();
        // renderer.write_vertices(vertex_data, 0);
        // renderer.write_indices(index_data, 0);

        // let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        // renderer.write_instances(&[ident_transform], 0);

        // let world = ident_transform;
        // let view_proj = ident_transform;
        // renderer.write_uniform(&[world, view_proj], 0);
    }

    fn update(&mut self, _renderer: &OdcCore) {}

    // fn draw_info(&self) -> Vec<(u64, Vec<DrawData>)> {
    //     let draw = DrawData {
    //         indices: 0..3,
    //         base_vertex: 0,
    //         instances: 0..1,
    //     };

    //     vec![(0, vec![draw])]
    // }
}

fn main() {
    common::run_example(Triangle)
}
