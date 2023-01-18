#version 330 core

uniform mat4 camera_view;
uniform mat4 camera_proj;
uniform sampler2D tex_atlas;

layout(location = 0) in vec3 vertex_pos;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in float vertex_tex;
layout(location = 3) in vec2 vertex_uv;

out vec3 frag_pos;
out vec4 frag_color;
out float frag_tex;
out vec2 frag_uv;

void main() {
    frag_pos = vertex_pos;
    frag_color = vertex_color;
    frag_tex = vertex_tex;
    frag_uv = vertex_uv;

    gl_Position = camera_proj * camera_view * vec4(vertex.pos, 1.0);
}