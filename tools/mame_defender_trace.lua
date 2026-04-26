-- Local MAME trace exporter for Williams Defender red-label comparison.
--
-- This script is run by tools/generate_reference_traces.py via MAME's
-- -autoboot_script option. It writes the checked-in trace schema header plus
-- one TSV row per scripted frame. Generated traces are local fixtures and must
-- not be committed.

local inputs_path = os.getenv("DEFENDER_TRACE_INPUTS")
local output_path = os.getenv("DEFENDER_TRACE_OUTPUT")
local schema_path = os.getenv("DEFENDER_TRACE_SCHEMA")
local frame_limit = tonumber(os.getenv("DEFENDER_TRACE_FRAMES") or "0")
local debug_path = os.getenv("DEFENDER_TRACE_DEBUG")

if not inputs_path or not output_path or not schema_path or frame_limit <= 0 then
    error("DEFENDER_TRACE_INPUTS, DEFENDER_TRACE_OUTPUT, DEFENDER_TRACE_SCHEMA, and DEFENDER_TRACE_FRAMES are required")
end

local function read_all(path)
    local handle = assert(io.open(path, "r"))
    local text = handle:read("*a")
    handle:close()
    return text
end

local function trim(text)
    return (text:gsub("^%s+", ""):gsub("%s+$", ""))
end

local function split(text, delimiter)
    local values = {}
    if text == "" then
        return values
    end
    local pattern = string.format("([^%s]+)", delimiter)
    for value in string.gmatch(text, pattern) do
        values[#values + 1] = trim(value)
    end
    return values
end

local schema = trim(read_all(schema_path))
local input_frames = split(trim(read_all(inputs_path)), ";")
if #input_frames ~= frame_limit then
    error(string.format("input script has %d frame(s), expected %d", #input_frames, frame_limit))
end

local machine = manager.machine
local maincpu = assert(machine.devices[":maincpu"])
local program = assert(maincpu.spaces["program"])
local output = assert(io.open(output_path, "w"))
output:write(schema, "\n")
local debug_output = nil
if debug_path then
    -- Local-only diagnostic stream for mapping fixture drift to ROM execution.
    -- It is intentionally separate from the checked fixture schema.
    debug_output = assert(io.open(debug_path, "w"))
    debug_output:write(
        "frame\tpc\tstatus\tp1_lives\tp1_wave\tp1_bombs\tseed\thseed\tlseed\tobject_table_crc32\tshell_table_crc32\n"
    )
end

local input_masks = {
    fire = { port = "IN0", mask = 0x01 },
    thrust = { port = "IN0", mask = 0x02 },
    smart_bomb = { port = "IN0", mask = 0x04 },
    smartbomb = { port = "IN0", mask = 0x04 },
    hyperspace = { port = "IN0", mask = 0x08 },
    start_two = { port = "IN0", mask = 0x10 },
    start2 = { port = "IN0", mask = 0x10 },
    start_one = { port = "IN0", mask = 0x20 },
    start1 = { port = "IN0", mask = 0x20 },
    reverse = { port = "IN0", mask = 0x40 },
    altitude_down = { port = "IN0", mask = 0x80 },
    down = { port = "IN0", mask = 0x80 },
    altitude_up = { port = "IN1", mask = 0x01 },
    up = { port = "IN1", mask = 0x01 },
    auto_up_manual_down = { port = "IN2", mask = 0x01 },
    service_advance = { port = "IN2", mask = 0x02 },
    advance = { port = "IN2", mask = 0x02 },
    coin_three = { port = "IN2", mask = 0x04 },
    coin3 = { port = "IN2", mask = 0x04 },
    high_score_reset = { port = "IN2", mask = 0x08 },
    coin = { port = "IN2", mask = 0x10 },
    coin_one = { port = "IN2", mask = 0x10 },
    coin1 = { port = "IN2", mask = 0x10 },
    coin_two = { port = "IN2", mask = 0x20 },
    coin2 = { port = "IN2", mask = 0x20 },
    tilt = { port = "IN2", mask = 0x40 },
}

local held_fields = {}

local function ioport(port_name)
    local ports = machine.ioport.ports
    return ports[":" .. port_name] or ports[port_name]
end

local function field_for(port_name, mask)
    local port = assert(ioport(port_name), "missing MAME input port " .. port_name)
    return assert(port:field(mask), string.format("missing %s mask 0x%02X", port_name, mask))
end

for _, binding in pairs(input_masks) do
    local key = binding.port .. ":" .. string.format("%02X", binding.mask)
    if not held_fields[key] then
        held_fields[key] = field_for(binding.port, binding.mask)
    end
end

local function clear_inputs()
    for _, field in pairs(held_fields) do
        field:clear_value()
    end
end

local function apply_inputs(frame_text)
    clear_inputs()
    local bits = 0
    local ports = { IN0 = 0, IN1 = 0, IN2 = 0 }
    if frame_text == "-" or frame_text == "none" then
        return bits, ports
    end
    for _, action in ipairs(split(frame_text, ",")) do
        local binding = input_masks[action]
        if not binding then
            error("unknown trace input action " .. action)
        end
        local key = binding.port .. ":" .. string.format("%02X", binding.mask)
        held_fields[key]:set_value(1)
        ports[binding.port] = ports[binding.port] | binding.mask
    end

    bits = 0
    if (ports.IN2 & 0x10) ~= 0 then bits = bits | 0x0001 end
    if (ports.IN0 & 0x20) ~= 0 then bits = bits | 0x0002 end
    if (ports.IN0 & 0x10) ~= 0 then bits = bits | 0x0004 end
    if (ports.IN0 & 0x80) ~= 0 then bits = bits | 0x0008 end
    if (ports.IN1 & 0x01) ~= 0 then bits = bits | 0x0010 end
    if (ports.IN0 & 0x40) ~= 0 then bits = bits | 0x0020 end
    if (ports.IN0 & 0x02) ~= 0 then bits = bits | 0x0040 end
    if (ports.IN0 & 0x01) ~= 0 then bits = bits | 0x0080 end
    if (ports.IN0 & 0x04) ~= 0 then bits = bits | 0x0100 end
    if (ports.IN0 & 0x08) ~= 0 then bits = bits | 0x0200 end
    if (ports.IN2 & 0x20) ~= 0 then bits = bits | 0x0400 end
    if (ports.IN2 & 0x04) ~= 0 then bits = bits | 0x0800 end
    if (ports.IN2 & 0x02) ~= 0 then bits = bits | 0x1000 end
    if (ports.IN2 & 0x08) ~= 0 then bits = bits | 0x2000 end
    if (ports.IN2 & 0x40) ~= 0 then bits = bits | 0x4000 end
    if (ports.IN2 & 0x01) ~= 0 then bits = bits | 0x8000 end
    return bits, ports
end

local function read_u8(address)
    return program:read_u8(address) & 0xff
end

local function crc32(bytes)
    local crc = 0xffffffff
    for _, byte in ipairs(bytes) do
        crc = crc ~ byte
        for _ = 1, 8 do
            local mask = -(crc & 1)
            crc = (crc >> 1) ~ (0xedb88320 & mask)
        end
    end
    return (~crc) & 0xffffffff
end

local function crc_range(start_address, length)
    local bytes = {}
    for index = 0, length - 1 do
        bytes[#bytes + 1] = read_u8(start_address + index)
    end
    return crc32(bytes)
end

local function bcd_score(address)
    local score = 0
    for offset = 0, 3 do
        local byte = read_u8(address + offset)
        score = (score * 100) + (((byte >> 4) & 0x0f) * 10) + (byte & 0x0f)
    end
    return score
end

local function phase_label()
    local status = read_u8(0xA0BA)
    if (status & 0x80) ~= 0 then
        return "game_over"
    end
    if read_u8(0xA1C9) ~= 0 then
        return "playing"
    end
    return "attract"
end

local function format_commands()
    return "-"
end

for frame_number, frame_text in ipairs(input_frames) do
    local input_bits, input_ports = apply_inputs(frame_text)
    emu.wait_next_frame()

    output:write(string.format(
        "%d\t0x%04X\t0x%02X\t0x%02X\t0x%02X\t%s\t%d\t%d\t%d\t%d\t%d\t0x%02X\t0x%02X\t0x%02X\t0x%08X\t0x%08X\t-\t%s\t-\n",
        frame_number,
        input_bits,
        input_ports.IN0,
        input_ports.IN1,
        input_ports.IN2,
        phase_label(),
        bcd_score(0xA1C2),
        bcd_score(0xA1FF),
        read_u8(0xA1CA),
        read_u8(0xA1C9),
        read_u8(0xA1CB),
        read_u8(0xA0DF),
        read_u8(0xA0E0),
        read_u8(0xA0E1),
        crc_range(0xA23C, 0x17 * 95),
        crc_range(0xA06D, 2),
        format_commands()
    ))
    if debug_output then
        debug_output:write(string.format(
            "%d\t0x%04X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%08X\t0x%08X\n",
            frame_number,
            maincpu.state["PC"].value,
            read_u8(0xA0BA),
            read_u8(0xA1C9),
            read_u8(0xA1CA),
            read_u8(0xA1CB),
            read_u8(0xA0DF),
            read_u8(0xA0E0),
            read_u8(0xA0E1),
            crc_range(0xA23C, 0x17 * 95),
            crc_range(0xA06D, 2)
        ))
    end
end

clear_inputs()
output:close()
if debug_output then
    debug_output:close()
end
machine:exit()
