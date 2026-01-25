#pragma once

import type.magic;
#include "bvh.hexpat"

bitfield CollisionType {
    unsigned base_type           : 30;
    bool disable_cursor          : 1;
    bool disable_camera_collision: 1;
};

struct Vertex {
    float x;
    float y;
    float z;
};

struct Triangle {
    u16 indices[3];
};

struct CollisionDataEntry {
    s32 should_skip_entry;
    if (!should_skip_entry) {
        s32 vertex_count;
        Vertex vertices[vertex_count];
        s32 triangle_count;
        Triangle triangles[triangle_count];
        s32 bvh_byte_count;
        padding[16];
        Bvh bvh_data;
    }
};

struct Cdt {
    type::Magic<"CDTA"> magic;
    // always 1
    s32 version;
    // handled at 00ea42c0, 00ea3fcb, 00d87bd0
    CollisionType collision_type;
    // always 1
    s32 entry_count;
    CollisionDataEntry data[entry_count];
};

Cdt cdt_at_0x00 @ 0x00;
