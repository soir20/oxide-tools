#pragma once

import type.color;
#include "vec.hexpat"

struct SubMeshBakedLighting {
    s32 vertex_count;
    // Use u32 instead of type::RGBA8 to avoid ImHex pattern limits/performance issues
    u32 vertex_baked_lighting[vertex_count];
};

struct RuntimeObject {
    // Runtime objects are duplicated for every tile the object appears in.
    // The duplicate entries all share the same GUID
    s32 guid;
    char adr_name[];
    char unknown2[];
    Vec4 pos;
    Vec4 rot;
    float scale;
    if (gcnk_version >= 6) {
        char texture_alias[];
        char tint_alias[];
        if (tint_alias[0] != '\0') {
            Vec4 tint_rgba;
        }
    }
    padding[4];
    if (gcnk_version >= 5) {
        u32 terrain_object_id;
    } else {
        char terrain_object_name[];
    }
    // Might be unused; only applied to large models but can't tell a difference in-game
    float min_render_radius;
    if (gcnk_version >= 3) {
        s32 sub_mesh_count;
        // Might be unused; can't tell a difference in-game when this is removed
        SubMeshBakedLighting baked_lighting[sub_mesh_count];
    }
};

struct RawLight {
    char name[];
    char color_name[];
    u8 type;
    Vec4 pos;
    float range;
    float intensity;
    type::RGBA8 color;
};

struct RawArea {
    char name[];
    s32 unknown1;
    char unknown2[];
    Vec4 pos;
    Vec4 rot;
    float scale;
    Vec3 dimensions;
};

struct RawGroup {
    char name[];
    Vec4 pos;
    Vec4 rot;
    float scale;
};

struct Tile {
    s32 x;
    s32 y;
    Vec4 pos;
    s32 unknown1;
    if (unknown1 > 0) {
        s32 unknown2;
        s32 unknown3;
        s32 unknown4;
        s32 unknown5;
    }
    float unknown6;
    s32 eco_data_count;
    s32 eco_data[eco_data_count];
    s32 runtime_obj_count;
    RuntimeObject runtime_objs[runtime_obj_count];
    s32 lights_count;
    RawLight lights[lights_count];
    s32 area_count;
    RawArea areas[area_count];
    s32 group_count;
    RawGroup groups[group_count];
    s32 index;
    padding[4];
};

struct RenderBatch {
    s32 index_offset;
    s32 index_count;
    s32 vertex_offset;
    s32 vertex_count;
};

struct Vertex {
    Vec3 pos;
    u8 normal[4];
    type::RGBA8 color;
    type::RGBA8 color2;
    u16 tex_coord[2];
    u16 tex_coord2[2];
};

struct GcnkChunk {
    s32 tile_count;
    Tile tiles[tile_count];
    s32 render_batch_count;
    RenderBatch render_batches[render_batch_count];
    s32 detail_mask_channels;
    if (detail_mask_channels > 0) {
        s32 detail_mask_size;
        if (gcnk_version >= 4) {
            // Channels are 4 bits each.
            // detail_mask_channels == 1 -> single 4-bit channel (R4)
            // detail_mask_channels == 2 -> two 4-bit channels (RG44)
            // detail_mask_channels == 4 -> four 4-bit channels (RGBA4444)
            u8 detail_mask_pixels[detail_mask_size * detail_mask_size * detail_mask_channels / 2];
        } else {
            // Channels are 8 bits each.
            // detail_mask_channels == 1 -> single 8-bit channel (R8)
            // detail_mask_channels == 2 -> two 8-bit channels (RG88)
            // detail_mask_channels == 4 -> four 8-bit channels (RGBA8888)
            u8 detail_mask_pixels[detail_mask_size * detail_mask_size * detail_mask_channels];
        }
    }
    s32 index_count;
    u16 indices[index_count];
    s32 vertex_count;
    Vertex vertices[vertex_count];
};

s32 gcnk_version in;
#ifndef ROOT_PATTERN
GcnkChunk chunk @ 0x00;
#endif