#version 330 core

uniform vec2 anchor;
uniform vec2 offset_scale;
uniform sampler2D tex_atlas;

layout(location = 0) in vec2 vertex_offset;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in vec2 vertex_uv;

out vec4 frag_color;
out vec2 frag_uv;

void main() {
    frag_color = vertex_color;
    frag_uv = vertex_uv / textureSize(tex_atlas, 0);

    gl_Position = vec4(anchor + vertex_offset * offset_scale, 0.0, 1.0);
}
