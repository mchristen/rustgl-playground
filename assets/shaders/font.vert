#version 330

layout (location = 0) in vec4 position;
layout (location = 1) in vec2 texture;

out VS_OUTPUT {
    vec2 textCoord;
} OUT;

uniform mat4 cameraToClipMatrix;
uniform mat4 modelToCameraMatrix;

void main()
{
	OUT.textCoord = texture;
	gl_Position = cameraToClipMatrix * (modelToCameraMatrix * position);
}
