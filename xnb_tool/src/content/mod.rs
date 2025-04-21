use std::io::Read;

use item::Item;

use crate::ext::MyReadBytesExt;
use crate::xnb::TypeReader;

pub mod attack_property;
pub mod color;
pub mod element;
pub mod event;
pub mod item;
pub mod light;
pub mod passive_ability;
pub mod resistance;
pub mod sound;
pub mod special_ability;
pub mod weapon_class;

const ITEM_READER_NAME: &str = "LMagicka.ContentReaders.ItemReader";

#[derive(Debug)]
pub enum Content {
    Null,
    Item(Item),
}

impl Content {
    pub fn read(mut reader: impl Read, type_readers: &[TypeReader]) -> anyhow::Result<Self> {
        let type_id = reader.read_7bit_encoded_i32()? as usize;
        if type_id == 0 {
            return Ok(Content::Null);
        }
        let type_reader = &type_readers[type_id - 1];
        dbg!(type_reader);

        if type_reader.name.starts_with(ITEM_READER_NAME) {
            let item = Item::read(&mut reader)?;
            return Ok(Content::Item(item));
        } else {
            anyhow::bail!("unknown type reader: {}", type_reader.name);
        }
    }
}
