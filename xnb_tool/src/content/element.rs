use std::io::Read;

use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Element: u16 {
        const None = 0;
        const Earth = 1;
        const Water = 2;
        const Cold = 4;
        const Fire = 8;
        const Lightning = 16;
        const Arcane = 32;
        const Life = 64;
        const Shield = 128;
        const Ice = 256;
        const Steam = 512;
        const Poison = 1024;
        const Offensive = 1855; // originally 65343, upper bits removed
        const Defensive = 176;
        const All = 65535;
        const Magick = 65535;
        const Basic = 255;
        const Instant = 881;
        const InstantPhysical = 369;
        const InstantNonPhysical = 624;
        const StatusEffect = 1614;
        const ShieldElement = 224;
        const Beam = 96;
    }
}

impl Element {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_i32::<LittleEndian>()?;
        let element =
            Element::from_bits(value as u16).ok_or_else(|| anyhow!("unknown element: {value}"))?;
        Ok(element)
    }
}
