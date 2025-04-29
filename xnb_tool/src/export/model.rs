use std::{borrow::Cow, collections::BTreeMap};

use anyhow::Context;
use glam::Vec3;
use gltf::{
    Glb, Semantic,
    binary::Header,
    buffer::Target,
    json::{
        self, Accessor, Buffer, Image, Index, Material, Node, Root, Scene, Texture,
        accessor::{ComponentType, GenericComponentType, Type},
        buffer::{Stride, View},
        material::{PbrBaseColorFactor, PbrMetallicRoughness, StrengthFactor},
        mesh::Primitive,
        texture,
        validation::{Checked, USize64},
    },
};

use crate::content::{
    Content,
    model::{
        ElementFormat, ElementUsage, IndexBuffer, Mesh, MeshPart, Model, VertexDeclaration,
        VertexElement,
    },
};

impl Model {
    pub fn to_glb(&self, shared_content: &[Content]) -> anyhow::Result<Vec<u8>> {
        let mut root = Root::default();

        let buffer = build_buffer(&mut root, self);

        let materials = build_materials(&mut root, shared_content);

        let mesh_nodes: Vec<Index<Node>> = self
            .meshes
            .iter()
            .enumerate()
            .map(|(mesh_idx, mesh)| {
                build_mesh(&mut root, &buffer, self, mesh, mesh_idx, &materials)
            })
            .collect();

        let scene = root.push(Scene {
            nodes: mesh_nodes,
            name: None,
            extensions: Default::default(),
            extras: Default::default(),
        });
        root.scene = Some(scene);

        let json_string = serde_json::to_string(&root)?;
        let json_padded_len = pad_to_multiple_of_four(json_string.len());

        let mut bin = buffer.data.clone();
        while bin.len() % 4 != 0 {
            bin.push(0);
        }

        let glb = Glb {
            header: Header {
                magic: *b"glTF",
                version: 2,
                length: (json_padded_len + bin.len())
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

        let normalized = match self.format {
            ElementFormat::Color => true,
            _ => false,
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
            normalized,
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
            ElementFormat::Color => ComponentType::U8,
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
            ElementFormat::Color => Type::Vec4,
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

struct FullBuffer {
    index: Index<Buffer>,
    data: Vec<u8>,
    vertex_offsets: Vec<usize>,
    index_offsets: Vec<usize>,
}

fn build_buffer(root: &mut Root, model: &Model) -> FullBuffer {
    let mut vertex_offsets = Vec::new();
    let mut index_offsets = Vec::new();
    let mut buffer_data = Vec::new();

    for mesh in &model.meshes {
        vertex_offsets.push(buffer_data.len());
        buffer_data.extend_from_slice(&mesh.vertex_buffer.data);

        index_offsets.push(buffer_data.len());
        let reversed_indices = reverse_winding(&mesh.index_buffer);
        buffer_data.extend_from_slice(&reversed_indices.data);
    }

    let buffer_index = root.push(Buffer {
        byte_length: USize64(buffer_data.len() as u64),
        name: None,
        uri: None,
        extensions: Default::default(),
        extras: Default::default(),
    });

    FullBuffer {
        index: buffer_index,
        data: buffer_data,
        vertex_offsets,
        index_offsets,
    }
}

fn build_materials(root: &mut Root, shared_content: &[Content]) -> Vec<Option<Index<Material>>> {
    let materials: Vec<Option<Index<Material>>> = shared_content
        .iter()
        .map(|content| match content {
            Content::RenderDeferredEffect(effect) => {
                let color = effect.material_0.diffuse_color;
                let color = PbrBaseColorFactor([color.r, color.g, color.b, 1.0]);
                let base_texture_path =
                    format!("{}.texture2d.png", &effect.material_0.diffuse_texture);

                let base_image = root.push(Image {
                    uri: Some(base_texture_path),
                    buffer_view: None,
                    mime_type: None,
                    name: None,
                    extensions: Default::default(),
                    extras: Default::default(),
                });

                let base_texture = root.push(Texture {
                    source: base_image,
                    sampler: None,
                    name: None,
                    extensions: Default::default(),
                    extras: Default::default(),
                });

                let material = root.push(Material {
                    pbr_metallic_roughness: PbrMetallicRoughness {
                        base_color_factor: color,
                        base_color_texture: Some(texture::Info {
                            index: base_texture,
                            tex_coord: 0,
                            extensions: Default::default(),
                            extras: Default::default(),
                        }),
                        metallic_factor: StrengthFactor(0.0),
                        roughness_factor: StrengthFactor(1.0),
                        metallic_roughness_texture: None,
                        extensions: Default::default(),
                        extras: Default::default(),
                    },
                    ..Default::default()
                });

                Some(material)
            }
            Content::AdditiveEffect(effect) => {
                let color = effect.color_tint;
                let color = PbrBaseColorFactor([color.r, color.g, color.b, 1.0]);

                let texture = if effect.texture_enabled {
                    let texture_path = format!("{}.texture2d.png", &effect.texture);

                    let image = root.push(Image {
                        uri: Some(texture_path),
                        buffer_view: None,
                        mime_type: None,
                        name: None,
                        extensions: Default::default(),
                        extras: Default::default(),
                    });

                    let texture = root.push(Texture {
                        source: image,
                        sampler: None,
                        name: None,
                        extensions: Default::default(),
                        extras: Default::default(),
                    });

                    Some(texture::Info {
                        index: texture,
                        tex_coord: 0,
                        extensions: Default::default(),
                        extras: Default::default(),
                    })
                } else {
                    None
                };

                let material = root.push(Material {
                    pbr_metallic_roughness: PbrMetallicRoughness {
                        base_color_factor: color,
                        base_color_texture: texture,
                        metallic_factor: StrengthFactor(0.0),
                        roughness_factor: StrengthFactor(1.0),
                        metallic_roughness_texture: None,
                        extensions: Default::default(),
                        extras: Default::default(),
                    },
                    ..Default::default()
                });

                Some(material)
            }
            _ => None,
        })
        .collect();

    materials
}

fn build_mesh(
    root: &mut Root,
    buffer: &FullBuffer,
    model: &Model,
    mesh: &Mesh,
    mesh_idx: usize,
    materials: &[Option<Index<Material>>],
) -> Index<Node> {
    let part_nodes: Vec<Index<Node>> = mesh
        .parts
        .iter()
        .map(|part| build_mesh_part(root, buffer, model, mesh, mesh_idx, part, materials))
        .collect();

    let node = root.push(Node {
        children: Some(part_nodes),
        name: Some(mesh.name.clone()),
        ..Default::default()
    });

    node
}

fn build_mesh_part(
    root: &mut Root,
    buffer: &FullBuffer,
    model: &Model,
    mesh: &Mesh,
    mesh_idx: usize,
    part: &MeshPart,
    materials: &[Option<Index<Material>>],
) -> Index<Node> {
    // vertices
    let vertex_decl = &model.vertex_decls[part.vertex_decl_index as usize];
    let vertex_stride = vertex_decl.stride();
    let vertex_buffer_length = part.vertex_count as usize * vertex_stride;
    let vertex_buffer_local_offset = part.base_vertex as usize * vertex_stride;
    let vertex_buffer_full_offset = vertex_buffer_local_offset + buffer.vertex_offsets[mesh_idx];

    let vertex_view = root.push(View {
        buffer: buffer.index,
        byte_length: USize64(vertex_buffer_length as u64),
        byte_offset: Some(USize64(vertex_buffer_full_offset as u64)),
        byte_stride: Some(Stride(vertex_stride)),
        target: Some(Checked::Valid(Target::ArrayBuffer)),
        name: None,
        extensions: Default::default(),
        extras: Default::default(),
    });

    let (pos_min, pos_max) = calculate_bounds(
        &mesh.vertex_buffer.data
            [vertex_buffer_local_offset..vertex_buffer_local_offset + vertex_buffer_length],
        &vertex_decl,
    );
    let vertex_accessors =
        vertex_decl.accessors(vertex_view, part.vertex_count as u64, pos_min, pos_max);
    let vertex_accessors: Vec<Index<Accessor>> =
        vertex_accessors.into_iter().map(|a| root.push(a)).collect();

    // indices
    let index_type = if mesh.index_buffer.is_16_bit {
        ComponentType::U16
    } else {
        ComponentType::U32
    };
    let index_count = part.primitive_count * 3; // assuming primitives are always triangles
    let index_buffer_length = index_count as usize * index_type.size();
    let index_buffer_local_offset = part.start_index as usize * index_type.size();
    let index_buffer_full_offset = index_buffer_local_offset + buffer.index_offsets[mesh_idx];

    let index_view = root.push(View {
        buffer: buffer.index,
        byte_length: USize64(index_buffer_length as u64),
        byte_offset: Some(USize64(index_buffer_full_offset as u64)),
        byte_stride: None,
        target: Some(Checked::Valid(Target::ElementArrayBuffer)),
        name: None,
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

    // everything else
    let material = if part.shared_content_material_idx > 0 {
        Some(materials[part.shared_content_material_idx as usize - 1].unwrap())
    } else {
        None
    };

    let primitive = Primitive {
        attributes: {
            let mut map = BTreeMap::new();
            for (i, el) in vertex_decl.elements.iter().enumerate() {
                map.insert(Checked::Valid(el.usage.into()), vertex_accessors[i]);
            }
            map
        },
        indices: Some(index_accessor),
        material,
        targets: None,
        mode: Checked::Valid(gltf::mesh::Mode::Triangles),
        extensions: Default::default(),
        extras: Default::default(),
    };

    let mesh = root.push(json::Mesh {
        primitives: vec![primitive],
        weights: None,
        name: None,
        extensions: Default::default(),
        extras: Default::default(),
    });

    let node = root.push(Node {
        mesh: Some(mesh),
        ..Default::default()
    });

    node
}

fn pad_to_multiple_of_four(n: usize) -> usize {
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
