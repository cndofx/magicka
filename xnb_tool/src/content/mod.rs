use std::io::Read;

use character::Character;
use item::Item;
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;
use crate::xnb::TypeReader;

pub mod ability;
pub mod ai;
pub mod animation_action;
pub mod animation_clip;
pub mod animation_set;
pub mod attachment;
pub mod attack_property;
pub mod aura;
pub mod blood_kind;
pub mod boned_effect;
pub mod boned_light;
pub mod character;
pub mod character_model;
pub mod color;
pub mod damage;
pub mod element;
pub mod event;
pub mod faction;
pub mod gib;
pub mod item;
pub mod light;
pub mod movement;
pub mod passive_ability;
pub mod resistance;
pub mod sound;
pub mod special_ability;
pub mod vector3;
pub mod weapon_class;

const ITEM_READER_NAME: &str = "Magicka.ContentReaders.ItemReader";
const CHARACTER_READER_NAME: &str = "Magicka.ContentReaders.CharacterTemplateReader";

#[derive(Serialize, Deserialize, Debug)]
pub enum Content {
    Null,
    Item(Item),
    Character(Character),
}

impl Content {
    pub fn read(reader: &mut impl Read, type_readers: &[TypeReader]) -> anyhow::Result<Self> {
        let type_id = reader.read_7bit_encoded_i32()? as usize;
        if type_id == 0 {
            return Ok(Content::Null);
        }
        let type_reader = &type_readers[type_id - 1];

        let name = type_reader.name.split(",").next().unwrap();
        match name {
            ITEM_READER_NAME => {
                let item = Item::read(reader)?;
                return Ok(Content::Item(item));
            }
            CHARACTER_READER_NAME => {
                let character = Character::read(reader)?;
                return Ok(Content::Character(character));
            }
            _ => {
                anyhow::bail!("unknown type reader: {}", type_reader.name);
            }
        }
    }
}
