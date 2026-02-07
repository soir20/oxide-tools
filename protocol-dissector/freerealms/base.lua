local modules = {
    PlayerUpdate = require("playerupdate"),
    Combat       = require("combat"),
}

local OpCode = {
    [0x23] = {
        name       = "PlayerUpdate",
        subopcodes = modules.PlayerUpdate.SubOpcodes,
    },

    [0x20] = {
        name       = "Combat",
        subopcodes = modules.Combat.SubOpcodes,
    },

    [0x7D] = {
        name   = "BasePlayerUpdatePosition",
        fields = {
            { name = "guid",            type = "uint64" },
            { name = "posX",            type = "float"  },
            { name = "posY",            type = "float"  },
            { name = "posZ",            type = "float"  },
            { name = "rotX",            type = "float"  },
            { name = "rotY",            type = "float"  },
            { name = "rotZ",            type = "float"  },
            { name = "character_state", type = "uint8"  },
            { name = "unknown1",        type = "uint8"  },
        }
    },
}

local Struct = {

    Pos = {
        name = "Pos",
        fields = {
            { name = "x", type = "float" },
            { name = "y", type = "float" },
            { name = "z", type = "float" },
            { name = "w", type = "float" },
        }
    },

    EffectTag = {
        name = "EffectTag",
        fields = {
            { name = "instanceid", type = "int32" },
            { name = "unknown2",   type = "int32" },
            { name = "unknown3",   type = "int32" },
            { name = "unknown4",   type = "int32" },
            { name = "unknown5",   type = "int32" },
            { name = "unknown6",   type = "int32" },
            { name = "duration",   type = "int32" },
            { name = "unknown8",   type = "bool"   },
            { name = "unknown9",   type = "int64"  },
            { name = "starttime",  type = "uint64" },
            { name = "unknown11",  type = "uint64" },
            { name = "unknown12",  type = "int32" },
            { name = "unknown13",  type = "int32" },
            { name = "unknown14",  type = "int64"  },
            { name = "unknown15",  type = "int32" },
            { name = "unknown16",  type = "int32" },
            { name = "unknown17",  type = "bool" },
            { name = "unknown18",  type = "bool" },
            { name = "unknown19",  type = "bool" },
        }
    },

    Target = {

        None = {
            name = "None",
            fields = {}
        },

        Guid = {
            name = "GuidTarget",
            fields = {
                { name = "fallback_pos", type = "Pos"    },
                { name = "guid",         type = "uint64" },
            }
        },

        BoundingBox = {
            name = "BoundingBoxTarget",
            fields = {
                { name = "fallback_pos", type = "Pos" },
                { name = "min_pos",      type = "Pos" },
                { name = "max_pos",      type = "Pos" },
            }
        },

        CharacterBoneName = {
            name = "CharacterBoneNameTarget",
            fields = {
                { name = "fallback_pos",   type = "Pos"      },
                { name = "character_guid", type = "uint64"   },
                { name = "bone_name",      type = "string32" },
            }
        },

        CharacterBoneId = {
            name = "CharacterBoneIdTarget",
            fields = {
                { name = "fallback_pos",   type = "Pos"    },
                { name = "character_guid", type = "uint64" },
                { name = "bone_id",        type = "int32"  },
            }
        },

        ActorBoneName = {
            name = "ActorBoneNameTarget",
            fields = {
                { name = "fallback_pos", type = "Pos"      },
                { name = "actor_id",     type = "int32"    },
                { name = "bone_name",    type = "string32" },
            }
        },

        ActorBoneId = {
            name = "ActorBoneIdTarget",
            fields = {
                { name = "fallback_pos", type = "Pos"    },
                { name = "actor_id",     type = "int32"  },
                { name = "bone_id",      type = "int32"  },
            }
        },
    },
}

-- Merge Structs from all modules
for _, mod in pairs(modules) do
    -- Normal structs
    for name, def in pairs(mod.Structs or {}) do
        Struct[name] = def
    end

    -- Struct-like entries inside SubOpcodes (string keys)
    for key, value in pairs(mod.SubOpcodes or {}) do
        if type(key) == "string" then
            Struct[key] = value
        end
    end
end

local EnumType = {
    Target = {
        [0] = Struct.Target.None,
        [1] = Struct.Target.Guid,
        [2] = Struct.Target.BoundingBox,
        [3] = Struct.Target.CharacterBoneName,
        [4] = Struct.Target.CharacterBoneId,
        [5] = Struct.Target.ActorBoneName,
        [6] = Struct.Target.ActorBoneId,
    }
}

return {
    opcodes   = OpCode,
    structs   = Struct,
    enumtypes = EnumType,
}
