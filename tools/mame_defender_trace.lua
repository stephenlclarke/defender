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
local skip_video_crc = os.getenv("DEFENDER_TRACE_SKIP_VIDEO_CRC") == "1"
local state_steer = os.getenv("DEFENDER_TRACE_STEER") or ""
local state_steer_frame = tonumber(os.getenv("DEFENDER_TRACE_STEER_FRAME") or "0") or 0
local sound_dac_output_path = os.getenv("DEFENDER_TRACE_SOUND_DAC_OUTPUT")

local MAIN_BOARD_SOUND_COMMAND_HIGH_BITS = 0xc0
local SOUND_COMMAND_IDLE_BYTE = 0xff
local PIA1_PORT_B_DATA_REGISTER = 0xcc02
local PIA1_PORT_B_CONTROL_REGISTER = 0xcc03
local SOUND_CPU_PIA_PORT_A_DATA_REGISTER = 0x0400
local SOUND_CPU_PIA_PORT_A_CONTROL_REGISTER = 0x0401
local PIA_CONTROL_DATA_REGISTER_SELECT = 0x04
local DEFENDER_BANK_SELECT_REGISTER_START = 0xd000
local DEFENDER_BANK_SELECT_REGISTER_END = 0xdfff
local CREDIT_ADDED_SOUND_COMMAND = 0xe6
local ONE_PLAYER_START_SOUND_COMMAND = 0xf5
local WILLIAMS_VIDEO_PAIR_STRIDE = 256
local DEFENDER_VISIBLE_WIDTH = 292
local DEFENDER_VISIBLE_HEIGHT = 240
local DEFENDER_VISIBLE_X_START = 12
local DEFENDER_VISIBLE_Y_START = 7
local ACTIVE_OBJECT_LIST_HEAD = 0xa065
local ACTIVE_PROCESS_LIST_HEAD = 0xa05f
local FREE_PROCESS_LIST_HEAD = 0xa061
local FREE_OBJECT_LIST_HEAD = 0xa067
local SHELL_OBJECT_LIST_HEAD = 0xa06d
local OBJECT_TABLE_START = 0xa23c
local OBJECT_TABLE_END = 0xaa0d
local OBJECT_TABLE_STRIDE = 0x17
local PROCESS_TABLE_START = 0xaac5
local PROCESS_TABLE_END = 0xaf2a
local PROCESS_TABLE_STRIDE = 0x0f
local APPEARANCE_RAM_START = 0x9c00
local APPEARANCE_RAM_STRIDE = 0x40
local APPEARANCE_RAM_ENTRIES = 16
local TERRAIN_ALTITUDE_TABLE_START = 0xb300
local BASE_PAGE_BACKGROUND_LEFT = 0xa020
local BASE_PAGE_PREVIOUS_BACKGROUND_LEFT = 0xa022
local BASE_PAGE_LAST_EXPANDED_OBJECT_SLOT = 0xa0e2
local BASE_PAGE_PCRAM = 0xa026
local BASE_PAGE_OVCNT = 0xa05e
local BASE_PAGE_REVFLG = 0xa0af
local BASE_PAGE_LFLG = 0xa0b5
local BASE_PAGE_LCOLRX = 0xa0b6
local BASE_PAGE_PWRFLG = 0xa0b7
local BASE_PAGE_PLADIR = 0xa0bd
local BASE_PAGE_PLAXC = 0xa0bf
local BASE_PAGE_PLAYC = 0xa0c0
local BASE_PAGE_NPLAXC = 0xa0c1
local BASE_PAGE_NPLAYC = 0xa0c2
local BASE_PAGE_PLAX16 = 0xa0c3
local BASE_PAGE_PLAY16 = 0xa0c5
local BASE_PAGE_PLAXV = 0xa0c7
local BASE_PAGE_PLAYV = 0xa0ca
local BASE_PAGE_PLABX = 0xa0cc
local BASE_PAGE_PCFLG = 0xa0de
local BASE_PAGE_ASTCNT = 0xa0fa
local SYSTEM_PROCESS_TYPE = 0x00
local ROUTINE_TERBLO = 0xedea
local ROUTINE_ASTKIL = 0xed70
local ROUTINE_AKIL1 = 0xed91
local ROUTINE_AFALL = 0xf216
local PICTURE_ASTP1 = 0xf901
local PICTURE_SCZP1 = 0xf8ce
local PICTURE_SWXP1 = 0xf8e2
local PICTURE_NULOB = 0xf8ec
local PICTURE_PRBP1 = 0xf8f7
local PICTURE_TIEP3 = 0xf93d
local PICTURE_LNDP3 = 0xf999
local PICTURE_SWPIC1 = 0xf97b
local PICTURE_UFOP3 = 0xf9b7
local HUMAN_SCANNER_COLOR = 0x1111

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

local function williams_screen_byte_offset(screen_x, screen_y)
    return screen_y + ((screen_x >> 1) * WILLIAMS_VIDEO_PAIR_STRIDE)
end

local function visible_pixel_nibbles_from_reader(read_byte)
    local nibbles = {}
    for visible_y = 0, DEFENDER_VISIBLE_HEIGHT - 1 do
        local screen_y = DEFENDER_VISIBLE_Y_START + visible_y
        for visible_x = 0, DEFENDER_VISIBLE_WIDTH - 1 do
            local screen_x = DEFENDER_VISIBLE_X_START + visible_x
            local byte = read_byte(williams_screen_byte_offset(screen_x, screen_y)) & 0xff
            if (screen_x & 1) == 0 then
                nibbles[#nibbles + 1] = (byte >> 4) & 0x0f
            else
                nibbles[#nibbles + 1] = byte & 0x0f
            end
        end
    end
    return nibbles
end

local function visible_video_crc32_from_reader(read_byte)
    return crc32(visible_pixel_nibbles_from_reader(read_byte))
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

local function append_sound_dac_write(writes, data)
    writes[#writes + 1] = data & 0xff
end

local function copy_bytes(bytes)
    local copied = {}
    for index, byte in ipairs(bytes) do
        copied[index] = byte
    end
    return copied
end

local function clear_array(values)
    for index = #values, 1, -1 do
        values[index] = nil
    end
end

local function format_hex_byte_list(bytes, edge_count)
    if #bytes == 0 then
        return "-"
    end

    local values = {}
    if #bytes <= edge_count * 2 then
        for index, byte in ipairs(bytes) do
            values[#values + 1] = string.format("0x%02X", byte)
        end
        return table.concat(values, ",")
    end

    for index = 1, edge_count do
        values[#values + 1] = string.format("0x%02X", bytes[index])
    end
    values[#values + 1] = "..."
    for index = #bytes - edge_count + 1, #bytes do
        values[#values + 1] = string.format("0x%02X", bytes[index])
    end
    return table.concat(values, ",")
end

local function sound_dac_frame_row(frame_number, writes)
    if #writes == 0 then
        return string.format("%d\t0\t-\t-\t-\t-", frame_number)
    end

    return string.format(
        "%d\t%d\t0x%02X\t0x%02X\t0x%08X\t%s",
        frame_number,
        #writes,
        writes[1],
        writes[#writes],
        crc32(writes),
        format_hex_byte_list(writes, 32)
    )
end

local function install_sound_dac_tap(sound_program, writes)
    local port_a_data_selected = false
    return sound_program:install_write_tap(
        SOUND_CPU_PIA_PORT_A_DATA_REGISTER,
        SOUND_CPU_PIA_PORT_A_CONTROL_REGISTER,
        "defender_sound_dac_trace",
        function(offset, data, mask)
            if offset == SOUND_CPU_PIA_PORT_A_CONTROL_REGISTER then
                port_a_data_selected = (data & PIA_CONTROL_DATA_REGISTER_SELECT) ~= 0
            elseif offset == SOUND_CPU_PIA_PORT_A_DATA_REGISTER and port_a_data_selected then
                append_sound_dac_write(writes, data)
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

    local installed_dac = {}
    local sound_program = {
        install_write_tap = function(_, start_address, end_address, name, callback)
            installed_dac.start_address = start_address
            installed_dac.end_address = end_address
            installed_dac.name = name
            installed_dac.callback = callback
            return { remove = function() end }
        end,
    }
    local dac_writes = {}
    install_sound_dac_tap(sound_program, dac_writes)
    assert_equal(installed_dac.start_address, SOUND_CPU_PIA_PORT_A_DATA_REGISTER, "DAC tap start")
    assert_equal(installed_dac.end_address, SOUND_CPU_PIA_PORT_A_CONTROL_REGISTER, "DAC tap end")
    assert_equal(installed_dac.name, "defender_sound_dac_trace", "DAC tap name")
    installed_dac.callback(SOUND_CPU_PIA_PORT_A_DATA_REGISTER, 0x12, 0xff)
    assert_equal(#dac_writes, 0, "DAC DDR write is ignored until data register is selected")
    installed_dac.callback(SOUND_CPU_PIA_PORT_A_CONTROL_REGISTER, 0x04, 0xff)
    installed_dac.callback(SOUND_CPU_PIA_PORT_A_DATA_REGISTER, 0xa5, 0xff)
    installed_dac.callback(SOUND_CPU_PIA_PORT_A_DATA_REGISTER, 0x5a, 0xff)
    assert_equal(sound_dac_frame_row(42, dac_writes), "42\t2\t0xA5\t0x5A\t0x1917E2B9\t0xA5,0x5A", "DAC frame row")
    clear_array(dac_writes)
    assert_equal(sound_dac_frame_row(43, dac_writes), "43\t0\t-\t-\t-\t-", "empty DAC frame row")
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

    local video_ram = {}
    video_ram[williams_screen_byte_offset(12, 7)] = 0xab
    video_ram[williams_screen_byte_offset(14, 7)] = 0xcd
    video_ram[williams_screen_byte_offset(303, 246)] = 0xef
    local function read_video(address)
        return video_ram[address] or 0
    end
    local visible_nibbles = visible_pixel_nibbles_from_reader(read_video)
    assert_equal(#visible_nibbles, DEFENDER_VISIBLE_WIDTH * DEFENDER_VISIBLE_HEIGHT, "visible pixel count")
    assert_equal(visible_nibbles[1], 0x0a, "top-left high nibble")
    assert_equal(visible_nibbles[2], 0x0b, "top-left low nibble")
    assert_equal(visible_nibbles[3], 0x0c, "next visible pair high nibble")
    assert_equal(visible_nibbles[4], 0x0d, "next visible pair low nibble")
    assert_equal(visible_nibbles[#visible_nibbles], 0x0f, "bottom-right low nibble")
    assert_equal(
        visible_video_crc32_from_reader(read_video),
        crc32(visible_nibbles),
        "visible video CRC uses decoded pixel nibbles"
    )
    print("mame_defender_trace.lua self-test ok")
end

if os.getenv("DEFENDER_TRACE_SELF_TEST") == "1" then
    run_self_test()
    os.exit(0)
end

local valid_state_steers = {
    afall_fall = true,
    afall_player_catch = true,
    afall_safe_landing = true,
    enemy_explosion_matrix = true,
    enemy_materialize_matrix = true,
    sound_baiter_hit = true,
    sound_bomber_hit = true,
    sound_command_matrix = true,
    sound_command_f3 = true,
    sound_command_f8 = true,
    sound_command_fa = true,
    sound_command_fe = true,
    sound_pod_hit = true,
    sound_swarmer_hit = true,
    sound_swarmer_shot = true,
    sound_tie_hit = true,
    terrain_blow = true,
}

if not inputs_path or not output_path or not schema_path or frame_limit <= 0 then
    error("DEFENDER_TRACE_INPUTS, DEFENDER_TRACE_OUTPUT, DEFENDER_TRACE_SCHEMA, and DEFENDER_TRACE_FRAMES are required")
end
if state_steer ~= "" then
    if not valid_state_steers[state_steer] then
        error("unknown DEFENDER_TRACE_STEER " .. state_steer)
    end
    if state_steer_frame <= 0 then
        error("DEFENDER_TRACE_STEER_FRAME must be positive when DEFENDER_TRACE_STEER is set")
    end
end

local schema = trim(read_all(schema_path))
local input_frames = split(trim(read_all(inputs_path)), ";")
if #input_frames ~= frame_limit then
    error(string.format("input script has %d frame(s), expected %d", #input_frames, frame_limit))
end

local machine = manager.machine
local maincpu = assert(machine.devices[":maincpu"])
local program = assert(maincpu.spaces["program"])
local soundcpu = assert(machine.devices[":soundcpu"])
local sound_program = assert(soundcpu.spaces["program"])
local sound_commands = {}
local sound_dac_writes = {}
local bank_select_writes = {}
local current_bank_select = 0
local sound_command_tap = install_sound_command_tap(program, sound_commands)
local sound_dac_tap = install_sound_dac_tap(sound_program, sound_dac_writes)
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
        "frame\tpc\tinput_read_in0\tinput_read_in1\tinput_read_in2\tbank_select\tbank_writes\tstatus\tp1_lives\tp1_wave\tp1_bombs\tseed\thseed\tlseed\tpladir\trevflg\tlflg\tlcolrx\tpwrflg\tplaxc\tplayc\tnplaxc\tnplayc\tplax16\tplay16\tplaxv\tplayv\tplabx\tastcnt\tterrain_blown\tpcram0\tovcnt\tlndres\twavtim\twavsiz\tldstim\tlndcnt\twavtmr\tobject_table_crc32\tprocess_table_crc32\tsuper_process_table_crc32\tshell_table_crc32\tactive_objects\tactive_processes\tobject_slots\texpanded_objects\tshell_objects\n"
    )
end
local sound_dac_output = nil
if sound_dac_output_path then
    sound_dac_output = assert(io.open(sound_dac_output_path, "w"))
    sound_dac_output:write("frame\tcount\tfirst\tlast\tcrc32\tvalues\n")
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

local function crc_range(start_address, length)
    local bytes = {}
    for index = 0, length - 1 do
        bytes[#bytes + 1] = read_u8(start_address + index)
    end
    return crc32(bytes)
end

local function read_u16(address)
    return ((read_u8(address) << 8) | read_u8(address + 1)) & 0xffff
end

local function write_u8(address, value)
    program:write_u8(address, value & 0xff)
end

local function write_u16(address, value)
    write_u8(address, (value >> 8) & 0xff)
    write_u8(address + 1, value & 0xff)
end

local function read_u24(address)
    return ((read_u8(address) << 16) | (read_u8(address + 1) << 8) | read_u8(address + 2)) & 0xffffff
end

local function is_object_cell_address(address)
    if address < OBJECT_TABLE_START or address >= OBJECT_TABLE_END then
        return false
    end
    return ((address - OBJECT_TABLE_START) % OBJECT_TABLE_STRIDE) == 0
end

local function is_process_cell_address(address)
    return address >= PROCESS_TABLE_START
        and address < PROCESS_TABLE_END
        and ((address - PROCESS_TABLE_START) % PROCESS_TABLE_STRIDE) == 0
end

local function pop_list_cell(list_head_address, label)
    local cell = read_u16(list_head_address)
    if cell == 0 then
        error(label .. " free list is empty")
    end
    local next_cell = read_u16(cell)
    write_u16(list_head_address, next_cell)
    return cell
end

local function insert_active_process(process_address)
    local current_process = read_u16(ACTIVE_PROCESS_LIST_HEAD)
    if current_process ~= 0 and is_process_cell_address(current_process) then
        local old_next = read_u16(current_process)
        write_u16(current_process, process_address)
        write_u16(process_address, old_next)
    else
        write_u16(process_address, current_process)
        write_u16(ACTIVE_PROCESS_LIST_HEAD, process_address)
    end
end

local function allocate_process(routine_address, process_type)
    local process_address = pop_list_cell(FREE_PROCESS_LIST_HEAD, "process")
    if not is_process_cell_address(process_address) then
        error(string.format("free process list returned invalid cell 0x%04X", process_address))
    end

    for offset = 0, PROCESS_TABLE_STRIDE - 1 do
        write_u8(process_address + offset, 0)
    end
    write_u16(process_address + 0x02, routine_address)
    write_u8(process_address + 0x04, 1)
    write_u8(process_address + 0x05, process_type)
    write_u8(process_address + 0x06, 0)
    insert_active_process(process_address)
    return process_address
end

local function allocate_object(process_address, picture_address, collision_vector_address, scanner_color)
    local object_address = pop_list_cell(FREE_OBJECT_LIST_HEAD, "object")
    if not is_object_cell_address(object_address) then
        error(string.format("free object list returned invalid cell 0x%04X", object_address))
    end

    for offset = 0, OBJECT_TABLE_STRIDE - 1 do
        write_u8(object_address + offset, 0)
    end
    local old_active = read_u16(ACTIVE_OBJECT_LIST_HEAD)
    write_u16(object_address + 0x00, old_active)
    write_u16(object_address + 0x02, picture_address)
    write_u16(object_address + 0x06, process_address)
    write_u16(object_address + 0x08, collision_vector_address)
    write_u16(object_address + 0x12, scanner_color)
    write_u16(ACTIVE_OBJECT_LIST_HEAD, object_address)
    return object_address
end

local function allocate_hidden_appearance_object(picture_address, top_left_x, top_left_y)
    local object_address = pop_list_cell(FREE_OBJECT_LIST_HEAD, "object")
    if not is_object_cell_address(object_address) then
        error(string.format("free object list returned invalid cell 0x%04X", object_address))
    end

    for offset = 0, OBJECT_TABLE_STRIDE - 1 do
        write_u8(object_address + offset, 0)
    end
    local old_active = read_u16(ACTIVE_OBJECT_LIST_HEAD)
    write_u16(object_address + 0x00, old_active)
    write_u16(object_address + 0x02, PICTURE_NULOB)
    write_u16(object_address + 0x0a, (top_left_x & 0xff) << 6)
    write_u16(object_address + 0x0c, (top_left_y & 0xff) << 8)
    write_u8(object_address + 0x14, 0x02)
    write_u16(ACTIVE_OBJECT_LIST_HEAD, object_address)
    return object_address
end

local function set_falling_astronaut_common(x16, y16, y_velocity, terrain_altitude)
    local process_address = allocate_process(ROUTINE_AFALL, SYSTEM_PROCESS_TYPE)
    local object_address = allocate_object(
        process_address,
        PICTURE_ASTP1,
        ROUTINE_AKIL1,
        HUMAN_SCANNER_COLOR
    )
    write_u16(process_address + 0x07, object_address)
    write_u16(object_address + 0x0a, x16)
    write_u16(object_address + 0x0c, y16)
    write_u16(object_address + 0x10, y_velocity)
    write_u8(TERRAIN_ALTITUDE_TABLE_START + (x16 >> 6), terrain_altitude)
    return process_address, object_address
end

local function clear_appearance_ram()
    for slot = 0, APPEARANCE_RAM_ENTRIES - 1 do
        local address = APPEARANCE_RAM_START + slot * APPEARANCE_RAM_STRIDE
        for offset = 0, APPEARANCE_RAM_STRIDE - 1 do
            write_u8(address + offset, 0)
        end
    end
end

local function write_expanded_explosion_slot(slot, picture_address, top_left_x, top_left_y, width, height)
    local address = APPEARANCE_RAM_START + slot * APPEARANCE_RAM_STRIDE
    local center_x = top_left_x + (width >> 1)
    local center_y = top_left_y + (height >> 1)
    write_u16(address + 0x00, 0x0100)
    write_u16(address + 0x02, picture_address)
    write_u16(address + 0x04, address + APPEARANCE_RAM_STRIDE)
    write_u16(address + 0x06, ((center_x & 0xff) << 8) | (center_y & 0xff))
    write_u16(address + 0x08, ((top_left_x & 0xff) << 8) | (top_left_y & 0xff))
    write_u16(address + 0x0a, 0)
end

local function write_expanded_appearance_slot(slot, picture_address, top_left_x, top_left_y, width, height)
    local address = APPEARANCE_RAM_START + slot * APPEARANCE_RAM_STRIDE
    local center_x = top_left_x + (width >> 1)
    local center_y = top_left_y + (height >> 1)
    local object_address = allocate_hidden_appearance_object(picture_address, top_left_x, top_left_y)
    write_u16(address + 0x00, 0xad00)
    write_u16(address + 0x02, picture_address)
    write_u16(address + 0x04, address + APPEARANCE_RAM_STRIDE)
    write_u16(address + 0x06, ((center_x & 0xff) << 8) | (center_y & 0xff))
    write_u16(address + 0x08, ((top_left_x & 0xff) << 8) | (top_left_y & 0xff))
    write_u16(address + 0x0a, object_address)
end

local SOUND_COMMAND_MATRIX = {
    [0] = 0xfe,
    [240] = 0xfa,
    [480] = 0xf8,
    [720] = 0xf3,
}

local SINGLE_SOUND_COMMAND_STEERS = {
    sound_baiter_hit = 0xf8,
    sound_bomber_hit = 0xfe,
    sound_command_f3 = 0xf3,
    sound_command_f8 = 0xf8,
    sound_command_fa = 0xfa,
    sound_command_fe = 0xfe,
    sound_pod_hit = 0xfa,
    sound_swarmer_hit = 0xf8,
    sound_swarmer_shot = 0xf3,
    sound_tie_hit = 0xfe,
}

local function emit_sound_command(command)
    write_u8(PIA1_PORT_B_CONTROL_REGISTER, PIA_CONTROL_DATA_REGISTER_SELECT)
    write_u8(PIA1_PORT_B_DATA_REGISTER, SOUND_COMMAND_IDLE_BYTE & 0x3f)
    write_u8(PIA1_PORT_B_DATA_REGISTER, command & 0x3f)
end

local function emit_sound_command_matrix(relative_frame)
    local command = SOUND_COMMAND_MATRIX[relative_frame]
    if command then
        emit_sound_command(command)
    end
end

local function seed_enemy_explosion_matrix()
    clear_appearance_ram()
    write_u16(BASE_PAGE_LAST_EXPANDED_OBJECT_SLOT, APPEARANCE_RAM_START)
    write_expanded_explosion_slot(0, PICTURE_LNDP3, 0x20, 0x40, 5, 8)
    write_expanded_explosion_slot(1, PICTURE_SCZP1, 0x58, 0x40, 5, 8)
    write_expanded_explosion_slot(2, PICTURE_TIEP3, 0x90, 0x40, 4, 8)
    write_expanded_explosion_slot(3, PICTURE_PRBP1, 0xc8, 0x40, 4, 8)
    write_expanded_explosion_slot(4, PICTURE_UFOP3, 0x20, 0x80, 6, 4)
    write_expanded_explosion_slot(5, PICTURE_SWXP1, 0x58, 0x80, 4, 8)
end

local function seed_enemy_materialize_matrix()
    clear_appearance_ram()
    write_u16(BASE_PAGE_LAST_EXPANDED_OBJECT_SLOT, APPEARANCE_RAM_START)
    write_expanded_appearance_slot(0, PICTURE_LNDP3, 0x20, 0x40, 5, 8)
    write_expanded_appearance_slot(1, PICTURE_SCZP1, 0x58, 0x40, 5, 8)
    write_expanded_appearance_slot(2, PICTURE_TIEP3, 0x90, 0x40, 4, 8)
    write_expanded_appearance_slot(3, PICTURE_PRBP1, 0xc8, 0x40, 4, 8)
    write_expanded_appearance_slot(4, PICTURE_UFOP3, 0x20, 0x80, 6, 4)
    write_expanded_appearance_slot(5, PICTURE_SWPIC1, 0x58, 0x80, 3, 4)
end

local function apply_state_steer(mode)
    if mode == "afall_fall" then
        set_falling_astronaut_common(0x1000, 0x5000, 0x0010, 0x60)
    elseif mode == "afall_safe_landing" then
        local process_address, object_address =
            set_falling_astronaut_common(0x1000, 0x6000, 0x00d0, 0x50)
        write_u16(object_address + 0x08, ROUTINE_ASTKIL)
        write_u16(process_address + 0x07, object_address)
    elseif mode == "afall_player_catch" then
        local player_x = (read_u8(BASE_PAGE_PLAXC) << 6) & 0xffff
        local player_y = (read_u8(BASE_PAGE_PLAYC) << 8) & 0xffff
        local process_address, object_address =
            set_falling_astronaut_common(player_x, player_y, 0x0010, 0x90)
        write_u16(object_address + 0x08, ROUTINE_AKIL1)
        write_u16(object_address + 0x06, process_address)
        write_u8(BASE_PAGE_PCFLG, 1)
    elseif mode == "enemy_explosion_matrix" then
        seed_enemy_explosion_matrix()
    elseif mode == "enemy_materialize_matrix" then
        seed_enemy_materialize_matrix()
    elseif mode == "terrain_blow" then
        allocate_process(ROUTINE_TERBLO, SYSTEM_PROCESS_TYPE)
    end
end

local function format_active_objects()
    local values = {}
    local address = read_u16(ACTIVE_OBJECT_LIST_HEAD)
    local guard = 0
    while address ~= 0 and guard < 95 do
        if not is_object_cell_address(address) then
            values[#values + 1] = string.format("invalid:0x%04X", address)
            break
        end
        local process_address = read_u16(address + 0x06)
        local process_shot_timer = 0
        if is_process_cell_address(process_address) then
            process_shot_timer = read_u8(process_address + 0x0d)
        end
        values[#values + 1] = string.format(
            "0x%04X:pic=0x%04X:x=0x%02X:y=0x%02X:proc=0x%04X:pd6=0x%02X:x16=0x%04X:y16=0x%04X:xv=0x%04X:yv=0x%04X:typ=0x%02X:col=0x%04X",
            address,
            read_u16(address + 0x02),
            read_u8(address + 0x04),
            read_u8(address + 0x05),
            process_address,
            process_shot_timer,
            read_u16(address + 0x0a),
            read_u16(address + 0x0c),
            read_u16(address + 0x0e),
            read_u16(address + 0x10),
            read_u8(address + 0x14),
            read_u16(address + 0x12)
        )
        address = read_u16(address)
        guard = guard + 1
    end
    if address ~= 0 and guard >= 95 then
        values[#values + 1] = "truncated"
    end
    if #values == 0 then
        return "-"
    end
    return table.concat(values, ",")
end

local function format_active_processes()
    local values = {}
    local address = read_u16(ACTIVE_PROCESS_LIST_HEAD)
    local guard = 0
    while address ~= 0 and guard < 75 do
        if not is_process_cell_address(address) then
            values[#values + 1] = string.format("invalid:0x%04X", address)
            break
        end
        local object_address = read_u16(address + 0x07)
        local object_summary = "-"
        if is_object_cell_address(object_address) then
            object_summary = string.format(
                "obj=0x%04X:pic=0x%04X:x=0x%02X:y=0x%02X:x16=0x%04X:y16=0x%04X:xv=0x%04X:yv=0x%04X:typ=0x%02X",
                object_address,
                read_u16(object_address + 0x02),
                read_u8(object_address + 0x04),
                read_u8(object_address + 0x05),
                read_u16(object_address + 0x0a),
                read_u16(object_address + 0x0c),
                read_u16(object_address + 0x0e),
                read_u16(object_address + 0x10),
                read_u8(object_address + 0x14)
            )
        end
        values[#values + 1] = string.format(
            "0x%04X:paddr=0x%04X:ptime=0x%02X:ptype=0x%02X:pcod=0x%02X:pd=0x%04X:pd2=0x%04X:pd4=0x%04X:pd6=0x%02X:%s",
            address,
            read_u16(address + 0x02),
            read_u8(address + 0x04),
            read_u8(address + 0x05),
            read_u8(address + 0x06),
            read_u16(address + 0x07),
            read_u16(address + 0x09),
            read_u16(address + 0x0b),
            read_u8(address + 0x0d),
            object_summary
        )
        address = read_u16(address)
        guard = guard + 1
    end
    if address ~= 0 and guard >= 75 then
        values[#values + 1] = "truncated"
    end
    if #values == 0 then
        return "-"
    end
    return table.concat(values, ",")
end

local function format_object_slots()
    local values = {}
    for slot = 0, 15 do
        local address = OBJECT_TABLE_START + slot * OBJECT_TABLE_STRIDE
        values[#values + 1] = string.format(
            "%02d:0x%04X:link=0x%04X:pic=0x%04X:x=0x%02X:y=0x%02X:proc=0x%04X:x16=0x%04X:y16=0x%04X:xv=0x%04X:yv=0x%04X:typ=0x%02X",
            slot,
            address,
            read_u16(address),
            read_u16(address + 0x02),
            read_u8(address + 0x04),
            read_u8(address + 0x05),
            read_u16(address + 0x06),
            read_u16(address + 0x0a),
            read_u16(address + 0x0c),
            read_u16(address + 0x0e),
            read_u16(address + 0x10),
            read_u8(address + 0x14)
        )
    end
    return table.concat(values, ",")
end

local function format_shell_objects()
    local values = {}
    local address = read_u16(SHELL_OBJECT_LIST_HEAD)
    local guard = 0
    while address ~= 0 and guard < 95 do
        if not is_object_cell_address(address) then
            values[#values + 1] = string.format("invalid:0x%04X", address)
            break
        end
        values[#values + 1] = string.format(
            "0x%04X:link=0x%04X:pic=0x%04X:x=0x%02X:y=0x%02X:proc=0x%04X:x16=0x%04X:y16=0x%04X:xv=0x%04X:yv=0x%04X:typ=0x%02X:col=0x%04X",
            address,
            read_u16(address),
            read_u16(address + 0x02),
            read_u8(address + 0x04),
            read_u8(address + 0x05),
            read_u16(address + 0x06),
            read_u16(address + 0x0a),
            read_u16(address + 0x0c),
            read_u16(address + 0x0e),
            read_u16(address + 0x10),
            read_u8(address + 0x14),
            read_u16(address + 0x12)
        )
        address = read_u16(address)
        guard = guard + 1
    end
    if address ~= 0 and guard >= 95 then
        values[#values + 1] = "truncated"
    end
    if #values == 0 then
        return "-"
    end
    return table.concat(values, ",")
end

local function format_expanded_objects()
    local values = {
        string.format(
            "lsexpl=0x%04X:bgl=0x%04X:bglx=0x%04X",
            read_u16(BASE_PAGE_LAST_EXPANDED_OBJECT_SLOT),
            read_u16(BASE_PAGE_BACKGROUND_LEFT),
            read_u16(BASE_PAGE_PREVIOUS_BACKGROUND_LEFT)
        ),
    }
    for slot = 0, APPEARANCE_RAM_ENTRIES - 1 do
        local address = APPEARANCE_RAM_START + slot * APPEARANCE_RAM_STRIDE
        local size = read_u16(address)
        if size ~= 0 then
            values[#values + 1] = string.format(
                "0x%04X:size=0x%04X:pic=0x%04X:center=0x%04X:top=0x%04X:erase=0x%04X:obj=0x%04X",
                address,
                size,
                read_u16(address + 0x02),
                read_u16(address + 0x06),
                read_u16(address + 0x08),
                read_u16(address + 0x04),
                read_u16(address + 0x0a)
            )
        end
    end
    return table.concat(values, ",")
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
    if state_steer == "sound_command_matrix" then
        emit_sound_command_matrix(frame_number - state_steer_frame)
    elseif SINGLE_SOUND_COMMAND_STEERS[state_steer] and frame_number == state_steer_frame then
        emit_sound_command(SINGLE_SOUND_COMMAND_STEERS[state_steer])
    elseif state_steer ~= "" and frame_number == state_steer_frame then
        apply_state_steer(state_steer)
    end
    emu.wait_next_frame()
    local frame_sound_commands = copy_commands(sound_commands)
    local frame_sound_dac_writes = copy_bytes(sound_dac_writes)
    local sound_command_text = take_commands(sound_commands)
    clear_array(sound_dac_writes)
    if sound_dac_output then
        sound_dac_output:write(sound_dac_frame_row(frame_number, frame_sound_dac_writes), "\n")
    end
    local event_text = format_events(events_for_commands(frame_sound_commands))
    local visible_video_crc32 = 0
    if not skip_video_crc then
        visible_video_crc32 = visible_video_crc32_from_reader(read_u8)
    end

    output:write(string.format(
        "%d\t0x%04X\t0x%02X\t0x%02X\t0x%02X\t%s\t%d\t%d\t%d\t%d\t%d\t0x%02X\t0x%02X\t0x%02X\t0x%08X\t0x%08X\t0x%08X\t0x%08X\t0x%08X\t%s\t%s\n",
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
        crc_range(0xAAC5, 0x0F * 75),
        crc_range(0xAF2A, 0x17 * 5),
        crc_range(0xA06D, 2),
        visible_video_crc32,
        sound_command_text,
        event_text
    ))
    if debug_output then
        local input_read_ports = read_input_ports()
        local status = read_u8(0xA0BA)
        debug_output:write(string.format(
            "%d\t0x%04X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t%s\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%04X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%04X\t0x%04X\t0x%06X\t0x%04X\t0x%04X\t0x%02X\t%s\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%02X\t0x%08X\t0x%08X\t0x%08X\t0x%08X\t%s\t%s\t%s\t%s\t%s\n",
            frame_number,
            maincpu.state["PC"].value,
            input_read_ports.IN0,
            input_read_ports.IN1,
            input_read_ports.IN2,
            current_bank_select,
            take_bank_select_writes(bank_select_writes),
            status,
            read_u8(0xA1C9),
            read_u8(0xA1CA),
            read_u8(0xA1CB),
            read_u8(0xA0DF),
            read_u8(0xA0E0),
            read_u8(0xA0E1),
            read_u16(BASE_PAGE_PLADIR),
            read_u8(BASE_PAGE_REVFLG),
            read_u8(BASE_PAGE_LFLG),
            read_u8(BASE_PAGE_LCOLRX),
            read_u8(BASE_PAGE_PWRFLG),
            read_u8(BASE_PAGE_PLAXC),
            read_u8(BASE_PAGE_PLAYC),
            read_u8(BASE_PAGE_NPLAXC),
            read_u8(BASE_PAGE_NPLAYC),
            read_u16(BASE_PAGE_PLAX16),
            read_u16(BASE_PAGE_PLAY16),
            read_u24(BASE_PAGE_PLAXV),
            read_u16(BASE_PAGE_PLAYV),
            read_u16(BASE_PAGE_PLABX),
            read_u8(BASE_PAGE_ASTCNT),
            ((status & 0x02) ~= 0) and "true" or "false",
            read_u8(BASE_PAGE_PCRAM),
            read_u8(BASE_PAGE_OVCNT),
            read_u8(0xA0FB),
            read_u8(0xA100),
            read_u8(0xA101),
            read_u8(0xA105),
            read_u8(0xA112),
            read_u8(0xA118),
            crc_range(0xA23C, 0x17 * 95),
            crc_range(0xAAC5, 0x0F * 75),
            crc_range(0xAF2A, 0x17 * 5),
            crc_range(0xA06D, 2),
            format_active_objects(),
            format_active_processes(),
            format_object_slots(),
            format_expanded_objects(),
            format_shell_objects()
        ))
    end
end

clear_inputs()
sound_command_tap:remove()
sound_dac_tap:remove()
bank_select_tap:remove()
output:close()
if debug_output then
    debug_output:close()
end
if sound_dac_output then
    sound_dac_output:close()
end
machine:exit()
