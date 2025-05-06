shader_type spatial;
render_mode depth_prepass_alpha;

uniform sampler2D diffuse_texture : source_color;
uniform sampler2D damage_texture : source_color;
uniform sampler2D normal_texture;
uniform sampler2D normal_damage_texture;
uniform sampler2D material_texture;
uniform sampler2D spg_texture;
uniform vec3 diffuse_color : source_color;
uniform float emissive_amount = 1.0;
uniform float specular_amount = 1.0;
uniform float specular_power = 1.0;
uniform float normal_power = 1.0;
uniform bool damage_enabled = false;

void fragment() {
	vec4 diffuse = texture(diffuse_texture, UV);
	float base_alpha = diffuse.a;
	vec4 damage = texture(damage_texture, UV);
	vec4 normal = texture(normal_texture, UV);
	vec4 normal_damage = texture(normal_damage_texture, UV);
	vec4 spg = texture(spg_texture, UV);
	if (damage_enabled) {
		diffuse = mix(diffuse, damage, damage.a);
	}
	diffuse = diffuse * vec4(diffuse_color, 1.0);

	ALBEDO = diffuse.rgb;
	ALPHA = base_alpha;
	EMISSION = diffuse.rgb * spg.r * emissive_amount;
	SPECULAR = spg.g * specular_amount;
	NORMAL_MAP = normal.rgb;
}
