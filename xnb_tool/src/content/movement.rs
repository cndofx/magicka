use std::io::Read;

use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Movement {
    properties: MovementProperties,
    animations: Vec<String>,
}

impl Movement {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let properties = MovementProperties::read(reader)?;
        let num_animations = reader.read_i32::<LittleEndian>()?;
        let mut animations = Vec::with_capacity(num_animations as usize);
        for _ in 0..num_animations {
            let animation = reader.read_7bit_length_string()?;
            animations.push(animation);
        }
        Ok(Movement {
            properties,
            animations,
        })
    }
}

bitflags! {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct MovementProperties: u8 {
        const DEFAULT = 0;
        const WATER = 1;
        const JUMP = 2;
        const FLY = 4;
        const DYNAMIC = 128;
        const ALL = 255;
    }
}

impl MovementProperties {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let properties = MovementProperties::from_bits(value)
            .ok_or_else(|| anyhow!("unknown movement properties: {value}"))?;
        Ok(properties)
    }
}
