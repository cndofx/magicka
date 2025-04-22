use std::io::Read;

use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Factions: u16 {
        const None = 0;
        const Evil = 1;
        const Wild = 2;
        const Friendly = 4;
        const Demon = 8;
        const Undead = 16;
        const Human = 32;
        const Wizard = 64;
        const Neutral = 255;
        const Player0 = 256;
        const Player1 = 512;
        const Player2 = 1024;
        const Player3 = 2048;
        const TeamRed = 4096;
        const TeamBlue = 8192;
        const Player = 16128;
    }
}

impl Factions {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_i32::<LittleEndian>()?;
        let bank =
            Factions::from_bits(value as u16).ok_or_else(|| anyhow!("unknown faction: {value}"))?;
        Ok(bank)
    }
}
