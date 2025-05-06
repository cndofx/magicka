use std::{io::Read, vec};

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::{ext::MyReadBytesExt, xnb::TypeReader};

use super::{Content, color::Color};

#[derive(Serialize, Deserialize, Debug)]
pub struct Effect {
    pub bytecode: Vec<u8>,
}

impl Effect {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let length = reader.read_u32::<LittleEndian>()?;
        let mut bytecode = vec![0; length as usize];
        reader.read_exact(&mut bytecode)?;
        Ok(Effect { bytecode })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BasicEffect {
    pub texture: String,
    pub diffuse_color: Color,
    pub emissive_color: Color,
    pub specular_color: Color,
    pub specular_power: f32,
    pub alpha: f32,
    pub vertex_color_enabled: bool,
}

impl BasicEffect {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let texture = reader.read_7bit_length_string()?;
        let diffuse_color = Color::read(reader)?;
        let emissive_color = Color::read(reader)?;
        let specular_color = Color::read(reader)?;
        let specular_power = reader.read_f32::<LittleEndian>()?;
        let alpha = reader.read_f32::<LittleEndian>()?;
        let vertex_color_enabled = reader.read_bool()?;
        Ok(BasicEffect {
            texture,
            diffuse_color,
            emissive_color,
            specular_color,
            specular_power,
            alpha,
            vertex_color_enabled,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinnedModelBasicEffect {
    pub method: u8,
    pub emissive_amount: f32,
    pub diffuse_color: Color,
    pub specular_amount: f32,
    pub specular_power: f32,
    pub alpha: f32,
    pub use_soft_light_blend: bool,
    pub map_0_diffuse: Option<String>,
    pub map_1_diffuse: Option<String>,
    pub map_0_damage: Option<String>,
    pub map_1_damage: Option<String>,
    pub material_map: Option<String>,
    pub normal_map: Option<String>,
}

impl SkinnedModelBasicEffect {
    pub fn read(reader: &mut impl Read, type_readers: &[TypeReader]) -> anyhow::Result<Self> {
        let method = reader.read_u8()?;
        let emissive_amount = reader.read_f32::<LittleEndian>()?;
        let diffuse_color = Color::read(reader)?;
        let specular_amount = reader.read_f32::<LittleEndian>()?;
        let specular_power = reader.read_f32::<LittleEndian>()?;
        let alpha = reader.read_f32::<LittleEndian>()?;
        let use_soft_light_blend = reader.read_bool()?;
        let map_0_diffuse_enabled = reader.read_bool()?;
        let map_1_diffuse_enabled = reader.read_bool()?;
        let material_map_enabled = reader.read_bool()?;
        let map_0_damage_enabled = reader.read_bool()?;
        let map_1_damage_enabled = reader.read_bool()?;
        let normal_map_enabled = reader.read_bool()?;

        fn read_external_reference(
            enabled: bool,
            reader: &mut impl Read,
            type_readers: &[TypeReader],
        ) -> anyhow::Result<Option<String>> {
            let reference = if enabled {
                let reference = Content::read(reader, type_readers)?;
                let Content::ExternalReference(reference) = reference else {
                    anyhow::bail!("expected external reference");
                };
                Some(reference)
            } else {
                let z = reader.read_u8()?;
                if z != 0 {
                    anyhow::bail!("expected zero byte for unused external reference");
                }
                None
            };
            Ok(reference)
        }

        let map_0_diffuse = read_external_reference(map_0_diffuse_enabled, reader, type_readers)?;
        let map_1_diffuse = read_external_reference(map_1_diffuse_enabled, reader, type_readers)?;
        let material_map = read_external_reference(material_map_enabled, reader, type_readers)?;
        let map_0_damage = read_external_reference(map_0_damage_enabled, reader, type_readers)?;
        let map_1_damage = read_external_reference(map_1_damage_enabled, reader, type_readers)?;
        let normal_map = read_external_reference(normal_map_enabled, reader, type_readers)?;

        Ok(SkinnedModelBasicEffect {
            method,
            emissive_amount,
            diffuse_color,
            specular_amount,
            specular_power,
            alpha,
            use_soft_light_blend,
            map_0_diffuse,
            map_1_diffuse,
            map_0_damage,
            map_1_damage,
            material_map,
            normal_map,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdditiveEffect {
    pub color_tint: Color,
    pub vertex_color_enabled: bool,
    pub texture_enabled: bool,
    pub texture: String,
}

impl AdditiveEffect {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let color_tint = Color::read(reader)?;
        let vertex_color_enabled = reader.read_bool()?;
        let texture_enabled = reader.read_bool()?;
        let texture = reader.read_7bit_length_string()?;
        Ok(AdditiveEffect {
            color_tint,
            vertex_color_enabled,
            texture_enabled,
            texture,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RenderDeferredEffect {
    pub alpha: f32,
    pub sharpness: f32,
    pub vertex_color_enabled: bool,
    pub use_material_texture_for_reflectiveness: bool,
    pub reflection_map: String,
    pub material_0: RenderDeferredEffectMaterial,
    pub material_1: Option<RenderDeferredEffectMaterial>,
}

impl RenderDeferredEffect {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let alpha = reader.read_f32::<LittleEndian>()?;
        let sharpness = reader.read_f32::<LittleEndian>()?;
        let vertex_color_enabled = reader.read_bool()?;
        let use_material_texture_for_reflectiveness = reader.read_bool()?;
        let reflection_map = reader.read_7bit_length_string()?;
        let material_0 = RenderDeferredEffectMaterial::read(reader)?;
        let has_material_1 = reader.read_bool()?;
        let material_1 = if has_material_1 {
            Some(RenderDeferredEffectMaterial::read(reader)?)
        } else {
            None
        };
        Ok(RenderDeferredEffect {
            alpha,
            sharpness,
            vertex_color_enabled,
            use_material_texture_for_reflectiveness,
            reflection_map,
            material_0,
            material_1,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RenderDeferredEffectMaterial {
    pub diffuse_texture_alpha_disabled: bool,
    pub alpha_mask_enabled: bool,
    pub diffuse_color: Color,
    pub spec_amount: f32,
    pub spec_power: f32,
    pub emissive_amount: f32,
    pub normal_power: f32,
    pub reflectiveness: f32,
    pub diffuse_texture: String,
    pub material_texture: String,
    pub normal_texture: String,
}

impl RenderDeferredEffectMaterial {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let diffuse_texture_alpha_disabled = reader.read_bool()?;
        let alpha_mask_enabled = reader.read_bool()?;
        let diffuse_color = Color::read(reader)?;
        let spec_amount = reader.read_f32::<LittleEndian>()?;
        let spec_power = reader.read_f32::<LittleEndian>()?;
        let emissive_amount = reader.read_f32::<LittleEndian>()?;
        let normal_power = reader.read_f32::<LittleEndian>()?;
        let reflectiveness = reader.read_f32::<LittleEndian>()?;
        let diffuse_texture = reader.read_7bit_length_string()?;
        let material_texture = reader.read_7bit_length_string()?;
        let normal_texture = reader.read_7bit_length_string()?;
        Ok(RenderDeferredEffectMaterial {
            diffuse_texture_alpha_disabled,
            alpha_mask_enabled,
            diffuse_color,
            spec_amount,
            spec_power,
            emissive_amount,
            normal_power,
            reflectiveness,
            diffuse_texture,
            material_texture,
            normal_texture,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkinnedModelDeferredNormalMappedEffect {
    pub diffuse_color: Color,
    pub specular_amount: f32,
    pub specular_power: f32,
    pub emissive_amount: f32,
    pub normal_power: f32,
    pub diffuse_texture: String,
    pub material_texture: String,
    pub damage_texture: String,
    pub normal_texture: String,
    pub normal_damage_texture: String,
}

impl SkinnedModelDeferredNormalMappedEffect {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let diffuse_color = Color::read(reader)?;
        let specular_amount = reader.read_f32::<LittleEndian>()?;
        let specular_power = reader.read_f32::<LittleEndian>()?;
        let emissive_amount = reader.read_f32::<LittleEndian>()?;
        let normal_power = reader.read_f32::<LittleEndian>()?;
        let diffuse_texture = reader.read_7bit_length_string()?;
        let material_texture = reader.read_7bit_length_string()?;
        let damage_texture = reader.read_7bit_length_string()?;
        let normal_texture = reader.read_7bit_length_string()?;
        let normal_damage_texture = reader.read_7bit_length_string()?;
        Ok(SkinnedModelDeferredNormalMappedEffect {
            diffuse_color,
            specular_amount,
            specular_power,
            emissive_amount,
            normal_power,
            diffuse_texture,
            material_texture,
            damage_texture,
            normal_texture,
            normal_damage_texture,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BonedEffect {
    bone: String,
    effect: String,
}

impl BonedEffect {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let bone = reader.read_7bit_length_string()?;
        let effect = reader.read_7bit_length_string()?;
        Ok(BonedEffect { bone, effect })
    }
}
