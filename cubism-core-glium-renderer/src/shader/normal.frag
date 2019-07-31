#version 410

layout (location = 0) in vec2 in_tex_coords;

uniform sampler2D us_tex0;
//uniform vec4 u_baseColor;

out vec4 Target0;

void main() {
    vec4 color = texture2D(us_tex0 , in_tex_coords);
    //color = color * u_baseColor;
    Target0 = vec4(color.rgb * color.a,  color.a);
}
