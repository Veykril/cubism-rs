#version 410

layout (location = 0) in vec2 in_pos;
layout (location = 1) in vec2 in_tex_coords;

uniform mat4 u_mvp;

layout (location = 0) out vec2 out_tex_coords;

void main() {
    gl_Position = u_mvp * vec4(in_pos, 0.0, 1.0);
    out_tex_coords = in_tex_coords;
}
