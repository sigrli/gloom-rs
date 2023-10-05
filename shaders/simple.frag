#version 430 core

//Task 1 ii in Assignment 2
in vec4 fragColor; // Input color from the vertex shader
in vec3 fragNormal;

out vec4 color;

uniform vec3 lightDirection = normalize(vec3(0.8, -0.5, 0.6));

void main()
{
    // Lambertian lighting model
    //float diffuseFactor = max(0.0, dot(normalize(fragNormal.xyz), -lightDirection));
    float diffuseFactor = max(0.0, dot(normalize(fragNormal), -lightDirection)); //removed xyz

    vec3 finalColor = fragColor.xyz * diffuseFactor;

    color = vec4(finalColor, fragColor.a);
}