use std::io::Read;

use anyhow::anyhow;
use byteorder::{LittleEndian, ReadBytesExt};
use glam::{Mat4, Vec3};
use serde::{Deserialize, Serialize};

use crate::ext::MyReadBytesExt;
use crate::xnb::TypeReader;

use super::Content;

#[derive(Serialize, Deserialize, Debug)]
pub struct Model {
    pub bones: Vec<Bone>,
    pub bones_hierarchy: Vec<BoneHierarchy>,
    pub vertex_decls: Vec<VertexDeclaration>,
    pub meshes: Vec<Mesh>,
    pub root_bone_ref: u32,
    pub tag: u8,
}

impl Model {
    pub fn read(reader: &mut impl Read, type_readers: &[TypeReader]) -> anyhow::Result<Self> {
        let num_bones = reader.read_u32::<LittleEndian>()?;
        let mut bones = Vec::with_capacity(num_bones as usize);
        for _ in 0..num_bones {
            let bone = Bone::read(reader, type_readers)?;
            bones.push(bone);
        }
        let mut bones_hierarchy = Vec::with_capacity(num_bones as usize);
        for _ in 0..num_bones {
            let parent_ref = read_bone_ref(reader, num_bones)?;
            let num_children = reader.read_u32::<LittleEndian>()? as usize;
            let mut children_refs = Vec::with_capacity(num_children);
            for _ in 0..num_children {
                let child_ref = read_bone_ref(reader, num_bones)?;
                children_refs.push(child_ref);
            }
            bones_hierarchy.push(BoneHierarchy {
                parent_ref,
                children_refs,
            });
        }

        let num_vertex_decls = reader.read_u32::<LittleEndian>()?;
        let mut vertex_decls = Vec::with_capacity(num_vertex_decls as usize);
        for _ in 0..num_vertex_decls {
            let content = Content::read(reader, type_readers)?;
            let Content::VertexDeclaration(decl) = content else {
                anyhow::bail!("expected vertex declaration");
            };
            vertex_decls.push(decl);
        }

        let num_meshes = reader.read_u32::<LittleEndian>()?;
        let mut meshes = Vec::with_capacity(num_meshes as usize);
        for _ in 0..num_meshes {
            let mesh = Mesh::read(reader, type_readers)?;
            meshes.push(mesh);
        }

        let root_bone_ref = read_bone_ref(reader, num_bones)?;
        let tag = reader.read_u8()?;

        Ok(Model {
            bones,
            bones_hierarchy,
            vertex_decls,
            meshes,
            root_bone_ref,
            tag,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bone {
    pub name: String,
    pub transform: Mat4,
}

impl Bone {
    pub fn read(reader: &mut impl Read, type_readers: &[TypeReader]) -> anyhow::Result<Self> {
        let name = Content::read(reader, type_readers)?;
        let Content::String(name) = name else {
            anyhow::bail!("expected bone name to be a string");
        };
        let transform = reader.read_mat4()?;
        Ok(Bone { name, transform })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoneHierarchy {
    pub parent_ref: u32,
    pub children_refs: Vec<u32>,
}

fn read_bone_ref(reader: &mut impl Read, num_bones: u32) -> std::io::Result<u32> {
    let bone_ref = if num_bones <= 255 {
        reader.read_u8()? as u32
    } else {
        reader.read_u32::<LittleEndian>()?
    };
    Ok(bone_ref)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Mesh {
    name: String,
    parent_bone_ref: u32,
    bounds: BoundingSphere,
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,
    parts: Vec<MeshPart>,
    tag: u8,
}

impl Mesh {
    pub fn read(reader: &mut impl Read, type_readers: &[TypeReader]) -> anyhow::Result<Self> {
        let name = Content::read(reader, type_readers)?;
        let Content::String(name) = name else {
            anyhow::bail!("expected bone name to be a string");
        };

        let parent_bone_ref = read_bone_ref(reader, 0)?;
        let bounds = BoundingSphere::read(reader)?;

        let vertex_buffer = Content::read(reader, type_readers)?;
        let Content::VertexBuffer(vertex_buffer) = vertex_buffer else {
            anyhow::bail!("expected vertex buffer");
        };

        let index_buffer = Content::read(reader, type_readers)?;
        let Content::IndexBuffer(index_buffer) = index_buffer else {
            anyhow::bail!("expected index buffer");
        };

        let tag = reader.read_u8()?;

        let num_parts = reader.read_u32::<LittleEndian>()? as usize;
        let mut parts = Vec::with_capacity(num_parts);
        for _ in 0..num_parts {
            let part = MeshPart::read(reader)?;
            parts.push(part);
        }

        Ok(Mesh {
            name,
            parent_bone_ref,
            bounds,
            vertex_buffer,
            index_buffer,
            parts,
            tag,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MeshPart {
    pub stream_offset: u32,
    pub base_vertex: u32,
    pub vertex_count: u32,
    pub start_index: u32,
    pub primitive_count: u32,
    pub vertex_decl_index: u32,
    pub tag: u8,
    pub shared_content_id: i32,
}

impl MeshPart {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let stream_offset = reader.read_u32::<LittleEndian>()?;
        let base_vertex = reader.read_u32::<LittleEndian>()?;
        let vertex_count = reader.read_u32::<LittleEndian>()?;
        let start_index = reader.read_u32::<LittleEndian>()?;
        let primitive_count = reader.read_u32::<LittleEndian>()?;
        let vertex_decl_index = reader.read_u32::<LittleEndian>()?;
        let tag = reader.read_u8()?;
        let shared_content_id = reader.read_7bit_encoded_i32()?;
        Ok(MeshPart {
            stream_offset,
            base_vertex,
            vertex_count,
            start_index,
            primitive_count,
            vertex_decl_index,
            tag,
            shared_content_id,
        })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BoundingSphere {
    pub center: Vec3,
    pub radius: f32,
}

impl BoundingSphere {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let center = reader.read_vec3()?;
        let radius = reader.read_f32::<LittleEndian>()?;
        Ok(BoundingSphere { center, radius })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VertexDeclaration {
    pub elements: Vec<VertexElement>,
}

impl VertexDeclaration {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let num_elements = reader.read_u32::<LittleEndian>()? as usize;
        let mut elements = Vec::with_capacity(num_elements);
        for _ in 0..num_elements {
            let element = VertexElement::read(reader)?;
            elements.push(element);
        }
        Ok(VertexDeclaration { elements })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VertexElement {
    pub stream: u16,
    pub offset: u16,
    pub format: ElementFormat,
    pub method: ElementMethod,
    pub usage: ElementUsage,
    pub usage_index: u8,
}

impl VertexElement {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let stream = reader.read_u16::<LittleEndian>()?;
        let offset = reader.read_u16::<LittleEndian>()?;
        let format = ElementFormat::read(reader)?;
        let method = ElementMethod::read(reader)?;
        let usage = ElementUsage::read(reader)?;
        let usage_index = reader.read_u8()?;
        Ok(VertexElement {
            stream,
            offset,
            format,
            method,
            usage,
            usage_index,
        })
    }
}

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug)]
pub enum ElementFormat {
    Single,
    Vector2,
    Vector3,
    Vector4,
    Color,
    Byte4,
    Short2,
    Short4,
    RGBA32,
    NormalizedShort2,
    NormalizedShort4,
    RGB32,
    RGBA64,
    UInt40,
    Normalized40,
    HalfVector2,
    HalfVector4,
}

impl ElementFormat {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let format = ElementFormat::from_repr(value as u8)
            .ok_or_else(|| anyhow!("unknown element format: {value}"))?;
        Ok(format)
    }
}

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug)]
pub enum ElementMethod {
    Default,
    UV = 4,
    LookUp = 5,
    LookUpPresampled,
}

impl ElementMethod {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let method = ElementMethod::from_repr(value as u8)
            .ok_or_else(|| anyhow!("unknown element method: {value}"))?;
        Ok(method)
    }
}

#[repr(u8)]
#[derive(strum::FromRepr, Serialize, Deserialize, Debug)]
pub enum ElementUsage {
    Position,
    BlendWeight,
    BlendIndices,
    Normal,
    PointSize,
    TextureCoordinate,
    Tangent,
    Binormal,
    TessellateFactor,
    Color = 10,
    Fog,
    Depth,
    Sample,
}

impl ElementUsage {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let value = reader.read_u8()?;
        let usage = ElementUsage::from_repr(value as u8)
            .ok_or_else(|| anyhow!("unknown element usage: {value}"))?;
        Ok(usage)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VertexBuffer {
    pub data: Vec<u8>,
}

impl VertexBuffer {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let size = reader.read_u32::<LittleEndian>()? as usize;
        let mut data = vec![0; size];
        reader.read_exact(&mut data)?;
        Ok(VertexBuffer { data })
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IndexBuffer {
    pub is_16_bit: bool,
    pub data: Vec<u8>,
}

impl IndexBuffer {
    pub fn read(reader: &mut impl Read) -> anyhow::Result<Self> {
        let is_16_bit = reader.read_bool()?;
        let size = reader.read_u32::<LittleEndian>()? as usize;
        let mut data = vec![0; size];
        reader.read_exact(&mut data)?;
        Ok(IndexBuffer { is_16_bit, data })
    }
}
