#version 450

#define ROOT_3 1.73205080757f

layout(local_size_x = 8, local_size_y = 8, local_size_z = 1) in;

layout(set = 0, binding = 0, rgba8) uniform writeonly image2D img;

layout(push_constant) uniform PushConstants {
    float time;
} push_constants;

int get_index(in ivec2 pos) {
    const ivec2 dims = ivec2(imageSize(img));
    return pos.y * dims.x + pos.x;
}

float equalateral_triangle_signed_distance_function(in vec2 point, in float radius) {
    const float k = ROOT_3;
    point.x = abs(point.x) - radius;
    point.y = point.y + radius / k;

    if(point.x + k * point.y > 0.0) {
        point = vec2(point.x - k * point.y, -k * point.x - point.y) / 2.0;
    }

    point.x -= clamp(point.x, -2.0 * radius, 0.0);
    return -length(point) * sign(point.y);
}

vec3 palette(in float t) {
    const vec3 a = vec3(0.500f, 0.500f, 0.500f);
    const vec3 b = vec3(0.420f, 0.420f, 0.420f);
    const vec3 c = vec3(0.760f, 0.760f, 0.760f);
    const vec3 d = vec3(1.588f, 1.922f, 2.255f);
    return a + b * cos(6.28318f * (c*t+d));
}


void main() {
    const ivec2 pos = ivec2(gl_GlobalInvocationID.xy);
    const ivec2 dims = ivec2(imageSize(img));
    const vec2 centered_pos = (vec2(pos.xy) * 2.0f - vec2(dims.xy)) / float(dims.x);;

    const vec3 triangle_gradient_colour = palette(length(centered_pos) + push_constants.time);
    const float triangle_distance_length = 0.015f / abs(sin(equalateral_triangle_signed_distance_function(centered_pos, 1.0f) * 8.0f + push_constants.time) / 8.0f);

    const vec3 final_colour = triangle_gradient_colour * triangle_distance_length;
    imageStore(img, pos, vec4(final_colour, 1.0f));
}