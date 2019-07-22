#version 330

in vec2 v_tex_coord;

uniform sampler2D u_tex0;

out vec4 Target0;

void main() {
    Target0 = texture(u_tex0, v_tex_coord);
}
