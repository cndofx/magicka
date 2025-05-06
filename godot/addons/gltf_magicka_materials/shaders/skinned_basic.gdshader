shader_type spatial;

uniform sampler2D diffuse_texture_0 : source_color;
uniform sampler2D diffuse_texture_1 : source_color;
uniform sampler2D damage_texture_0 : source_color;
uniform bool damage_0_set = false;
uniform sampler2D damage_texture_1 : source_color;
uniform bool damage_1_set = false;
uniform sampler2D normal_texture;
uniform bool normal_set = false;
uniform sampler2D spg_texture;
uniform vec3 diffuse_color : source_color;
uniform float emissive_amount = 1.0;
uniform float specular_amount = 1.0;
uniform float specular_power = 1.0;
uniform float alpha = 1.0;
uniform bool damage_enabled = false;

void fragment() {
	vec4 diffuse_0 = texture(diffuse_texture_0, UV);
	vec4 diffuse_1 = texture(diffuse_texture_1, UV);
	vec4 damage_0 = texture(damage_texture_0, UV);
	vec4 damage_1 = texture(damage_texture_1, UV);
	vec4 normal = texture(normal_texture, UV);
	vec4 spg = texture(spg_texture, UV);
	if (damage_enabled) {
		// TODO: doesnt always seem to work quite right
		if (damage_0_set) {
			diffuse_0 = mix(diffuse_0, damage_0, damage_0.a);
		}
		if (damage_1_set) {
			diffuse_1 = mix(diffuse_1, damage_1, damage_1.a);
		}
	}
	vec4 diffuse = mix(diffuse_1 * vec4(diffuse_color, 1.0), diffuse_0, diffuse_0.a);

	ALBEDO = diffuse.rgb;
	//ALPHA = diffuse.a * alpha;
	EMISSION = diffuse.rgb * spg.r * emissive_amount;
	SPECULAR = spg.g * specular_amount; // TODO: is specular_amount set correctly?
	if (normal_set) {
		NORMAL_MAP = normal.rgb;
	}
}
