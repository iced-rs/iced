[[block]]
struct Globals {
    transform: mat4x4<f32>;
};

[[group(0), binding(0)]] var<uniform> globals: Globals;

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.color = input.color;
    out.position = globals.transform * vec4<f32>(input.position, 0.0, 1.0);

    return out;
}

[[stage(fragment)]]
fn fs_main(input: VertexOutput) -> [[location(0)]] vec4<f32> {
    return input.color;
}
