#pragma once

#include <std/mem.pat>
#include "bvh.hexpat"
#include "vec.hexpat"

struct GcnkCollisionEntry {
    s32 len;
    padding[28];
    Bvh bvh;
};

struct GcnkCollision {
    GcnkCollisionEntry entries[while(std::mem::read_unsigned($, 4) != 0x00)];
};

#ifndef ROOT_PATTERN
GcnkCollision collision @ 0x00;
#endif
