#type vertex
#version 330 core

layout (location = 0) in vec2 aPos;
layout (location = 1) in vec3 aColor;
layout (location = 2) in float aStatic;

out vec3 fColor;

uniform mat4 uView;
uniform mat4 uProjection;
uniform mat4 uStaticProjection;

void main() {
    fColor = aColor;
    gl_Position = (aStatic == 0 ? uProjection * uView : uStaticProjection) * vec4(aPos, -10, 1);
}

#type fragment
#version 330 core

in vec3 fColor;

out vec4 color;

void main() {
    color = vec4(fColor, 1);
}
