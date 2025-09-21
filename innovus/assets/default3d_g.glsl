#version 330 core

uniform vec3 camera_pos;
uniform vec3 ambient_color;
uniform vec3 pt_light_pos;
uniform vec3 pt_light_color;
uniform float pt_light_power;

layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

in Vertex {
    vec3 position;
    vec4 color;
    vec2 uv;
    vec3 normal;
} tri[];

out vec3 frag_position;
out vec4 frag_color;
out vec2 frag_uv;
out vec3 frag_normal;

void main() {
    vec3 face_normal = normalize(cross(tri[1].position - tri[0].position, tri[2].position - tri[0].position));

    gl_Position = gl_in[0].gl_Position;
    frag_position = tri[0].position;
    frag_color = tri[0].color;
    frag_uv = tri[0].uv;
    frag_normal = bool(tri[0].normal) ? normalize(tri[0].normal) : face_normal;
    EmitVertex();

    gl_Position = gl_in[1].gl_Position;
    frag_position = tri[1].position;
    frag_color = tri[1].color;
    frag_uv = tri[1].uv;
    frag_normal = bool(tri[1].normal) ? normalize(tri[1].normal) : face_normal;
    EmitVertex();

    gl_Position = gl_in[2].gl_Position;
    frag_position = tri[2].position;
    frag_color = tri[2].color;
    frag_uv = tri[2].uv;
    frag_normal = bool(tri[2].normal) ? normalize(tri[2].normal) : face_normal;
    EmitVertex();

    EndPrimitive();
}
