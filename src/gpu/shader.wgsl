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


fn mandel(p: vec2<f32>) -> vec2<f32> {
    var z: vec2<f32> = p;

    var i: u32 = 0u;
    var d: f32 = 0.0;

    loop {
        if (i > 32u) {
            break;
        }

        d = z.x*z.x + z.y*z.y;
        if (d > 1024.0) {
            let j = f32(i) - log2(log2(d)) + 4.0;
            return vec2<f32>(j, j / 32.0);
        }

        z = vec2<f32>(
            z.x*z.x - z.y*z.y + p.x,
            z.x*z.y*2.0       + p.y
        );

        i = i + 1u;
    }

    return vec2<f32>(0.0, 1.0);
}

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
    let v = mandel((in.uv.xy*2.0 - 1.0)*1.5);
    let t = v.x*0.1;
    let y = v.y;

    let lo = 0.0;
    let hi = 1.0;

    let pi     = 3.141592653;
    let pi_1_3 = pi / 3.0;
    let pi_2_3 = pi * 2.0 / 3.0;

    let x = (0.5 - t) * pi;

    let r = sin(x);
    let g = sin(x+pi_1_3);
    let b = sin(x+pi_2_3);

    let r = r*r;
    let g = g*g;
    let b = b*b;

    let r = lo*(1.0-r) + hi*r;
    let g = lo*(1.0-g) + hi*g;
    let b = lo*(1.0-b) + hi*b;

    let col = textureSample(texture, sampler, in.uv, in.ix);
    return vec4<f32>(col.rgb, 1.0);
}
