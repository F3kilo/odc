struct VertexInput {
    [[location(0)]] position: vec4<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct InstanceInput {
    [[location(2)]] transform_0: vec4<f32>;
    [[location(3)]] transform_1: vec4<f32>;
    [[location(4)]] transform_2: vec4<f32>;
    [[location(5)]] transform_3: vec4<f32>;
};

struct RenderInfo {
    world: mat4x4<f32>;
    view_proj: mat4x4<f32>;
};


[[group(0), binding(0)]]
var<uniform> render_info: RenderInfo;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let pos = vec4<f32>(vertex.position.xyz, 1.0);
    let instance_transform = mat4x4<f32>(
        instance.transform_0, 
        instance.transform_1, 
        instance.transform_2, 
        instance.transform_3
    );
    return VertexOutput(render_info.view_proj * render_info.world * instance_transform * pos, vertex.color);
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color);
}
