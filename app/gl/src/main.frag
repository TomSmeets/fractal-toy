#version 330 core

in vec3 frag_col;
in vec2 frag_tex;

out vec4 out_col;

uniform sampler2D tex;

void main(void) {
    vec3 col = texture(tex, frag_tex).rgb;
    col *= frag_col;
    out_col = vec4(col.rgb, 1.0);
}
