//! Player, projectile, and object process-family helper primitives.

use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlayerVerticalAction {
    Idle,
    Up,
    Down,
}

pub(super) fn player_vertical_action(pia21: u8, pia31: u8) -> PlayerVerticalAction {
    if pia31 & 0x01 != 0 {
        PlayerVerticalAction::Up
    } else if pia21 & 0x80 != 0 {
        PlayerVerticalAction::Down
    } else {
        PlayerVerticalAction::Idle
    }
}

pub(super) fn player_scroll_adjusted_x(previous_x16: u16, calculated_x: u16) -> (u16, u16) {
    let delta = calculated_x.wrapping_sub(previous_x16);
    if delta == 0 {
        return (calculated_x, 0);
    }

    if calculated_x >= previous_x16 {
        if delta <= 0x0100 {
            (calculated_x, 0)
        } else {
            (previous_x16.wrapping_add(0x0100), 0x0040)
        }
    } else if signed_word_greater_than(delta, 0xFF00) {
        (calculated_x, 0)
    } else {
        (previous_x16.wrapping_sub(0x0100), 0xFFC0)
    }
}

pub(super) fn clamp_player_x_velocity_high_word(value: u16) -> u16 {
    if signed_word_greater_or_equal(value, 0x0100) {
        0x0100
    } else if signed_word_less_or_equal(value, 0xFF00) {
        0xFF00
    } else {
        value
    }
}

pub(super) fn player_absolute_x(player_x16: u16, background_left: u16) -> u16 {
    let mut shifted = player_x16 >> 2;
    shifted &= 0xFFE0;
    shifted.wrapping_add(background_left)
}

pub(super) fn object_display_y_in_band(y: u8, upper_bound: u8, lower_bound: u8) -> bool {
    y <= upper_bound && y > lower_bound
}

pub(super) fn active_object_next_y(previous_y16: u16, y_velocity: u16) -> u16 {
    let [mut y, fraction] = previous_y16.wrapping_add(y_velocity).to_be_bytes();
    if y < RED_LABEL_Y_MIN {
        y = RED_LABEL_Y_MAX;
    } else if y > RED_LABEL_Y_MAX {
        y = RED_LABEL_Y_MIN;
    }
    u16::from_be_bytes([y, fraction])
}

pub(super) fn tie_onscreen_y_velocity_delta(object_y: u8, player_y: u8) -> Option<u16> {
    let delta = object_y.wrapping_sub(player_y);
    let signed_delta = delta as i8;
    if signed_delta >= 0 {
        if delta >= 0x20 {
            Some(0xFFF0)
        } else if delta > 0x10 {
            None
        } else {
            Some(0x0010)
        }
    } else if signed_delta <= -32 {
        Some(0x0010)
    } else if signed_delta < -16 {
        None
    } else {
        Some(0xFFF0)
    }
}

pub(super) fn signed_word_greater_than(left: u16, right: u16) -> bool {
    (left as i16) > (right as i16)
}

pub(super) fn signed_word_greater_or_equal(left: u16, right: u16) -> bool {
    (left as i16) >= (right as i16)
}

pub(super) fn signed_word_less_or_equal(left: u16, right: u16) -> bool {
    (left as i16) <= (right as i16)
}

pub(super) fn sign_extend_24_to_i32(bytes: [u8; 3]) -> i32 {
    let sign = if bytes[0] & 0x80 == 0 { 0x00 } else { 0xFF };
    i32::from_be_bytes([sign, bytes[0], bytes[1], bytes[2]])
}

pub(super) fn sign_extend_u8_to_u16(value: u8) -> u16 {
    let sign = if value & 0x80 == 0 { 0x00 } else { 0xFF };
    u16::from_be_bytes([sign, value])
}

pub(super) fn mini_swarmer_damping_adjustment(value: u16) -> u16 {
    let [mut a, mut b] = value.to_be_bytes();
    a = !a;
    b = !b;
    for _ in 0..2 {
        let carry = b & 0x80 != 0;
        b = b.wrapping_shl(1);
        a = a.wrapping_shl(1) | u8::from(carry);
    }
    sign_extend_u8_to_u16(a)
}

pub(super) fn arithmetic_shift_right_word(mut value: u16, shifts: u8) -> u16 {
    for _ in 0..shifts {
        let [mut a, mut b] = value.to_be_bytes();
        let carry = a & 0x01 != 0;
        a = (a >> 1) | (a & 0x80);
        b = (b >> 1) | if carry { 0x80 } else { 0x00 };
        value = u16::from_be_bytes([a, b]);
    }
    value
}
