use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

use super::{
    aura::Aura,
    event::EventConditions,
    light::Light,
    passive_ability::PassiveAbility,
    resistance::Resistance,
    sound::{Bank, Sound},
    special_ability::SpecialAbility,
    weapon_class::WeaponClass,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {}

impl Character {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        Ok(Character {})
    }
}
