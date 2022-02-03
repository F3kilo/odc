# ODC

Simple and fast render engine based on [wgpu](https://github.com/gfx-rs/wgpu) crate.

## Triangle example
```rust
struct Triangle;

impl Example for Triangle {
    fn render_model() -> RenderModel {
        models::color_mesh_model()
    }

    fn init(&mut self, renderer: &OdcCore) {
        let (vertex_data, index_data) = common::triangle_mesh();
        renderer.write_buffer("vertex", vertex_data, 0);
        renderer.write_buffer("index", index_data, 0);

        let ident = Mat4::IDENTITY.to_cols_array_2d();
        renderer.write_buffer("uniform", &[ident, ident], 0);
        renderer.write_buffer("instance", &[ident], 0);
    }

    fn update(&mut self, _renderer: &OdcCore) {}

    fn draw_info(&self) -> (Vec<DrawData>, Vec<Range<usize>>) {
        let draw = DrawData {
            indices: 0..3,
            base_vertex: 0,
            instances: 0..1,
        };

        (vec![draw], vec![0..1])
    }
}

fn main() {
    common::run_example(Triangle)
}
```

## Next steps
- Resize window