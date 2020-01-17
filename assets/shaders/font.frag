#version 330

in VS_OUTPUT {
    vec2 textCoord;
} IN;

out vec4 Color;
uniform sampler2D fontTexture;

void main()
{
	//Color = vec4(1,1,1,0);
	Color = vec4(1,1,1, texture2D(fontTexture, IN.textCoord).r);
}
