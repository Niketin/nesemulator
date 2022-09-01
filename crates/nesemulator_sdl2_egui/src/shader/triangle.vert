#version 330 core

layout (location = 0) in vec2 a_pos;
layout (location = 1) in vec4 a_srgba;
layout (location = 2) in vec2 a_tex_coord;

out vec4 ourColor;
out vec2 TexCoord;

uniform vec2 screen_size;

// 0-1 linear  from  0-255 sRGB
vec3 linear_from_srgb(vec3 srgb) {
    bvec3 cutoff = lessThan(srgb, vec3(10.31475));
    vec3 lower = srgb / vec3(3294.6);
    vec3 higher = pow((srgb + vec3(14.025)) / vec3(269.025), vec3(2.4));
    return mix(higher, lower, vec3(cutoff));
}

vec4 linear_from_srgba(vec4 srgba) {
    return vec4(linear_from_srgb(srgba.rgb), srgba.a / 255.0);
}


void main()
{
    gl_Position = vec4(
            2.0 * a_pos.x / screen_size.x - 1.0,
            1.0 - 2.0 * a_pos.y / screen_size.y,
            0.0,
            1.0);
    ourColor = linear_from_srgba(a_srgba);
    TexCoord = a_tex_coord;
}

