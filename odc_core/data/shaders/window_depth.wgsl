[[group(0), binding(0)]]
var color_map: texture_depth_2d;

[[group(0), binding(1)]]
var color_sampler: sampler;

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] tex_coord: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] vertex_id: u32) -> VertexOutput {
    let x = (vertex_id << u32(1)) & u32(2);
    let y = vertex_id & u32(2);
    let tex_coord = vec2<f32>(f32(x), f32(y));
    let position = vec4<f32>(tex_coord * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0), 0.0, 1.0);
    return VertexOutput(position, tex_coord);
}


[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let depth = textureSample(color_map, color_sampler, in.tex_coord);
    
    return vec4<f32>(depth, 0.0, 0.0, 1.0);
}