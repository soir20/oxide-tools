return {
    SubOpcodes = {

        [0x01] = { name = "AutoAttackTarget" },
        [0x03] = { name = "SingleAttackTarget" },
        [0x04] = { name = "AttackTargetDamage" },
        [0x05] = { name = "AttackAttackerMissed" },
        [0x06] = { name = "AttackTargetDodged" },

        [0x07] = {
            name = "AttackProcessed",
            fields = {
                { name = "attacker_guid1",            type = "uint64" },
                { name = "attacker_guid2",            type = "uint64" },
                { name = "receiver_guid",             type = "uint64" },

                { name = "damage_dealt",              type = "int32" },
                { name = "max_health",                type = "int32" },
                { name = "receiver_composite_effect", type = "int32" },

                { name = "use_hurt_animation",        type = "bool"   },
                { name = "unknown1",                  type = "bool"   },

                { name = "attacker_composite_effect", type = "int32" },
                { name = "current_health",            type = "int32" },
            }
        },

        [0x09] = { name = "EnableBossDisplay" },
    }
}
