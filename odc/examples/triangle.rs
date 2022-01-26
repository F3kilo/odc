mod common;

use crate::common::{InstanceInfo, Example};
use glam::Mat4;
use odc::{Odc, RenderInfo, StaticMesh};

struct Triangle;

impl Example for Triangle {
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
        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        let info = RenderInfo {
            world: ident_transform,
            view_proj: ident_transform,
        };

        let draw = StaticMesh {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };
        
        (info, vec!(draw))
    }
}

fn main() {
    common::run_example(Triangle)
}