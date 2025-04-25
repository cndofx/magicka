use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    slot: i32,
    bone: String,
    rotation: Vec3,
    item: String,
}

impl Attachment {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let slot = reader.read_i32::<LittleEndian>()?;
        let bone = reader.read_7bit_length_string()?;
        let rotation = reader.read_vec3()?;
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
