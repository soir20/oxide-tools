import hex.dec;
import type.magic;

std::mem::Section decompressed_chunk = std::mem::create_section("Decompressed chunk data");
std::mem::Section decompressed_collision = std::mem::create_section("Decompressed collision data");

struct Gcnk {
    type::Magic<"GCNK"> magic;
    u32 version;
    
    u32 decompressed_chunk_len;
    u32 compressed_chunk_len;
    u8 compressed_chunk[compressed_chunk_len];
    std::assert(
        hex::dec::zlib_decompress(compressed_chunk, decompressed_chunk) == compressed_chunk_len,
        "Decompressed too little or too much chunk data"
    );
    
    u32 decompressed_collision_len;
    u32 compressed_collision_len;
    u8 compressed_collision[compressed_collision_len];
    std::assert(
        hex::dec::zlib_decompress(compressed_collision, decompressed_collision) == compressed_collision_len,
        "Decompressed too little or too much collision data"
    );
    
};

Gcnk gcnk @ 0x00;
