local base = require("base")

local opcodes   = base.opcodes
local structs   = base.structs
local enumtypes = base.enumtypes

game_protocol = Proto("freerealms", "Free Realms Game Protocol")

local opcode_vs = {}
local sub_vs    = {}

-- Build value strings
for id, opcode in pairs(opcodes) do
    if type(id) == "number" then
        opcode_vs[id] = opcode.name
    end

    local sub = opcode.subopcodes
    if sub then
        for sid, subdef in pairs(sub) do
            if type(sid) == "number" then
                sub_vs[sid] = subdef.name
            end
        end
    end
end

game_opcode    = ProtoField.uint8("freerealms.opcode",    "GameOpcode",    base.HEX, opcode_vs)
game_subopcode = ProtoField.string("freerealms.subopcode", "GameSubOpcode")

function read_le_uint64(tvb, offset)
    local lo = tvb(offset, 4):le_uint()
    local hi = tvb(offset + 4, 4):le_uint()
    return hi * 0x100000000 + lo
end

local primitive_pf_map = {
    uint8    = ProtoField.uint8,
    uint16   = ProtoField.uint16,
    int32    = ProtoField.int32,
    uint64   = ProtoField.uint64,
    float    = ProtoField.float,
    bool     = ProtoField.bool,
    string32 = ProtoField.bytes,
}

function make_primitive_pf(t, fullname, label)
    local ctor = primitive_pf_map[t]
    if ctor then
        return ctor(fullname, label)
    end

    -- Enums and arrays handled elsewhere
    if enumtypes[t] or t == "array" then
        return nil
    end

    return ProtoField.bytes(fullname, label)
end

function walk_struct_fields(prefix, struct, callback)
    if not struct or not struct.fields then
        return
    end

    for _, f in ipairs(struct.fields) do
        local t = f.type
        local fullname = prefix .. "." .. f.name

        callback(fullname, t, f)

        local nested_type = (t == "array") and f.elem or t

        local sdef = structs[nested_type]
        if sdef then
            walk_struct_fields(fullname, sdef, callback)
        else
            local enum_variants = enumtypes[nested_type]
            if enum_variants then
                for _, variant in pairs(enum_variants) do
                    walk_struct_fields(fullname, variant, callback)
                end
            end
        end
    end
end

local all_registered_fields = {}

-- Convert enum definition -> Wireshark value-string table
function enum_value_strings(enumdef)
    local t = {}
    for key, val in pairs(enumdef) do
        if type(val) == "table" and val.name then
            t[key] = tostring(val.name)
        elseif type(val) == "string" then
            t[key] = val
        else
            t[key] = tostring(val)
        end
    end
    return t
end

function register_fields(def)
    if not def.fields then
        return
    end

    local base = "freerealms." .. def.name

    walk_struct_fields(base, def, function(fullname, t, f)

        if enumtypes[t] then
            local value_strings = enum_value_strings(enumtypes[t])
            local pf = ProtoField.uint32(fullname, f.name, base.DEC, value_strings)
            f.protofield = pf
            table.insert(all_registered_fields, pf)
            return
        end

        -- Array length ProtoField (renamed)
        if t == "array" then
            local pf = ProtoField.uint32(fullname .. "_count", f.name .. " count")
            f.arr_len_protofield = pf
            table.insert(all_registered_fields, pf)
            return
        end

        if primitive_pf_map[t] then
            local pf = make_primitive_pf(t, fullname, f.name)
            f.protofield = pf
            table.insert(all_registered_fields, pf)
            return
        end

        local pf = ProtoField.bytes(fullname, f.name)
        f.protofield = pf
        table.insert(all_registered_fields, pf)
    end)
end

-- Register fields for all opcodes and subopcodes
for _, def in pairs(opcodes) do
    register_fields(def)

    local sub = def.subopcodes
    if sub then
        for _, subdef in pairs(sub) do
            register_fields(subdef)
        end
    end
end

-- Register fields for all enums
for _, enum_map in pairs(enumtypes) do
    for _, variant in pairs(enum_map) do
        register_fields(variant)
    end
end
----------------------------------------------------------------------------------------------------------------
function add_val(tree, f, slice, text)
    local pf = f.protofield
    if pf then
        tree:add(pf, slice):set_text(text)
    else
        tree:add(text, slice)
    end
end

function add_name(tree, f, slice, v)
    add_val(tree, f, slice, f.name .. ": " .. tostring(v))
end

function decode_primitive(tvb, off, tree, f)
    local t = f.type

    local function add(size, val)
        local s = tvb(off, size)
        tree:add(f.protofield, s, val)
        return size
    end

    if t == "uint8"  then return add(1, tvb(off,1):uint()) end
    if t == "uint16" then return add(2, tvb(off,2):le_uint()) end
    if t == "int32"  then return add(4, tvb(off,4):le_int()) end

    if t == "uint64" then
        local s = tvb(off,8)
        tree:add(f.protofield, s)
        return 8
    end

    if t == "float" then
        local s = tvb(off,4)
        local v = s:le_float()

        local formatted =
            (math.floor(v) == v)
            and string.format("%.1f", v)
            or string.format("%.6f", v):gsub("0+$",""):gsub("%.$","")

        local node = tree:add(f.protofield, s, v)
        node:set_text(f.name .. ": " .. formatted)
        return 4
    end

    if t == "bool" then
        return add(1, tvb(off,1):uint() ~= 0)
    end

    if t == "string32" then
        local len_s = tvb(off,4)
        local n = len_s:le_int()

        if n == -1 then
            tree:add(f.protofield, len_s, "<null>")
            return 4
        end

        if n < 0 or off+4+n > tvb:len() then
            return 4
        end

        local s = tvb(off+4,n)
        tree:add(f.protofield, s, s:string())
        return 4+n
    end

    return nil
end

function decode_enum(tvb, off, tree, f)
    local tag_s = tvb(off,4)
    local tag = tag_s:le_uint()

    local node = tree:add(f.protofield, tag_s, tag)

    local vdef = enumtypes[f.type][tag]
    if vdef then
        return 4 + decode_struct(tvb, off+4, node, vdef)
    end

    return 4
end

function decode_struct_inner(tvb, off, tree, f)
    local node = tree:add(f.name .. ":")
    return decode_struct(tvb, off, node, structs[f.type])
end

function decode_array(tvb, off, tree, f)
    local count_s = tvb(off,4)
    local count = count_s:le_int()
    local label = string.format("%s_count: %d", f.name, count)

    local parent = f.arr_len_protofield
        and tree:add(f.arr_len_protofield, count_s, count)
        or  tree:add(label, count_s)

    if f.arr_len_protofield then
        parent:set_text(label)
    end

    local cursor = 4
    if count <= 0 then return cursor end

    local elem = f.elem
    local arr_off = off + 4

    local enum_def = enumtypes[elem]
    if enum_def then
        for i = 1, count do
            if arr_off + 4 > tvb:len() then break end

            local tag_s = tvb(arr_off,4)
            local tag = tag_s:le_uint()
            local vdef = enum_def[tag]
            local typename = vdef and vdef.name or ("Unknown(" .. tag .. ")")

            local node = parent:add(
                string.format("%s[%d] (%s: %s)", f.name, i-1, elem, typename),
                tag_s
            )

            local used = 4
            if vdef then
                used = used + decode_struct(tvb, arr_off + 4, node, vdef)
            end

            arr_off = arr_off + used
        end

        return arr_off - off
    end

    local elem_def = structs[elem]
    if elem_def and elem_def.fields then
        for i = 1, count do
            local node = parent:add(string.format("%s[%d]", f.name, i-1))
            local used = decode_struct(tvb, arr_off, node, elem_def)
            arr_off = arr_off + used
        end

        return arr_off - off
    end

    return cursor
end

function decode_struct(tvb, base_off, tree, def)
    if not def or not def.fields then return 0 end

    local cursor = 0

    for _, f in ipairs(def.fields) do
        local off = base_off + cursor
        if off >= tvb:len() then break end

        local t = f.type

        local dec =
            enumtypes[t] and decode_enum or
            structs[t]   and decode_struct_inner or
            t == "array" and decode_array or
            nil

        if dec then
            cursor = cursor + dec(tvb, off, tree, f)
        else
            local used = decode_primitive(tvb, off, tree, f)
            if used then
                cursor = cursor + used
            else
                local sz = size_for_type(t)
                if not sz or off+sz > tvb:len() then
                    tree:add("unknown type (" .. tostring(t) .. ")")
                    break
                end
                local slice = tvb(off,sz)
                add_val(tree, f, slice, string.format("%s (%s)", f.name, t))
                cursor = cursor + sz
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
        local subhex = string.format("0x%02X", subval)

        pkt_tree:add(game_subopcode, subname)
            :set_text("SubOpcode: " .. subname .. " (" .. subhex .. ")")

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
            parent_tree:add(game_data, tvb(pos, len - pos))
            break
        end

        if def.fields and not def.subopcodes then
            local consumed = decode_game_packet(tvb, pos, parent_tree, def, op, nil)
            if consumed <= 0 then break end
            pos = pos + consumed

        else
            if pos + 3 > len then break end

            local sub    = tvb(pos + 2, 1):uint()
            local subdef = def.subopcodes and def.subopcodes[sub]

            if not subdef then
                local opname = def.name or string.format("0x%02X", op)

                parent_tree:add(game_opcode, tvb(pos, 1))
                    :set_text("Opcode: " .. opname .. " (" .. string.format("0x%02X", op) .. ")")

                parent_tree:add(game_subopcode, tvb(pos + 2, 1))
                    :set_text("SubOpcode: UNKNOWN (" .. string.format("0x%02X", sub) .. ")")

                parent_tree:add(game_data, tvb(pos, len - pos))
                break
            end

            local consumed = decode_game_packet(tvb, pos, parent_tree, subdef, op, sub)
            if consumed <= 0 then break end
            pos = pos + consumed
        end
    end
end

game_protocol.fields = {
    game_opcode,
    game_subopcode,
    table.unpack(all_registered_fields)
}

return {
    decode_game_stream = decode_game_stream,
}
