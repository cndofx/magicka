use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct BonedEffect {
    bone: String,
    effect: String,
}

impl BonedEffect {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let bone = reader.read_7bit_length_string()?;
        let effect = reader.read_7bit_length_string()?;
        Ok(BonedEffect { bone, effect })
    }
}
