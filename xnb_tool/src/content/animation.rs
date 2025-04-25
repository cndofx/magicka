use std::io::Read;

use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

use super::{
    damage::Damage,
    sound::{Bank, Sound},
    special_ability::SpecialAbility,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimationSet {
    clips: Vec<AnimationClip>,
}

impl AnimationSet {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let num_clips = reader.read_i32::<LittleEndian>()?;
        let mut clips = Vec::with_capacity(num_clips as usize);
        for _ in 0..num_clips {
            let clip = AnimationClip::read(reader)?;
            clips.push(clip);
        }
        Ok(AnimationSet { clips })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimationClip {
    kind: String,
    key: String,
    speed: f32,
    blend_time: f32,
    loops: bool,
    actions: Vec<AnimationAction>,
}

impl AnimationClip {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_7bit_length_string()?;
        let key = reader.read_7bit_length_string()?;
        let speed = reader.read_f32::<LittleEndian>()?;
        let blend_time = reader.read_f32::<LittleEndian>()?;
        let loops = reader.read_bool()?;
        let num_actions = reader.read_i32::<LittleEndian>()?;
        let mut actions = Vec::with_capacity(num_actions as usize);
        for _ in 0..num_actions {
            let action = AnimationAction::read(reader)?;
            actions.push(action);
        }
        Ok(AnimationClip {
            kind,
            key,
            speed,
            blend_time,
            loops,
            actions,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnimationAction {
    kind: AnimationActionKind,
    start: f32,
    end: f32,
}

impl AnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_7bit_length_string()?;
        let start = reader.read_f32::<LittleEndian>()?;
        let end = reader.read_f32::<LittleEndian>()?;
        let kind = match kind.as_str() {
            "Footstep" => {
                let action = FootstepAnimationAction;
                AnimationActionKind::Footstep(action)
            }
            "Move" => {
                let action = MoveAnimationAction::read(reader)?;
                AnimationActionKind::Move(action)
            }
            "Jump" => {
                let action = JumpAnimationAction::read(reader)?;
                AnimationActionKind::Jump(action)
            }
            "Crouch" => {
                let action = CrouchAnimationAction::read(reader)?;
                AnimationActionKind::Crouch(action)
            }
            "Block" => {
                let action = BlockAnimationAction::read(reader)?;
                AnimationActionKind::Block(action)
            }
            "Grip" => {
                let action = GripAnimationAction::read(reader)?;
                AnimationActionKind::Grip(action)
            }
            "DamageGrip" => {
                let action = DamageGripAnimationAction::read(reader)?;
                AnimationActionKind::DamageGrip(action)
            }
            "OverkillGrip" => {
                let action = OverkillGripAnimationAction;
                AnimationActionKind::OverkillGrip(action)
            }
            "ThrowGrip" => {
                let action = ThrowGripAnimationAction;
                AnimationActionKind::ThrowGrip(action)
            }
            "ReleaseGrip" => {
                let action = ReleaseGripAnimationAction;
                AnimationActionKind::ReleaseGrip(action)
            }
            "BreakFree" => {
                let action = BreakFreeAnimationAction::read(reader)?;
                AnimationActionKind::BreakFree(action)
            }
            "Gunfire" => {
                let action = GunfireAnimationAction::read(reader)?;
                AnimationActionKind::Gunfire(action)
            }
            "DealDamage" => {
                let action = DealDamageAnimationAction::read(reader)?;
                AnimationActionKind::DealDamage(action)
            }
            "RemoveStatus" => {
                let action = RemoveStatusAnimationAction::read(reader)?;
                AnimationActionKind::RemoveStatus(action)
            }
            "SpecialAbility" => {
                let action = SpecialAbilityAnimationAction::read(reader)?;
                AnimationActionKind::SpecialAbility(action)
            }
            "CastSpell" => {
                let action = CastSpellAnimationAction::read(reader)?;
                AnimationActionKind::CastSpell(action)
            }
            "SpawnMissile" => {
                let action = SpawnMissileAnimationAction::read(reader)?;
                AnimationActionKind::SpawnMissile(action)
            }
            "Tongue" => {
                let action = TongueAnimationAction::read(reader)?;
                AnimationActionKind::Tongue(action)
            }
            "Invisible" => {
                let action = InvisibleAnimationAction::read(reader)?;
                AnimationActionKind::Invisible(action)
            }
            "Ethereal" => {
                let action = EtherealAnimationAction::read(reader)?;
                AnimationActionKind::Ethereal(action)
            }
            "Immortal" => {
                let action = ImmortalAnimationAction::read(reader)?;
                AnimationActionKind::Immortal(action)
            }
            "Suicide" => {
                let action = SuicideAnimationAction::read(reader)?;
                AnimationActionKind::Suicide(action)
            }
            "WeaponVisibility" => {
                let action = WeaponVisibilityAnimationAction::read(reader)?;
                AnimationActionKind::WeaponVisibility(action)
            }
            "DetachItem" => {
                let action = DetachItemAnimationAction::read(reader)?;
                AnimationActionKind::DetachItem(action)
            }
            "CameraShake" => {
                let action = CameraShakeAnimationAction::read(reader)?;
                AnimationActionKind::CameraShake(action)
            }
            "PlaySound" => {
                let action = PlaySoundAnimationAction::read(reader)?;
                AnimationActionKind::PlaySound(action)
            }
            "PlayEffect" => {
                let action = PlayEffectAnimationAction::read(reader)?;
                AnimationActionKind::PlayEffect(action)
            }
            v => {
                anyhow::bail!("unknown animation action kind: {v}");
            }
        };
        Ok(AnimationAction { kind, start, end })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AnimationActionKind {
    Footstep(FootstepAnimationAction),
    Move(MoveAnimationAction),
    Jump(JumpAnimationAction),
    Crouch(CrouchAnimationAction),
    Block(BlockAnimationAction),
    Grip(GripAnimationAction),
    DamageGrip(DamageGripAnimationAction),
    OverkillGrip(OverkillGripAnimationAction),
    ThrowGrip(ThrowGripAnimationAction),
    ReleaseGrip(ReleaseGripAnimationAction),
    BreakFree(BreakFreeAnimationAction),
    Gunfire(GunfireAnimationAction),
    DealDamage(DealDamageAnimationAction),
    RemoveStatus(RemoveStatusAnimationAction),
    SpecialAbility(SpecialAbilityAnimationAction),
    CastSpell(CastSpellAnimationAction),
    SpawnMissile(SpawnMissileAnimationAction),
    Tongue(TongueAnimationAction),
    Invisible(InvisibleAnimationAction),
    Ethereal(EtherealAnimationAction),
    Immortal(ImmortalAnimationAction),
    Suicide(SuicideAnimationAction),
    WeaponVisibility(WeaponVisibilityAnimationAction),
    DetachItem(DetachItemAnimationAction),
    CameraShake(CameraShakeAnimationAction),
    PlaySound(PlaySoundAnimationAction),
    PlayEffect(PlayEffectAnimationAction),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FootstepAnimationAction;

#[derive(Serialize, Deserialize, Debug)]
pub struct MoveAnimationAction {
    pub velocity: Vec3,
}

impl MoveAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let velocity = reader.read_vec3()?;
        Ok(MoveAnimationAction { velocity })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JumpAnimationAction {
    pub elevation: f32,
    pub min_range: f32,
    pub max_range: f32,
}

impl JumpAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let elevation = reader.read_f32::<LittleEndian>()?;
        let has_min_range = reader.read_bool()?;
        let min_range = if has_min_range {
            reader.read_f32::<LittleEndian>()?
        } else {
            0.0
        };
        let has_max_range = reader.read_bool()?;
        let max_range = if has_max_range {
            reader.read_f32::<LittleEndian>()?
        } else {
            0.0
        };
        Ok(JumpAnimationAction {
            elevation,
            min_range,
            max_range,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrouchAnimationAction {
    pub radius: f32,
    pub length: f32,
}

impl CrouchAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let radius = reader.read_f32::<LittleEndian>()?;
        let length = reader.read_f32::<LittleEndian>()?;
        Ok(CrouchAnimationAction { radius, length })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockAnimationAction {
    pub weapon_slot: i32,
}

impl BlockAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let weapon_slot = reader.read_i32::<LittleEndian>()?;
        Ok(BlockAnimationAction { weapon_slot })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GripAnimationAction {
    pub kind: GripKind,
    pub radius: f32,
    pub break_free_tolerance: f32,
    pub bone_a: String,
    pub bone_b: String,
    pub finish_on_grip: bool,
}

impl GripAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = GripKind::read(reader)?;
        let radius = reader.read_f32::<LittleEndian>()?;
        let break_free_tolerance = reader.read_f32::<LittleEndian>()?;
        let bone_a = reader.read_7bit_length_string()?;
        let bone_b = reader.read_7bit_length_string()?;
        let finish_on_grip = reader.read_bool()?;
        Ok(GripAnimationAction {
            kind,
            radius,
            break_free_tolerance,
            bone_a,
            bone_b,
            finish_on_grip,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DamageGripAnimationAction {
    pub damage_owner: bool,
    pub damages: Vec<Damage>,
}

impl DamageGripAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let damage_owner = reader.read_bool()?;
        let num_damages = reader.read_i32::<LittleEndian>()?;
        let mut damages = Vec::with_capacity(num_damages as usize);
        for _ in 0..num_damages {
            let damage = Damage::read(reader)?;
            damages.push(damage);
        }
        Ok(DamageGripAnimationAction {
            damage_owner,
            damages,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OverkillGripAnimationAction;

#[derive(Serialize, Deserialize, Debug)]
pub struct ThrowGripAnimationAction;

#[derive(Serialize, Deserialize, Debug)]
pub struct ReleaseGripAnimationAction;

#[derive(Serialize, Deserialize, Debug)]
pub struct BreakFreeAnimationAction {
    pub weapon_slot: i32,
    pub magnitude: f32,
}

impl BreakFreeAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let magnitude = reader.read_f32::<LittleEndian>()?;
        let weapon_slot = reader.read_i32::<LittleEndian>()?;
        Ok(BreakFreeAnimationAction {
            weapon_slot,
            magnitude,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GunfireAnimationAction {
    pub weapon_slot: i32,
    pub accuracy: f32,
}

impl GunfireAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let weapon_slot = reader.read_i32::<LittleEndian>()?;
        let accuracy = reader.read_f32::<LittleEndian>()?;
        Ok(GunfireAnimationAction {
            weapon_slot,
            accuracy,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DealDamageAnimationAction {
    pub weapon_slot: i32,
    pub targets: AnimationTargets,
}

impl DealDamageAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let weapon_slot = reader.read_i32::<LittleEndian>()?;
        let targets = AnimationTargets::read(reader)?;
        Ok(DealDamageAnimationAction {
            weapon_slot,
            targets,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveStatusAnimationAction {
    pub status: String,
}

impl RemoveStatusAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let status = reader.read_7bit_length_string()?;
        Ok(RemoveStatusAnimationAction { status })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpecialAbilityAnimationAction {
    pub weapon_slot: i32,
    pub ability: Option<SpecialAbility>,
}

impl SpecialAbilityAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let weapon_slot = reader.read_i32::<LittleEndian>()?;
        let ability = if weapon_slot < 0 {
            let ability = SpecialAbility::read(reader)?;
            Some(ability)
        } else {
            None
        };
        Ok(SpecialAbilityAnimationAction {
            weapon_slot,
            ability,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CastSpellAnimationAction {
    pub from_staff: bool,
    pub bone: Option<String>,
}

impl CastSpellAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let from_staff = reader.read_bool()?;
        let bone = if !from_staff {
            Some(reader.read_7bit_length_string()?)
        } else {
            None
        };
        Ok(CastSpellAnimationAction { from_staff, bone })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnMissileAnimationAction {
    pub weapon_slot: i32,
    pub velocity: Vec3,
    pub aligned: bool,
}

impl SpawnMissileAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let weapon_slot = reader.read_i32::<LittleEndian>()?;
        let velocity = reader.read_vec3()?;
        let aligned = reader.read_bool()?;
        Ok(SpawnMissileAnimationAction {
            weapon_slot,
            velocity,
            aligned,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TongueAnimationAction {
    pub max_length: f32,
}

impl TongueAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let max_length = reader.read_f32::<LittleEndian>()?;
        Ok(TongueAnimationAction { max_length })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InvisibleAnimationAction {
    pub shimmer: bool,
}

impl InvisibleAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let shimmer = reader.read_bool()?;
        Ok(InvisibleAnimationAction { shimmer })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EtherealAnimationAction {
    pub is_ethereal: bool,
    pub alpha: f32,
    pub speed: f32,
}

impl EtherealAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let is_ethereal = reader.read_bool()?;
        let alpha = reader.read_f32::<LittleEndian>()?;
        let speed = reader.read_f32::<LittleEndian>()?;
        Ok(EtherealAnimationAction {
            is_ethereal,
            alpha,
            speed,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImmortalAnimationAction {
    pub collide: bool,
}

impl ImmortalAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let collide = reader.read_bool()?;
        Ok(ImmortalAnimationAction { collide })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SuicideAnimationAction {
    pub overkill: bool,
}

impl SuicideAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let overkill = reader.read_bool()?;
        Ok(SuicideAnimationAction { overkill })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WeaponVisibilityAnimationAction {
    pub weapon_slot: i32,
    pub visible: bool,
}

impl WeaponVisibilityAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let weapon_slot = reader.read_i32::<LittleEndian>()?;
        let visible = reader.read_bool()?;
        Ok(WeaponVisibilityAnimationAction {
            weapon_slot,
            visible,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DetachItemAnimationAction {
    pub weapon_slot: i32,
    pub velocity: Vec3,
}

impl DetachItemAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let weapon_slot = reader.read_i32::<LittleEndian>()?;
        let velocity = reader.read_vec3()?;
        Ok(DetachItemAnimationAction {
            weapon_slot,
            velocity,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CameraShakeAnimationAction {
    pub unk: String,
    pub duration: f32,
    pub magnitude: f32,
}

impl CameraShakeAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let unk = reader.read_7bit_length_string()?;
        let duration = reader.read_f32::<LittleEndian>()?;
        let magnitude = reader.read_f32::<LittleEndian>()?;
        Ok(CameraShakeAnimationAction {
            unk,
            duration,
            magnitude,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlaySoundAnimationAction {
    pub sound: Sound,
}

impl PlaySoundAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let cue = reader.read_7bit_length_string()?;
        let bank = Bank::read(reader)?;
        let sound = Sound { cue, bank };
        Ok(PlaySoundAnimationAction { sound })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayEffectAnimationAction {
    pub bone: String,
    pub attached: bool,
    pub effect: String,
    pub value: f32,
}

impl PlayEffectAnimationAction {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let bone = reader.read_7bit_length_string()?;
        let attached = reader.read_bool()?;
        let effect = reader.read_7bit_length_string()?;
        let value = reader.read_f32::<LittleEndian>()?;
        Ok(PlayEffectAnimationAction {
            bone,
            attached,
            effect,
            value,
        })
    }
}

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug)]
pub enum GripKind {
    Pickup,
    Ride,
    Hold,
}

impl GripKind {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let kind =
            GripKind::from_repr(value).ok_or_else(|| anyhow!("unknown grip kind: {value}"))?;
        Ok(kind)
    }
}

bitflags! {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct AnimationTargets: u8 {
        const Friendly = 1;
        const Enemy = 2;
        const NonCharacters = 4;
        const All = 255;
    }
}

impl AnimationTargets {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let targets = AnimationTargets::from_bits(value)
            .ok_or_else(|| anyhow!("unknown animation targets: {value}"))?;
        Ok(targets)
    }
}
