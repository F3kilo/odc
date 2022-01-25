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

[[group(0), binding(0)]]
var position: texture_2d<f32>;

[[group(0), binding(1)]]
var albedo: texture_2d<f32>;

[[group(0), binding(2)]]
var depth: texture_depth_2d;

[[group(0), binding(3)]]
var color_sampler: sampler;

[[group(0), binding(4)]]
var depth_sampler: sampler_comparison;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let pos: vec4<f32> = textureSample(position, color_sampler, in.tex_coord);
    let color: vec4<f32> = textureSample(albedo, color_sampler, in.tex_coord);
    let depth = textureSampleCompare(depth, depth_sampler, in.tex_coord, in.position.w);
    
    return color;
    // return vec4<f32>(in.tex_coord, 0.0, 1.0);
    // return vec4<f32>(pos.xy / 800.0, 0.0, 1.0);
    // return vec4<f32>(depth, 0.0, 0.0, 1.0);
}
