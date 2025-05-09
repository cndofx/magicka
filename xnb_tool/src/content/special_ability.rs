use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use super::element::Elements;
use crate::ext::MyReadBytesExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct SpecialAbility {
    pub kind: String,
    pub animation: String,
    pub hash: String,
    pub elements: Vec<Elements>,
}

impl SpecialAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_7bit_length_string()?;
        let animation = reader.read_7bit_length_string()?;
        let hash = reader.read_7bit_length_string()?;
        let num_elements = reader.read_i32::<LittleEndian>()?;
        let mut elements = Vec::with_capacity(num_elements as usize);
        for _ in 0..num_elements {
            let element = Elements::read(reader)?;
            elements.push(element);
        }

        let ability = SpecialAbility {
            kind,
            animation,
            hash,
            elements,
        };
        Ok(ability)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpecialAbilityWithCooldown {
    pub ability: SpecialAbility,
    pub cooldown: f32,
}

impl SpecialAbilityWithCooldown {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let cooldown = reader.read_f32::<LittleEndian>()?;
        let ability = SpecialAbility::read(reader)?;
        Ok(SpecialAbilityWithCooldown { ability, cooldown })
    }
}
