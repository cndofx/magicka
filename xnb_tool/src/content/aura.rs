use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::content::{color::Color, faction::Faction};
use crate::ext::MyReadBytesExt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Aura {
    kind: AuraKind,
}

impl Aura {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let target = AuraTarget::read(reader)?;
        let kind = reader.read_u8()?;
        let visual_category = VisualCategory::read(reader)?;
        let color = Color::read(reader)?;
        let effect = reader.read_7bit_length_string()?;
        let duration = reader.read_f32::<LittleEndian>()?;
        let radius = reader.read_f32::<LittleEndian>()?;
        let types = reader.read_7bit_length_string()?;
        let factions = Faction::read(reader)?;

        match kind {
            0 => {
                todo!("buff aura");
            }
            1 => {
                todo!("deflect aura");
            }
            2 => {
                todo!("boost aura")
            }
            3 => {
                todo!("life steal aura");
            }
            4 => {
                todo!("love aura");
            }
            v => {
                return Err(anyhow!("unknown aura kind: {v}"));
            }
        }

        todo!()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AuraKind {}

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuraTarget {
    Friendly,
    FriendlyButSelf,
    Enemy,
    All,
    AllButSelf,
    OnlySelf,
    Type,
    TypeButSelf,
    Faction,
    FactionButSelf,
}

impl AuraTarget {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let target =
            AuraTarget::from_repr(value).ok_or_else(|| anyhow!("unknown aura target: {value}"))?;
        Ok(target)
    }
}

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualCategory {
    None,
    Offensive,
    Defensive,
    Special,
}

impl VisualCategory {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let category = VisualCategory::from_repr(value)
            .ok_or_else(|| anyhow!("unknown visual category: {value}"))?;
        Ok(category)
    }
}
