meta:
  id: xnb_decompressed
  endian: le
seq:
  - id: type_readers
    type: type_readers
  - id: shared_resource_count
    type: int_7bit
  - id: primary
    type: entry
  - id: shared_resources
    type: entry
    repeat: expr
    repeat-expr: shared_resource_count.value
types:
  type_readers:
    seq:
      - id: length
        type: u1
      - id: readers
        type: type_reader
        repeat: expr
        repeat-expr: length
  type_reader:
    seq:
      - id: length
        type: u1
      - id: name
        type: str
        size: length
        encoding: ASCII
      - id: version
        type: u4
  entry:
    seq:
      - id: reader_idx
        type: u1
      - id: inner
        type:
          switch-on: reader
          cases:
            '"Microsoft.Xna.Framework.Content.StringReader"': entry_string
            '"Microsoft.Xna.Framework.Content.ModelReader"': entry_model
            '"Microsoft.Xna.Framework.Content.VertexDeclarationReader"': entry_vertex_decl
            '"Microsoft.Xna.Framework.Content.VertexBufferReader"': entry_vertex_buffer
            '"Microsoft.Xna.Framework.Content.IndexBufferReader"': entry_index_buffer
            '"PolygonHead.Pipeline.RenderDeferredEffectReader, PolygonHead, Version=1.0.0.0, Culture=neutral"': entry_render_deferred_effect
    instances:
      reader:
        value: _root.type_readers.readers[reader_idx - 1].name
  entry_string:
    seq:
      - id: length
        type: u1
      - id: value
        type: str
        size: length
        encoding: ASCII
  entry_model:
    seq:
      - id: bone_count
        type: u4
      - id: bones
        type: bone
        repeat: expr
        repeat-expr: bone_count
      - id: bones_hierarchy
        type: bone_hierarchy
        repeat: expr
        repeat-expr: bone_count
      - id: vertex_decl_count
        type: u4
      - id: vertex_decls
        type: entry
      - id: mesh_count
        type: u4
      - id: meshes
        type: mesh
      - id: root_bone_ref_u1
        type: u1
        if: bone_count <= 255
      - id: root_bone_ref_u4
        type: u4
        if: bone_count > 255
      - id: tag
        type: u1
  entry_vertex_decl:
    seq:
      - id: element_count
        type: u4
      - id: elements
        type: vertex_element
        repeat: expr
        repeat-expr: element_count
  entry_vertex_buffer:
    seq:
      - id: length
        type: u4
      - id: data
        type: u1
        repeat: expr
        repeat-expr: length
  entry_index_buffer:
    seq:
      - id: is_16_bit
        type: u1
      - id: length
        type: u4
      - id: data
        type: u1
        repeat: expr
        repeat-expr: length
  entry_render_deferred_effect:
    seq:
      - id: alpha
        type: f4
      - id: sharpness
        type: f4
      - id: vertex_color_enabled
        type: u1
      - id: use_material_texture_for_reflectiveness
        type: u1
      - id: reflection_map
        type: external_reference
      - id: texture_0
        type: render_deferred_effect_texture
      - id: has_texture_1
        type: u1
      - id: texture_1
        type: render_deferred_effect_texture
        if: has_texture_1 != 0
  render_deferred_effect_texture:
    seq:
      - id: diffuse_texture_alpha_disabled
        type: u1
      - id: alpha_mask_enabled
        type: u1
      - id: diffuse_color
        type: vector3f
      - id: spec_amount
        type: f4
      - id: spec_power
        type: f4
      - id: emissive_amount
        type: f4
      - id: normal_power
        type: f4
      - id: reflectiveness
        type: f4
      - id: diffuse_texture
        type: external_reference
      - id: material_texture
        type: external_reference
      - id: normal_texture
        type: external_reference
  bone:
    seq:
      - id: name
        type: entry
      - id: transform
        type: matrix4f
  bone_hierarchy:
    seq:
      - id: bone_ref_u1
        type: u1
        if: _parent.bone_count <= 255
      - id: bone_ref_u4
        type: u4
        if: _parent.bone_count > 255
      - id: children_count
        type: u4
      - id: children_u1
        type: u1
        repeat: expr
        repeat-expr: children_count
        if: _parent.bone_count <= 255
      - id: children_u4
        type: u4
        repeat: expr
        repeat-expr: children_count
        if: _parent.bone_count > 255
    instances:
      bone_ref:
        value: _parent.bone_count <= 255 ? bone_ref_u1 : bone_ref_u4
  mesh:
    seq:
      - id: name
        type: entry
      - id: parent_bone_ref_u1
        type: u1
        if: _parent.bone_count <= 255
      - id: parent_bone_ref_u4
        type: u4
        if: _parent.bone_count > 255
      - id: bounds
        type: bounding_sphere
      - id: vertex_buffer
        type: entry
      - id: index_buffer
        type: entry
      - id: tag
        type: u1
      - id: mesh_part_count
        type: u4
      - id: mesh_parts
        type: mesh_part
        repeat: expr
        repeat-expr: mesh_part_count
  mesh_part:
    seq:
      - id: stream_offset
        type: u4
      - id: base_vertex
        type: u4
      - id: vertex_count
        type: u4
      - id: start_index
        type: u4
      - id: primitive_count
        type: u4
      - id: vertex_decl_index
        type: u4
      - id: tag
        type: u1
      - id: shared_content_id
        type: int_7bit
  vertex_element:
    seq:
      - id: stream
        type: u2
      - id: offset
        type: u2
      - id: format
        type: u1
        enum: format
      - id: method
        type: u1
        enum: method
      - id: usage
        type: u1
        enum: usage
      - id: unk1
        type: u1
    enums:
      format:
        0: single
        1: vector2
        2: vector3
        3: vector4
        4: color
        5: byte4
      method:
        0: default
        4: uv
        5: lookup
        6: lookup_presampled
      usage:
        0: position
        1: blend_weight
        2: blend_indices
        3: normal
        4: point_size
        5: texture_coordinate
        6: tangent
        7: binormal
        8: tesselate_factor
        10: color
        11: fog
        12: depth
        13: sample
  external_reference:
    seq:
      - id: length
        type: int_7bit
      - id: value
        type: str
        size: length.value
        encoding: ascii
  bounding_sphere:
    seq:
      - id: center
        type: vector3f
      - id: radius
        type: f4
  vector3f:
    seq:
      - id: x
        type: f4
      - id: y
        type: f4
      - id: z
        type: f4
  matrix4f:
    seq:
      - id: values
        type: f4
        repeat: expr
        repeat-expr: 16
  int_7bit:
    seq:
      - id: value
        type: u1
