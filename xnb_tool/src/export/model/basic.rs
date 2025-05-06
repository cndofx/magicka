use gltf::json::{Root, Scene};

use crate::content::{Content, model::Model};

use super::{
    build_bones, build_buffer, build_glb_bytes, build_materials, build_mesh_parts,
    transformed_model::TransformedModel,
};

impl Model {
    pub fn to_glb(&self, shared_content: &[Content]) -> anyhow::Result<Vec<u8>> {
        let mut root = Root::default();

        let transformed_model = TransformedModel::from(self);

        let buffer = build_buffer(&mut root, &transformed_model, shared_content);

        let materials = build_materials(&mut root, shared_content);

        let (root_bone_node, bone_nodes) = build_bones(&mut root, self)?;

        for (mesh_idx, mesh) in self.meshes.iter().enumerate() {
            build_mesh_parts(
                &mut root,
                &buffer,
                &transformed_model,
                mesh,
                mesh_idx,
                &materials,
                Some(&bone_nodes),
            );
        }

        let scene = root.push(Scene {
            nodes: vec![root_bone_node],
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });
        root.scene = Some(scene);

        let json_string = serde_json::to_string(&root)?;

        let glb = build_glb_bytes(json_string, buffer.data)?;
        Ok(glb)
    }
}
