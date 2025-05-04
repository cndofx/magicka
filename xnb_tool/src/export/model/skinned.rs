use anyhow::anyhow;
use glam::Mat4;
use gltf::{
    animation::{Interpolation, Property},
    json::{
        Accessor, Animation, Index, Node, Root, Scene, Skin,
        accessor::{ComponentType, GenericComponentType, Type},
        animation::{Channel, Sampler, Target},
        buffer::View,
        scene::UnitQuaternion,
        validation::{Checked, USize64},
    },
};
use serde_json::Value;

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

        // build animation
        build_animations(&mut root, &buffer, self, shared_content)?;

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
            (buffer.inverse_bind_matrices.count * std::mem::size_of::<Mat4>()) as u64,
        ),
        byte_offset: Some(USize64(buffer.inverse_bind_matrices.offset as u64)),
        byte_stride: None,
        target: None,
        name: None,
        extensions: Default::default(),
        extras: Default::default(),
    });

    let inverse_bind_accessor = root.push(Accessor {
        buffer_view: Some(inverse_bind_view),
        byte_offset: Some(USize64(0)),
        count: USize64(buffer.inverse_bind_matrices.count as u64),
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

fn build_animations(
    root: &mut Root,
    buffer: &FullBuffer,
    model: &SkinnedModel,
    shared_content: &[Content],
) -> anyhow::Result<()> {
    for anim_ref in &model.shared_animation_refs {
        let Content::SkinnedModelAnimationClip(anim) = &shared_content[anim_ref - 1] else {
            anyhow::bail!("expected animation at shared content index {}", anim_ref);
        };

        let mut max_timestamp = 0.0;
        for (_, keyframes) in &anim.channels {
            for keyframe in keyframes {
                max_timestamp = f32::max(max_timestamp, keyframe.time);
            }
        }

        // dbg!(&buffer.animation_timestamp_offsets, &anim.name);
        let timestamp_offset = buffer.animation_timestamp_offsets[&anim.name];
        let timestamp_view = root.push(View {
            buffer: buffer.index,
            byte_length: USize64((timestamp_offset.count * std::mem::size_of::<f32>()) as u64),
            byte_offset: Some(USize64(timestamp_offset.offset as u64)),
            byte_stride: None,
            target: None,
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });
        let timestamp_accessor = root.push(Accessor {
            buffer_view: Some(timestamp_view),
            byte_offset: Some(USize64(0)),
            count: USize64(timestamp_offset.count as u64),
            component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
            type_: Checked::Valid(Type::Scalar),
            name: None,
            sparse: None,
            min: Some(Value::from(vec![0.0])),
            max: Some(Value::from(vec![max_timestamp])),
            normalized: false,
            extensions: Default::default(),
            extras: Default::default(),
        });

        // // let transform_offset = buffer.animation_transform_offsets[&anim.name];
        // let transform_size = std::mem::size_of::<f32>() * (3 + 4 + 3);
        // let transform_view = root.push(View {
        //     buffer: buffer.index,
        //     byte_length: USize64((transform_offset.count * transform_size) as u64),
        //     byte_offset: Some(USize64(transform_offset.offset as u64)),
        //     // byte_stride: None,
        //     byte_stride: Some(Stride(transform_size)),
        //     target: None,
        //     name: None,
        //     extensions: Default::default(),
        //     extras: Default::default(),
        // });

        let translation_offset = buffer.animation_translation_offsets[&anim.name];
        let translation_view = root.push(View {
            buffer: buffer.index,
            byte_length: USize64(
                (translation_offset.count * std::mem::size_of::<f32>() * 3) as u64,
            ),
            byte_offset: Some(USize64(translation_offset.offset as u64)),
            byte_stride: None,
            target: None,
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });
        let translation_accessor = root.push(Accessor {
            buffer_view: Some(translation_view),
            byte_offset: Some(USize64(0)),
            count: USize64(translation_offset.count as u64),
            component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
            type_: Checked::Valid(Type::Vec3),
            name: None,
            sparse: None,
            min: None,
            max: None,
            normalized: false,
            extensions: Default::default(),
            extras: Default::default(),
        });

        let rotation_offset = buffer.animation_orientation_offsets[&anim.name];
        let rotation_view = root.push(View {
            buffer: buffer.index,
            byte_length: USize64((rotation_offset.count * std::mem::size_of::<f32>() * 4) as u64),
            byte_offset: Some(USize64(rotation_offset.offset as u64)),
            byte_stride: None,
            target: None,
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });
        let rotation_accessor = root.push(Accessor {
            buffer_view: Some(rotation_view),
            byte_offset: Some(USize64(0)),
            count: USize64(rotation_offset.count as u64),
            component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
            type_: Checked::Valid(Type::Vec4),
            name: None,
            sparse: None,
            min: None,
            max: None,
            normalized: false,
            extensions: Default::default(),
            extras: Default::default(),
        });

        let scale_offset = buffer.animation_scale_offsets[&anim.name];
        let scale_view = root.push(View {
            buffer: buffer.index,
            byte_length: USize64((scale_offset.count * std::mem::size_of::<f32>() * 4) as u64),
            byte_offset: Some(USize64(scale_offset.offset as u64)),
            byte_stride: None,
            target: None,
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });
        let scale_accessor = root.push(Accessor {
            buffer_view: Some(scale_view),
            byte_offset: Some(USize64(0)),
            count: USize64(scale_offset.count as u64),
            component_type: Checked::Valid(GenericComponentType(ComponentType::F32)),
            type_: Checked::Valid(Type::Vec3),
            name: None,
            sparse: None,
            min: None,
            max: None,
            normalized: false,
            extensions: Default::default(),
            extras: Default::default(),
        });

        let mut samplers = Vec::with_capacity(3);
        let translation_sampler = Index::push(
            &mut samplers,
            Sampler {
                input: timestamp_accessor,
                output: translation_accessor,
                interpolation: Checked::Valid(Interpolation::Linear),
                extensions: Default::default(),
                extras: Default::default(),
            },
        );
        let rotation_sampler = Index::push(
            &mut samplers,
            Sampler {
                input: timestamp_accessor,
                output: rotation_accessor,
                interpolation: Checked::Valid(Interpolation::Linear),
                extensions: Default::default(),
                extras: Default::default(),
            },
        );
        let scale_sampler = Index::push(
            &mut samplers,
            Sampler {
                input: timestamp_accessor,
                output: scale_accessor,
                interpolation: Checked::Valid(Interpolation::Linear),
                extensions: Default::default(),
                extras: Default::default(),
            },
        );

        let mut channels = Vec::with_capacity(3);
        for (target_node_name, _) in &anim.channels {
            let (target_node_index, _) = root
                .nodes
                .iter()
                .enumerate()
                .find(|(_, node)| node.name.as_ref() == Some(target_node_name))
                .unwrap();
            let target_node = Index::<Node>::new(target_node_index as u32);

            channels.push(Channel {
                sampler: translation_sampler,
                target: Target {
                    node: target_node,
                    path: Checked::Valid(Property::Translation),
                    extensions: Default::default(),
                    extras: Default::default(),
                },
                extensions: Default::default(),
                extras: Default::default(),
            });
            channels.push(Channel {
                sampler: rotation_sampler,
                target: Target {
                    node: target_node,
                    path: Checked::Valid(Property::Rotation),
                    extensions: Default::default(),
                    extras: Default::default(),
                },
                extensions: Default::default(),
                extras: Default::default(),
            });
            channels.push(Channel {
                sampler: scale_sampler,
                target: Target {
                    node: target_node,
                    path: Checked::Valid(Property::Scale),
                    extensions: Default::default(),
                    extras: Default::default(),
                },
                extensions: Default::default(),
                extras: Default::default(),
            });
        }

        root.push(Animation {
            name: Some(anim.name.clone()),
            channels,
            samplers,
            extensions: Default::default(),
            extras: Default::default(),
        });
        println!("added animation");
    }

    Ok(())
}
