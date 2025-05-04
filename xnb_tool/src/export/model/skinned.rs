use anyhow::anyhow;
use glam::Mat4;
use gltf::json::{
    Accessor, Index, Node, Root, Scene, Skin,
    accessor::{ComponentType, GenericComponentType, Type},
    buffer::View,
    scene::UnitQuaternion,
    validation::{Checked, USize64},
};

use crate::content::{
    Content,
    skinned_model::{SkinnedModel, SkinnedModelBone},
};

use super::{FullBuffer, build_buffer, build_glb_bytes, build_materials, build_mesh_parts};

impl SkinnedModel {
    pub fn to_glb(&self, shared_content: &[Content]) -> anyhow::Result<Vec<u8>> {
        let mut root = Root::default();

        let buffer = build_buffer(&mut root, &self.model, shared_content);

        let materials = build_materials(&mut root, shared_content);

        let mut mesh_node_indices = Vec::new();
        let mut mesh_part_node_indices = Vec::new();
        for (mesh_idx, mesh) in self.model.meshes.iter().enumerate() {
            let (mesh_node_index, part_node_indices) = build_mesh_parts(
                &mut root,
                &buffer,
                &self.model,
                mesh,
                mesh_idx,
                &materials,
                None,
            );
            mesh_node_indices.push(mesh_node_index);
            mesh_part_node_indices.extend_from_slice(&part_node_indices);
        }

        let (skin, root_skin_bone_node) = build_skin(&mut root, &buffer, self, shared_content)?;

        for index in &mesh_part_node_indices {
            root.nodes[index.value()].skin = Some(skin);
        }

        let mut scene_nodes = mesh_node_indices.clone();
        scene_nodes.push(root_skin_bone_node);

        let scene = root.push(Scene {
            nodes: scene_nodes,
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

fn build_skin(
    root: &mut Root,
    buffer: &FullBuffer,
    model: &SkinnedModel,
    shared_content: &[Content],
) -> anyhow::Result<(Index<Skin>, Index<Node>)> {
    let root_bone = model
        .shared_bone_refs
        .iter()
        .copied()
        .find_map(|bone_ref| {
            if let Content::SkinnedModelBone(bone) = &shared_content[bone_ref - 1] {
                if bone.shared_parent_ref == 0 {
                    Some(bone)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("could not find root bone"))?;

    let root_bone_node = root.push(Node {
        name: Some(root_bone.name.clone()),
        translation: Some(root_bone.translation.into()),
        rotation: Some(UnitQuaternion(root_bone.orientation.normalize().to_array())),
        scale: Some(root_bone.scale.into()),
        ..Default::default()
    });

    fn build_bone_tree_recursive(
        root: &mut Root,
        parent_node: Index<Node>,
        parent_bone: &SkinnedModelBone,
        shared_content: &[Content],
        joint_nodes: &mut Vec<Index<Node>>,
    ) -> anyhow::Result<()> {
        for child_ref in parent_bone.shared_child_refs.iter().copied() {
            let Content::SkinnedModelBone(child_bone) = &shared_content[child_ref - 1] else {
                anyhow::bail!("expected child bone at shared content index {}", child_ref);
            };
            let child_bone_node = root.push(Node {
                name: Some(child_bone.name.clone()),
                translation: Some(child_bone.translation.into()),
                rotation: Some(UnitQuaternion(
                    child_bone.orientation.normalize().to_array(),
                )),
                scale: Some(child_bone.scale.into()),
                ..Default::default()
            });
            joint_nodes.push(child_bone_node);
            let parent_node = &mut root.nodes[parent_node.value()];
            let parent_node_children = parent_node.children.get_or_insert_default();
            parent_node_children.push(child_bone_node);
            build_bone_tree_recursive(
                root,
                child_bone_node,
                child_bone,
                shared_content,
                joint_nodes,
            )?;
        }
        Ok(())
    }

    let mut joint_nodes = Vec::new();
    joint_nodes.push(root_bone_node);

    build_bone_tree_recursive(
        root,
        root_bone_node,
        root_bone,
        shared_content,
        &mut joint_nodes,
    )?;

    let inverse_bind_view = root.push(View {
        buffer: buffer.index,
        byte_length: USize64(
            (buffer.inverse_bind_matrices_count * std::mem::size_of::<Mat4>()) as u64,
        ),
        byte_offset: Some(USize64(buffer.inverse_bind_matrices_offset as u64)),
        byte_stride: None,
        target: None,
        name: None,
        extensions: Default::default(),
        extras: Default::default(),
    });

    let inverse_bind_accessor = root.push(Accessor {
        buffer_view: Some(inverse_bind_view),
        byte_offset: Some(USize64(0)),
        count: USize64(buffer.inverse_bind_matrices_count as u64),
        component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
        type_: Checked::Valid(Type::Mat4),
        name: None,
        min: None,
        max: None,
        sparse: None,
        normalized: false,
        extensions: Default::default(),
        extras: Default::default(),
    });

    let skin = root.push(Skin {
        skeleton: Some(root_bone_node),
        joints: joint_nodes,
        inverse_bind_matrices: Some(inverse_bind_accessor),
        name: None,
        extensions: Default::default(),
        extras: Default::default(),
    });

    Ok((skin, root_bone_node))
}
