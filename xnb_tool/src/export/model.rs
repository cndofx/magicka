use std::{borrow::Cow, collections::BTreeMap};

use anyhow::Context;
use glam::Vec3;
use gltf::{
    Glb, Semantic,
    binary::Header,
    buffer::Target,
    json::{
        Accessor, Buffer, Index, Mesh, Node, Root, Scene,
        accessor::{ComponentType, GenericComponentType, Type},
        buffer::{Stride, View},
        mesh::Primitive,
        validation::{Checked, USize64},
    },
};

use crate::content::model::{
    ElementFormat, ElementUsage, IndexBuffer, Model, VertexDeclaration, VertexElement,
};

impl Model {
    pub fn to_glb(&self) -> anyhow::Result<Vec<u8>> {
        let (root, length) = build_root(self);

        let json_string = serde_json::to_string(&root)?;
        let json_offset = align_to_multiple_of_four(json_string.len());

        let reversed_indices = reverse_winding(&self.meshes[0].index_buffer);

        let mut bin = Vec::new();
        bin.extend_from_slice(&self.meshes[0].vertex_buffer.data);
        bin.extend_from_slice(&reversed_indices.data);
        while bin.len() % 4 != 0 {
            bin.push(0);
        }

        let glb = Glb {
            header: Header {
                magic: *b"glTF",
                version: 2,
                length: (json_offset + length)
                    .try_into()
                    .context("file size exceeds binary glTF size limit")?,
            },
            json: Cow::Owned(json_string.into_bytes()),
            bin: Some(Cow::Owned(bin)),
        };

        let bytes = glb.to_vec().context("failed to serialize glb")?;
        Ok(bytes)
    }
}

impl VertexDeclaration {
    pub fn accessors(
        &self,
        view: Index<View>,
        num_vertices: u64,
        pos_min: Vec3,
        pos_max: Vec3,
    ) -> Vec<Accessor> {
        self.elements
            .iter()
            .map(|el| el.accessor(view, num_vertices, pos_min, pos_max))
            .collect()
    }
}

impl VertexElement {
    pub fn accessor(
        &self,
        view: Index<View>,
        num_vertices: u64,
        pos_min: Vec3,
        pos_max: Vec3,
    ) -> Accessor {
        let (min, max) = match self.usage {
            ElementUsage::Position => (
                Some(serde_json::Value::from(vec![
                    pos_min.x, pos_min.y, pos_min.z,
                ])),
                Some(serde_json::Value::from(vec![
                    pos_max.x, pos_max.y, pos_max.z,
                ])),
            ),
            _ => (None, None),
        };

        Accessor {
            buffer_view: Some(view),
            byte_offset: Some(USize64(self.offset as u64)),
            count: USize64(num_vertices),
            component_type: Checked::Valid(GenericComponentType(self.format.into())),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Checked::Valid(self.format.into()),
            min,
            max,
            name: None,
            normalized: false,
            sparse: None,
        }
    }
}

impl From<ElementFormat> for ComponentType {
    fn from(value: ElementFormat) -> Self {
        match value {
            ElementFormat::Single => ComponentType::F32,
            ElementFormat::Vector2 => ComponentType::F32,
            ElementFormat::Vector3 => ComponentType::F32,
            ElementFormat::Vector4 => ComponentType::F32,
            v => unimplemented!("component type for element format: {v:?}"),
        }
    }
}

impl From<ElementFormat> for Type {
    fn from(value: ElementFormat) -> Self {
        match value {
            ElementFormat::Single => Type::Scalar,
            ElementFormat::Vector2 => Type::Vec2,
            ElementFormat::Vector3 => Type::Vec3,
            ElementFormat::Vector4 => Type::Vec4,
            v => unimplemented!("type for element format: {v:?}"),
        }
    }
}

impl From<ElementUsage> for Semantic {
    fn from(value: ElementUsage) -> Self {
        match value {
            ElementUsage::Position => Semantic::Positions,
            ElementUsage::Normal => Semantic::Normals,
            ElementUsage::Color => Semantic::Colors(0),
            ElementUsage::TextureCoordinate => Semantic::TexCoords(0),
            v => unimplemented!("semantic for element usage: {v:?}"),
        }
    }
}

fn build_root(model: &Model) -> (Root, usize) {
    let mut root = Root::default();

    let mesh = &model.meshes[0];
    let part = &mesh.parts[0];

    let vertex_decl_index = part.vertex_decl_index;
    let vertex_decl = &model.vertex_decls[vertex_decl_index as usize];
    let stride = vertex_decl.stride();

    let vertex_buffer_length = part.vertex_count as usize * stride;
    let vertex_buffer_offset = part.base_vertex as usize * stride;
    dbg!(stride, vertex_buffer_length);

    let full_buffer_length = mesh.vertex_buffer.data.len() + mesh.index_buffer.data.len();
    let buffer = root.push(Buffer {
        byte_length: USize64(vertex_buffer_length as u64),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        uri: None,
    });

    let vertex_view = root.push(View {
        buffer,
        byte_length: USize64(vertex_buffer_length as u64),
        byte_offset: Some(USize64(vertex_buffer_offset as u64)),
        byte_stride: Some(Stride(stride)),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Checked::Valid(Target::ArrayBuffer)),
    });

    let (pos_min, pos_max) = calculate_bounds(
        &mesh.vertex_buffer.data[vertex_buffer_offset..vertex_buffer_offset + vertex_buffer_length],
        &vertex_decl,
    );
    let vertex_accessors =
        vertex_decl.accessors(vertex_view, part.vertex_count as u64, pos_min, pos_max);
    dbg!(&vertex_accessors);

    let vertex_accessors: Vec<Index<Accessor>> =
        vertex_accessors.into_iter().map(|a| root.push(a)).collect();

    let index_type = if mesh.index_buffer.is_16_bit {
        ComponentType::U16
    } else {
        ComponentType::U32
    };
    let index_count = part.primitive_count * 3; // assuming primitives are always triangles
    let index_buffer_length = index_count as usize * index_type.size();
    let index_buffer_offset =
        mesh.vertex_buffer.data.len() + part.start_index as usize * index_type.size();
    let index_view = root.push(View {
        buffer,
        byte_length: USize64(index_buffer_length as u64),
        byte_offset: Some(USize64(index_buffer_offset as u64)),
        byte_stride: None,
        name: None,
        target: Some(Checked::Valid(Target::ElementArrayBuffer)),
        extensions: Default::default(),
        extras: Default::default(),
    });
    let index_accessor = root.push(Accessor {
        buffer_view: Some(index_view),
        byte_offset: Some(USize64(0)),
        count: USize64(index_count as u64),
        component_type: Checked::Valid(GenericComponentType(index_type)),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Checked::Valid(Type::Scalar),
        min: None,
        max: None,
        name: None,
        normalized: false,
        sparse: None,
    });

    let primitive = Primitive {
        attributes: {
            let mut map = BTreeMap::new();
            for (i, el) in vertex_decl.elements.iter().enumerate() {
                map.insert(Checked::Valid(el.usage.into()), vertex_accessors[i]);
            }
            map
        },
        extensions: Default::default(),
        extras: Default::default(),
        indices: Some(index_accessor),
        material: None,
        mode: Checked::Valid(gltf::mesh::Mode::Triangles),
        targets: None,
    };

    let mesh = root.push(Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        primitives: vec![primitive],
        weights: None,
    });

    let node = root.push(Node {
        mesh: Some(mesh),
        ..Default::default()
    });

    root.push(Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        nodes: vec![node],
    });

    // let full_length = vertex_buffer_length + index_buffer_length;

    (root, full_buffer_length)
}

fn align_to_multiple_of_four(n: usize) -> usize {
    (n + 3) & !3
}

fn calculate_bounds(vertices: &[u8], decl: &VertexDeclaration) -> (Vec3, Vec3) {
    let offset = decl
        .elements
        .iter()
        .find(|el| matches!(el.usage, ElementUsage::Position))
        .unwrap()
        .offset as usize;

    let mut min = Vec3::new(f32::MAX, f32::MAX, f32::MAX);
    let mut max = Vec3::new(f32::MIN, f32::MIN, f32::MIN);
    for i in (0..vertices.len()).step_by(decl.stride()) {
        let i = i + offset;
        let x = f32::from_le_bytes(vertices[i..i + 4].try_into().unwrap());
        let y = f32::from_le_bytes(vertices[i + 4..i + 8].try_into().unwrap());
        let z = f32::from_le_bytes(vertices[i + 8..i + 12].try_into().unwrap());
        min.x = min.x.min(x);
        min.y = min.y.min(y);
        min.z = min.z.min(z);
        max.x = max.x.max(x);
        max.y = max.y.max(y);
        max.z = max.z.max(z);
    }
    (min, max)
}

fn reverse_winding(indices: &IndexBuffer) -> IndexBuffer {
    let mut data = Vec::with_capacity(indices.data.len());

    if indices.is_16_bit {
        assert!(indices.data.len() % 2 == 0);
        let indices_u16: Vec<u16> = indices
            .data
            .chunks_exact(2)
            .map(|i| u16::from_le_bytes([i[0], i[1]]))
            .collect();

        assert!(indices_u16.len() % 3 == 0);
        for triangle in indices_u16.chunks_exact(3) {
            data.extend_from_slice(&triangle[0].to_le_bytes());
            data.extend_from_slice(&triangle[2].to_le_bytes());
            data.extend_from_slice(&triangle[1].to_le_bytes());
        }
    } else {
        assert!(indices.data.len() % 4 == 0);
        let indices_u32: Vec<u32> = indices
            .data
            .chunks_exact(4)
            .map(|i| u32::from_le_bytes([i[0], i[1], i[2], i[3]]))
            .collect();

        assert!(indices_u32.len() % 3 == 0);
        for triangle in indices_u32.chunks_exact(3) {
            data.extend_from_slice(&triangle[0].to_le_bytes());
            data.extend_from_slice(&triangle[2].to_le_bytes());
            data.extend_from_slice(&triangle[1].to_le_bytes());
        }
    }

    IndexBuffer {
        is_16_bit: indices.is_16_bit,
        data,
    }
}
