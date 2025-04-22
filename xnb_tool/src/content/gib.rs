use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Gib {
    model: String,
    mass: f32,
    scale: f32,
}

impl Gib {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let model = reader.read_7bit_length_string()?;
        let mass = reader.read_f32::<LittleEndian>()?;
        let scale = reader.read_f32::<LittleEndian>()?;
        Ok(Gib { model, mass, scale })
    }
}
