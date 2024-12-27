#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in float height;


void main() {
    gl_Position = vec4(position.x, height, position.y, 1.0);
}