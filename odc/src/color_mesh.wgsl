struct VertexInput {
    [[location(0)]] position: vec4<f32>;
    [[location(1)]] normal: vec4<f32>;
    [[location(2)]] color: vec4<f32>;
};

struct RenderInfo {
    world: mat4x4<f32>;
    view_proj: mat4x4<f32>;
};

struct InstanceInfo {
    transform: array<mat4x4<f32>>;
};

[[group(0), binding(0)]]
var<storage> instance_info: InstanceInfo;

[[group(1), binding(0)]]
var<uniform> render_info: RenderInfo;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] world_position: vec4<f32>;
    [[location(1)]] normal: vec4<f32>;
    [[location(2)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(vertex: VertexInput, [[builtin(instance_index)]] inst_index: u32) -> VertexOutput {
    let position = vec4<f32>(vertex.position.xyz, 1.0);
    let normal = vec4<f32>(vertex.normal.xyz, 1.0);
    
    let instance_transform = instance_info.transform[inst_index];
    let world_transform = render_info.world * instance_transform;
    let world_position = world_transform * position;
    let screen_position = render_info.view_proj * world_position;

    let world_normal = normalize(world_transform * vec4<f32>(normal.xyz, 0.0));
    
    return VertexOutput(screen_position, world_position, world_normal, vertex.color);
}

struct FragmentOutput {
    [[location(0)]] position: vec4<f32>;
    [[location(1)]] normal: vec4<f32>;
    [[location(2)]] albedo: vec4<f32>;
};

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> FragmentOutput {
    return FragmentOutput(in.world_position, in.normal, in.color);
}