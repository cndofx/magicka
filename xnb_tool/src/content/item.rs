use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;

use super::{
    aura::Aura,
    event::EventConditions,
    light::Light,
    passive_ability::PassiveAbility,
    resistance::Resistance,
    sound::{Bank, Sound},
    special_ability::{SpecialAbility, SpecialAbilityWithCooldown},
    weapon_class::WeaponClass,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    name: String,
    locale_name: String,
    locale_description: String,
    sounds: Vec<Sound>,
    pickupable: bool,
    bound: bool,
    block_value: i32,
    weapon_class: WeaponClass,
    cooldown_time: f32,
    hide_model: bool,
    hide_effect: bool,
    pause_sounds: bool,
    resistances: Vec<Resistance>,
    passive_ability: PassiveAbility,
    effects: Vec<String>,
    lights: Vec<Light>,
    special_ability: Option<SpecialAbilityWithCooldown>,
    melee_range: f32,
    melee_multi_hit: bool,
    melee_conditions: Vec<EventConditions>,
    ranged_range: f32,
    facing: bool,
    homing_strength: f32,
    ranged_elevation: f32,
    ranged_danger: f32,
    gun_range: f32,
    gun_clip: i32,
    gun_rate: i32,
    gun_accuracy: f32,
    gun_sound_cue: String,
    gun_muzzle_effect: String,
    gun_shell_effect: String,
    gun_tracer_velocity: f32,
    gun_non_tracer: String,
    gun_tracer: String,
    gun_conditions: Vec<EventConditions>,
    projectile_model: String,
    ranged_conditions: Vec<EventConditions>,
    scale: f32,
    model: String,
    auras: Vec<Aura>,
}

impl Item {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let name = reader.read_7bit_length_string()?;
        let locale_name = reader.read_7bit_length_string()?;
        let locale_description = reader.read_7bit_length_string()?;

        let num_sounds = reader.read_i32::<LittleEndian>()?;
        let mut sounds = Vec::with_capacity(num_sounds as usize);
        for _ in 0..num_sounds {
            let cue = reader.read_7bit_length_string()?;
            let bank = Bank::read(reader)?;
            let sound = Sound { cue, bank };
            sounds.push(sound);
        }

        let pickupable = reader.read_bool()?;
        let bound = reader.read_bool()?;
        let block_value = reader.read_i32::<LittleEndian>()?;
        let weapon_class = reader.read_u8()?;
        let weapon_class = WeaponClass::from_repr(weapon_class)
            .ok_or_else(|| anyhow!("unknown weapon class: {weapon_class}"))?;
        let cooldown_time = reader.read_f32::<LittleEndian>()?;
        let hide_model = reader.read_bool()?;
        let hide_effect = reader.read_bool()?;
        let pause_sounds = reader.read_bool()?;

        let num_resistances = reader.read_i32::<LittleEndian>()?;
        let mut resistances = Vec::with_capacity(num_resistances as usize);
        for _ in 0..num_resistances {
            let resistance = Resistance::read(reader)?;
            resistances.push(resistance);
        }

        let passive_ability = PassiveAbility::read(reader)?;

        let num_effects = reader.read_i32::<LittleEndian>()?;
        let mut effects = Vec::with_capacity(num_effects as usize);
        for _ in 0..num_effects {
            let effect = reader.read_7bit_length_string()?;
            effects.push(effect);
        }

        let num_lights = reader.read_i32::<LittleEndian>()?;
        let mut lights = Vec::with_capacity(num_lights as usize);
        for _ in 0..num_lights {
            let light = Light::read(reader)?;
            lights.push(light);
        }

        let has_special_ability = reader.read_bool()?;
        let special_ability = if has_special_ability {
            let ability = SpecialAbilityWithCooldown::read(reader)?;
            Some(ability)
        } else {
            None
        };

        let melee_range = reader.read_f32::<LittleEndian>()?;
        let melee_multi_hit = reader.read_bool()?;
        let num_melee_conditions = reader.read_i32::<LittleEndian>()?;
        let mut melee_conditions = Vec::with_capacity(num_melee_conditions as usize);
        for _ in 0..num_melee_conditions {
            let condition = EventConditions::read(reader)?;
            melee_conditions.push(condition);
        }

        let ranged_range = reader.read_f32::<LittleEndian>()?;
        let facing = reader.read_bool()?;
        let homing_strength = reader.read_f32::<LittleEndian>()?;
        let ranged_elevation = reader.read_f32::<LittleEndian>()?;
        let ranged_danger = reader.read_f32::<LittleEndian>()?;

        let gun_range = reader.read_f32::<LittleEndian>()?;
        let gun_clip = reader.read_i32::<LittleEndian>()?;
        let gun_rate = reader.read_i32::<LittleEndian>()?;
        let gun_accuracy = reader.read_f32::<LittleEndian>()?;
        let gun_sound_cue = reader.read_7bit_length_string()?;
        let gun_muzzle_effect = reader.read_7bit_length_string()?;
        let gun_shell_effect = reader.read_7bit_length_string()?;
        let gun_tracer_velocity = reader.read_f32::<LittleEndian>()?;
        let gun_non_tracer = reader.read_7bit_length_string()?;
        let gun_tracer = reader.read_7bit_length_string()?;

        let num_gun_conditions = reader.read_i32::<LittleEndian>()?;
        let mut gun_conditions = Vec::with_capacity(num_gun_conditions as usize);
        for _ in 0..num_gun_conditions {
            let condition = EventConditions::read(reader)?;
            gun_conditions.push(condition);
        }

        let projectile_model = reader.read_7bit_length_string()?;

        let num_ranged_conditions = reader.read_i32::<LittleEndian>()?;
        let mut ranged_conditions = Vec::with_capacity(num_ranged_conditions as usize);
        for _ in 0..num_ranged_conditions {
            let condition = EventConditions::read(reader)?;
            ranged_conditions.push(condition);
        }

        let scale = reader.read_f32::<LittleEndian>()?;
        let model = reader.read_7bit_length_string()?;

        let num_auras = reader.read_i32::<LittleEndian>()?;
        let mut auras = Vec::with_capacity(num_auras as usize);
        for _ in 0..num_auras {
            let aura = Aura::read(reader)?;
            auras.push(aura);
        }

        let item = Item {
            name,
            locale_name,
            locale_description,
            sounds,
            pickupable,
            bound,
            block_value,
            weapon_class,
            cooldown_time,
            hide_model,
            hide_effect,
            pause_sounds,
            resistances,
            passive_ability,
            effects,
            lights,
            special_ability,
            melee_range,
            melee_multi_hit,
            melee_conditions,
            ranged_range,
            facing,
            homing_strength,
            ranged_elevation,
            ranged_danger,
            gun_range,
            gun_clip,
            gun_rate,
            gun_accuracy,
            gun_sound_cue,
            gun_muzzle_effect,
            gun_shell_effect,
            gun_tracer_velocity,
            gun_non_tracer,
            gun_tracer,
            gun_conditions,
            projectile_model,
            ranged_conditions,
            scale,
            model,
            auras,
        };
        Ok(item)
    }
}
