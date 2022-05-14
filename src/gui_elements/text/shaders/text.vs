#version 150

const mat4 INVERT_Y_AXIS = mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, -1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(0.0, 0.0, 0.0, 1.0)
);

uniform mat4 proj;
uniform mat3 window_transform;

in vec3 left_top;
in vec2 right_bottom;
in vec2 tex_left_top;
in vec2 tex_right_bottom;
in vec4 color;

out vec2 f_tex_pos;
out vec4 f_color;

// generate positional data based on vertex ID
void main() {
    vec3 pos = vec3(0.0, 0.0, 1.0);
    float left = left_top.x;
    float right = right_bottom.x;
    float top = left_top.y;
    float bottom = right_bottom.y;

    switch (gl_VertexID) {
        case 0:
            pos = window_transform * vec3(left, top, 1.0);
            f_tex_pos = tex_left_top;
            break;
        case 1:
            pos = window_transform * vec3(right, top, 1.0);
            f_tex_pos = vec2(tex_right_bottom.x, tex_left_top.y);
            break;
        case 2:
            pos = window_transform * vec3(left, bottom, 1.0);
            f_tex_pos = vec2(tex_left_top.x, tex_right_bottom.y);
            break;
        case 3:
            pos = window_transform * vec3(right, bottom, 1.0);
            f_tex_pos = tex_right_bottom;
            break;
    }

    f_color = color;
    gl_Position =  INVERT_Y_AXIS * proj * vec4(pos.xy, left_top.z, 1.0);
}
