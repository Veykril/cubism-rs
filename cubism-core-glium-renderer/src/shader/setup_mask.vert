#version 410

in vec4 in_position;
in vec2 in_texCoord;

uniform mat4 u_clipMatrix;

layout (location = 0) out vec2 out_texCoord;
layout (location = 1) out vec4 out_myPos;

void main() {
    gl_Position = u_clipMatrix * in_position;
    out_myPos = u_clipMatrix * in_position;
    out_texCoord = in_texCoord;
    out_texCoord.y = 1.0 - out_texCoord.y;
}
