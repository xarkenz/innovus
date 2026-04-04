struct GuiParams {
    @binding(0) offset_scale: vec2<f32>,
}

struct Atlas {
    @binding(0) texture: texture_2d<f32>,
    @binding(1) texture_sampler: sampler,
}

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

@group(0) var atlas: Atlas;
@group(2) var<uniform> params: GuiParams;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.color = in.color;
    out.uv = in.uv;
    out.clip_position = vec4<f32>(
        (in.anchor * 2.0 - vec2<f32>(1.0) + in.offset * params.offset_scale) * vec2(1.0, -1.0),
        0.0,
        1.0,
    );

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var out_color: vec4<f32> = in.color;

    out_color *= select(vec4<f32>(1.0), textureSample(atlas.texture, atlas.texture_sampler, in.uv), all(in.uv == in.uv));
    if out_color.a <= 0.0 {
        discard;
    }

    return out_color;
}
