#version 330 core

layout(location = 0) in vec2 vert_pos;
layout(location = 1) in vec3 vert_col;

out vec3 frag_col;

void main(void) {
    frag_col = vert_col;
    gl_Position = vec4(vert_pos, 0.0, 1.0);
}
