#version 330 core

layout(location = 0) in vec2 vert_pos;
layout(location = 1) in vec3 vert_col;

out vec3 frag_col;

void main(void) {
    frag_col = vert_col;
    vec2 p = vert_pos * 2.0f - 1.0f;
    gl_Position = vec4(p, 0.0, 1.0);
}
