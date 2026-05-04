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

local MAIN_BOARD_SOUND_COMMAND_HIGH_BITS = 0xc0
local SOUND_COMMAND_IDLE_BYTE = 0xff
local PIA1_PORT_B_DATA_REGISTER = 0xcc02
local PIA1_PORT_B_CONTROL_REGISTER = 0xcc03
local PIA_CONTROL_DATA_REGISTER_SELECT = 0x04
local DEFENDER_BANK_SELECT_REGISTER_START = 0xd000
local DEFENDER_BANK_SELECT_REGISTER_END = 0xdfff
local CREDIT_ADDED_SOUND_COMMAND = 0xe6
local ONE_PLAYER_START_SOUND_COMMAND = 0xf5

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

local function command_from_pia_port_b(data)
    return (data | MAIN_BOARD_SOUND_COMMAND_HIGH_BITS) & 0xff
end

local function append_sound_command(commands, data)
    local command = command_from_pia_port_b(data)
    if command ~= SOUND_COMMAND_IDLE_BYTE then
        commands[#commands + 1] = command
    end
end

local function format_commands(commands)
    if #commands == 0 then
        return "-"
    end

    local values = {}
    for index, command in ipairs(commands) do
        values[index] = string.format("0x%02X", command)
    end
    return table.concat(values, ",")
end

local function copy_commands(commands)
    local copied = {}
    for index, command in ipairs(commands) do
        copied[index] = command
    end
    return copied
end

local function take_commands(commands)
    local text = format_commands(commands)
    for index = #commands, 1, -1 do
        commands[index] = nil
    end
    return text
end

local function command_list_contains(commands, expected_command)
    for _, command in ipairs(commands) do
        if command == expected_command then
            return true
        end
    end
    return false
end

local function events_for_commands(commands)
    local events = {}
    if command_list_contains(commands, CREDIT_ADDED_SOUND_COMMAND) then
        events[#events + 1] = "credit_added"
    end
    if command_list_contains(commands, ONE_PLAYER_START_SOUND_COMMAND) then
        events[#events + 1] = "game_started"
    end
    return events
end

local function format_events(events)
    if #events == 0 then
        return "-"
    end
    return table.concat(events, ",")
end

local function selected_bank_from_data(data)
    return data & 0x0f
end

local function append_bank_select_write(writes, address, data)
    local selected_bank = selected_bank_from_data(data)
    writes[#writes + 1] = {
        address = address & 0xffff,
        data = data & 0xff,
        selected_bank = selected_bank,
    }
    return selected_bank
end

local function format_bank_select_writes(writes)
    if #writes == 0 then
        return "-"
    end

    local values = {}
    for index, write in ipairs(writes) do
        values[index] = string.format(
            "0x%04X=0x%02X->0x%02X",
            write.address,
            write.data,
            write.selected_bank
        )
    end
    return table.concat(values, ",")
end

local function take_bank_select_writes(writes)
    local text = format_bank_select_writes(writes)
    for index = #writes, 1, -1 do
        writes[index] = nil
    end
    return text
end

local function input_ports_from_reader(read_port)
    return {
        IN0 = read_port("IN0") & 0xff,
        IN1 = read_port("IN1") & 0xff,
        IN2 = read_port("IN2") & 0xff,
    }
end

local function install_sound_command_tap(program, commands)
    local port_b_data_selected = false
    return program:install_write_tap(
        PIA1_PORT_B_DATA_REGISTER,
        PIA1_PORT_B_CONTROL_REGISTER,
        "defender_sound_command_trace",
        function(offset, data, mask)
            if offset == PIA1_PORT_B_CONTROL_REGISTER then
                port_b_data_selected = (data & PIA_CONTROL_DATA_REGISTER_SELECT) ~= 0
            elseif offset == PIA1_PORT_B_DATA_REGISTER and port_b_data_selected then
                append_sound_command(commands, data)
            end
        end
    )
end

local function install_bank_select_tap(program, writes, set_current_bank)
    return program:install_write_tap(
        DEFENDER_BANK_SELECT_REGISTER_START,
        DEFENDER_BANK_SELECT_REGISTER_END,
        "defender_bank_select_trace",
        function(offset, data, mask)
            local selected_bank = append_bank_select_write(writes, offset, data)
            set_current_bank(selected_bank)
        end
    )
end

local function assert_equal(actual, expected, label)
    if actual ~= expected then
        error(string.format("%s: expected %s, got %s", label, tostring(expected), tostring(actual)))
    end
end

local function run_self_test()
    local installed = {}
    local program = {
        install_write_tap = function(_, start_address, end_address, name, callback)
            installed.start_address = start_address
            installed.end_address = end_address
            installed.name = name
            installed.callback = callback
            return { remove = function() end }
        end,
    }
    local commands = {}

    install_sound_command_tap(program, commands)
    assert_equal(installed.start_address, PIA1_PORT_B_DATA_REGISTER, "tap start")
    assert_equal(installed.end_address, PIA1_PORT_B_CONTROL_REGISTER, "tap end")
    assert_equal(installed.name, "defender_sound_command_trace", "tap name")

    installed.callback(PIA1_PORT_B_DATA_REGISTER, 0x35, 0xff)
    assert_equal(format_commands(commands), "-", "reset-time DDR write is ignored")
    installed.callback(PIA1_PORT_B_CONTROL_REGISTER, 0x04, 0xff)
    installed.callback(PIA1_PORT_B_DATA_REGISTER, 0x3f, 0xff)
    assert_equal(format_commands(commands), "-", "idle command is suppressed")
    installed.callback(PIA1_PORT_B_DATA_REGISTER, 0x35, 0xff)
    assert_equal(format_commands(commands), "0xF5", "active command is recorded")
    installed.callback(PIA1_PORT_B_CONTROL_REGISTER, 0x00, 0xff)
    installed.callback(PIA1_PORT_B_DATA_REGISTER, 0x29, 0xff)
    assert_equal(format_commands(commands), "0xF5", "DDRB write is ignored")
    installed.callback(PIA1_PORT_B_CONTROL_REGISTER, 0x04, 0xff)
    installed.callback(PIA1_PORT_B_DATA_REGISTER, 0x29, 0xff)
    assert_equal(format_events(events_for_commands(copy_commands(commands))), "game_started", "start sound command marks game start")
    assert_equal(take_commands(commands), "0xF5,0xE9", "commands are formatted and drained")
    assert_equal(format_commands(commands), "-", "drained command list is empty")
    assert_equal(format_events(events_for_commands({ 0xe6, 0xf5 })), "credit_added,game_started", "coin/start events are source markers")
    assert_equal(format_events(events_for_commands({})), "-", "empty event list is formatted as dash")

    local bank_installed = {}
    local bank_program = {
        install_write_tap = function(_, start_address, end_address, name, callback)
            bank_installed.start_address = start_address
            bank_installed.end_address = end_address
            bank_installed.name = name
            bank_installed.callback = callback
            return { remove = function() end }
        end,
    }
    local bank_writes = {}
    local current_bank = 0xff

    install_bank_select_tap(bank_program, bank_writes, function(selected_bank)
        current_bank = selected_bank
    end)
    assert_equal(bank_installed.start_address, DEFENDER_BANK_SELECT_REGISTER_START, "bank tap start")
    assert_equal(bank_installed.end_address, DEFENDER_BANK_SELECT_REGISTER_END, "bank tap end")
    assert_equal(bank_installed.name, "defender_bank_select_trace", "bank tap name")

    bank_installed.callback(0xd123, 0x17, 0xff)
    assert_equal(current_bank, 0x07, "bank select uses low nibble")
    bank_installed.callback(0xdfff, 0x00, 0xff)
    assert_equal(current_bank, 0x00, "bank select records I/O bank")
    assert_equal(
        take_bank_select_writes(bank_writes),
        "0xD123=0x17->0x07,0xDFFF=0x00->0x00",
        "bank select writes are formatted and drained"
    )
    assert_equal(format_bank_select_writes(bank_writes), "-", "drained bank select list is empty")

    local input_ports = input_ports_from_reader(function(port_name)
        return ({ IN0 = 0x120, IN1 = 0x101, IN2 = 0x210 })[port_name]
    end)
    assert_equal(input_ports.IN0, 0x20, "debug IN0 read is byte-sized")
    assert_equal(input_ports.IN1, 0x01, "debug IN1 read is byte-sized")
    assert_equal(input_ports.IN2, 0x10, "debug IN2 read is byte-sized")
    print("mame_defender_trace.lua self-test ok")
end

if os.getenv("DEFENDER_TRACE_SELF_TEST") == "1" then
    run_self_test()
    os.exit(0)
end

if not inputs_path or not output_path or not schema_path or frame_limit <= 0 then
    error("DEFENDER_TRACE_INPUTS, DEFENDER_TRACE_OUTPUT, DEFENDER_TRACE_SCHEMA, and DEFENDER_TRACE_FRAMES are required")
end

local schema = trim(read_all(schema_path))
local input_frames = split(trim(read_all(inputs_path)), ";")
if #input_frames ~= frame_limit then
    error(string.format("input script has %d frame(s), expected %d", #input_frames, frame_limit))
end

local machine = manager.machine
local maincpu = assert(machine.devices[":maincpu"])
local program = assert(maincpu.spaces["program"])
local sound_commands = {}
local bank_select_writes = {}
local current_bank_select = 0
local sound_command_tap = install_sound_command_tap(program, sound_commands)
local bank_select_tap = install_bank_select_tap(program, bank_select_writes, function(selected_bank)
    current_bank_select = selected_bank
end)
local output = assert(io.open(output_path, "w"))
output:write(schema, "\n")
local debug_output = nil
if debug_path then
    -- Local-only diagnostic stream for mapping fixture drift to ROM execution.
    -- It is intentionally separate from the checked fixture schema.
    debug_output = assert(io.open(debug_path, "w"))
    debug_output:write(
        "frame\tpc\tinput_read_in0\tinput_read_in1\tinput_read_in2\tbank_select\tbank_writes\tstatus\tp1_lives\tp1_wave\tp1_bombs\tseed\thseed\tlseed\tobject_table_crc32\tshell_table_crc32\n"
    )
end

local input_masks = {
    fire = { port = "IN0", mask = 0x01, trace_bit = 0x0080 },
    thrust = { port = "IN0", mask = 0x02, trace_bit = 0x0040 },
    smart_bomb = { port = "IN0", mask = 0x04, trace_bit = 0x0100 },
    smartbomb = { port = "IN0", mask = 0x04, trace_bit = 0x0100 },
    hyperspace = { port = "IN0", mask = 0x08, trace_bit = 0x0200 },
    start_two = { port = "IN0", mask = 0x10, trace_bit = 0x0004 },
    start2 = { port = "IN0", mask = 0x10, trace_bit = 0x0004 },
    start_one = { port = "IN0", mask = 0x20, trace_bit = 0x0002 },
    start1 = { port = "IN0", mask = 0x20, trace_bit = 0x0002 },
    reverse = { port = "IN0", mask = 0x40, trace_bit = 0x0020 },
    altitude_down = { port = "IN0", mask = 0x80, trace_bit = 0x0010 },
    down = { port = "IN0", mask = 0x80, trace_bit = 0x0010 },
    altitude_up = { port = "IN1", mask = 0x01, trace_bit = 0x0008 },
    up = { port = "IN1", mask = 0x01, trace_bit = 0x0008 },
    auto_up_manual_down = { port = "IN2", mask = 0x01, trace_bit = 0x1000 },
    service_advance = { port = "IN2", mask = 0x02, trace_bit = 0x2000 },
    advance = { port = "IN2", mask = 0x02, trace_bit = 0x2000 },
    coin_three = { port = "IN2", mask = 0x04, trace_bit = 0x0800 },
    coin3 = { port = "IN2", mask = 0x04, trace_bit = 0x0800 },
    high_score_reset = { port = "IN2", mask = 0x08, trace_bit = 0x4000 },
    coin = { port = "IN2", mask = 0x10, trace_bit = 0x0001 },
    coin_one = { port = "IN2", mask = 0x10, trace_bit = 0x0001 },
    coin1 = { port = "IN2", mask = 0x10, trace_bit = 0x0001 },
    coin_two = { port = "IN2", mask = 0x20, trace_bit = 0x0400 },
    coin2 = { port = "IN2", mask = 0x20, trace_bit = 0x0400 },
    tilt = { port = "IN2", mask = 0x40, trace_bit = 0x8000 },
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

local function read_input_port(port_name)
    local port = assert(ioport(port_name), "missing MAME input port " .. port_name)
    return port:read() & 0xff
end

local function read_input_ports()
    return input_ports_from_reader(read_input_port)
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
        bits = bits | binding.trace_bit
    end
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

for frame_number, frame_text in ipairs(input_frames) do
    local input_bits, input_ports = apply_inputs(frame_text)
    emu.wait_next_frame()
    local frame_sound_commands = copy_commands(sound_commands)
    local sound_command_text = take_commands(sound_commands)
    local event_text = format_events(events_for_commands(frame_sound_commands))

    output:write(string.format(
        "%d\t0x%04X\t0x%02X\t0x%02X\t0x%02X\t%s\t%d\t%d\t%d\t%d\t%d\t0x%02X\t0x%02X\t0x%02X\t0x%08X\t0x%08X\t-\t%s\t%s\n",
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
        sound_command_text,
        event_text
    ))
    if debug_output then
        local input_read_ports = read_input_ports()
        debug_output:write(string.format(
            "%d\t0x%04X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t%s\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%08X\t0x%08X\n",
            frame_number,
            maincpu.state["PC"].value,
            input_read_ports.IN0,
            input_read_ports.IN1,
            input_read_ports.IN2,
            current_bank_select,
            take_bank_select_writes(bank_select_writes),
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
sound_command_tap:remove()
bank_select_tap:remove()
output:close()
if debug_output then
    debug_output:close()
end
machine:exit()
