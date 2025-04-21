use std::io::Read;

use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

bitflags! {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Bank: u16 {
        const WaveBank = 1;
        const Music = 2;
        const Ambience = 4;
        const UI = 8;
        const Spells = 16;
        const Characters = 32;
        const Footsteps = 64;
        const Weapons = 128;
        const Misc = 256;
        const Additional = 512;
        const AdditionalMusic = 1024;
    }
}

impl Bank {
    pub fn read(mut reader: impl Read) -> anyhow::Result<Self> {
        let value = reader.read_i32::<LittleEndian>()?;
        let bank =
            Bank::from_bits(value as u16).ok_or_else(|| anyhow!("unknown sound bank: {value}"))?;
        Ok(bank)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Sound {
    pub cue: String,
    pub bank: Bank,
}
