struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) frag_color: vec4<f32>,
}

struct Globals {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
}

@group(0) @binding(0)
var<uniform> globals: Globals;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let clip_x = (input.position.x / globals.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (input.position.y / globals.screen_size.y) * 2.0;

    output.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    output.frag_color = input.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.frag_color;
}
