@group(0) @binding(0)
var atlas_texture: texture_2d<f32>;
@group(0) @binding(1)
var atlas_sampler: sampler;

@group(2) @binding(0)
var<uniform> offset_scale: vec2<f32>;

struct VertexInput {
    @location(0) anchor: vec2<f32>,
    @location(1) offset: vec2<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
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
    out.clip_position = vec4<f32>(
        (in.anchor * 2.0 - vec2<f32>(1.0) + in.offset * offset_scale) * vec2(1.0, -1.0),
        0.0,
        1.0,
    );

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
