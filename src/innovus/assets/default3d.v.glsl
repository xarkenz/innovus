#version 330 core

uniform float time;
uniform mat4 camera_view;
uniform mat4 camera_proj;
uniform sampler2D tex_atlas;

layout(location = 0) in vec3 vertex_pos;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in float vertex_tex;
layout(location = 3) in vec2 vertex_uv;
layout(location = 4) in vec3 vertex_norm;

out Vertex {
    vec3 pos;
    vec4 color;
    float tex;
    vec2 uv;
    vec3 norm;
} vertex;

void main() {
    vertex.pos = vertex_pos;
    vertex.color = vertex_color;
    vertex.tex = vertex_tex;
    vertex.uv = vertex_uv;
    vertex.norm = vertex_norm;

    gl_Position = camera_proj * camera_view * vec4(vertex.pos, 1.0);
}