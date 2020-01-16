#version 450

layout (location = 0) in vec2 i_uv;

layout(set = 1, binding = 0) uniform texture2D u_tex;
layout(set = 1, binding = 1) uniform sampler s_tex;

layout (location = 0) out vec4 o_target;

void main() {
    o_target = texture(sampler2D(u_tex, s_tex), i_uv);
}
