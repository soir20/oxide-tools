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

struct Vec3 {
    float x;
    float y;
    float z;
    float w;
};

struct Node {
    Vec3 aabb_min_org;
    Vec3 aabb_max_org;
    u32 escape_index;
    u32 sub_part;
    u32 triangle_index;
    padding[20];
};

struct QuantizedNode {
    u16 quantized_aabb_min[3];
    u16 quantized_aabb_max[3];
    u32 escape_index_or_triangle_index;
};

struct SubtreeHeader {
    u16 quantized_aabb_min[3];
    u16 quantized_aabb_max[3];
    u32 root_node_index;
    u32 subtree_size;
    padding[12];
};

enum TraversalMode : u32 {
    Stackless = 0,
    StacklessCacheFriendly = 1,
    Recursive = 2
};

// https://github.com/vancegroup-mirrors/bullet-physics/blob/bullet-2.76/src/BulletCollision/BroadphaseCollision/btQuantizedBvh.cpp#L1031
struct Bvh {
    Vec3 aabb_min;
    Vec3 aabb_max;
    Vec3 quantization;
    u32 bullet_version;
    u32 node_count;
    bool use_quantization;
    
    padding[83];
    TraversalMode traversal_mode;
    padding[20];
    u32 subtree_header_count;
    padding[8];
    if (use_quantization) {
        QuantizedNode nodes[node_count];
    } else {
        Node nodes[node_count];
    }
    SubtreeHeader subtree_headers[subtree_header_count];
};

struct CollisionDataEntry {
    u32 should_skip_entry;
    if (!should_skip_entry) {
        u32 vertex_count;
        Vertex vertices[vertex_count];
        u32 triangle_count;
        Triangle triangles[triangle_count];
        u32 bvh_byte_count;
        padding[16];
        Bvh bvh_data;
    }
};

struct Cdt {
    // CDTA (exact case match)
    char magic[0x4];
    // always 1
    u32 version;
    // handled at 00ea42c0, 00ea3fcb, 00d87bd0
    CollisionType collision_type;
    // always 1
    u32 entry_count;
    CollisionDataEntry data[entry_count];
};

Cdt cdt_at_0x00 @ 0x00;