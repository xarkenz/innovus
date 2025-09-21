#version 330 core

uniform mat4 camera_view;
uniform mat4 camera_proj;
uniform sampler2D tex_atlas;

layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in vec2 vertex_uv;

out vec4 frag_color;
out vec2 frag_uv;

void main() {
    frag_color = vertex_color;
    frag_uv = vertex_uv / textureSize(tex_atlas, 0);

    gl_Position = camera_proj * camera_view * vec4(vertex_position, 1.0);
}
