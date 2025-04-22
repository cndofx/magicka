use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use super::color::Color;

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightVariation {
    None = 0,
    Sine,
    Flicker,
    Candle,
    Strobe,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Light {
    radius: f32,
    diffuse_color: Color,
    ambient_color: Color,
    specular_amount: f32,
    variation: LightVariation,
    variation_amount: f32,
    variation_speed: f32,
}

impl Light {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let radius = reader.read_f32::<LittleEndian>()?;
        let diffuse_color = Color::read(reader)?;
        let ambient_color = Color::read(reader)?;
        let specular_amount = reader.read_f32::<LittleEndian>()?;
        let variation_kind = reader.read_u8()?;
        let variation_kind = LightVariation::from_repr(variation_kind)
            .ok_or_else(|| anyhow!("unknown light variation kind: {variation_kind}"))?;
        let variation_amount = reader.read_f32::<LittleEndian>()?;
        let variation_speed = reader.read_f32::<LittleEndian>()?;

        let light = Light {
            radius,
            diffuse_color,
            ambient_color,
            specular_amount,
            variation: variation_kind,
            variation_amount,
            variation_speed,
        };
        Ok(light)
    }
}
