return {

    Structs = {

        CharacterVariables = {
            name = "CharacterVariables",
            fields = {
                { name = "unknown1", type = "int32"   },
                { name = "unknown2", type = "string32"},
                { name = "unknown3", type = "int32"   },
            }
        },

        CharacterAttachmentData = {
            name = "CharacterAttachmentData",
            fields = {
                { name = "modelname",          type = "string32" },
                { name = "texturealias",       type = "string32" },
                { name = "tintalias",          type = "string32" },
                { name = "tintid",             type = "int32"   },
                { name = "compositeeffectid",  type = "int32"   },
                { name = "slot",               type = "int32"   },
            }
        },

        CustomizationDetail = {
            name = "CustomizationDetail",
            fields = {
                { name = "type",            type = "int32"   },
                { name = "texturealias",    type = "string32" },
                { name = "tintalias",       type = "string32" },
                { name = "tintid",          type = "int32"   },
                { name = "textureoverride", type = "string32" },
            }
        },
    },

    SubOpcodes = {

        [0x01] = { name = "AddPc" },

        [0x02] = {
            name = "AddNpc",
            fields = {
                { name = "guid",                 type = "uint64" },
                { name = "nameid",               type = "int32"  },
                { name = "modelid",              type = "int32"  },
                { name = "unknown1",             type = "bool"   },
                { name = "chatbubble_fg_color",  type = "int32"  },
                { name = "chatbubble_bg_color",  type = "int32"  },
                { name = "chatbubble_size",      type = "int32"  },

                { name = "scale",                type = "float"  },
                { name = "pos",                  type = "Pos"    },
                { name = "rot",                  type = "Pos"    },
                { name = "animationid",          type = "int32"  },

                -- Attachments
                { name = "attachments", type = "array", elem = "CharacterAttachmentData" },

                { name = "disposition",          type = "int32"  },
                { name = "texturealias",         type = "string32" },
                { name = "tintalias",            type = "string32" },
                { name = "tintid",               type = "int32"  },
                { name = "unknown2",             type = "bool"   },
                { name = "yoffset",              type = "float"  },
                { name = "compositeeffectid",    type = "int32"  },
                { name = "wieldtype",            type = "int32"  },
                { name = "customname",           type = "string32" },
                { name = "hidenameplate",        type = "bool"   },
                { name = "unknown3",             type = "float"  },
                { name = "unknown4",             type = "float"  },
                { name = "unknown5",             type = "float"  },
                { name = "terrainobjectid",      type = "int32"  },
                { name = "hasattachments",       type = "bool"   },
                { name = "speed",                type = "float"  },
                { name = "unknown6",             type = "bool"   },
                { name = "interactrange",        type = "int32"  },
                { name = "walkanimid",           type = "int32"  },
                { name = "runanimid",            type = "int32"  },
                { name = "standanimid",          type = "int32"  },
                { name = "unknown7",             type = "bool"   },
                { name = "unknown8",             type = "bool"   },
                { name = "subtextnameid",        type = "int32"  },
                { name = "unknown9",             type = "int32"  },
                { name = "temporaryappearance",  type = "int32"  },

                -- Effect tags
                { name = "effecttags", type = "array", elem = "EffectTag" },

                { name = "unknown38",            type = "bool"   },
                { name = "unknown39",            type = "int32"  },

                { name = "unknown40",            type = "bool"   },
                { name = "unknown41",            type = "bool"   },
                { name = "unknown42",            type = "bool"   },

                { name = "has_tilt",             type = "bool"   },
                { name = "customization",        type = "CustomizationDetail" },
                { name = "tilt",                 type = "Pos"    },

                { name = "namecolor",            type = "float"  },
                { name = "areadefinitionid",     type = "int32"  },
                { name = "imagesetid",           type = "int32"  },

                { name = "is_interactable",      type = "bool"   },
                { name = "rider_guid",           type = "uint64" },

                { name = "movement_type",        type = "int32"  },
                { name = "unknown51",            type = "float"  },

                { name = "target",               type = "Target" },
                { name = "variables",            type = "CharacterVariables" },

                { name = "unknown52",            type = "int32"  },
                { name = "unknown53",            type = "float"  },
                { name = "unknown54",            type = "Pos"    },
                { name = "unknown55",            type = "int32"  },

                { name = "unknown56",            type = "float"  },
                { name = "unknown57",            type = "float"  },
                { name = "unknown58",            type = "float"  },

                { name = "head",                 type = "string32" },
                { name = "hair",                 type = "string32" },
                { name = "modelcustomization",   type = "string32" },

                { name = "replace_terrain_object", type = "bool" },

                { name = "unknown63",            type = "int32"  },
                { name = "unknown64",            type = "int32"  },

                { name = "flyby_effect_id",      type = "int32"  },

                { name = "active_profile",       type = "int32"  },

                { name = "unknown67",            type = "int32"  },
                { name = "unknown68",            type = "int32"  },

                { name = "name_scale",           type = "float"  },
                { name = "nameplate_image_id",   type = "int32"  },
            }
        },

        [0x03] = { name = "RemovePlayer" },
        [0x04] = { name = "Knockback" },
        [0x05] = { name = "UpdateHitpoints" },
        [0x06] = { name = "EquipItemChange" },
        [0x07] = { name = "EquippedItemsChange" },
        [0x08] = { name = "SetAnimation" },
        [0x09] = { name = "UpdateMana" },
        [0x0A] = { name = "AddNotifications" },
        [0x0B] = { name = "RemoveNotifications" },

        [0x0C] = {
            name = "NpcRelevance",
            fields = {
                {
                    name = "npcData",
                    type = "array",
                    elem = {
                        { name = "guid",     type = "uint64" },
                        { name = "unknown1", type = "bool"   },
                        { name = "cursor",   type = "uint8"  },
                        { name = "unknown2", type = "bool"   },
                    }
                }
            }
        },

        [0x0D] = { name = "UpdateScale" },
        [0x0E] = { name = "UpdateTemporaryAppearance" },
        [0x0F] = { name = "RemoveTemporaryAppearance" },
        [0x10] = { name = "PlayCompositeEffect" },
        [0x11] = { name = "SetLookAt" },
        [0x12] = { name = "UpdateLivesRemaining" },
        [0x13] = { name = "RenamePlayer" },
        [0x14] = { name = "UpdateCharacterState" },
        [0x15] = { name = "UpdateWalkAnim" },
        [0x16] = { name = "QueueAnimation" },

        [0x17] = {
            name = "ExpectedSpeed",
            fields = {
                { name = "guid",  type = "uint64" },
                { name = "speed", type = "float"  },
            }
        },

        [0x18] = { name = "ScriptedAnimation" },
        [0x19] = { name = "UpdateRunAnim" },
        [0x1A] = { name = "UpdateIdleAnim" },
        [0x1B] = { name = "ThoughtBubble" },
        [0x1C] = { name = "SetDisposition" },
        [0x1D] = { name = "LootEvent" },
        [0x1E] = { name = "HeadInflationScale" },
        [0x1F] = { name = "SlotCompositeEffectOverride" },
        [0x20] = { name = "Freeze" },
        [0x21] = { name = "RequestStripEffect" },
        [0x22] = { name = "ItemDefinitionRequest" },
        [0x23] = { name = "HitPointModification" },
        [0x24] = { name = "TriggerEffectPackage" },
        [0x25] = { name = "ItemDefinitions" },
        [0x26] = { name = "PreferredLanguages" },
        [0x27] = { name = "CustomizationChange" },
        [0x28] = { name = "PlayerTitle" },
        [0x29] = { name = "AddEffectTagCompositeEffect" },
        [0x2A] = { name = "RemoveEffectTagCompositeEffect" },
        [0x2B] = { name = "EffectTagCompositeEffectsEnable" },
        [0x2C] = { name = "StartRentalUpsell" },
        [0x2D] = { name = "SetSpawnAnimation" },
        [0x2E] = { name = "CustomizeNpc" },
        [0x2F] = { name = "SetSpawnerActivationEffect" },
        [0x30] = { name = "RemoveNpcCustomization" },
        [0x31] = { name = "ReplaceBaseModel" },
        [0x32] = { name = "SetCollidable" },

        [0x33] = {
            name = "UpdateOwner",
            fields = {
                { name = "child_guid", type = "uint64" },
                { name = "owner",      type = "Target" },
                { name = "attach",     type = "bool"   },
            }
        },

        [0x34] = { name = "UpdateTintAlias" },
        [0x35] = { name = "MoveOnRail" },
        [0x36] = { name = "ClearMovementRail" },
        [0x37] = { name = "MoveOnRelativeRail" },
        [0x38] = { name = "Destroyed" },
        [0x39] = { name = "UpdateShields" },
        [0x3A] = { name = "HitPointAndShieldsModification" },
        [0x3B] = { name = "SeekTarget" },
        [0x3C] = { name = "SeekTargetUpdate" },
        [0x3D] = { name = "UpdateActiveWieldType" },
        [0x3E] = { name = "LaunchProjectile" },
        [0x3F] = { name = "SetSynchronizedAnimations" },
        [0x40] = { name = "HudMessage" },
        [0x41] = { name = "CustomizationData" },
        [0x42] = { name = "MemberStatusUpdate" },
        [0x46] = { name = "Popup" },
        [0x47] = { name = "ProfileNameplateImageId" },
    }
}
