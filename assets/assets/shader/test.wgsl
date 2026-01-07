
struct TimeUniforms {
    time: f32,
    _padding1: f32,
    _padding2: f32,
    _padding3: f32,
};

@group(0) @binding(0)
var<uniform> time_uniforms: TimeUniforms;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);

    let time = time_uniforms.time;
    let r = (sin(time * 1.0 + x * 3.14159) + 1.0) * 0.5;
    let g = (sin(time * 1.5 + y * 3.14159) + 1.0) * 0.5;
    let b = (sin(time * 2.0) + 1.0) * 0.5;

    out.color = vec3<f32>(r, g, b);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}
