#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 in_position;
layout(location = 1) in vec2 in_tex_coord;

layout(location = 0) out struct dto
{
	vec2 tex_coord;
} out_dto;

void main()
{
	gl_Position = vec4(in_position, 1.0);

	out_dto.tex_coord = in_tex_coord;
}
