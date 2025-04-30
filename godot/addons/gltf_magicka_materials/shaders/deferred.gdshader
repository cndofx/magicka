shader_type spatial;

uniform vec3 diffuse_color : source_color;
uniform sampler2D diffuse_texture : source_color;
uniform sampler2D spg_texture;
uniform float specular_amount = 1.0;
uniform float emissive_amount = 1.0;
uniform bool vertex_color_enabled = true;

void vertex() {
	if (!vertex_color_enabled) {
		COLOR = vec4(1.0);
	}
}

void fragment() {
	vec4 diffuse = texture(diffuse_texture, UV) * vec4(diffuse_color, 1.0);
	vec4 spg = texture(spg_texture, UV);
	ALBEDO = diffuse.rgb;
	EMISSION = spg.rrr * diffuse.rgb * emissive_amount;
	SPECULAR = spg.g * specular_amount; // TODO: is specular_amount set correctly?
	//SPECULAR = spg.g;
}
