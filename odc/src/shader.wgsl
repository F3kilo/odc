struct VertexInput {
    [[location(0)]] position: vec4<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct RenderInfo {
    world: mat4x4<f32>;
    view_proj: mat4x4<f32>;
};

struct InstanceInfo {
    transform: array<mat4x4<f32>>;
};

[[group(0), binding(0)]]
var<uniform> render_info: RenderInfo;

[[group(0), binding(1)]]
var<storage> instance_info: InstanceInfo;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(vertex: VertexInput, [[builtin(instance_index)]] inst_index: u32) -> VertexOutput {
    let pos = vec4<f32>(vertex.position.xyz, 1.0);
    let instance_transform = instance_info.transform[inst_index];
    return VertexOutput(render_info.view_proj * render_info.world * instance_transform * pos, vertex.color);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color);
}
