#version 430 core

//Task 1 ii in Assignment 2
in vec4 fragColor; // Input color from the vertex shader

out vec4 color;

void main()
{
    color = fragColor;
}