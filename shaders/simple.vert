#version 430 core

in vec3 position; //Vertex position 
in vec4 color;   // Vertex color 
in vec3 normal_vec; //Vertex normal vector

out vec4 fragColor; // Output color to the fragment shader
out vec3 fragNormal; //Output

//uniform mat4 identityM; // identity matrix

// Task 5b)
uniform mat4 MVP; //mvp matrix;
uniform mat4 model; //model matrix

void main()
{

    gl_Position = MVP * vec4(position, 1.0);
    fragColor = color;
    mat3 model_matrix_3 = mat3(model);
    vec3 modified_matrix = model_matrix_3 * normal_vec;
    vec3 normalized_modified_matrix = normalize(modified_matrix);
    fragNormal = normalized_modified_matrix; 
  
}