use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    content::{blood_kind::BloodKind, boned_light::BonedLight, faction::Factions, gib::Gib},
    ext::MyReadBytesExt,
};

use super::{
    animation_action::AnimationAction,
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
pub struct AnimationClip {
    kind: String,
    key: String,
    speed: f32,
    blend_time: f32,
    loops: bool,
    actions: Vec<AnimationAction>,
}

impl AnimationClip {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_7bit_length_string()?;
        let key = reader.read_7bit_length_string()?;
        let speed = reader.read_f32::<LittleEndian>()?;
        let blend_time = reader.read_f32::<LittleEndian>()?;
        let loops = reader.read_bool()?;
        let num_actions = reader.read_i32::<LittleEndian>()?;
        let mut actions = Vec::with_capacity(num_actions as usize);
        for _ in 0..num_actions {
            let action = AnimationAction::read(reader)?;
            actions.push(action);
        }
        Ok(AnimationClip {
            kind,
            key,
            speed,
            blend_time,
            loops,
            actions,
        })
    }
}
