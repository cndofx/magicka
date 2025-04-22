use std::io::Read;

use anyhow::anyhow;
use bitflags::bitflags;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use super::{attack_property::AttackProperty, element::Element, sound::Bank};
use crate::ext::MyReadBytesExt;

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    Damage(DamageEvent),
    Splash(SplashEvent),
    Sound(SoundEvent),
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
            _ => Err(anyhow!("unknown event kind: {kind}")),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DamageEvent {
    attack_properties: AttackProperty,
    elements: Element,
    amount: f32,
    magnitude: f32,
    velocity_based: bool,
}

impl DamageEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let attack_properties = AttackProperty::read(reader)?;
        let elements = Element::read(reader)?;
        let amount = reader.read_f32::<LittleEndian>()?;
        let magnitude = reader.read_f32::<LittleEndian>()?;
        let velocity_based = reader.read_bool()?;

        let event = DamageEvent {
            attack_properties,
            elements,
            amount,
            magnitude,
            velocity_based,
        };
        Ok(event)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SplashEvent {
    attack_properties: AttackProperty,
    elements: Element,
    amount: i32,
    magnitude: f32,
    radius: f32,
}

impl SplashEvent {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        todo!()
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

        let event = SoundEvent {
            banks: bank,
            cue,
            magnitude,
            stop_on_remove,
        };
        Ok(event)
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
    element: Element,
    threshold: f32,
    time: f32,
    repeat: bool,
    events: Vec<Event>,
}

impl EventConditions {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let kind = EventConditionKind::read(reader)?;
        let hitpoints = reader.read_i32::<LittleEndian>()? as f32;
        let element = Element::read(reader)?;
        let threshold = reader.read_f32::<LittleEndian>()?;
        let time = reader.read_f32::<LittleEndian>()?;
        let repeat = reader.read_bool()?;

        let num_events = reader.read_i32::<LittleEndian>()?;
        let mut events = Vec::with_capacity(num_events as usize);
        for _ in 0..num_events {
            let event = Event::read(reader)?;
            dbg!(&event);
            events.push(event);
        }

        let conditions = EventConditions {
            kind,
            hitpoints,
            element,
            threshold,
            time,
            repeat,
            events,
        };
        Ok(conditions)
    }
}
