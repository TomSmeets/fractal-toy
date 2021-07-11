struct VertexOutput {
    [[builtin(position)]] pos: vec4<f32>;
    [[location(0)]] uv: vec2<REAL>;
};

fn cpx_sqr(z: vec2<REAL>) -> vec2<REAL> {
    return vec2<REAL>(
        z.x*z.x - z.y*z.y,
        z.x*z.y*REAL(2.0)
    );
}

fn cpx_cube(z: vec2<REAL>) -> vec2<REAL> {
    return vec2<REAL>(
        z.x*z.x*z.x - REAL(3.0)*z.x*z.y*z.y,
        REAL(3.0)*z.x*z.x*z.y - z.y*z.y*z.y
    );
}

fn cpx_abs(z: vec2<REAL>) -> vec2<REAL> {
    return vec2<REAL>(
        abs(z.x),
        abs(z.y),
    );
}

fn mandel(c: vec2<REAL>) -> REAL {
    var z: vec2<REAL> = vec2<REAL>(0.0, 0.0);

    var i: u32 = 0u;
    var t: REAL = REAL(0.0);

    loop {
        if (i >= 1024u) {
            break;
        }

        @IMPL@

        let d = z.x*z.x + z.y*z.y;
        if (d > REAL(256.0)) {
            t = t - log2(log2(d)) + REAL(4.0);
            break;
        }

        i = i + 1u;
    }

    return t;
}

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] pos: vec2<f32>,
    [[location(1)]] uv:  vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.pos = vec4<f32>(pos, 0.0, 1.0);
    out.uv.x  = REAL(uv.x);
    out.uv.y  = REAL(uv.y);
    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let t0 = f32(mandel(in.uv));

    let pi_3 = 1.04719755119659774615421446109316763;
    let a = (1.0 - ((t0*t0) / (1024.0*1024.0)));
    let a = max(min(a, 1.0), 0.0);
    let t = t0 * 0.005;
    let r = a * sin((0.5 - t) * pi_3 * 3.0 + pi_3 * 0.0);
    let g = a * sin((0.5 - t) * pi_3 * 3.0 + pi_3 * 1.0);
    let b = a * sin((0.5 - t) * pi_3 * 3.0 + pi_3 * 2.0);

    let r = r * r;
    let g = g * g;
    let b = b * b;

    // FIXME: Gamma correction, are we in srgb or linear space now?
    let r = r * r;
    let g = g * g;
    let b = b * b;

    // FIXME: images are RGBA, but our target is BGRA, so we swap A and R here to make it work kind of
    return vec4<f32>(b, g, r, 1.0);
}
