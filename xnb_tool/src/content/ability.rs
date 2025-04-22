use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

use super::{damage::Damage, element::Elements, vector3::Vector3};

#[derive(Serialize, Deserialize, Debug)]
pub struct Ability {
    kind: AbilityKind,
    cooldown: f32,
    target: AbilityTarget,
    fuzzy_expression: Option<String>,
    animations: Vec<String>,
}

impl Ability {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_7bit_length_string()?;
        let cooldown = reader.read_f32::<LittleEndian>()?;
        let target = AbilityTarget::read(reader)?;
        let has_fuzzy_expression = reader.read_bool()?;
        let fuzzy_expression = if has_fuzzy_expression {
            Some(reader.read_7bit_length_string()?)
        } else {
            None
        };
        let num_animations = reader.read_i32::<LittleEndian>()?;
        let mut animations = Vec::with_capacity(num_animations as usize);
        for _ in 0..num_animations {
            let animation = reader.read_7bit_length_string()?;
            animations.push(animation);
        }

        let kind = match kind.as_str() {
            "Jump" => {
                let ability = JumpAbility::read(reader)?;
                AbilityKind::Jump(ability)
            }
            "Dash" => {
                let ability = DashAbility::read(reader)?;
                AbilityKind::Dash(ability)
            }
            "Block" => {
                let ability = BlockAbility::read(reader)?;
                AbilityKind::Block(ability)
            }
            "Melee" => {
                let ability = MeleeAbility::read(reader)?;
                AbilityKind::Melee(ability)
            }
            "Ranged" => {
                let ability = RangedAbility::read(reader)?;
                AbilityKind::Ranged(ability)
            }
            "ConfuseGrip" => {
                let ability = ConfuseGripAbility;
                AbilityKind::ConfuseGrip(ability)
            }
            "DamageGrip" => {
                let ability = DamageGripAbility;
                AbilityKind::DamageGrip(ability)
            }
            "ThrowGrip" => {
                let ability = ThrowGripAbility::read(reader)?;
                AbilityKind::ThrowGrip(ability)
            }
            "GripCharacterFromBehind" => {
                let ability = GripCharacterFromBehindAbility::read(reader)?;
                AbilityKind::GripCharacterFromBehind(ability)
            }
            "PickUpCharacter" => {
                let ability = PickUpCharacterAbility::read(reader)?;
                AbilityKind::PickUpCharacter(ability)
            }
            "RemoveStatus" => {
                let ability = RemoveStatusAbility;
                AbilityKind::RemoveStatus(ability)
            }
            "CastSpell" => {
                let ability = CastSpellAbility::read(reader)?;
                AbilityKind::CastSpell(ability)
            }
            "SpecialAbilityAbility" => {
                let ability = SpecialAbilityAbility::read(reader)?;
                AbilityKind::SpecialAbility(ability)
            }
            v => {
                anyhow::bail!("unknown ability kind: {v}");
            }
        };

        Ok(Ability {
            kind,
            cooldown,
            target,
            fuzzy_expression,
            animations,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AbilityKind {
    Jump(JumpAbility),
    Dash(DashAbility),
    Block(BlockAbility),
    Melee(MeleeAbility),
    Ranged(RangedAbility),
    ConfuseGrip(ConfuseGripAbility),
    DamageGrip(DamageGripAbility),
    ThrowGrip(ThrowGripAbility),
    GripCharacterFromBehind(GripCharacterFromBehindAbility),
    PickUpCharacter(PickUpCharacterAbility),
    RemoveStatus(RemoveStatusAbility),
    CastSpell(CastSpellAbility),
    SpecialAbility(SpecialAbilityAbility),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JumpAbility {
    pub min_range: f32,
    pub max_range: f32,
    pub angle: f32,
    pub elevation: f32,
}

impl JumpAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let max_range = reader.read_f32::<LittleEndian>()?;
        let min_range = reader.read_f32::<LittleEndian>()?;
        let angle = reader.read_f32::<LittleEndian>()?;
        let elevation = reader.read_f32::<LittleEndian>()?;
        Ok(JumpAbility {
            min_range,
            max_range,
            angle,
            elevation,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DashAbility {
    pub min_range: f32,
    pub max_range: f32,
    pub arc: f32,
    pub velocity: Vector3,
}

impl DashAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let min_range = reader.read_f32::<LittleEndian>()?;
        let max_range = reader.read_f32::<LittleEndian>()?;
        let arc = reader.read_f32::<LittleEndian>()?;
        let velocity = Vector3::read(reader)?;
        Ok(DashAbility {
            min_range,
            max_range,
            arc,
            velocity,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockAbility {
    pub arc: f32,
    pub shield: i32,
}

impl BlockAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let arc = reader.read_f32::<LittleEndian>()?;
        let shield = reader.read_i32::<LittleEndian>()?;
        Ok(BlockAbility { arc, shield })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MeleeAbility {
    pub min_range: f32,
    pub max_range: f32,
    pub arc_angle: f32,
    pub weapon_slots: Vec<i32>,
    pub rotate: bool,
}

impl MeleeAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let min_range = reader.read_f32::<LittleEndian>()?;
        let max_range = reader.read_f32::<LittleEndian>()?;
        let arc_angle = reader.read_f32::<LittleEndian>()?;
        let num_weapon_slots = reader.read_i32::<LittleEndian>()?;
        let mut weapon_slots = Vec::with_capacity(num_weapon_slots as usize);
        for _ in 0..num_weapon_slots {
            let weapon_slot = reader.read_i32::<LittleEndian>()?;
            weapon_slots.push(weapon_slot);
        }
        let rotate = reader.read_bool()?;
        Ok(MeleeAbility {
            min_range,
            max_range,
            arc_angle,
            weapon_slots,
            rotate,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RangedAbility {
    pub min_range: f32,
    pub max_range: f32,
    pub elevation: f32,
    pub arc: f32,
    pub accuracy: f32,
    pub weapon_slots: Vec<i32>,
}

impl RangedAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let min_range = reader.read_f32::<LittleEndian>()?;
        let max_range = reader.read_f32::<LittleEndian>()?;
        let elevation = reader.read_f32::<LittleEndian>()?;
        let arc = reader.read_f32::<LittleEndian>()?;
        let accuracy = reader.read_f32::<LittleEndian>()?;
        let num_weapon_slots = reader.read_i32::<LittleEndian>()?;
        let mut weapon_slots = Vec::with_capacity(num_weapon_slots as usize);
        for _ in 0..num_weapon_slots {
            let weapon_slot = reader.read_i32::<LittleEndian>()?;
            weapon_slots.push(weapon_slot);
        }
        Ok(RangedAbility {
            min_range,
            max_range,
            elevation,
            arc,
            accuracy,
            weapon_slots,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfuseGripAbility;

#[derive(Serialize, Deserialize, Debug)]
pub struct DamageGripAbility;

#[derive(Serialize, Deserialize, Debug)]
pub struct ThrowGripAbility {
    pub min_range: f32,
    pub max_range: f32,
    pub elevation: f32,
    pub damages: Vec<Damage>,
}

impl ThrowGripAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let max_range = reader.read_f32::<LittleEndian>()?;
        let min_range = reader.read_f32::<LittleEndian>()?;
        let elevation = reader.read_f32::<LittleEndian>()?;
        let num_damages = reader.read_i32::<LittleEndian>()?;
        let mut damages = Vec::with_capacity(num_damages as usize);
        for _ in 0..num_damages {
            let damage = Damage::read(reader)?;
            damages.push(damage);
        }
        Ok(ThrowGripAbility {
            min_range,
            max_range,
            elevation,
            damages,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GripCharacterFromBehindAbility {
    pub min_range: f32,
    pub max_range: f32,
    pub angle: f32,
    pub max_weight: f32,
}

impl GripCharacterFromBehindAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let max_range = reader.read_f32::<LittleEndian>()?;
        let min_range = reader.read_f32::<LittleEndian>()?;
        let angle = reader.read_f32::<LittleEndian>()?;
        let max_weight = reader.read_f32::<LittleEndian>()?;
        Ok(GripCharacterFromBehindAbility {
            min_range,
            max_range,
            angle,
            max_weight,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PickUpCharacterAbility {
    pub min_range: f32,
    pub max_range: f32,
    pub angle: f32,
    pub max_weight: f32,
    pub drop_animation: String,
}

impl PickUpCharacterAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let max_range = reader.read_f32::<LittleEndian>()?;
        let min_range = reader.read_f32::<LittleEndian>()?;
        let angle = reader.read_f32::<LittleEndian>()?;
        let max_weight = reader.read_f32::<LittleEndian>()?;
        let drop_animation = reader.read_7bit_length_string()?;
        Ok(PickUpCharacterAbility {
            min_range,
            max_range,
            angle,
            max_weight,
            drop_animation,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveStatusAbility;

#[derive(Serialize, Deserialize, Debug)]
pub struct CastSpellAbility {
    pub min_range: f32,
    pub max_range: f32,
    pub angle: f32,
    pub chant_time: f32,
    pub power: f32,
    pub cast_kind: CastKind,
    pub elements: Vec<Elements>,
}

impl CastSpellAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let min_range = reader.read_f32::<LittleEndian>()?;
        let max_range = reader.read_f32::<LittleEndian>()?;
        let angle = reader.read_f32::<LittleEndian>()?;
        let chant_time = reader.read_f32::<LittleEndian>()?;
        let power = reader.read_f32::<LittleEndian>()?;
        let cast_kind = CastKind::read(reader)?;
        let num_elements = reader.read_i32::<LittleEndian>()?;
        let mut elements = Vec::with_capacity(num_elements as usize);
        for _ in 0..num_elements {
            let element = Elements::read(reader)?;
            elements.push(element);
        }
        Ok(CastSpellAbility {
            min_range,
            max_range,
            angle,
            chant_time,
            power,
            cast_kind,
            elements,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpecialAbilityAbility {
    pub min_range: f32,
    pub max_range: f32,
    pub angle: f32,
    pub weapon_slot: i32,
}

impl SpecialAbilityAbility {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let min_range = reader.read_f32::<LittleEndian>()?;
        let max_range = reader.read_f32::<LittleEndian>()?;
        let angle = reader.read_f32::<LittleEndian>()?;
        let weapon_slot = reader.read_i32::<LittleEndian>()?;
        Ok(SpecialAbilityAbility {
            min_range,
            max_range,
            angle,
            weapon_slot,
        })
    }
}

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug)]
pub enum AbilityTarget {
    User = 1,
    Enemy = 2,
    Friendly = 3,
}

impl AbilityTarget {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let kind =
            AbilityTarget::from_repr(value).ok_or_else(|| anyhow!("unknown cast kind: {value}"))?;
        Ok(kind)
    }
}

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug)]
pub enum CastKind {
    None,
    Force,
    Area,
    User,
    Weapon,
    Magick,
}

impl CastKind {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_i32::<LittleEndian>()?;
        let kind = CastKind::from_repr(value as u8)
            .ok_or_else(|| anyhow!("unknown cast kind: {value}"))?;
        Ok(kind)
    }
}
