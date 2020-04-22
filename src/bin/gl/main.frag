#version 330 core

in vec3 frag_col;

out vec4 out_col;

void main(void) {
    out_col = vec4(frag_col, 1.0);
}
