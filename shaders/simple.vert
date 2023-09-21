#version 430 core

in vec3 position; //Vertex position 
in vec4 color;   // Vertex color 

out vec4 fragColor; // Output color to the fragment shader

//Assignment 2 Task 3b
uniform float time;

// identity matrix
mat4 i_m = mat4(1.0);


 
void main()
{
    //modify one at a time 
    i_m[1][3] = time;

    gl_Position = i_m * vec4(position, 1.0f);
    // Pass the color to the fragment shader
    fragColor = color;
    
    //Task 2d
    //gl_Position = vec4(-position.x,-position.y,position.z, 1.0f);
}