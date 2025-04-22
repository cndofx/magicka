use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassiveAbilityKind {
    None = 0,
    ShieldBoost,
    AreaLifeDrain,
    ZombieDeterrent,
    ReduceAggro,
    EnhanceAllyMelee,
    AreaRegeneration,
    InverseArcaneLife,
    Zap,
    BirchSteam,
    WetLightning,
    MoveSpeed,
    Glow,
    Mjolnir,
    Gungnir,
    MasterSword,
    DragonSlayer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PassiveAbility {
    pub kind: PassiveAbilityKind,
    pub value: f32,
}

impl PassiveAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_u8()?;
        let kind = PassiveAbilityKind::from_repr(kind)
            .ok_or_else(|| anyhow!("unknown passive ability kind: {kind}"))?;
        let value = reader.read_f32::<LittleEndian>()?;

        let ability = PassiveAbility { kind, value };
        Ok(ability)
    }
}
