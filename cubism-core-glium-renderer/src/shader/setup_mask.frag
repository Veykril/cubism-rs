#version 410

uniform sampler2D us_texture0;
uniform vec4 u_channelFlag;
uniform vec4 u_baseColor;

layout (location = 0) in vec2 in_texCoord;
layout (location = 1) in vec4 in_myPos;

void main() {
    float isInside = step(u_baseColor.x, in_myPos.x/in_myPos.w)
                        * step(u_baseColor.y, in_myPos.y/in_myPos.w)
                        * step(in_myPos.x/in_myPos.w, u_baseColor.z)
                        * step(in_myPos.y/in_myPos.w, u_baseColor.w);

    gl_FragColor = u_channelFlag * texture2D(us_texture0 , in_texCoord).a * isInside;
}