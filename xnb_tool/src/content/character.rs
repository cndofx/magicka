use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};

use crate::{
    content::{
        ability::Ability, attachment::Attachment, aura::Buff, blood_kind::BloodKind,
        boned_effect::BonedEffect, boned_light::BonedLight, character_model::CharacterModel,
        faction::Factions, gib::Gib, movement::Movement,
    },
    ext::MyReadBytesExt,
};

use super::{
    animation::AnimationSet,
    aura::Aura,
    event::EventConditions,
    resistance::Resistance,
    sound::{Bank, Sound},
};

pub const MAX_ANIMATION_SETS: usize = 27;

#[derive(Serialize, Deserialize, Debug)]
pub struct Character {
    pub name: String,
    pub locale_name: String,
    pub factions: Factions,
    pub blood: BloodKind,
    pub is_ethereal: bool,
    pub looks_ethereal: bool,
    pub fearless: bool,
    pub uncharmable: bool,
    pub non_slippery: bool,
    pub has_fairy: bool,
    pub can_see_invisible: bool,
    pub sounds: Vec<Sound>,
    pub gibs: Vec<Gib>,
    pub lights: Vec<BonedLight>,
    pub max_hitpoints: f32,
    pub num_healthbars: i32,
    pub undying: bool,
    pub undie_time: f32,
    pub undie_hitpoints: f32,
    pub pain_tolerance: i32,
    pub knockdown_tolerance: f32,
    pub score_value: i32,
    pub xp_value: i32,
    pub reward_on_kill: bool,
    pub reward_on_overkill: bool,
    pub regeneration: i32,
    pub max_panic: f32,
    pub zap_modifier: f32,
    pub length: f32,
    pub radius: f32,
    pub mass: f32,
    pub speed: f32,
    pub turn_speed: f32,
    pub bleed_rate: f32,
    pub stun_time: f32,
    pub summon_element_sound: Sound,
    pub resistances: Vec<Resistance>,
    pub models: Vec<CharacterModel>,
    pub animation_skeleton: String,
    pub effects: Vec<BonedEffect>,
    pub animations: Vec<AnimationSet>,
    pub equipment: Vec<Attachment>,
    pub conditions: Vec<EventConditions>,
    pub alert_radius: f32,
    pub group_chase: f32,
    pub group_separation: f32,
    pub group_cohesion: f32,
    pub group_alignment: f32,
    pub group_wander: f32,
    pub friendly_avoidance: f32,
    pub enemy_avoidance: f32,
    pub sight_avoidance: f32,
    pub danger_avoidance: f32,
    pub anger_weight: f32,
    pub distance_weight: f32,
    pub health_weight: f32,
    pub flocking: bool,
    pub break_free_strength: f32,
    pub abilities: Vec<Ability>,
    pub movements: Vec<Movement>,
    pub buffs: Vec<Buff>,
    pub auras: Vec<Aura>,
}

impl Character {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let name = reader.read_7bit_length_string()?;
        let locale_name = reader.read_7bit_length_string()?;
        let factions = Factions::read(reader)?;
        let blood = BloodKind::read(reader)?;
        let is_ethereal = reader.read_bool()?;
        let looks_ethereal = reader.read_bool()?;
        let fearless = reader.read_bool()?;
        let uncharmable = reader.read_bool()?;
        let non_slippery = reader.read_bool()?;
        let has_fairy = reader.read_bool()?;
        let can_see_invisible = reader.read_bool()?;

        let num_sounds = reader.read_i32::<LittleEndian>()?;
        let mut sounds = Vec::with_capacity(num_sounds as usize);
        for _ in 0..num_sounds {
            let cue = reader.read_7bit_length_string()?;
            let bank = Bank::read(reader)?;
            let sound = Sound { cue, bank };
            sounds.push(sound);
        }

        let num_gibs = reader.read_i32::<LittleEndian>()?;
        let mut gibs = Vec::with_capacity(num_gibs as usize);
        for _ in 0..num_gibs {
            let gib = Gib::read(reader)?;
            gibs.push(gib);
        }

        let num_lights = reader.read_i32::<LittleEndian>()?;
        let mut lights = Vec::with_capacity(num_lights as usize);
        for _ in 0..num_lights {
            let light = BonedLight::read(reader)?;
            lights.push(light);
        }

        let max_hitpoints = reader.read_f32::<LittleEndian>()?;
        let num_healthbars = reader.read_i32::<LittleEndian>()?;
        let undying = reader.read_bool()?;
        let undie_time = reader.read_f32::<LittleEndian>()?;
        let undie_hitpoints = reader.read_f32::<LittleEndian>()?;
        let pain_tolerance = reader.read_i32::<LittleEndian>()?;
        let knockdown_tolerance = reader.read_f32::<LittleEndian>()?;
        let score_value = reader.read_i32::<LittleEndian>()?;

        // modern only (?)
        let xp_value = reader.read_i32::<LittleEndian>()?;
        let reward_on_kill = reader.read_bool()?;
        let reward_on_overkill = reader.read_bool()?;

        let regeneration = reader.read_i32::<LittleEndian>()?;
        let max_panic = reader.read_f32::<LittleEndian>()?;
        let zap_modifier = reader.read_f32::<LittleEndian>()?;
        let length = reader.read_f32::<LittleEndian>()?;
        let radius = reader.read_f32::<LittleEndian>()?;
        let mass = reader.read_f32::<LittleEndian>()?;
        let speed = reader.read_f32::<LittleEndian>()?;
        let turn_speed = reader.read_f32::<LittleEndian>()?;
        let bleed_rate = reader.read_f32::<LittleEndian>()?;
        let stun_time = reader.read_f32::<LittleEndian>()?;

        let summon_element_bank = Bank::read(reader)?;
        let summon_element_cue = reader.read_7bit_length_string()?;
        let summon_element_sound = Sound {
            cue: summon_element_cue,
            bank: summon_element_bank,
        };

        let num_resistances = reader.read_i32::<LittleEndian>()?;
        let mut resistances = Vec::with_capacity(num_resistances as usize);
        for _ in 0..num_resistances {
            let resistance = Resistance::read(reader)?;
            resistances.push(resistance);
        }

        let num_models = reader.read_i32::<LittleEndian>()?;
        let mut models = Vec::with_capacity(num_models as usize);
        for _ in 0..num_models {
            let model = CharacterModel::read(reader)?;
            models.push(model);
        }

        let animation_skeleton = reader.read_7bit_length_string()?;

        let num_effects = reader.read_i32::<LittleEndian>()?;
        let mut effects = Vec::with_capacity(num_effects as usize);
        for _ in 0..num_effects {
            let effect = BonedEffect::read(reader)?;
            effects.push(effect);
        }

        let mut animations = Vec::with_capacity(MAX_ANIMATION_SETS);
        for _ in 0..MAX_ANIMATION_SETS {
            let set = AnimationSet::read(reader)?;
            animations.push(set);
        }

        let num_equipment = reader.read_i32::<LittleEndian>()?;
        let mut equipment = Vec::with_capacity(num_equipment as usize);
        for _ in 0..num_equipment {
            let attachment = Attachment::read(reader)?;
            equipment.push(attachment);
        }

        let num_conditions = reader.read_i32::<LittleEndian>()?;
        let mut conditions = Vec::with_capacity(num_conditions as usize);
        for _ in 0..num_conditions {
            let condition = EventConditions::read(reader)?;
            conditions.push(condition);
        }

        let alert_radius = reader.read_f32::<LittleEndian>()?;
        let group_chase = reader.read_f32::<LittleEndian>()?;
        let group_separation = reader.read_f32::<LittleEndian>()?;
        let group_cohesion = reader.read_f32::<LittleEndian>()?;
        let group_alignment = reader.read_f32::<LittleEndian>()?;
        let group_wander = reader.read_f32::<LittleEndian>()?;
        let friendly_avoidance = reader.read_f32::<LittleEndian>()?;
        let enemy_avoidance = reader.read_f32::<LittleEndian>()?;
        let sight_avoidance = reader.read_f32::<LittleEndian>()?;
        let danger_avoidance = reader.read_f32::<LittleEndian>()?;
        let anger_weight = reader.read_f32::<LittleEndian>()?;
        let distance_weight = reader.read_f32::<LittleEndian>()?;
        let health_weight = reader.read_f32::<LittleEndian>()?;
        let flocking = reader.read_bool()?;
        let break_free_strength = reader.read_f32::<LittleEndian>()?;

        let num_abilities = reader.read_i32::<LittleEndian>()?;
        let mut abilities = Vec::with_capacity(num_abilities as usize);
        for _ in 0..num_abilities {
            let ability = Ability::read(reader)?;
            abilities.push(ability);
        }

        let num_movements = reader.read_i32::<LittleEndian>()?;
        let mut movements = Vec::with_capacity(num_movements as usize);
        for _ in 0..num_movements {
            let movement = Movement::read(reader)?;
            movements.push(movement);
        }

        let num_buffs = reader.read_i32::<LittleEndian>()?;
        let mut buffs = Vec::with_capacity(num_buffs as usize);
        for _ in 0..num_buffs {
            let buff = Buff::read(reader)?;
            buffs.push(buff);
        }

        let num_auras = reader.read_i32::<LittleEndian>()?;
        let mut auras = Vec::with_capacity(num_auras as usize);
        for _ in 0..num_auras {
            let aura = Aura::read(reader)?;
            auras.push(aura);
        }

        Ok(Character {
            name,
            locale_name,
            factions,
            blood,
            is_ethereal,
            looks_ethereal,
            fearless,
            uncharmable,
            non_slippery,
            has_fairy,
            can_see_invisible,
            sounds,
            gibs,
            lights,
            max_hitpoints,
            num_healthbars,
            undying,
            undie_time,
            undie_hitpoints,
            pain_tolerance,
            knockdown_tolerance,
            score_value,
            xp_value,
            reward_on_kill,
            reward_on_overkill,
            regeneration,
            max_panic,
            zap_modifier,
            length,
            radius,
            mass,
            speed,
            turn_speed,
            bleed_rate,
            stun_time,
            summon_element_sound,
            resistances,
            models,
            animation_skeleton,
            effects,
            animations,
            equipment,
            conditions,
            alert_radius,
            group_chase,
            group_separation,
            group_cohesion,
            group_alignment,
            group_wander,
            friendly_avoidance,
            enemy_avoidance,
            sight_avoidance,
            danger_avoidance,
            anger_weight,
            distance_weight,
            health_weight,
            flocking,
            break_free_strength,
            abilities,
            movements,
            buffs,
            auras,
        })
    }
}
