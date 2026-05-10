//! Wave, world-state, and BCD helper primitives.

pub(super) fn getwv_restore_wave_hits(wave: u8, restore_wave: u8) -> bool {
    if restore_wave == 0 {
        return false;
    }

    let mut remainder = wave;
    loop {
        let (next, borrowed) = remainder.overflowing_sub(restore_wave);
        if borrowed {
            return false;
        }
        if next == 0 {
            return true;
        }
        remainder = next;
    }
}

pub(super) fn getwv_inter_wall_delta_iterations(
    wave: u8,
    difficulty_initial: u8,
    difficulty_ceiling: u8,
) -> u16 {
    let wave_delta = if wave >= 4 { wave.wrapping_sub(4) } else { 0 };
    let pre_ceiling = difficulty_initial.wrapping_add(wave_delta);
    if pre_ceiling == 0 {
        return 0;
    }

    let capped = if difficulty_ceiling >= pre_ceiling {
        pre_ceiling
    } else {
        difficulty_ceiling
    };
    if capped == 0 { 256 } else { u16::from(capped) }
}

pub(super) fn decimal_to_bcd_byte(value: u8) -> u8 {
    ((value / 10) << 4) | (value % 10)
}

pub(super) fn bcd_number_visible_digits(bcd_number: u8) -> Vec<u8> {
    let high = bcd_number >> 4;
    let low = bcd_number & 0x0F;
    if high == 0 {
        vec![low]
    } else {
        vec![high, low]
    }
}
