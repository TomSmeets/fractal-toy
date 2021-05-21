[[stage(vertex)]]
fn vs_main([[location(0)]] pos: vec2<f32>) -> [[builtin(position)]] vec4<f32> {
    return vec4<f32>(pos.x, pos.y, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main([[builtin(position)]] pos: vec4<f32>) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(sin(pos.xy * 0.1)*0.5+0.5, 0.0, 1.0);
}

