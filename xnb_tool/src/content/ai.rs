use std::io::Read;

use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use super::{
    attack_property::AttackProperties, element::Elements, light::Light, sound::Bank,
    vector3::Vector3,
};
use crate::ext::MyReadBytesExt;

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug)]
pub enum Order {
    None,
    Idle,
    Attack,
    Defend,
    Flee,
    Wander,
    Panic,
}

impl Order {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let order = Order::from_repr(value).ok_or_else(|| anyhow!("unknown order: {value}"))?;
        Ok(order)
    }
}

bitflags! {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct ReactionTriggers: u8 {
        const None = 0;
        const Attack = 1;
        const Proximity = 2;
    }
}

impl ReactionTriggers {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let triggers = ReactionTriggers::from_bits(value)
            .ok_or_else(|| anyhow!("unknown reaction triggers: {value}"))?;
        Ok(triggers)
    }
}
