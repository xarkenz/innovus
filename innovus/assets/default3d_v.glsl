#version 330 core

uniform float time;
uniform mat4 camera_view;
uniform mat4 camera_proj;

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in vec2 vertex_uv;
layout(location = 3) in vec3 vertex_normal;

out Vertex {
    vec3 position;
    vec4 color;
    vec2 uv;
    vec3 normal;
} vertex;

void main() {
    vertex.position = vertex_position;
    vertex.color = vertex_color;
    vertex.uv = vertex_uv;
    vertex.normal = vertex_normal;

    gl_Position = camera_proj * camera_view * vec4(vertex_position, 1.0);
}