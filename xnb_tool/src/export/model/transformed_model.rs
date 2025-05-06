use glam::{Vec3, Vec4};
use gltf::{
    Semantic,
    json::{
        Accessor, Index,
        accessor::{ComponentType, GenericComponentType, Type},
        buffer::View,
        validation::{Checked, USize64},
    },
};

use crate::content::model::{
    Bone, BoneHierarchy, ElementFormat, ElementUsage, Mesh, Model, VertexBuffer, VertexDeclaration,
};

/// model transformed into a format more suitable for gltf export
pub struct TransformedModel {
    pub bones: Vec<Bone>,
    pub bones_hierarchy: Vec<BoneHierarchy>,
    pub vertex_decls: Vec<TransformedVertexDeclaration>,
    pub meshes: Vec<Mesh>,
    pub root_bone_ref: u32,
    pub tag: u8,
}

impl From<&Model> for TransformedModel {
    fn from(model: &Model) -> Self {
        let (meshes, vertex_decls) = transform_meshes(&model.meshes, &model.vertex_decls);
        TransformedModel {
            vertex_decls,
            meshes,
            bones: model.bones.clone(),
            bones_hierarchy: model.bones_hierarchy.clone(),
            root_bone_ref: model.root_bone_ref,
            tag: model.tag,
        }
    }
}

pub struct TransformedVertexDeclaration {
    pub elements: Vec<TransformedVertexElement>,
}

impl TransformedVertexDeclaration {
    pub fn stride(&self) -> usize {
        let mut end = 0;
        for el in &self.elements {
            end = usize::max(end, el.offset as usize + el.size());
        }
        end
    }

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

impl From<&VertexDeclaration> for TransformedVertexDeclaration {
    fn from(decl: &VertexDeclaration) -> Self {
        let elements = decl
            .elements
            .iter()
            .filter_map(|el| {
                let semantic = match el.usage {
                    ElementUsage::Position => Some(Semantic::Positions),
                    ElementUsage::Normal => Some(Semantic::Normals),
                    ElementUsage::Tangent => Some(Semantic::Tangents),
                    ElementUsage::Color => Some(Semantic::Colors(el.usage_index as u32)),
                    ElementUsage::TextureCoordinate => {
                        Some(Semantic::TexCoords(el.usage_index as u32))
                    }
                    ElementUsage::BlendWeight => Some(Semantic::Weights(el.usage_index as u32)),
                    ElementUsage::BlendIndices => Some(Semantic::Joints(el.usage_index as u32)),
                    ElementUsage::Binormal => None,
                    v => todo!("semantic for element usage: {v:?}"),
                };

                let Some(semantic) = semantic else {
                    return None;
                };

                let element_type = if semantic == Semantic::Tangents {
                    Type::Vec4
                } else {
                    match el.format {
                        ElementFormat::Single => Type::Scalar,
                        ElementFormat::Vector2 => Type::Vec2,
                        ElementFormat::Vector3 => Type::Vec3,
                        ElementFormat::Vector4 => Type::Vec4,
                        ElementFormat::Color => Type::Vec4,
                        ElementFormat::Byte4 => Type::Vec4,
                        v => unimplemented!("type for element format: {v:?}"),
                    }
                };

                let component_type = match el.format {
                    ElementFormat::Single => ComponentType::F32,
                    ElementFormat::Vector2 => ComponentType::F32,
                    ElementFormat::Vector3 => ComponentType::F32,
                    ElementFormat::Vector4 => ComponentType::F32,
                    ElementFormat::Color => ComponentType::U8,
                    ElementFormat::Byte4 => ComponentType::U8,
                    v => unimplemented!("component type for element format: {v:?}"),
                };

                let offset = el.offset as usize;

                let normalized = match el.format {
                    ElementFormat::Color => true,
                    _ => false,
                };

                Some(TransformedVertexElement {
                    semantic,
                    element_type,
                    component_type,
                    offset,
                    normalized,
                })
            })
            .collect();
        TransformedVertexDeclaration { elements }
    }
}

pub struct TransformedVertexElement {
    pub semantic: Semantic,
    pub element_type: Type,
    pub component_type: ComponentType,
    pub offset: usize,
    pub normalized: bool,
}

impl TransformedVertexElement {
    pub fn size(&self) -> usize {
        self.element_type.multiplicity() * self.component_type.size()
    }

    pub fn accessor(
        &self,
        view: Index<View>,
        num_vertices: u64,
        pos_min: Vec3,
        pos_max: Vec3,
    ) -> Accessor {
        let (min, max) = match self.semantic {
            Semantic::Positions => (
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
            component_type: Checked::Valid(GenericComponentType(self.component_type)),
            type_: Checked::Valid(self.element_type),
            normalized: self.normalized,
            sparse: None,
            name: None,
            min,
            max,
            extensions: Default::default(),
            extras: Default::default(),
        }
    }
}

fn transform_meshes(
    meshes: &[Mesh],
    vertex_decls: &[VertexDeclaration],
) -> (Vec<Mesh>, Vec<TransformedVertexDeclaration>) {
    let transformed_decls: Vec<TransformedVertexDeclaration> =
        vertex_decls.iter().map(|decl| decl.into()).collect();

    let transformed_meshes = meshes
        .iter()
        .map(|mesh| transform_mesh(mesh, vertex_decls))
        .collect();

    (transformed_meshes, transformed_decls)
}

fn transform_mesh(mesh: &Mesh, vertex_decls: &[VertexDeclaration]) -> Mesh {
    let mut transformed_vertex_buffer = Vec::with_capacity(mesh.vertex_buffer.data.len());
    for part in &mesh.parts {
        let decl = &vertex_decls[part.vertex_decl_index as usize];
        let stride = decl.stride();
        let base = part.base_vertex as usize;
        let count = part.vertex_count as usize;
        let part_data = &mesh.vertex_buffer.data[base * stride..(base + count) * stride];
        if decl
            .elements
            .iter()
            .any(|el| el.usage == ElementUsage::Tangent)
        {
            transform_vertex_data_tangent(part_data, decl, &mut transformed_vertex_buffer);
        } else {
            transformed_vertex_buffer.extend_from_slice(part_data);
        }
    }
    Mesh {
        name: mesh.name.clone(),
        parent_bone_ref: mesh.parent_bone_ref,
        bounds: mesh.bounds.clone(),
        vertex_buffer: VertexBuffer {
            data: transformed_vertex_buffer,
        },
        index_buffer: mesh.index_buffer.clone(),
        parts: mesh.parts.clone(),
        tag: mesh.tag,
    }
}

fn transform_vertex_data_tangent(data: &[u8], decl: &VertexDeclaration, dest: &mut Vec<u8>) {
    let normal_offset = decl
        .elements
        .iter()
        .find_map(|el| {
            if el.usage == ElementUsage::Normal {
                Some(el.offset)
            } else {
                None
            }
        })
        .unwrap() as usize;
    let tangent_offset = decl
        .elements
        .iter()
        .find_map(|el| {
            if el.usage == ElementUsage::Tangent {
                Some(el.offset)
            } else {
                None
            }
        })
        .unwrap() as usize;
    let binormal_offset = decl
        .elements
        .iter()
        .find_map(|el| {
            if el.usage == ElementUsage::Binormal {
                Some(el.offset)
            } else {
                None
            }
        })
        .unwrap() as usize;

    for vertex in data.chunks(decl.stride()) {
        let vec3_size = std::mem::size_of::<Vec3>();
        let normal_bytes = &vertex[normal_offset..normal_offset + vec3_size];
        let tangent_bytes = &vertex[tangent_offset..tangent_offset + vec3_size];
        let binormal_bytes = &vertex[binormal_offset..binormal_offset + vec3_size];
        let normal = Vec3::from_slice(bytemuck::cast_slice(normal_bytes));
        let tangent = Vec3::from_slice(bytemuck::cast_slice(tangent_bytes));
        let binormal = Vec3::from_slice(bytemuck::cast_slice(binormal_bytes));
        let w = if normal.cross(tangent).dot(binormal) < 0.0 {
            -1.0
        } else {
            1.0
        };
        let tangent = Vec4::new(tangent.x, tangent.y, tangent.z, w);

        for el in &decl.elements {
            match el.usage {
                ElementUsage::Tangent => dest.extend_from_slice(bytemuck::cast_slice(&[tangent])),
                ElementUsage::Binormal => {}
                _ => {
                    let offset = el.offset as usize;
                    let bytes = &vertex[offset..offset + el.format.size()];
                    dest.extend_from_slice(bytes);
                }
            }
        }
    }
}
