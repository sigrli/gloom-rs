#version 430 core

in vec3 position; //Vertex position 
in vec4 color;   // Vertex color 
in vec4 normal_vec; //Vertex normal vector

out vec4 fragColor; // Output color to the fragment shader
out vec4 fragNormal; //Output

uniform mat4 identityM;

void main()
{
    gl_Position = identityM * vec4(position, 1.0f);

    // Pass the color to the fragment shader
    fragColor = color;
    fragNormal = normal_vec;
}