use anyhow::anyhow;
use gltf::json::{Index, Node, Root, Scene};

use crate::content::{Content, model::Model};

use super::{build_bones, build_buffer, build_glb_bytes, build_materials, build_mesh_parts};

impl Model {
    pub fn to_glb(&self, shared_content: &[Content]) -> anyhow::Result<Vec<u8>> {
        let mut root = Root::default();

        let buffer = build_buffer(&mut root, self, shared_content);

        let materials = build_materials(&mut root, shared_content);

        let (root_bone_node, bone_nodes) = build_bones(&mut root, self)?;

        // let mesh_nodes: Vec<Index<Node>> = self
        //     .meshes
        //     .iter()
        //     .enumerate()
        //     .map(|(mesh_idx, mesh)| {
        //         build_mesh(
        //             &mut root,
        //             &buffer,
        //             self,
        //             mesh,
        //             mesh_idx,
        //             &materials,
        //             &bone_nodes,
        //         )
        //     })
        //     .collect();

        for (mesh_idx, mesh) in self.meshes.iter().enumerate() {
            build_mesh_parts(
                &mut root,
                &buffer,
                self,
                mesh,
                mesh_idx,
                &materials,
                Some(&bone_nodes),
            );
        }

        // let mut scene_nodes = mesh_nodes.clone();
        // scene_nodes.push(root_bone_node);

        let scene = root.push(Scene {
            nodes: vec![root_bone_node],
            // nodes: scene_nodes,
            // nodes: mesh_nodes,
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
