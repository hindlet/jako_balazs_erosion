#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in float height;



layout(set = 0, binding = 0) uniform Data {
    // mat4 world;
    mat4 view;
    mat4 proj;
} uniforms;


void main() {
    gl_Position = uniforms.view * uniforms.proj * vec4(position.x, height, position.y, 1.0);
}