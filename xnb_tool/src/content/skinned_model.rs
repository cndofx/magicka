use std::collections::HashMap;
use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;
use crate::xnb::TypeReader;

use super::Content;
use super::model::Model;

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinnedModel {
    pub model: Model,
    pub shared_bone_refs: Vec<usize>,
    pub shared_animation_refs: Vec<usize>,
}

impl SkinnedModel {
    pub fn read(reader: &mut impl Read, type_readers: &[TypeReader]) -> anyhow::Result<Self> {
        let model = Content::read(reader, type_readers)?;
        let Content::Model(model) = model else {
            return Err(anyhow!("expected model"));
        };

        let num_bones = reader.read_i32::<LittleEndian>()?;
        let mut shared_bone_refs = Vec::with_capacity(num_bones as usize);
        for _ in 0..num_bones {
            let bone_ref = reader.read_7bit_encoded_i32()?;
            shared_bone_refs.push(bone_ref as usize);
        }

        let num_animations = reader.read_i32::<LittleEndian>()?;
        let mut shared_animation_refs = Vec::with_capacity(num_animations as usize);
        for _ in 0..num_animations {
            let animation_ref = reader.read_7bit_encoded_i32()?;
            shared_animation_refs.push(animation_ref as usize);
        }

        Ok(SkinnedModel {
            model,
            shared_bone_refs,
            shared_animation_refs,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinnedModelBone {
    index: u16,
    name: String,
    translation: Vec3,
    orientation: Quat,
    scale: Vec3,
    inverse_bind_pose_transform: Mat4,
    shared_parent_ref: usize,
    shared_child_refs: Vec<usize>,
}

impl SkinnedModelBone {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let index = reader.read_u16::<LittleEndian>()?;
        let name = reader.read_7bit_length_string()?;
        let translation = reader.read_vec3()?;
        let orientation = reader.read_quat()?;
        let scale = reader.read_vec3()?;
        let inverse_bind_pose_transform = reader.read_mat4()?;
        let shared_parent_ref = reader.read_7bit_encoded_i32()? as usize;
        let num_children = reader.read_i32::<LittleEndian>()?;
        let mut shared_child_refs = Vec::with_capacity(num_children as usize);
        for _ in 0..num_children {
            let child_ref = reader.read_7bit_encoded_i32()?;
            shared_child_refs.push(child_ref as usize);
        }

        Ok(SkinnedModelBone {
            index,
            name,
            translation,
            orientation,
            scale,
            inverse_bind_pose_transform,
            shared_parent_ref,
            shared_child_refs,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinnedModelAnimationClip {
    name: String,
    duration: f32,
    channels: HashMap<String, Vec<SkinnedModelAnimationKeyframe>>,
}

impl SkinnedModelAnimationClip {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let name = reader.read_7bit_length_string()?;
        let duration = reader.read_f32::<LittleEndian>()?;
        let num_channels = reader.read_i32::<LittleEndian>()?;
        let mut channels = HashMap::with_capacity(num_channels as usize);
        for _ in 0..num_channels {
            let channel_name = reader.read_7bit_length_string()?;
            let num_frames = reader.read_i32::<LittleEndian>()?;
            let mut frames = Vec::with_capacity(num_frames as usize);
            for _ in 0..num_frames {
                let time = reader.read_f32::<LittleEndian>()?;
                let translation = reader.read_vec3()?;
                let orienation = reader.read_quat()?;
                let scale = reader.read_vec3()?;
                let pose = SkinnedModelPose {
                    translation,
                    orienation,
                    scale,
                };
                let frame = SkinnedModelAnimationKeyframe { time, pose };
                frames.push(frame);
            }
            channels.insert(channel_name, frames);
        }

        Ok(SkinnedModelAnimationClip {
            name,
            duration,
            channels,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinnedModelAnimationKeyframe {
    time: f32,
    pose: SkinnedModelPose,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinnedModelPose {
    translation: Vec3,
    orienation: Quat,
    scale: Vec3,
}
