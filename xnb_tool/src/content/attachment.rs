use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

use super::vector3::Vector3;

#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    slot: i32,
    bone: String,
    rotation: Vector3,
    item: String,
}

impl Attachment {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let slot = reader.read_i32::<LittleEndian>()?;
        let bone = reader.read_7bit_length_string()?;
        let rotation = Vector3::read(reader)?;
        let item = reader.read_7bit_length_string()?;

        let attachment = Attachment {
            slot,
            bone,
            rotation,
            item,
        };
        Ok(attachment)
    }
}
