#version 330 core

uniform vec3 ambient_color;
uniform vec3 pt_light_pos;
uniform vec3 pt_light_color;
uniform float pt_light_power;
uniform sampler2D tex_atlas;

in vec3 frag_position;
in vec4 frag_color;
in vec2 frag_uv;
in vec3 frag_normal;

out vec4 final_frag_color;

void main() {
    if (frag_color.a <= 0.0) {
        discard;
    }

    vec3 dir_to_light = normalize(pt_light_pos - frag_position);
    float normal_dot_light = dot(frag_normal, dir_to_light);
    vec3 light_color = ambient_color + pt_light_color * max(0.2 + (normal_dot_light >= 0.0 ? 0.8 : 0.2) * normal_dot_light, 0.0) * pt_light_power;
    final_frag_color = vec4(clamp(frag_color.rgb * light_color, vec3(0.0), vec3(1.0)), frag_color.a);

    if (!any(isnan(frag_uv))) {
        final_frag_color *= texture(tex_atlas, frag_uv);
    }

    if (final_frag_color.a <= 0.0) {
        discard;
    }
}
