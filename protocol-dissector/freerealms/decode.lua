local base = require("base")

local opcodes   = base.opcodes
local structs   = base.structs
local enumtypes = base.enumtypes

game_protocol = Proto("freerealms", "Free Realms Game Protocol")

-- Build lookup tables
local opcode_vs = {}
local sub_vs    = {}

-- Opcode and subopcode tables contain both numeric IDs and string keys
-- Only numeric IDs belong in a value_string table, so we filter for type == "number"

-- Build opcode value strings
for id, opcode in pairs(opcodes) do
    if type(id) == "number" then
        opcode_vs[id] = opcode.name
    end
end

for _, opcode in pairs(opcodes) do
    local sub = opcode.subopcodes
    if sub then
        for id, subdef in pairs(sub) do
            if type(id) == "number" then
                sub_vs[id] = subdef.name
            end
        end
    end
end

game_opcode    = ProtoField.uint8("freerealms.opcode",    "GameOpcode",    base.HEX, opcode_vs)
game_subopcode = ProtoField.uint8("freerealms.subopcode", "GameSubOpcode", base.HEX, sub_vs)

-- Wireshark doesn't have a manual 64-bit reader, so we manually decode.
function read_le_uint64(tvb, offset)
    local lo = tvb(offset, 4):le_uint()
    local hi = tvb(offset + 4, 4):le_uint()
    return hi * 0x100000000 + lo
end

function size_for_type(t)
    if t == "uint8"  then return 1 end
    if t == "uint16" then return 2 end
    if t == "int32"  then return 4 end
    if t == "uint64" then return 8 end
    if t == "float"  then return 4 end
    if t == "bool"   then return 1 end
    return 0
end

-- Build Wireshark value_string tables for enums
local enum_vs_tables = {}

for enum_name, enum_map in pairs(enumtypes) do
    local vs = {}
    for id, struct_def in pairs(enum_map) do
        vs[id] = struct_def.name
    end
    enum_vs_tables[enum_name] = vs
end

function pf_for_type(t, fullname, label)
    -- Primitive types
    if t == "uint8"    then return ProtoField.uint8(fullname,  label) end
    if t == "uint16"   then return ProtoField.uint16(fullname, label) end
    if t == "int32"    then return ProtoField.int32(fullname, label) end
    if t == "uint64"   then return ProtoField.bytes(fullname,  label) end
    if t == "float"    then return ProtoField.bytes(fullname,  label) end
    if t == "bool"     then return ProtoField.bool(fullname,   label) end
    if t == "string32" then return ProtoField.bytes(fullname,  label) end

    -- Not a simple numeric value_strings, so we do NOT create a ProtoField here.
    if enumtypes[t] then
        return nil
    end

    -- Arrays handled separately in registration (no top-level field)
    if t == "array" then
        return nil
    end

    -- Treat unknown as bytes
    return ProtoField.bytes(fullname, label)
end

-- Register fields for filtering
local generated_fields = {}

function register_struct_fields(prefix, struct)
    if not struct or not struct.fields then return end

    for _, f in ipairs(struct.fields) do
        local t = f.type
        local fullname = prefix .. "." .. f.name

        -- Primitive -> register ProtoField
        if not structs[t] and not enumtypes[t] and t ~= "array" then
            f.pf = pf_for_type(t, fullname, f.name)
            if f.pf then
                table.insert(generated_fields, f.pf)
            end

        -- Nested struct -> recurse
        elseif structs[t] then
            register_struct_fields(fullname, structs[t])

        -- Enum -> recurse into each variant
        elseif enumtypes[t] then
            for _, variant in pairs(enumtypes[t]) do
                register_struct_fields(fullname, variant)
            end

        -- Array -> recurse into element
        elseif t == "array" and f.elem then
            register_struct_fields(fullname, { fields = f.elem })
        end
    end
end

function register_fields(def)
    if not def.fields then return end

    for _, f in ipairs(def.fields) do
        local fullname = "freerealms." .. def.name .. "." .. f.name
        local pf_type  = f.type

        -- Don't register a top level blob of bytes field.
        if pf_type ~= "array" then
            f.pf = pf_for_type(pf_type, fullname, f.name)
            if f.pf then
                table.insert(generated_fields, f.pf)
            end
        end

        -- Register ProtoFields for inline array element fields.
        if pf_type == "array" and f.elem then
            for _, ef in ipairs(f.elem) do
                local e_fullname = fullname .. "." .. ef.name
                ef.pf = pf_for_type(ef.type, e_fullname, ef.name)
                if ef.pf then
                    table.insert(generated_fields, ef.pf)
                end
            end
        end
    end

    register_struct_fields("freerealms." .. def.name, def)
end

-- Register opcode + subopcode fields
for _, def in pairs(opcodes) do
    register_fields(def)
    if def.subopcodes then
        for _, subdef in pairs(def.subopcodes) do
            register_fields(subdef)
        end
    end
end

-- Register struct fields
for name, def in pairs(structs) do
    def.name = def.name or name
    register_fields(def)
end

function decode_struct(tvb, base_off, tree, def)
    if not def or not def.fields then return 0 end

    local cursor = 0
    local values = {}

    for _, f in ipairs(def.fields) do
        local t         = f.type
        local field_off = base_off + cursor

        if field_off >= tvb:len() then break end

        if enumtypes[t] then
            if field_off + 4 > tvb:len() then break end

            local tag_slice = tvb(field_off, 4)
            local tag       = tag_slice:le_uint()

            local vdef = enumtypes[t][tag]
            local typename = vdef and vdef.name or string.format("Unknown(%d)", tag)

            local node = tree:add(string.format("%s (%s: %s)", f.name, t, typename))

            local consumed = 4
            if vdef then
                consumed = consumed + decode_struct(tvb, field_off + 4, node, vdef)
            end

            cursor = cursor + consumed
        
        elseif t == "uint8" then
            local slice = tvb(field_off, 1)
            local v     = slice:uint()
            tree:add(f.pf, slice):set_text(f.name .. ": " .. v)
            values[f.name] = v
            cursor = cursor + 1

        elseif t == "uint16" then
            local slice = tvb(field_off, 2)
            local v     = slice:le_uint()
            tree:add(f.pf, slice):set_text(f.name .. ": " .. v)
            values[f.name] = v
            cursor = cursor + 2

        elseif t == "int32" then
            local slice = tvb(field_off, 4)
            local v     = slice:le_int()
            tree:add(f.pf, slice):set_text(f.name .. ": " .. v)
            values[f.name] = v
            cursor = cursor + 4

        elseif t == "uint64" then
            if field_off + 8 > tvb:len() then break end
            local slice = tvb(field_off, 8)
            local v     = read_le_uint64(tvb, field_off)
            tree:add(f.pf, slice):set_text(f.name .. ": " .. tostring(v))
            cursor = cursor + 8

        elseif t == "float" then
            if field_off + 4 > tvb:len() then break end
            local slice = tvb(field_off, 4)
            local v     = slice:le_float()
            tree:add(f.pf, slice):set_text(f.name .. ": " .. tostring(v))
            cursor = cursor + 4

        elseif t == "bool" then
            local slice = tvb(field_off, 1)
            local v     = (slice:uint() ~= 0)
            tree:add(f.pf, slice):set_text(f.name .. ": " .. tostring(v))
            values[f.name] = v
            cursor = cursor + 1

        elseif t == "string32" then
            if field_off + 4 > tvb:len() then break end

            local len_slice = tvb(field_off, 4)
            local strlen    = len_slice:le_int()

            if strlen == -1 then
                tree:add(f.pf, len_slice):set_text(f.name .. ": <null>")
                values[f.name] = nil
                cursor = cursor + 4
            else
                if strlen < 0 or field_off + 4 + strlen > tvb:len() then break end
                local slice = tvb(field_off + 4, strlen)
                local str   = slice:string()
                tree:add(f.pf, slice):set_text(f.name .. ": " .. str)
                values[f.name] = str
                cursor = cursor + 4 + strlen
            end

        elseif structs[t] then
            local structDef = structs[t]
            local sub = tree:add(string.format("%s:", f.name))
            local consumed = decode_struct(tvb, field_off, sub, structDef)
            cursor = cursor + consumed

        elseif t == "array" then
            -- Array length
            if field_off + 4 > tvb:len() then break end

            local count_slice = tvb(field_off, 4)
            local count = count_slice:le_int()

            local parent = tree:add(
                string.format("%s_count", f.name),
                count_slice
            )
            parent:set_text(string.format("%s_count: %d", f.name, count))

            cursor = cursor + 4

            -- If empty array, skip
            if count > 0 and f.elem then
                local arr_cursor = field_off + 4
                for i = 1, count do
                    local elem_node = parent:add(string.format("%s[%d]", f.name, i - 1))
                    local consumed = decode_struct(tvb, arr_cursor, elem_node, { fields = f.elem })
                    arr_cursor = arr_cursor + consumed
                end
                cursor = cursor + (arr_cursor - (field_off + 4))
            end

        -- Fallback
        else
            local sz = size_for_type(t)

            if sz and sz > 0 and field_off + sz <= tvb:len() then
                tree:add(f.pf, tvb(field_off, sz))
                cursor = cursor + sz
            else
                tree:add(string.format("unknown type (%s)", tostring(t)))
                break
            end
        end
    end

    return cursor
end

function packet_size_for_def(def)
    if not def or not def.fields or #def.fields == 0 then
        return 3 -- minimum header size
    end

    local size = 0
    for _, f in ipairs(def.fields) do
        local t = f.type

        -- variable-size types
        if t == "string32" or t == "array" or structs[t] or enumtypes[t] then
            return math.max(size, 3)
        end

        size = size + size_for_type(t)
    end

    if size < 3 then size = 3 end
    return size
end

function decode_game_packet(tvb, offset, parent_tree, def, op, sub)
    local opname = def and def.name or string.format("0x%02X", op)
    local pkt_tvb = tvb(offset):tvb("Game Packet")
    local pkt_tree = parent_tree:add(game_protocol, pkt_tvb(), "Decoded Game Packet (" .. opname .. ")")

    local opdef = opcodes[op]
    local opname = opdef and opdef.name or string.format("0x%02X", op)

    local opval = tvb(offset, 1):uint()
    pkt_tree:add(game_opcode, tvb(offset, 1))
        :set_text("Opcode: " .. opname .. " (" .. string.format("0x%02X", opval) .. ")")

    -- Determine header size based on whether this opcode uses subopcodes
    local header_size

    if opcodes[op] and opcodes[op].subopcodes then
        -- This opcode family uses subopcodes
        local subval = tvb(offset + 2, 1):uint()
        local subname = def and def.name or string.format("0x%02X", subval)

        pkt_tree:add(game_subopcode, tvb(offset + 2, 1))
            :set_text("SubOpcode: " .. subname .. " (" .. string.format("0x%02X", subval) .. ")")

        header_size = 4
    else
        -- No subopcode -> payload starts at byte 2
        header_size = 2
    end

    local payload_off = offset + header_size

    -- Decode the struct
    local consumed = 0
    if def and def.fields then
        consumed = decode_struct(tvb, payload_off, pkt_tree, def)
    end

    if consumed <= 0 then
        consumed = packet_size_for_def(def)
    end

    return header_size + consumed
end

function decode_game_stream(tvb, parent_tree)
    local pos = 0
    local len = tvb:len()

    while pos + 3 <= len do
        local op  = tvb(pos, 1):uint()
        local def = opcodes[op]

        if not def then
            -- Unknown opcode: dump remaining data
            parent_tree:add(game_data, tvb(pos, len - pos))
            break
        end

        -- No subopcode
        if def.fields and not def.subopcodes then
            local consumed = decode_game_packet(tvb, pos, parent_tree, def, op, nil)
            if consumed <= 0 then break end
            pos = pos + consumed

        else
            -- This opcode has a subopcode
            if pos + 3 > len then break end

            local sub    = tvb(pos + 2, 1):uint()
            local subdef = def.subopcodes and def.subopcodes[sub]

            if not subdef then
                -- Unknown subopcode: show opcode + unknown subopcode + raw data
                local opname = def.name or string.format("0x%02X", op)

                parent_tree:add(game_opcode, tvb(pos, 1))
                    :set_text("Opcode: " .. opname .. " (" .. string.format("0x%02X", op) .. ")")

                parent_tree:add(game_subopcode, tvb(pos + 2, 1))
                    :set_text("SubOpcode: UNKNOWN (" .. string.format("0x%02X", sub) .. ")")

                parent_tree:add(game_data, tvb(pos, len - pos))
                break
            end

            -- Decode known sub packet
            local consumed = decode_game_packet(tvb, pos, parent_tree, subdef, op, sub)
            if consumed <= 0 then break end
            pos = pos + consumed
        end
    end
end

game_protocol.fields = {
    game_opcode,
    game_subopcode,
    table.unpack(generated_fields)
}

return {
    decode_game_stream = decode_game_stream,
}
