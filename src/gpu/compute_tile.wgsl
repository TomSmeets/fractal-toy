struct VertexOutput {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]] uv: vec2<f32>;
};

fn mandel(p: vec2<f32>) -> f32 {
    var z: vec2<f32> = p;

    var i: f32 = 0.0;

    loop {
        if (i > 1024.0) {
            break;
        }

        z = vec2<f32>(
            z.x*z.x - z.y*z.y + p.x,
            z.x*z.y*2.0       + p.y
        );

        let d = z.x*z.x + z.y*z.y;
        if (d > 256.0) {
            i = i - log2(log2(d)) + 4.0;
            break;
        }

        i = i + 1.0;
    }

    return i;
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
    let t0 = mandel(in.uv);

    let pi = 3.1415926531;
    let a = (1.0 - ((t0*t0) / (1024.0*1024.0)));
    let a = max(min(a, 1.0), 0.0);
    let t = t0 * 0.005;
    let r = a * sin((0.5 - t) * pi + pi * 0.0 / 3.0);
    let g = a * sin((0.5 - t) * pi + pi * 1.0 / 3.0);
    let b = a * sin((0.5 - t) * pi + pi * 2.0 / 3.0);

    let r = r * r;
    let g = g * g;
    let b = b * b;

    let r = r * r;
    let g = g * g;
    let b = b * b;

    return vec4<f32>(b, g, r, 1.0);
}
