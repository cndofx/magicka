use std::io::Read;

use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AttackProperties: u16 {
        const Damage = 1;
        const Knockdown = 2;
        const Pushed = 4;
        const Knockback = 6;
        const Piercing = 8;
        const ArmorPiercing = 16;
        const Status = 32;
        const Entanglement = 64;
        const Stun = 128;
        const Bleed = 256;
    }
}

impl AttackProperties {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_i32::<LittleEndian>()?;
        let attack_properties = AttackProperties::from_bits(value as u16)
            .ok_or_else(|| anyhow!("unknown attack properties: {value}"))?;
        Ok(attack_properties)
    }
}
