struct Camera {
    @binding(0) view: mat4x4<f32>,
    @binding(1) proj: mat4x4<f32>,
}

struct Atlas {
    @binding(0) texture: texture_2d<f32>,
    @binding(1) texture_sampler: sampler,
}

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

@group(0) var atlas: Atlas;
@group(1) var<uniform> camera: Camera;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.color = in.color;
    out.uv = in.uv;
    out.clip_position = camera.proj * camera.view * vec4<f32>(in.position, 1.0);

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
