struct RenderInfo {
    world: mat4x4<f32>;
    view_proj: mat4x4<f32>;
};

struct InstanceInfo {
    transform: array<mat4x4<f32>>;
};

[[group(0), binding(0)]]
var<uniform> render_info: RenderInfo;

[[group(1), binding(0)]]
var<storage> instance_info: InstanceInfo;

struct VertexOutput {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]] screen_pos: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vert_index: u32, [[builtin(instance_index)]] inst_index: u32) -> VertexOutput {
    let x = f32(i32(vert_index) - 1);
    let y = f32(i32(vert_index & 1u) * 2 - 1);
    let pos = vec4<f32>(x, y, 0.0, 1.0);
    let instance_transform = instance_info.transform[inst_index];
    return VertexOutput(render_info.view_proj * render_info.world * instance_transform * pos, pos);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let x = (in.screen_pos.x + 1.0) / 2.0;
    let y = (in.screen_pos.y + 1.0) / 2.0;
    return vec4<f32>(1.0 - x - y, y, x - y, 1.0);
}
