//! Video, laser, star, and terrain helper primitives.

use super::*;

pub(super) fn step_laser_address(direction: RedLabelLaserDirection, address: u16) -> u16 {
    match direction {
        RedLabelLaserDirection::Right => address.wrapping_add(0x0100),
        RedLabelLaserDirection::Left => address.wrapping_sub(0x0100),
    }
}

pub(super) fn laser_reached_screen_edge(direction: RedLabelLaserDirection, address: u16) -> bool {
    match direction {
        RedLabelLaserDirection::Right => address >= 0x9800,
        RedLabelLaserDirection::Left => address <= 0x0500,
    }
}

pub(super) fn laser_fizzle_byte(rand_value: u8) -> u8 {
    (rand_value & 0x01) | ((rand_value & 0x02) << 3)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RedLabelStar {
    pub(super) x: u8,
    pub(super) y: u8,
    pub(super) color: u8,
}

pub(super) fn star_table_from_rand_values<I>(
    rand_values: I,
) -> Result<(Vec<RedLabelStar>, usize), String>
where
    I: IntoIterator<Item = u8>,
{
    let mut rand_values = rand_values.into_iter();
    let mut stars = Vec::with_capacity(16);
    let mut consumed = 0usize;
    let mut color = 0u8;

    for star_index in 0..16 {
        let x = loop {
            let value = next_stinit_rand(&mut rand_values, &mut consumed, star_index, "x")?;
            if value < 0x9C {
                break value;
            }
        };
        let y = loop {
            let value = next_stinit_rand(&mut rand_values, &mut consumed, star_index, "y")?;
            if value <= 0xA8 && value > RED_LABEL_Y_MIN {
                break value;
            }
        };

        stars.push(RedLabelStar { x, y, color });
        color = color.wrapping_add(0x11) & 0x77;
    }

    Ok((stars, consumed))
}

pub(super) fn next_stinit_rand<I>(
    rand_values: &mut I,
    consumed: &mut usize,
    star_index: usize,
    coordinate: &str,
) -> Result<u8, String>
where
    I: Iterator<Item = u8>,
{
    let value = rand_values.next().ok_or_else(|| {
        format!(
            "red-label STINIT exhausted RAND byte source while selecting star {star_index} {coordinate}"
        )
    })?;
    *consumed += 1;
    Ok(value)
}

pub(super) fn star_output_movement(background_left: u16, previous_background_left: u16) -> u8 {
    let background_phase = background_left & 0xFF80;
    let previous_background_phase = previous_background_left & 0xFF80;
    previous_background_phase
        .wrapping_sub(background_phase)
        .wrapping_shl(1)
        .to_be_bytes()[0]
}

pub(super) fn star_output_next_x(x: u8, movement: u8) -> u8 {
    let moved = x.wrapping_add(movement);
    if moved < 0x9C {
        moved
    } else if moved <= 0xC0 {
        0
    } else {
        0x9B
    }
}

pub(super) fn star_hyperspace_y(lseed: u8) -> u8 {
    (lseed & 0x3F).wrapping_mul(3).wrapping_add(RED_LABEL_Y_MIN)
}

pub(super) fn fireball_table_byte(rand_value: u8) -> u8 {
    let low_nibble = if rand_value & 0x80 == 0 { 0x09 } else { 0x0A };
    if rand_value & 0x01 == 0 {
        low_nibble + 0xA0
    } else {
        low_nibble + 0x90
    }
}

pub(super) fn terrain_altitude_step(offset: u8, terrain_byte: u8) -> u8 {
    if terrain_byte & 0x80 != 0 {
        offset.wrapping_sub(1)
    } else {
        offset.wrapping_add(1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TerrainBitState {
    pub(super) data_index: usize,
    pub(super) data_pointer: u16,
    pub(super) data_byte: u8,
    pub(super) bit_counter: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct TerrainTableGenerationState {
    pub(super) left: TerrainBitState,
    pub(super) right: TerrainBitState,
    pub(super) left_offset: u8,
    pub(super) right_offset: u8,
    pub(super) background_left: u16,
    pub(super) terrain_left: u16,
    pub(super) flavor_0_pointer: u16,
    pub(super) flavor_1_pointer: u16,
}

pub(super) fn advance_terrain_right_data(
    data: &[u8],
    base_address: u16,
    data_index: &mut usize,
    data_pointer: &mut u16,
    data_byte: &mut u8,
    bit_counter: &mut u8,
) {
    if *bit_counter == 0 {
        *data_index = (*data_index + 1) % data.len();
        *data_pointer = base_address.wrapping_add(*data_index as u16);
        *bit_counter = 7;
        *data_byte = data[*data_index];
    } else {
        *bit_counter -= 1;
        let carry = u8::from(*data_byte & 0x80 != 0);
        *data_byte = (*data_byte).wrapping_shl(1).wrapping_add(carry);
    }
}

pub(super) fn terrain_data_index_for_pointer(
    pointer: u16,
    base_address: u16,
    len: usize,
) -> Result<usize, String> {
    let offset = pointer.wrapping_sub(base_address);
    if usize::from(offset) < len {
        Ok(usize::from(offset))
    } else {
        Err(format!(
            "red-label terrain pointer 0x{pointer:04X} is outside TDATA range 0x{base_address:04X}..0x{:04X}",
            base_address.wrapping_add(len as u16)
        ))
    }
}

pub(super) fn validate_terrain_bit_counter(counter: u8, label: &str) -> Result<(), String> {
    if counter <= 7 {
        Ok(())
    } else {
        Err(format!(
            "red-label terrain bit counter {label}={counter} is outside 0..=7"
        ))
    }
}

pub(super) fn validate_terrain_flavor_pointer(
    pointer: u16,
    table_start: u16,
    label: &str,
) -> Result<(), String> {
    let offset = pointer.wrapping_sub(table_start);
    if offset < RED_LABEL_TERRAIN_FLAVOR_HALF_BYTES && offset.is_multiple_of(3) {
        Ok(())
    } else {
        Err(format!(
            "red-label {label} 0x{pointer:04X} is outside aligned terrain flavor half 0x{table_start:04X}..0x{:04X}",
            table_start.wrapping_add(RED_LABEL_TERRAIN_FLAVOR_HALF_BYTES)
        ))
    }
}

pub(super) fn rotate_terrain_right_byte(data_byte: u8) -> u8 {
    (data_byte >> 1).wrapping_add(if data_byte & 1 == 0 { 0 } else { 0x80 })
}

pub(super) fn advance_terrain_right_state(
    state: &mut TerrainBitState,
    data: &[u8],
    base_address: u16,
) {
    advance_terrain_right_data(
        data,
        base_address,
        &mut state.data_index,
        &mut state.data_pointer,
        &mut state.data_byte,
        &mut state.bit_counter,
    );
}

pub(super) fn advance_terrain_left_state(
    state: &mut TerrainBitState,
    data: &[u8],
    base_address: u16,
) {
    if state.bit_counter == 7 {
        state.data_index = if state.data_index == 0 {
            data.len() - 1
        } else {
            state.data_index - 1
        };
        state.data_pointer = base_address.wrapping_add(state.data_index as u16);
        state.bit_counter = 0;
        state.data_byte = rotate_terrain_right_byte(data[state.data_index]);
    } else {
        state.bit_counter += 1;
        state.data_byte = rotate_terrain_right_byte(state.data_byte);
    }
}
