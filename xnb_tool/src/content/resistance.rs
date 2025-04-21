use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use super::element::Element;
use crate::ext::MyReadBytesExt;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Resistance {
    pub element: Element,
    pub multiplier: f32,
    pub modifier: f32,
    pub status_immunity: bool,
}

impl Resistance {
    pub fn new(element: Element) -> Self {
        Self {
            element,
            multiplier: 1.0,
            modifier: 0.0,
            status_immunity: false,
        }
    }

    pub fn read(mut reader: impl Read) -> anyhow::Result<Self> {
        let element = Element::read(&mut reader)?;
        let multiplier = reader.read_f32::<LittleEndian>()?;
        let modifier = reader.read_f32::<LittleEndian>()?;
        let status_immunity = reader.read_bool()?;

        let resistance = Resistance {
            element,
            multiplier,
            modifier,
            status_immunity,
        };
        Ok(resistance)
    }
}
