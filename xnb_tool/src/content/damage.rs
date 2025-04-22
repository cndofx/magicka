use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

use super::{attack_property::AttackProperties, element::Elements};

#[derive(Serialize, Deserialize, Debug)]
pub struct Damage {
    pub attack_properties: AttackProperties,
    pub elements: Elements,
    pub amount: f32,
    pub magnitude: f32,
}

impl Damage {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let attack_properties = AttackProperties::read(reader)?;
        let elements = Elements::read(reader)?;
        let amount = reader.read_f32::<LittleEndian>()?;
        let magnitude = reader.read_f32::<LittleEndian>()?;
        Ok(Damage {
            attack_properties,
            elements,
            amount,
            magnitude,
        })
    }
}
