# ODC

Simple and fast render engine based on [wgpu](https://github.com/gfx-rs/wgpu) crate.

## Triangle example
```rust
struct Triangle;

impl Example for Triangle {
    fn render_model() -> RenderModel {
        common::models::color_mesh::color_mesh_model()
    }

    fn windows() -> Vec<(usize, String, Size2d)> {
        vec![(0, "color".into(), Size2d { x: 800, y: 600 })]
    }

    fn init(&mut self, renderer: &OdcCore) {
        let (vertex_data, index_data) = mesh::triangle_mesh();
        renderer.write_vertex(vertex_data, 0);
        renderer.write_index(index_data, 0);

        let ident = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_uniform(&[ident, ident], 0);
        renderer.write_instance(&[ident], 0);
    }

    fn update(&mut self, _renderer: &OdcCore) {}

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
    common::run_example(Triangle)
}
```

## Next steps
- Resource replacement.