@tool
extends GLTFDocumentExtension


func _import_post_parse(state: GLTFState) -> Error:
	_replace_materials(state)
	return OK


func _replace_materials(state: GLTFState) -> void:
	for mesh in state.get_meshes():
		var importer_mesh = mesh.mesh
		for i in range(importer_mesh.get_surface_count()):
			var material = importer_mesh.get_surface_material(i)
			var extras = material.get_meta("extras")
			if extras == null:
				continue
			var new_material = _create_material(extras, state.base_path, i)
			if new_material == null:
				continue
			importer_mesh.set_surface_material(i, new_material)


func _create_material(extras: Dictionary, base_path: String, index: int) -> Material:
	if extras.has("BasicEffect"):
		return _create_basic_material(extras.BasicEffect, base_path, index)
	elif extras.has("RenderDeferredEffect"):
		return _create_render_deferred_material(extras.RenderDeferredEffect, base_path, index)
	elif extras.has("AdditiveEffect"):
		return _create_additive_material(extras.AdditiveEffect, base_path, index)
	else:
		push_error("no supported material data found in extras")
		return null


func _create_basic_material(data: Dictionary, base_path: String, index: int) -> Material:
	var diffuse_texture = load(base_path.path_join(data.texture + ".texture2d.png"))
	var diffuse_color = data.diffuse_color
	diffuse_color = Color(diffuse_color.r, diffuse_color.g, diffuse_color.b)
	var emissive_color = data.emissive_color
	emissive_color = Color(emissive_color.r, emissive_color.g, emissive_color.b)
	var specular_color = data.specular_color
	specular_color = Color(specular_color.r, specular_color.g, specular_color.b)
	var material = ShaderMaterial.new()
	material.resource_name = "material_%d_basic" % index
	material.shader = load("res://addons/gltf_magicka_materials/shaders/basic.gdshader")
	material.set_shader_parameter("diffuse_texture", diffuse_texture)
	material.set_shader_parameter("diffuse_color", diffuse_color)
	material.set_shader_parameter("emissive_color", emissive_color)
	material.set_shader_parameter("specular_color", specular_color)
	material.set_shader_parameter("specular_power", data.specular_power)
	material.set_shader_parameter("alpha", data.alpha)
	material.set_shader_parameter("vertex_color_enabled", data.vertex_color_enabled)
	return material


func _create_render_deferred_material(data: Dictionary, base_path: String, index: int) -> Material:
	var diffuse_color = data.material_0.diffuse_color
	diffuse_color = Color(diffuse_color.r, diffuse_color.g, diffuse_color.b)
	var diffuse_texture = load(base_path.path_join(data.material_0.diffuse_texture + ".texture2d.png"))
	var spg_texture = load(base_path.path_join(data.material_0.material_texture + ".texture2d.png"))
	var material = ShaderMaterial.new()
	material.resource_name = "material_%d_deferred" % index
	material.shader = load("res://addons/gltf_magicka_materials/shaders/deferred.gdshader")
	material.set_shader_parameter("diffuse_color", diffuse_color)
	material.set_shader_parameter("diffuse_texture", diffuse_texture)
	material.set_shader_parameter("spg_texture", spg_texture)
	material.set_shader_parameter("specular_amount", data.material_0.spec_amount)
	material.set_shader_parameter("emissive_amount", data.material_0.emissive_amount)
	material.set_shader_parameter("vertex_color_enabled", data.vertex_color_enabled)
	return material


func _create_additive_material(data: Dictionary, base_path: String, index: int) -> Material:
	var color_texture = load(base_path.path_join(data.texture + ".texture2d.png"))
	var color_tint = data.color_tint
	color_tint = Color(color_tint.r, color_tint.g, color_tint.b)
	var material = ShaderMaterial.new()
	material.resource_name = "material_%d_additive" % index
	material.shader = load("res://addons/gltf_magicka_materials/shaders/additive.gdshader")
	material.set_shader_parameter("color_texture", color_texture)
	material.set_shader_parameter("color_texture_enabled", data.texture_enabled)
	material.set_shader_parameter("color_tint", color_tint)
	material.set_shader_parameter("vertex_color_enabled", data.vertex_color_enabled)
	return material
