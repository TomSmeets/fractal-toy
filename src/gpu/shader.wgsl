[[group(0), binding(0)]]
var texture: texture_2d<f32>;

[[group(0), binding(1)]]
var sampler: sampler;


struct VertexOutput {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]]       uv: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos: vec2<f32>,
    [[location(1)]] uv: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(pos.xy*0.5, 0.0, 1.0);
    out.uv = uv;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let col = textureSample(texture, sampler, in.uv.xy);
    return vec4<f32>(col);
}
