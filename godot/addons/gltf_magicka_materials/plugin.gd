@tool
extends EditorPlugin

const GLTF_EXTENSION: Script = preload("res://addons/gltf_magicka_materials/gltf_extension.gd")

var _gltf_extension: GLTFDocumentExtension


func _enter_tree() -> void:
	_gltf_extension = GLTF_EXTENSION.new()
	GLTFDocument.register_gltf_document_extension(_gltf_extension)


func _exit_tree() -> void:
	GLTFDocument.unregister_gltf_document_extension(_gltf_extension)
	_gltf_extension = null
