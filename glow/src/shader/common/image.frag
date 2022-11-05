#ifdef GL_ES
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
#endif

uniform sampler2D tex;
in vec2 tex_pos;

#ifdef HIGHER_THAN_300
out vec4 fragColor;
#define gl_FragColor fragColor
#endif
#ifdef GL_ES
#define texture texture2D
#endif

void main() {
    gl_FragColor = texture(tex, tex_pos);
}
