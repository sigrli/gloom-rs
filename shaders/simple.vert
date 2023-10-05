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
   // gl_Position = identityM * vec4(position, 1.0f);
    gl_Position = MVP * vec4(position, 1.0f);


    // Pass the color to the fragment shader
    fragColor = color;

    //fragNormal = normal_vec;
    //fragNormal = mat3(identityM) * normal_vec;
    fragNormal = mat3(model) * normal_vec;

    
    // mat3 normalMatrix = mat3(model); // extract the upper 3x3 part of model matrix
    // vec3 modifiedNormal = normalize(normalMatrix * normal_vec); // multiply normal_vec with normal matrix to apply same rotation/scaling
    // vec4 position4 = MVP * vec4(position, 1.0); //normalize result
    // fragColor = vec4(modifiedNormal, 1.0); // pass modified normal to fragment shader
    // gl_Position = position4;


    
}