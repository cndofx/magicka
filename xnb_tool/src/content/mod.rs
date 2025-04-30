use std::io::Read;

use character::Character;
use effect::{AdditiveEffect, BasicEffect, Effect, RenderDeferredEffect, SkinnedModelBasicEffect};
use item::Item;
use model::{IndexBuffer, Model, VertexBuffer, VertexDeclaration};
use serde::{Deserialize, Serialize};
use skinned_model::{SkinnedModel, SkinnedModelAnimationClip, SkinnedModelBone};
use texture::Texture2D;

use crate::ext::MyReadBytesExt;
use crate::xnb::TypeReader;

pub mod ability;
pub mod ai;
pub mod animation;
pub mod attack_property;
pub mod aura;
pub mod character;
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
pub mod skinned_model;
pub mod sound;
pub mod special_ability;
pub mod texture;
pub mod weapon_class;

const ITEM_READER_NAME: &str = "Magicka.ContentReaders.ItemReader";
const CHARACTER_READER_NAME: &str = "Magicka.ContentReaders.CharacterTemplateReader";

const STRING_READER_NAME: &str = "Microsoft.Xna.Framework.Content.StringReader";
const EXTERNAL_REFERENCE_READER_NAME: &str =
    "Microsoft.Xna.Framework.Content.ExternalReferenceReader";
const TEXTURE_2D_READER_NAME: &str = "Microsoft.Xna.Framework.Content.Texture2DReader";
const MODEL_READER_NAME: &str = "Microsoft.Xna.Framework.Content.ModelReader";
const VERTEX_DECL_READER_NAME: &str = "Microsoft.Xna.Framework.Content.VertexDeclarationReader";
const VERTEX_BUFFER_READER_NAME: &str = "Microsoft.Xna.Framework.Content.VertexBufferReader";
const INDEX_BUFFER_READER_NAME: &str = "Microsoft.Xna.Framework.Content.IndexBufferReader";
const EFFECT_READER_NAME: &str = "Microsoft.Xna.Framework.Content.EffectReader";
const BASIC_EFFECT_READER_NAME: &str = "Microsoft.Xna.Framework.Content.BasicEffectReader";

const SKINNED_MODEL_READER_NAME: &str = "XNAnimation.Pipeline.SkinnedModelReader";
const SKINNED_MODEL_BONE_READER_NAME: &str = "XNAnimation.Pipeline.SkinnedModelBoneReader";
const SKINNED_MODEL_ANIMATION_CLIP_READER_NAME: &str = "XNAnimation.Pipeline.AnimationClipReader";
const SKINNED_MODEL_BASIC_EFFECT_READER_NAME: &str =
    "XNAnimation.Pipeline.SkinnedModelBasicEffectReader";

const ADDITIVE_EFFECT_READER_NAME: &str = "PolygonHead.Pipeline.AdditiveEffectReader";
const RENDER_DEFERRED_EFFECT_READER_NAME: &str = "PolygonHead.Pipeline.RenderDeferredEffectReader";

#[derive(Serialize, Deserialize, Debug)]
pub enum Content {
    Null,
    Item(Item),
    Character(Character),
    String(String),
    ExternalReference(String),
    Texture2D(Texture2D),
    Model(Model),
    SkinnedModel(SkinnedModel),
    SkinnedModelBone(SkinnedModelBone),
    SkinnedModelAnimationClip(SkinnedModelAnimationClip),
    VertexDeclaration(VertexDeclaration),
    VertexBuffer(VertexBuffer),
    IndexBuffer(IndexBuffer),
    Effect(Effect),
    BasicEffect(BasicEffect),
    SkinnedModelBasicEffect(SkinnedModelBasicEffect),
    AdditiveEffect(AdditiveEffect),
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
            EXTERNAL_REFERENCE_READER_NAME => {
                let path = reader.read_7bit_length_string()?;
                return Ok(Content::ExternalReference(path));
            }
            ITEM_READER_NAME => {
                let item = Item::read(reader)?;
                return Ok(Content::Item(item));
            }
            CHARACTER_READER_NAME => {
                let character = Character::read(reader)?;
                return Ok(Content::Character(character));
            }
            TEXTURE_2D_READER_NAME => {
                let texture = Texture2D::read(reader)?;
                return Ok(Content::Texture2D(texture));
            }
            MODEL_READER_NAME => {
                let model = Model::read(reader, type_readers)?;
                return Ok(Content::Model(model));
            }
            SKINNED_MODEL_READER_NAME => {
                let model = SkinnedModel::read(reader, type_readers)?;
                return Ok(Content::SkinnedModel(model));
            }
            SKINNED_MODEL_BONE_READER_NAME => {
                let bone = SkinnedModelBone::read(reader)?;
                return Ok(Content::SkinnedModelBone(bone));
            }
            SKINNED_MODEL_ANIMATION_CLIP_READER_NAME => {
                let clip = SkinnedModelAnimationClip::read(reader)?;
                return Ok(Content::SkinnedModelAnimationClip(clip));
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
            EFFECT_READER_NAME => {
                let effect = Effect::read(reader)?;
                return Ok(Content::Effect(effect));
            }
            BASIC_EFFECT_READER_NAME => {
                let effect = BasicEffect::read(reader)?;
                return Ok(Content::BasicEffect(effect));
            }
            SKINNED_MODEL_BASIC_EFFECT_READER_NAME => {
                let effect = SkinnedModelBasicEffect::read(reader, type_readers)?;
                return Ok(Content::SkinnedModelBasicEffect(effect));
            }
            RENDER_DEFERRED_EFFECT_READER_NAME => {
                let effect = RenderDeferredEffect::read(reader)?;
                return Ok(Content::RenderDeferredEffect(effect));
            }
            ADDITIVE_EFFECT_READER_NAME => {
                let effect = AdditiveEffect::read(reader)?;
                return Ok(Content::AdditiveEffect(effect));
            }
            _ => {
                anyhow::bail!("unknown type reader: {}", type_reader.name);
            }
        }
    }
}
