struct RenderInfo {
    world: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> render_info: RenderInfo;

struct VertexOutput {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]] screen_pos: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> VertexOutput {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    let pos = vec4<f32>(x, y, 0.0, 1.0);
    return VertexOutput(render_info.world * pos, pos);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let x = (in.screen_pos.x + 1.0) / 2.0;
    let y = (in.screen_pos.y + 1.0) / 2.0;
    return vec4<f32>(1.0 - x - y, y, x - y, 1.0);
}
