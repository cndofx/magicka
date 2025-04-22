use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

use super::light::Light;

#[derive(Serialize, Deserialize, Debug)]
pub struct BonedLight {
    bone: String,
    light: Light,
}

impl BonedLight {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let bone = reader.read_7bit_length_string()?;
        let light = Light::read(reader)?;
        Ok(BonedLight { bone, light })
    }
}
