# ODC

Simple and fast render engine based on [wgpu](https://github.com/gfx-rs/wgpu) crate.

## Triangle example
```rust
mod common;

use crate::common::Example;
use glam::Mat4;
use odc::{DrawData, Odc};

struct Triangle;

impl Example for Triangle {
    fn init(&mut self, renderer: &mut Odc) {
        let material = renderer.create_material(&common::color_mesh_material_data().as_info());
        renderer.insert_material(0, material);

        let (vertex_data, index_data) = common::triangle_mesh();
        renderer.write_vertices(vertex_data, 0);
        renderer.write_indices(index_data, 0);

        let ident_transform = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_instances(&[ident_transform], 0);

        let world = ident_transform;
        let view_proj = ident_transform;
        renderer.write_uniform(&[world, view_proj], 0);
    }

    fn update(&mut self, _renderer: &Odc) {}

    fn draw_info(&self) -> Vec<(u64, Vec<DrawData>)> {
        let draw = DrawData {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };

        vec![(0, vec![draw])]
    }
}

fn main() {
    common::run_example(Triangle)
}
```

## Next steps
- Fix examples
- Depth buffer
- Resize window