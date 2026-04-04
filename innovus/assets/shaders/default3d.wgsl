struct Camera {
    @binding(0) view_proj: mat4x4<f32>,
}

struct Atlas {
    @binding(0) texture: texture_2d<f32>,
    @binding(1) texture_sampler: sampler,
}

struct Lighting {
    @binding(0) ambient_color: vec3<f32>,
    @binding(1) point_light_position: vec3<f32>,
    @binding(2) point_light_color: vec3<f32>,
    @binding(3) point_light_power: f32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) normal: vec3<f32>,
}

@group(0) var atlas: Atlas;
@group(1) var<uniform> camera: Camera;
@group(2) var<uniform> lighting: Lighting;

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    out.position = in.position;
    out.color = in.color;
    out.uv = in.uv;
    out.normal = in.normal;
    out.clip_position = camera.view_proj * vec4<f32>(in.position, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var out_color: vec4<f32>;

    let dir_to_light: vec3<f32> = normalize(lighting.point_light_position - in.position);
    let normal_dot_light: vec3<f32> = dot(in.normal, dir_to_light);
    let light_received: f32 = max(0.0, 0.2 + select(0.2, 0.8, normal_dot_light >= 0.0) * normal_dot_light) * lighting.point_light_power;
    let light_color: vec3<f32> = lighting.ambient_color + lighting.point_light_color * light_received;
    out_color = vec4<f32>(clamp(in.color.rgb * light_color, vec3<f32>(0.0), vec3<f32>(1.0)), in.color.a);

    out_color *= select(vec4<f32>(1.0), textureSample(atlas.texture, atlas.texture_sampler, in.uv), all(in.uv == in.uv));
    if out_color.a <= 0.0 {
        discard;
    }

    return out_color;
}
