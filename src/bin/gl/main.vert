#version 330 core

layout(location = 0) in vec2 vert_pos;
layout(location = 1) in vec3 vert_col;
layout(location = 2) in vec2 vert_tex;

out vec3 frag_col;
out vec2 frag_tex;

void main(void) {
    frag_col = vert_col;
    frag_tex = vert_tex;
    gl_Position = vec4(vert_pos, 0.0, 1.0);
}
