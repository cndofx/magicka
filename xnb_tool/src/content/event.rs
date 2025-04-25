use std::io::Read;

use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use glam::Vec3;
use serde::{Deserialize, Serialize};

use super::{
    ai::{Order, ReactionTriggers},
    attack_property::AttackProperties,
    element::Elements,
    light::Light,
    sound::Bank,
};
use crate::ext::MyReadBytesExt;

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    Damage(DamageEvent),
    Splash(SplashEvent),
    Sound(SoundEvent),
    Effect(EffectEvent),
    Remove(RemoveEvent),
    Spawn(SpawnEvent),
    SpawnGibs(SpawnGibsEvent),
    SpawnItem(SpawnItemEvent),
    SpawnMagick(SpawnMagickEvent),
    SpawnMissile(SpawnMissileEvent),
    Light(LightEvent),
    CastMagick(CastMagickEvent),
    DamageOwner(DamageOwnerEvent),
}

impl Event {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_u8()?;
        match kind {
            0 => {
                let event = DamageEvent::read(reader)?;
                Ok(Event::Damage(event))
            }
            1 => {
                let event = SplashEvent::read(reader)?;
                Ok(Event::Splash(event))
            }
            2 => {
                let event = SoundEvent::read(reader)?;
                Ok(Event::Sound(event))
            }
            3 => {
                let event = EffectEvent::read(reader)?;
                Ok(Event::Effect(event))
            }
            4 => {
                let event = RemoveEvent::read(reader)?;
                Ok(Event::Remove(event))
            }
            5 => {
                todo!("camera shake event");
            }
            6 => {
                todo!("decal event");
            }
            7 => {
                todo!("blast event (invalid?)");
            }
            8 => {
                let event = SpawnEvent::read(reader)?;
                Ok(Event::Spawn(event))
            }
            9 => {
                todo!("overkill event");
            }
            10 => {
                let event = SpawnGibsEvent::read(reader)?;
                Ok(Event::SpawnGibs(event))
            }
            11 => {
                let event = SpawnItemEvent::read(reader)?;
                Ok(Event::SpawnItem(event))
            }
            12 => {
                let event = SpawnMagickEvent::read(reader)?;
                Ok(Event::SpawnMagick(event))
            }
            13 => {
                let event = SpawnMissileEvent::read(reader)?;
                Ok(Event::SpawnMissile(event))
            }
            14 => {
                let event = LightEvent::read(reader)?;
                Ok(Event::Light(event))
            }
            15 => {
                let event = CastMagickEvent::read(reader)?;
                Ok(Event::CastMagick(event))
            }
            16 => {
                let event = DamageOwnerEvent::read(reader)?;
                Ok(Event::DamageOwner(event))
            }
            17 => {
                todo!("callback event (invalid?)");
            }
            _ => Err(anyhow!("unknown event kind: {kind}")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DamageEvent {
    attack_properties: AttackProperties,
    elements: Elements,
    amount: f32,
    magnitude: f32,
    velocity_based: bool,
}

impl DamageEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let attack_properties = AttackProperties::read(reader)?;
        let elements = Elements::read(reader)?;
        let amount = reader.read_f32::<LittleEndian>()?;
        let magnitude = reader.read_f32::<LittleEndian>()?;
        let velocity_based = reader.read_bool()?;
        Ok(DamageEvent {
            attack_properties,
            elements,
            amount,
            magnitude,
            velocity_based,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SplashEvent {
    attack_properties: AttackProperties,
    elements: Elements,
    amount: i32,
    magnitude: f32,
    radius: f32,
}

impl SplashEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let attack_properties = AttackProperties::read(reader)?;
        let element = Elements::read(reader)?;
        let amount = reader.read_i32::<LittleEndian>()?;
        let magnitude = reader.read_f32::<LittleEndian>()?;
        let radius = reader.read_f32::<LittleEndian>()?;
        Ok(SplashEvent {
            attack_properties,
            elements: element,
            amount,
            magnitude,
            radius,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SoundEvent {
    banks: Bank,
    cue: String,
    magnitude: f32,
    stop_on_remove: bool,
}

impl SoundEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let bank = Bank::read(reader)?;
        let cue = reader.read_7bit_length_string()?;
        let magnitude = reader.read_f32::<LittleEndian>()?;
        let stop_on_remove = reader.read_bool()?;
        Ok(SoundEvent {
            banks: bank,
            cue,
            magnitude,
            stop_on_remove,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EffectEvent {
    follow: bool,
    world_aligned: bool,
    effect: String,
}

impl EffectEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let follow = reader.read_bool()?;
        let world_aligned = reader.read_bool()?;
        let effect = reader.read_7bit_length_string()?;
        Ok(EffectEvent {
            follow,
            world_aligned,
            effect,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RemoveEvent {
    bounces: i32,
}

impl RemoveEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let bounces = reader.read_i32::<LittleEndian>()?;
        let event = RemoveEvent { bounces };
        Ok(event)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnEvent {
    pub kind: String,
    pub idle_animation: String,
    pub spawn_animation: String,
    pub health: f32,
    pub order: Order,
    pub react_to: ReactionTriggers,
    pub reaction: Order,
    pub rotation: f32,
    pub offset: Vec3,
}

impl SpawnEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_7bit_length_string()?;
        let idle_animation = reader.read_7bit_length_string()?;
        let spawn_animation = reader.read_7bit_length_string()?;
        let health = reader.read_f32::<LittleEndian>()?;
        let order = Order::read(reader)?;
        let react_to = ReactionTriggers::read(reader)?;
        let reaction = Order::read(reader)?;
        let rotation = reader.read_f32::<LittleEndian>()?;
        let offset = reader.read_vec3()?;
        Ok(SpawnEvent {
            kind,
            idle_animation,
            spawn_animation,
            health,
            order,
            react_to,
            reaction,
            rotation,
            offset,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnGibsEvent {
    start_index: i32,
    end_index: i32,
}

impl SpawnGibsEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let start_index = reader.read_i32::<LittleEndian>()?;
        let end_index = reader.read_i32::<LittleEndian>()?;
        Ok(SpawnGibsEvent {
            start_index,
            end_index,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnItemEvent {
    item: String,
}

impl SpawnItemEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let item = reader.read_7bit_length_string()?;
        Ok(SpawnItemEvent { item })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnMagickEvent {
    magick: String,
}

impl SpawnMagickEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let magick = reader.read_7bit_length_string()?;
        Ok(SpawnMagickEvent { magick })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SpawnMissileEvent {
    kind: String,
    velocity: Vec3,
    facing: bool,
}

impl SpawnMissileEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_7bit_length_string()?;
        let velocity = reader.read_vec3()?;
        let facing = reader.read_bool()?;

        Ok(SpawnMissileEvent {
            kind,
            velocity,
            facing,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LightEvent {
    light: Light,
}

impl LightEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let light = Light::read(reader)?;
        Ok(LightEvent { light })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CastMagickEvent {
    kind: String,
    elements: Vec<Elements>,
}

impl CastMagickEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = reader.read_7bit_length_string()?;
        let num_elements = reader.read_i32::<LittleEndian>()? as usize;
        let mut elements = Vec::with_capacity(num_elements);
        for _ in 0..num_elements {
            let element = Elements::read(reader)?;
            elements.push(element);
        }
        Ok(CastMagickEvent { kind, elements })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DamageOwnerEvent {
    attack_properties: AttackProperties,
    elements: Elements,
    amount: f32,
    magnitude: f32,
    velocity_based: bool,
}

impl DamageOwnerEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let attack_properties = AttackProperties::read(reader)?;
        let elements = Elements::read(reader)?;
        let amount = reader.read_f32::<LittleEndian>()?;
        let magnitude = reader.read_f32::<LittleEndian>()?;
        let velocity_based = reader.read_bool()?;
        Ok(DamageOwnerEvent {
            attack_properties,
            elements,
            amount,
            magnitude,
            velocity_based,
        })
    }
}

bitflags! {
    #[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
    pub struct EventConditionKind: u8 {
        const Default = 1;
        const Hit = 2;
        const Collision = 4;
        const Damaged = 8;
        const Timer = 16;
        const Death = 32;
        const Overkill = 64;
    }
}

impl EventConditionKind {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let kind = EventConditionKind::from_bits(value)
            .ok_or_else(|| anyhow!("unknown event condition kind: {value}"))?;
        Ok(kind)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EventConditions {
    kind: EventConditionKind,
    hitpoints: f32,
    element: Elements,
    threshold: f32,
    time: f32,
    repeat: bool,
    events: Vec<Event>,
}

impl EventConditions {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = EventConditionKind::read(reader)?;
        let hitpoints = reader.read_i32::<LittleEndian>()? as f32;
        let element = Elements::read(reader)?;
        let threshold = reader.read_f32::<LittleEndian>()?;
        let time = reader.read_f32::<LittleEndian>()?;
        let repeat = reader.read_bool()?;

        let num_events = reader.read_i32::<LittleEndian>()?;
        let mut events = Vec::with_capacity(num_events as usize);
        for _ in 0..num_events {
            let event = Event::read(reader)?;
            events.push(event);
        }

        Ok(EventConditions {
            kind,
            hitpoints,
            element,
            threshold,
            time,
            repeat,
            events,
        })
    }
}
