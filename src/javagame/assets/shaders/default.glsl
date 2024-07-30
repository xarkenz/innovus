#type vertex
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec4 aColor;
layout (location = 2) in vec2 aTexCoords;
layout (location = 3) in float aStatic;

uniform mat4 uWorldProj;
uniform mat4 uStaticProj;
uniform sampler2D uTexture;

out vec4 fColor;
out vec2 fTexCoords;

void main() {
    fColor = aColor;
    fTexCoords = aTexCoords / textureSize(uTexture, 0);
    gl_Position = (aStatic == 0 ? uWorldProj : uStaticProj) * vec4(aPos, 1);
}

#type fragment
#version 330 core

in vec4 fColor;
in vec2 fTexCoords;

uniform sampler2D uTexture;

out vec4 color;

void main() {
    color = fColor * texture(uTexture, fTexCoords);
}
