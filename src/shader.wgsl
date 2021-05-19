[[stage(vertex)]]
fn vs_main([[builtin(vertex_index)]] in_vertex_index: u32) -> [[builtin(position)]] vec4<f32> {
    let points = array<f32, 6>(
        -1.0, -1.0,
         1.0, -1.0,
         0.0,  1.0,
    );


    let x = points[i32(in_vertex_index)*2 + 0];
    let y = points[i32(in_vertex_index)*2 + 1];
    return vec4<f32>(x, y, 0.0, 1.0);
}

[[stage(fragment)]]
fn fs_main() -> [[location(0)]] vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}

