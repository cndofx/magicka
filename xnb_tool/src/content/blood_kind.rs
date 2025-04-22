use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Clone, Copy, Debug)]
pub enum BloodKind {
    Regular,
    Green,
    Black,
    Wood,
    Insect,
    None,
}

impl BloodKind {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_i32::<LittleEndian>()? as u32;
        let kind = BloodKind::from_repr(value as u8)
            .ok_or_else(|| anyhow!("unknown blood kind: {value}"))?;
        Ok(kind)
    }
}
