use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let r = reader.read_f32::<LittleEndian>()?;
        let g = reader.read_f32::<LittleEndian>()?;
        let b = reader.read_f32::<LittleEndian>()?;
        let color = Color { r, g, b };
        Ok(color)
    }
}
