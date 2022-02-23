struct VertexInput {
    [[location(0)]] position: vec4<f32>;
    [[location(1)]] uvs: vec4<f32>;
};

struct InstanceInput {
    [[location(2)]] transform_0: vec4<f32>;
    [[location(3)]] transform_1: vec4<f32>;
    [[location(4)]] transform_2: vec4<f32>;
    [[location(5)]] transform_3: vec4<f32>;
    [[location(6)]] uv_offset_scale: vec4<f32>;
};

struct RenderInfo {
    world: mat4x4<f32>;
    view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> render_info: RenderInfo;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uvs: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let position = vec4<f32>(vertex.position.xyz, 1.0);
    
    let instance_transform = mat4x4<f32>(
        instance.transform_0, 
        instance.transform_1, 
        instance.transform_2, 
        instance.transform_3
    );

    let world_transform = render_info.world * instance_transform;
    let world_position = world_transform * position;
    let screen_position = render_info.view_proj * world_position;

    let uvs = vertex.uvs.xy * instance.uv_offset_scale.zw + instance.uv_offset_scale.xy;

    return VertexOutput(screen_position, uvs);
}

[[group(0), binding(1)]]
var sprite: texture_2d<f32>;

[[group(0), binding(2)]]
var color_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color = textureSample(sprite, color_sampler, in.uvs);
    return color;
}