struct LightInput {
    [[location(1)]] position: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coord: vec2<f32>;
    [[location(1)]] light_position: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_id: u32, light: LightInput) -> VertexOutput {
    let x = (vertex_id << u32(1)) & u32(2);
    let y = vertex_id & u32(2);
    let tex_coord = vec2<f32>(f32(x), f32(y));
    let position = vec4<f32>(tex_coord * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    return VertexOutput(position, tex_coord, light.position);
}

[[group(0), binding(0)]]
var position_map: texture_2d<f32>;

[[group(0), binding(2)]]
var color_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let pos: vec4<f32> = textureSample(position_map, color_sampler, in.tex_coord);

    let dist = distance(pos, in.light_position);
    let light = 2.0 / exp(4.0 * dist);
    // return in.light_position;
    // return color * light;
    return vec4<f32>(light);
}