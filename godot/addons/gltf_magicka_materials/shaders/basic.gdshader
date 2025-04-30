shader_type spatial;
render_mode cull_disabled; // TODO: is this needed?

uniform sampler2D diffuse_texture : source_color;
uniform vec3 diffuse_color : source_color;
uniform vec3 emissive_color : source_color;
uniform vec3 specular_color: source_color;
uniform float specular_power = 1.0;
uniform float alpha = 1.0;
uniform bool vertex_color_enabled = true;

void vertex() {
	if (!vertex_color_enabled) {
		COLOR = vec4(1.0);
	}
}

void fragment() {
	vec4 diffuse = texture(diffuse_texture, UV) * vec4(diffuse_color, 1.0);
	ALBEDO = diffuse.rgb;
	ALPHA = diffuse.a * alpha;
	EMISSION = diffuse.rgb * emissive_color;
	// TODO: specular
}
