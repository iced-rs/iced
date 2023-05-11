@vertex
fn main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(1 - i32(in_vertex_index & 1u) * 2) * 0.5;
    return vec4<f32>(x, y, 0.0, 1.0);
}
