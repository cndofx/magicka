use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::content::{color::Color, faction::Faction};
use crate::ext::MyReadBytesExt;

use super::attack_property::AttackProperty;
use super::element::Element;
use super::resistance::Resistance;

#[derive(Serialize, Deserialize, Debug)]
pub struct Aura {
    kind: AuraKind,
    target: AuraTarget,
    visual_category: VisualCategory,
    color: Color,
    effect: String,
    duration: f32,
    radius: f32,
    types: String,
    factions: Faction,
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

        let kind = match kind {
            0 => {
                let kind = BuffAura::read(reader)?;
                AuraKind::Buff(kind)
            }
            1 => {
                let kind = DeflectAura::read(reader)?;
                AuraKind::Deflect(kind)
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
        };

        let aura = Aura {
            kind,
            target,
            visual_category,
            color,
            effect,
            duration,
            radius,
            types,
            factions,
        };
        Ok(aura)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AuraKind {
    Buff(BuffAura),
    Deflect(DeflectAura),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BuffAura {
    buff: Buff,
}

impl BuffAura {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let buff = Buff::read(reader)?;
        Ok(BuffAura { buff })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeflectAura {
    strength: f32,
}

impl DeflectAura {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let strength = reader.read_f32::<LittleEndian>()?;
        Ok(DeflectAura { strength })
    }
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Buff {
    kind: BuffKind,
    visual_category: VisualCategory,
    color: Color,
    time: f32,
    effect: String,
}

impl Buff {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_u8()?;
        let visual_category = VisualCategory::read(reader)?;
        let color = Color::read(reader)?;
        let duration = reader.read_f32::<LittleEndian>()?;
        let effect = reader.read_7bit_length_string()?;

        let kind = match kind {
            0 => {
                let kind = BoostDamageBuff::read(reader)?;
                BuffKind::BoostDamage(kind)
            }
            1 => {
                let kind = DealDamageBuff::read(reader)?;
                BuffKind::DealDamage(kind)
            }
            2 => {
                let kind = ResistanceBuff::read(reader)?;
                BuffKind::Resistance(kind)
            }
            3 => {
                let kind = UndyingBuff;
                BuffKind::Undying(kind)
            }
            4 => {
                let kind = BoostBuff::read(reader)?;
                BuffKind::Boost(kind)
            }
            5 => {
                let kind = ReduceAggroBuff::read(reader)?;
                BuffKind::ReduceAggro(kind)
            }
            6 => {
                let kind = ModifyHitPointsBuff::read(reader)?;
                BuffKind::ModifyHitPoints(kind)
            }
            7 => {
                let kind = ModifySpellDurationBuff::read(reader)?;
                BuffKind::ModifySpellDuration(kind)
            }
            8 => {
                let kind = ModifySpellRangeBuff::read(reader)?;
                BuffKind::ModifySpellRange(kind)
            }
            v => {
                return Err(anyhow!("unknown buff kind: {v}"));
            }
        };

        let buff = Buff {
            kind,
            visual_category,
            color,
            time: duration,
            effect,
        };
        Ok(buff)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum BuffKind {
    BoostDamage(BoostDamageBuff),
    DealDamage(DealDamageBuff),
    Resistance(ResistanceBuff),
    Undying(UndyingBuff),
    Boost(BoostBuff),
    ReduceAggro(ReduceAggroBuff),
    ModifyHitPoints(ModifyHitPointsBuff),
    ModifySpellDuration(ModifySpellDurationBuff),
    ModifySpellRange(ModifySpellRangeBuff),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoostDamageBuff {
    attack_properties: AttackProperty,
    elements: Element,
    amount: f32,
    magnitude: f32,
}

impl BoostDamageBuff {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let attack_properties = AttackProperty::read(reader)?;
        let elements = Element::read(reader)?;
        let amount = reader.read_f32::<LittleEndian>()?;
        let magnitude = reader.read_f32::<LittleEndian>()?;

        let buff = BoostDamageBuff {
            attack_properties,
            elements,
            amount,
            magnitude,
        };
        Ok(buff)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DealDamageBuff {
    attack_properties: AttackProperty,
    elements: Element,
    amount: f32,
    magnitude: f32,
}

impl DealDamageBuff {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let attack_properties = AttackProperty::read(reader)?;
        let elements = Element::read(reader)?;
        let amount = reader.read_f32::<LittleEndian>()?;
        let magnitude = reader.read_f32::<LittleEndian>()?;

        let buff = DealDamageBuff {
            attack_properties,
            elements,
            amount,
            magnitude,
        };
        Ok(buff)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResistanceBuff {
    resistance: Resistance,
}

impl ResistanceBuff {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let resistance = Resistance::read(reader)?;
        let buff = ResistanceBuff { resistance };
        Ok(buff)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UndyingBuff;

#[derive(Serialize, Deserialize, Debug)]
pub struct BoostBuff {
    amount: f32,
}

impl BoostBuff {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let amount = reader.read_f32::<LittleEndian>()?;
        let buff = BoostBuff { amount };
        Ok(buff)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ReduceAggroBuff {
    amount: f32,
}

impl ReduceAggroBuff {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let amount = reader.read_f32::<LittleEndian>()?;
        let buff = ReduceAggroBuff { amount };
        Ok(buff)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModifyHitPointsBuff {
    multiplier: f32,
    modifier: f32,
}

impl ModifyHitPointsBuff {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let multiplier = reader.read_f32::<LittleEndian>()?;
        let modifier = reader.read_f32::<LittleEndian>()?;

        let buff = ModifyHitPointsBuff {
            multiplier,
            modifier,
        };
        Ok(buff)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModifySpellDurationBuff {
    multiplier: f32,
    modifier: f32,
}

impl ModifySpellDurationBuff {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let multiplier = reader.read_f32::<LittleEndian>()?;
        let modifier = reader.read_f32::<LittleEndian>()?;

        let buff = ModifySpellDurationBuff {
            multiplier,
            modifier,
        };
        Ok(buff)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ModifySpellRangeBuff {
    multiplier: f32,
    modifier: f32,
}

impl ModifySpellRangeBuff {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let multiplier = reader.read_f32::<LittleEndian>()?;
        let modifier = reader.read_f32::<LittleEndian>()?;

        let buff = ModifySpellRangeBuff {
            multiplier,
            modifier,
        };
        Ok(buff)
    }
}
