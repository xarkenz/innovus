#version 330 core

uniform vec3 camera_pos;
uniform vec3 ambient_color;
uniform vec3 pt_light_pos;
uniform vec3 pt_light_color;
uniform float pt_light_power;

layout (triangles) in;
layout (triangle_strip, max_vertices = 3) out;

in Vertex {
    vec3 pos;
    vec4 color;
    vec2 uv;
    vec3 norm;
} tri[];

out vec3 frag_pos;
out vec4 frag_color;
out vec2 frag_uv;
out vec3 frag_norm;

void main() {
    vec3 face_normal = normalize(cross(tri[1].pos - tri[0].pos, tri[2].pos - tri[0].pos));

    gl_Position = gl_in[0].gl_Position;
    frag_pos = tri[0].pos;
    frag_color = tri[0].color;
    frag_uv = tri[0].uv;
    frag_norm = bool(tri[0].norm) ? normalize(tri[0].norm) : face_normal;
    EmitVertex();

    gl_Position = gl_in[1].gl_Position;
    frag_pos = tri[1].pos;
    frag_color = tri[1].color;
    frag_uv = tri[1].uv;
    frag_norm = bool(tri[1].norm) ? normalize(tri[1].norm) : face_normal;
    EmitVertex();

    gl_Position = gl_in[2].gl_Position;
    frag_pos = tri[2].pos;
    frag_color = tri[2].color;
    frag_uv = tri[2].uv;
    frag_norm = bool(tri[2].norm) ? normalize(tri[2].norm) : face_normal;
    EmitVertex();

    EndPrimitive();
}
