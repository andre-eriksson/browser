struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) frag_uv: vec2<f32>,
    @location(1) frag_color: vec4<f32>,
}

struct Globals {
    screen_size: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var t_texture: texture_2d<f32>;
@group(1) @binding(1) var s_texture: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let clip_x = (input.position.x / globals.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (input.position.y / globals.screen_size.y) * 2.0;

    output.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    output.frag_uv = input.uv;
    output.frag_color = input.color;

    return output;
}

@fragment
fn fs_image(input: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(t_texture, s_texture, input.frag_uv);

    return texture_color * input.frag_color;
}

@fragment
fn fs_text(input: VertexOutput) -> @location(0) vec4<f32> {
    let glyph_alpha = textureSample(t_texture, s_texture, input.frag_uv).r;
    let final_alpha = glyph_alpha * input.frag_color.a;

    return vec4<f32>(
        input.frag_color.rgb * final_alpha,
        final_alpha
    );
}
