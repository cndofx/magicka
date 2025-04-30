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
			importer_mesh.set_surface_material(i, new_material)


func _create_material(extras: Variant, base_path: String, index: int) -> Material:
	var diffuse_color = extras.material_0.diffuse_color
	diffuse_color = Color(diffuse_color.r, diffuse_color.g, diffuse_color.b)
	var diffuse_texture = load(base_path.path_join(extras.material_0.diffuse_texture + ".texture2d.png"))
	var spg_texture = load(base_path.path_join(extras.material_0.material_texture + ".texture2d.png"))
	var material = ShaderMaterial.new()
	material.resource_name = "magicka_material_%d" % index
	material.shader = load("res://addons/gltf_magicka_materials/shaders/magicka.gdshader")
	material.set_shader_parameter("diffuse_color", diffuse_color)
	material.set_shader_parameter("diffuse_texture", diffuse_texture)
	material.set_shader_parameter("spg_texture", spg_texture)
	material.set_shader_parameter("specular_amount", extras.material_0.spec_amount)
	material.set_shader_parameter("emissive_amount", extras.material_0.emissive_amount)
	return material
