use std::io::Read;

use character::Character;
use effect::RenderDeferredEffect;
use item::Item;
use model::{IndexBuffer, Model, VertexBuffer, VertexDeclaration};
use serde::{Deserialize, Serialize};
use texture::Texture2D;

use crate::ext::MyReadBytesExt;
use crate::xnb::TypeReader;

pub mod ability;
pub mod ai;
pub mod animation;
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
pub mod effect;
pub mod element;
pub mod event;
pub mod faction;
pub mod gib;
pub mod item;
pub mod light;
pub mod model;
pub mod movement;
pub mod passive_ability;
pub mod resistance;
pub mod sound;
pub mod special_ability;
pub mod texture;
pub mod weapon_class;

const ITEM_READER_NAME: &str = "Magicka.ContentReaders.ItemReader";
const CHARACTER_READER_NAME: &str = "Magicka.ContentReaders.CharacterTemplateReader";

const STRING_READER_NAME: &str = "Microsoft.Xna.Framework.Content.StringReader";
const TEXTURE_2D_READER_NAME: &str = "Microsoft.Xna.Framework.Content.Texture2DReader";
const MODEL_READER_NAME: &str = "Microsoft.Xna.Framework.Content.ModelReader";
const VERTEX_DECL_READER_NAME: &str = "Microsoft.Xna.Framework.Content.VertexDeclarationReader";
const VERTEX_BUFFER_READER_NAME: &str = "Microsoft.Xna.Framework.Content.VertexBufferReader";
const INDEX_BUFFER_READER_NAME: &str = "Microsoft.Xna.Framework.Content.IndexBufferReader";

const RENDER_DEFERRED_EFFECT_READER_NAME: &str = "PolygonHead.Pipeline.RenderDeferredEffectReader";

#[derive(Serialize, Deserialize, Debug)]
pub enum Content {
    Null,
    Item(Item),
    Character(Character),
    String(String),
    Texture2D(Texture2D),
    Model(Model),
    VertexDeclaration(VertexDeclaration),
    VertexBuffer(VertexBuffer),
    IndexBuffer(IndexBuffer),
    RenderDeferredEffect(RenderDeferredEffect),
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
            STRING_READER_NAME => {
                let string = reader.read_7bit_length_string()?;
                return Ok(Content::String(string));
            }
            ITEM_READER_NAME => {
                let item = Item::read(reader)?;
                return Ok(Content::Item(item));
            }
            CHARACTER_READER_NAME => {
                let character = Character::read(reader)?;
                return Ok(Content::Character(character));
            }
            RENDER_DEFERRED_EFFECT_READER_NAME => {
                let effect = RenderDeferredEffect::read(reader)?;
                return Ok(Content::RenderDeferredEffect(effect));
            }
            TEXTURE_2D_READER_NAME => {
                let texture = Texture2D::read(reader)?;
                return Ok(Content::Texture2D(texture));
            }
            MODEL_READER_NAME => {
                let model = Model::read(reader, type_readers)?;
                return Ok(Content::Model(model));
            }
            VERTEX_DECL_READER_NAME => {
                let decl = VertexDeclaration::read(reader)?;
                return Ok(Content::VertexDeclaration(decl));
            }
            VERTEX_BUFFER_READER_NAME => {
                let buffer = VertexBuffer::read(reader)?;
                return Ok(Content::VertexBuffer(buffer));
            }
            INDEX_BUFFER_READER_NAME => {
                let buffer = IndexBuffer::read(reader)?;
                return Ok(Content::IndexBuffer(buffer));
            }
            _ => {
                anyhow::bail!("unknown type reader: {}", type_reader.name);
            }
        }
    }
}
