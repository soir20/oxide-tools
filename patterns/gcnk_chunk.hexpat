s32 version in;

struct Vec4 {
    float x;
    float y;
    float z;
    float w;
};

struct RuntimeObjectNode {
    s32 len;
    u8 data[len];
};

struct RuntimeObject {
    s32 unknown1;
    char adr_name[];
    char unknown2[];
    Vec4 pos;
    Vec4 rot;
    float scale;
    if (version >= 6) {
        char texture_alias[];
        char tint_alias[];
        if (tint_alias[0] != '\0') {
            Vec4 unknown3;
        }
    }
    padding[4];
    if (version >= 5) {
        u32 terrain_object_id;
    } else {
        char unknown4[];
    }
    s32 unknown5;
    if (version >= 3) {
        s32 node_count;
        RuntimeObjectNode nodes[node_count];
    }
};

struct RawLight {

};

struct RawArea {

};

struct RawGroup {

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
    /*s32 lights_count;
    RawLight lights[lights_count];
    s32 area_count;
    RawArea areas[area_count];
    s32 group_count;
    RawGroup groups[group_count];
    s32 index;
    padding[4];*/
};

struct RenderBatch {

};

struct DetailMasks {

};

struct IndexBuffer {

};

struct VertexBuffer {

};

struct GcnkChunk {
    s32 tile_count;
    Tile tiles[1];
};

GcnkChunk chunk @ 0x00;