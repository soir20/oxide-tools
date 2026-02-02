#pragma once
#pragma endian little
#define ROOT_PATTERN

import hex.dec;
import type.magic;
#include "gcnk_chunk.hexpat"
#include "gcnk_collision.hexpat"

std::mem::Section decompressed_chunk = std::mem::create_section("Decompressed chunk data");
std::mem::Section decompressed_collision = std::mem::create_section("Decompressed collision data");

struct Gcnk {
    type::Magic<"GCNK"> magic;
    s32 version;
    // Set the version for the GcnkChunk pattern
    gcnk_version = version;
    
    s32 decompressed_chunk_len;
    s32 compressed_chunk_len;
    u8 compressed_chunk[compressed_chunk_len];
    std::assert(
        hex::dec::zlib_decompress(compressed_chunk, decompressed_chunk) == compressed_chunk_len,
        "Decompressed too little or too much chunk data"
    );
    GcnkChunk chunk @ 0x00 in decompressed_chunk;
    
    s32 decompressed_collision_len;
    s32 compressed_collision_len;
    u8 compressed_collision[compressed_collision_len];
    std::assert(
        hex::dec::zlib_decompress(compressed_collision, decompressed_collision) == compressed_collision_len,
        "Decompressed too little or too much collision data"
    );
    GcnkCollision collision @ 0x00 in decompressed_collision;
};

Gcnk gcnk @ 0x00;
