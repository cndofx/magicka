use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    content::{blood_kind::BloodKind, boned_light::BonedLight, faction::Factions, gib::Gib},
    ext::MyReadBytesExt,
};

use super::{
    animation_clip::AnimationClip,
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
pub struct AnimationSet {
    clips: Vec<AnimationClip>,
}

impl AnimationSet {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let num_clips = reader.read_i32::<LittleEndian>()?;
        let mut clips = Vec::with_capacity(num_clips as usize);
        for _ in 0..num_clips {
            let clip = AnimationClip::read(reader)?;
            clips.push(clip);
        }
        Ok(AnimationSet { clips })
    }
}
