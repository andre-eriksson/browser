struct VertexInput {
    @location(0) position: vec2<f32>, // Vertex position (x, y)
    @location(1) uv: vec2<f32>,       // Texture coordinates (u, v)
    @location(2) color: vec4<f32>,    // Vertex color (r, g, b, a)
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>, // Clip space position
    @location(0) frag_uv: vec2<f32>,             // Texture coordinates to pass to fragment
    @location(1) frag_color: vec4<f32>,          // Color to pass to fragment
}

struct Globals {
    screen_size: vec2<f32>,  // To convert pixel coords to clip space
    _padding: vec2<f32>,
};

@group(0) @binding(0) var<uniform> globals: Globals;
@group(1) @binding(0) var t_texture: texture_2d<f32>;
@group(1) @binding(1) var s_texture: sampler;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    // Convert position from pixel space to clip space
    let clip_x = (input.position.x / globals.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (input.position.y / globals.screen_size.y) * 2.0;

    output.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    output.frag_uv = input.uv;
    output.frag_color = input.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(t_texture, s_texture, input.frag_uv);

    // Text: texture_color.r is alpha (grayscale)
    // Image: texture_color is full RGBA

    return texture_color * input.frag_color;
}
