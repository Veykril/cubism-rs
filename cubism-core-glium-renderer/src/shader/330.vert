#version 330

in vec2 a_pos;
in vec2 a_tex_coords;

uniform mat4 u_mvp;

out vec2 v_tex_coord;

void main() {
    v_tex_coord = a_tex_coords;
    gl_Position = vec4(a_pos, 0.0, 1.0) * u_mvp;
}
