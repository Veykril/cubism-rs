#version 410

layout (location = 0) in vec2 out_tex_coords;

uniform sampler2D us_tex0;
//uniform vec4 u_baseColor;

layout (location = 0) out vec4 Target0;

void main() {
    vec4 color = texture(us_tex0, out_tex_coords);
    //color = color * u_baseColor;
    Target0 = vec4(color.rgb * color.a,  color.a);
}
