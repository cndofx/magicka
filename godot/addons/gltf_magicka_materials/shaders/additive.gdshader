shader_type spatial;
render_mode cull_disabled, blend_add, unshaded;

uniform sampler2D color_texture : source_color;
uniform bool color_texture_enabled = true;
uniform vec3 color_tint : source_color = vec3(1.0);
uniform bool vertex_color_enabled = true;

void vertex() {
	if (!vertex_color_enabled) {
		COLOR = vec4(1.0);
	}
}

void fragment() {
	vec4 color = texture(color_texture, UV) * vec4(color_tint, 1.0);
	ALBEDO = color.rgb;
	ALPHA = color.a;
}
