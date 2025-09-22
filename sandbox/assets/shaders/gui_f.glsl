#version 330 core

uniform sampler2D tex_atlas;

in vec4 frag_color;
in vec2 frag_uv;

out vec4 final_frag_color;

void main() {
    final_frag_color = frag_color;
    if (!any(isnan(frag_uv))) {
        final_frag_color *= texture(tex_atlas, frag_uv);
    }
    if (final_frag_color.a <= 0.0) {
        discard;
    }
}
