#version 330 core

uniform sampler2D tex_atlas;

in vec3 frag_pos;
in vec4 frag_color;
in float frag_tex;
in vec2 frag_uv;

out vec4 final_frag_color;

void main() {
    final_frag_color = frag_color;
    if (bool(frag_tex)) {
        final_frag_color *= texture(tex_atlas, frag_uv);
    }
}