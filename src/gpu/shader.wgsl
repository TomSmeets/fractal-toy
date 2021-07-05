struct VertexOutput {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]]       uv: vec2<f32>;
    [[location(1)]]       ix: i32;
};

[[block]]
struct UniformData {
    resolution: vec2<f32>;
};

[[group(0), binding(0)]]
var texture: texture_2d_array<f32>;

[[group(0), binding(1)]]
var sampler: sampler;

[[group(0), binding(2)]]
var uniform: UniformData;

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos: vec2<f32>,
    [[location(1)]] uv: vec2<f32>,
    [[location(2)]] ix: i32
) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(pos.xy * 1.0 / uniform.resolution * 2.0 - 1.0, 0.0, 1.0);
    out.pos.y = out.pos.y * -1.0;
    out.uv = uv;
    out.ix = ix;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let col = textureSample(texture, sampler, in.uv, in.ix);
    return col;
}
