use std::{borrow::Cow, collections::BTreeMap};

use anyhow::Context;
use glam::Vec3;
use gltf::{
    Glb, Semantic,
    binary::Header,
    buffer::Target,
    json::{
        self, Accessor, Buffer, Index, Material, Node, Root, Scene,
        accessor::{ComponentType, GenericComponentType, Type},
        buffer::{Stride, View},
        mesh::Primitive,
        validation::{Checked, USize64},
    },
};
use serde_json::value::RawValue;

use crate::content::{
    Content,
    model::{
        ElementFormat, ElementUsage, IndexBuffer, Mesh, MeshPart, Model, VertexDeclaration,
        VertexElement,
    },
    skinned_model::SkinnedModel,
};

impl SkinnedModel {
    pub fn to_glb(&self, shared_content: &[Content]) -> anyhow::Result<Vec<u8>> {
        let glb = self.model.to_glb(shared_content)?;
        Ok(glb)
    }
}
