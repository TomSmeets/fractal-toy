struct VertexOutput {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

fn mandel(p: vec2<f32>) -> f32 {
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
            return j;
        }

        z = vec2<f32>(
            z.x*z.x - z.y*z.y + p.x,
            z.x*z.y*2.0       + p.y
        );

        i = i + 1u;
    }

    return 0.0;
}

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos: vec2<f32>,
    [[location(1)]] uv:  vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    out.uv  = uv;
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let col = mandel(in.uv);
    return vec4<f32>(col, col, col, 1.0);
}
