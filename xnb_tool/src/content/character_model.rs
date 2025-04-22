use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    content::{blood_kind::BloodKind, boned_light::BonedLight, faction::Factions, gib::Gib},
    ext::MyReadBytesExt,
};

use super::{
    aura::Aura,
    color::Color,
    event::EventConditions,
    light::Light,
    passive_ability::PassiveAbility,
    resistance::Resistance,
    sound::{Bank, Sound},
    special_ability::SpecialAbility,
    weapon_class::WeaponClass,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct CharacterModel {
    model: String,
    scale: f32,
    tint: Color,
}

impl CharacterModel {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let model = reader.read_7bit_length_string()?;
        let scale = reader.read_f32::<LittleEndian>()?;
        let tint = Color::read(reader)?;
        Ok(CharacterModel { model, scale, tint })
    }
}
