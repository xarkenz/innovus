@group(0) @binding(0)
var atlas_texture: texture_2d<f32>;
@group(0) @binding(1)
var atlas_sampler: sampler;

@group(1) @binding(0)
var<uniform> camera_view_proj: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.color = in.color;
    out.uv = in.uv / vec2<f32>(textureDimensions(atlas_texture));
    out.clip_position = camera_view_proj * vec4<f32>(in.position, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var out_color: vec4<f32> = in.color;

    out_color *= select(vec4<f32>(1.0), textureSample(atlas_texture, atlas_sampler, in.uv), all(in.uv == in.uv));
    if out_color.a <= 0.0 {
        discard;
    }

    return out_color;
}
