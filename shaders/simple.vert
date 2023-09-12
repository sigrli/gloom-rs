#version 430 core

in vec3 position; //Vertex position 
in vec4 color;   // Vertex color 

out vec4 fragColor; // Output color to the fragment shader


void main()
{
    gl_Position = vec4(position, 1.0f);

    // Task 1ii Assignment 2
    // Pass the color to the fragment shader
    fragColor = color;
    
    //Task 2d
    //gl_Position = vec4(-position.x,-position.y,position.z, 1.0f);
}