#version 430 core

in vec3 position; //Vertex position 
in vec4 color;   // Vertex color 

out vec4 fragColor; // Output color to the fragment shader

uniform mat4 identityM;

void main()
{
    gl_Position = vec4(position, 1.0f) * identityM;

    // Pass the color to the fragment shader
    fragColor = color;
}