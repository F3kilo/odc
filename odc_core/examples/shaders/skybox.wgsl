struct VertexInput {
    [[location(0)]] position: vec4<f32>;
};

struct RenderInfo {
    world: mat4x4<f32>;
    view_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var<uniform> render_info: RenderInfo;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] uvs: vec3<f32>;
};

[[stage(vertex)]]
fn vs_main(vertex: VertexInput) -> VertexOutput {
    let position = vec4<f32>(vertex.position.xyz, 1.0);

    let world_position = render_info.world * position;
    let screen_position = render_info.view_proj * world_position;

    return VertexOutput(screen_position, position.xyz);
}

[[group(0), binding(1)]]
var skybox_tex: texture_cube<f32>;

[[group(0), binding(2)]]
var color_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    // let color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
    let color = textureSample(skybox_tex, color_sampler, in.uvs);
    return color;
}