//! Narrow Motorola 6821 PIA data/control register model.
//!
//! This implements the register-selection and DDR/data behavior used by MAME's
//! `pia6821_device`: offsets 0 and 2 access DDR or data depending on control
//! bit 2, control writes mask off read-only bits 6 and 7, and output callbacks
//! fire on data writes or DDR changes. CA1/CB1 and input-mode CA2/CB2 edge IRQ
//! flags follow MAME, including control-register IRQ flag bits, data-port read
//! clearing, and CA2/CB2 output and strobe behavior.
//! Source: <https://github.com/mamedev/mame/blob/master/src/devices/machine/6821pia.cpp>.

pub const PIA_IRQ1: u8 = 0x80;
pub const PIA_IRQ2: u8 = 0x40;
pub const PIA_CONTROL_WRITABLE_MASK: u8 = 0x3F;
pub const PIA_CONTROL_IRQ1_ENABLE: u8 = 0x01;
pub const PIA_CONTROL_C1_LOW_TO_HIGH: u8 = 0x02;
pub const PIA_CONTROL_OUTPUT_SELECTED: u8 = 0x04;
pub const PIA_CONTROL_IRQ2_ENABLE: u8 = 0x08;
pub const PIA_CONTROL_C2_LOW_TO_HIGH: u8 = 0x10;
pub const PIA_CONTROL_C2_OUTPUT: u8 = 0x20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PiaOutputEvent {
    PortA { data: u8, ddr: u8 },
    PortB { data: u8, ddr: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pia6821 {
    out_a: u8,
    out_b: u8,
    out_ca2: bool,
    out_cb2: bool,
    ddr_a: u8,
    ddr_b: u8,
    control_a: u8,
    control_b: u8,
    a_input_overrides_output_mask: u8,
    in_ca1: bool,
    in_ca2: bool,
    in_cb1: bool,
    in_cb2: bool,
    irq_a1: bool,
    irq_a2: bool,
    irq_b1: bool,
    irq_b2: bool,
    irq_a_state: bool,
    irq_b_state: bool,
}

impl Default for Pia6821 {
    fn default() -> Self {
        Self {
            out_a: 0,
            out_b: 0,
            out_ca2: false,
            out_cb2: false,
            ddr_a: 0,
            ddr_b: 0,
            control_a: 0,
            control_b: 0,
            a_input_overrides_output_mask: 0,
            in_ca1: true,
            in_ca2: true,
            in_cb1: false,
            in_cb2: false,
            irq_a1: false,
            irq_a2: false,
            irq_b1: false,
            irq_b2: false,
            irq_a_state: false,
            irq_b_state: false,
        }
    }
}

impl Pia6821 {
    pub fn read(&mut self, register: u8, input_a: u8, input_b: u8) -> u8 {
        match register & 0x03 {
            0x00 if output_selected(self.control_a) => self.port_a_read_with_side_effects(input_a),
            0x00 => self.ddr_a,
            0x01 => self.control_a_read(),
            0x02 if output_selected(self.control_b) => self.port_b_read_with_side_effects(input_b),
            0x02 => self.ddr_b,
            0x03 => self.control_b_read(),
            _ => unreachable!("PIA register is masked to two address bits"),
        }
    }

    pub fn write(
        &mut self,
        register: u8,
        value: u8,
        input_a: u8,
        _input_b: u8,
    ) -> Option<PiaOutputEvent> {
        match register & 0x03 {
            0x00 if output_selected(self.control_a) => {
                self.out_a = value;
                Some(PiaOutputEvent::PortA {
                    data: self.port_a_output(input_a),
                    ddr: self.ddr_a,
                })
            }
            0x00 => self.write_ddr_a(value, input_a),
            0x01 => {
                self.control_a = value & PIA_CONTROL_WRITABLE_MASK;
                self.update_ca2_after_control_write();
                self.update_interrupts();
                None
            }
            0x02 if output_selected(self.control_b) => {
                self.out_b = value;
                if c2_strobe_mode(self.control_b) {
                    self.out_cb2 = false;
                    if strobe_e_reset(self.control_b) {
                        self.out_cb2 = true;
                    }
                }
                Some(PiaOutputEvent::PortB {
                    data: self.port_b_output(),
                    ddr: self.ddr_b,
                })
            }
            0x02 => self.write_ddr_b(value),
            0x03 => {
                self.control_b = value & PIA_CONTROL_WRITABLE_MASK;
                self.update_cb2_after_control_write();
                self.update_interrupts();
                None
            }
            _ => unreachable!("PIA register is masked to two address bits"),
        }
    }

    pub fn out_a(&self) -> u8 {
        self.out_a
    }

    pub fn out_b(&self) -> u8 {
        self.out_b
    }

    pub fn ddr_a(&self) -> u8 {
        self.ddr_a
    }

    pub fn ddr_b(&self) -> u8 {
        self.ddr_b
    }

    pub fn control_a(&self) -> u8 {
        self.control_a
    }

    pub fn control_b(&self) -> u8 {
        self.control_b
    }

    pub fn ca2_output(&self) -> bool {
        self.out_ca2
    }

    pub fn ca2_output_z(&self) -> bool {
        self.out_ca2 || c2_input(self.control_a)
    }

    pub fn cb2_output(&self) -> bool {
        self.out_cb2
    }

    /// Matches MAME's `cb2_output_z`: true means CB2 is high-impedance because
    /// the control register selects input mode.
    pub fn cb2_output_z(&self) -> bool {
        c2_input(self.control_b)
    }

    pub fn set_a_input_overrides_output_mask(&mut self, mask: u8) {
        self.a_input_overrides_output_mask = mask;
    }

    pub fn ca1_w(&mut self, state: bool) {
        if self.in_ca1 != state && c1_transition_matches(self.control_a, state) {
            self.irq_a1 = true;
            if c2_output(self.control_a)
                && c2_strobe_mode(self.control_a)
                && strobe_c1_reset(self.control_a)
                && !self.out_ca2
            {
                self.out_ca2 = true;
            }
        }
        self.in_ca1 = state;
        self.update_interrupts();
    }

    pub fn ca2_w(&mut self, state: bool) {
        if c2_input(self.control_a)
            && self.in_ca2 != state
            && c2_transition_matches(self.control_a, state)
        {
            self.irq_a2 = true;
        }
        self.in_ca2 = state;
        self.update_interrupts();
    }

    pub fn cb1_w(&mut self, state: bool) {
        if self.in_cb1 != state && c1_transition_matches(self.control_b, state) {
            self.irq_b1 = true;
        }
        self.in_cb1 = state;
        self.update_interrupts();
    }

    pub fn cb2_w(&mut self, state: bool) {
        if c2_input(self.control_b)
            && self.in_cb2 != state
            && c2_transition_matches(self.control_b, state)
        {
            self.irq_b2 = true;
        }
        self.in_cb2 = state;
        self.update_interrupts();
    }

    pub fn irq_a_asserted(&self) -> bool {
        self.irq_a_state
    }

    pub fn irq_b_asserted(&self) -> bool {
        self.irq_b_state
    }

    pub fn in_ca1(&self) -> bool {
        self.in_ca1
    }

    pub fn in_cb1(&self) -> bool {
        self.in_cb1
    }

    fn write_ddr_a(&mut self, value: u8, input_a: u8) -> Option<PiaOutputEvent> {
        if self.ddr_a == value {
            return None;
        }

        self.ddr_a = value;
        Some(PiaOutputEvent::PortA {
            data: self.port_a_output(input_a),
            ddr: self.ddr_a,
        })
    }

    fn write_ddr_b(&mut self, value: u8) -> Option<PiaOutputEvent> {
        if self.ddr_b == value {
            return None;
        }

        self.ddr_b = value;
        Some(PiaOutputEvent::PortB {
            data: self.port_b_output(),
            ddr: self.ddr_b,
        })
    }

    fn port_a_read(&self, input_a: u8) -> u8 {
        (!self.ddr_a & input_a)
            | (self.ddr_a & self.out_a & !self.a_input_overrides_output_mask)
            | (self.ddr_a & input_a & self.a_input_overrides_output_mask)
    }

    fn port_a_read_with_side_effects(&mut self, input_a: u8) -> u8 {
        let value = self.port_a_read(input_a);
        self.irq_a1 = false;
        self.irq_a2 = false;
        self.update_interrupts();
        if c2_output(self.control_a) && c2_strobe_mode(self.control_a) {
            if self.out_ca2 {
                self.out_ca2 = false;
            }
            if strobe_e_reset(self.control_a) {
                self.out_ca2 = true;
            }
        }
        value
    }

    fn port_b_read(&self, input_b: u8) -> u8 {
        if self.ddr_b == 0xFF {
            self.out_b
        } else {
            (self.out_b & self.ddr_b) | (input_b & !self.ddr_b)
        }
    }

    fn port_b_read_with_side_effects(&mut self, input_b: u8) -> u8 {
        let value = self.port_b_read(input_b);
        if self.irq_b1 && c2_strobe_mode(self.control_b) && strobe_c1_reset(self.control_b) {
            self.out_cb2 = true;
        }
        self.irq_b1 = false;
        self.irq_b2 = false;
        self.update_interrupts();
        value
    }

    fn control_a_read(&self) -> u8 {
        self.control_a
            | if self.irq_a1 { PIA_IRQ1 } else { 0 }
            | if self.irq_a2 && c2_input(self.control_a) {
                PIA_IRQ2
            } else {
                0
            }
    }

    fn control_b_read(&self) -> u8 {
        self.control_b
            | if self.irq_b1 { PIA_IRQ1 } else { 0 }
            | if self.irq_b2 && c2_input(self.control_b) {
                PIA_IRQ2
            } else {
                0
            }
    }

    fn port_a_output(&self, input_a: u8) -> u8 {
        if self.ddr_a == 0xFF {
            self.out_a
        } else {
            (self.out_a & self.ddr_a) | (self.port_a_read(input_a) & !self.ddr_a)
        }
    }

    fn port_b_output(&self) -> u8 {
        self.out_b & self.ddr_b
    }

    fn update_ca2_after_control_write(&mut self) {
        if c2_output(self.control_a) {
            if c2_set_mode(self.control_a) {
                self.out_ca2 = c2_set(self.control_a);
            } else {
                self.out_ca2 = true;
            }
        }
    }

    fn update_cb2_after_control_write(&mut self) {
        if c2_set_mode(self.control_b) {
            self.out_cb2 = c2_set(self.control_b);
        } else {
            self.out_cb2 = true;
        }
    }

    fn update_interrupts(&mut self) {
        self.irq_a_state = (self.irq_a1 && irq1_enabled(self.control_a))
            || (self.irq_a2 && irq2_enabled(self.control_a));
        self.irq_b_state = (self.irq_b1 && irq1_enabled(self.control_b))
            || (self.irq_b2 && irq2_enabled(self.control_b));
    }
}

pub fn output_selected(control: u8) -> bool {
    control & PIA_CONTROL_OUTPUT_SELECTED != 0
}

pub fn irq1_enabled(control: u8) -> bool {
    control & PIA_CONTROL_IRQ1_ENABLE != 0
}

pub fn irq2_enabled(control: u8) -> bool {
    control & PIA_CONTROL_IRQ2_ENABLE != 0
}

pub fn c1_low_to_high(control: u8) -> bool {
    control & PIA_CONTROL_C1_LOW_TO_HIGH != 0
}

pub fn c2_low_to_high(control: u8) -> bool {
    control & PIA_CONTROL_C2_LOW_TO_HIGH != 0
}

pub fn c2_output(control: u8) -> bool {
    control & PIA_CONTROL_C2_OUTPUT != 0
}

pub fn c2_input(control: u8) -> bool {
    !c2_output(control)
}

pub fn c2_set(control: u8) -> bool {
    control & PIA_CONTROL_IRQ2_ENABLE != 0
}

pub fn c2_set_mode(control: u8) -> bool {
    control & PIA_CONTROL_C2_LOW_TO_HIGH != 0
}

pub fn c2_strobe_mode(control: u8) -> bool {
    !c2_set_mode(control)
}

pub fn strobe_e_reset(control: u8) -> bool {
    control & PIA_CONTROL_IRQ2_ENABLE != 0
}

pub fn strobe_c1_reset(control: u8) -> bool {
    !strobe_e_reset(control)
}

fn c1_transition_matches(control: u8, state: bool) -> bool {
    state == c1_low_to_high(control)
}

fn c2_transition_matches(control: u8, state: bool) -> bool {
    state == c2_low_to_high(control)
}

#[cfg(test)]
mod tests {
    use crate::pia::{
        PIA_CONTROL_WRITABLE_MASK, PIA_IRQ1, PIA_IRQ2, Pia6821, PiaOutputEvent, c1_low_to_high,
        c2_input, c2_output, c2_set_mode, c2_strobe_mode, irq1_enabled, output_selected,
        strobe_c1_reset, strobe_e_reset,
    };

    #[test]
    fn pia_defaults_select_ddr_registers() {
        let mut pia = Pia6821::default();

        assert_eq!(pia.read(0, 0xFF, 0xFF), 0x00);
        assert_eq!(pia.read(2, 0xFF, 0xFF), 0x00);
        assert_eq!(pia.control_a(), 0x00);
        assert_eq!(pia.control_b(), 0x00);
        assert!(!output_selected(pia.control_a()));
    }

    #[test]
    fn pia_default_line_levels_match_mame_reset() {
        let pia = Pia6821::default();

        assert!(pia.in_ca1());
        assert!(!pia.in_cb1());
        assert!(!pia.ca2_output());
        assert!(pia.ca2_output_z());
        assert!(!pia.cb2_output());
        assert!(pia.cb2_output_z());
    }

    #[test]
    fn pia_control_writes_mask_read_only_irq_bits_and_select_data_registers() {
        let mut pia = Pia6821::default();

        assert_eq!(pia.write(1, 0xFF, 0, 0), None);
        assert_eq!(pia.write(3, 0x44, 0, 0), None);

        assert_eq!(pia.control_a(), PIA_CONTROL_WRITABLE_MASK);
        assert_eq!(pia.control_b(), 0x04);
        assert!(output_selected(pia.control_a()));
        assert!(output_selected(pia.control_b()));
    }

    #[test]
    fn pia_port_reads_merge_input_and_output_by_ddr() {
        let mut pia = Pia6821::default();

        assert_eq!(
            pia.write(0, 0xF0, 0x0C, 0x00),
            Some(PiaOutputEvent::PortA {
                data: 0x0C,
                ddr: 0xF0,
            })
        );
        assert_eq!(pia.write(1, 0x04, 0x0C, 0x00), None);
        assert_eq!(
            pia.write(0, 0xA5, 0x0C, 0x00),
            Some(PiaOutputEvent::PortA {
                data: 0xAC,
                ddr: 0xF0,
            })
        );
        assert_eq!(pia.read(0, 0x0C, 0x00), 0xAC);

        assert_eq!(
            pia.write(2, 0xF0, 0x00, 0x03),
            Some(PiaOutputEvent::PortB {
                data: 0x00,
                ddr: 0xF0,
            })
        );
        assert_eq!(pia.write(3, 0x04, 0x00, 0x03), None);
        assert_eq!(
            pia.write(2, 0xA5, 0x00, 0x03),
            Some(PiaOutputEvent::PortB {
                data: 0xA0,
                ddr: 0xF0,
            })
        );
        assert_eq!(pia.read(2, 0x00, 0x03), 0xA3);
    }

    #[test]
    fn pia_ddr_write_only_fires_output_event_when_value_changes() {
        let mut pia = Pia6821::default();

        assert_eq!(
            pia.write(2, 0xFF, 0, 0),
            Some(PiaOutputEvent::PortB { data: 0, ddr: 0xFF })
        );
        assert_eq!(pia.write(2, 0xFF, 0, 0), None);
        assert_eq!(pia.ddr_b(), 0xFF);
    }

    #[test]
    fn pia_port_a_can_allow_input_to_override_output_bits() {
        let mut pia = Pia6821::default();
        pia.set_a_input_overrides_output_mask(0x80);

        pia.write(0, 0xFF, 0, 0);
        pia.write(1, 0x04, 0, 0);
        pia.write(0, 0xFF, 0, 0);

        assert_eq!(pia.read(0, 0x00, 0), 0x7F);
    }

    #[test]
    fn pia_c1_edges_set_irq_flags_and_assert_only_when_enabled() {
        let mut pia = Pia6821::default();

        pia.write(1, 0x02, 0, 0);
        assert!(c1_low_to_high(pia.control_a()));
        pia.ca1_w(false);
        pia.ca1_w(true);

        assert_eq!(pia.read(1, 0, 0), PIA_IRQ1 | 0x02);
        assert!(!pia.irq_a_asserted());

        pia.write(1, 0x03, 0, 0);
        assert!(irq1_enabled(pia.control_a()));
        assert!(pia.irq_a_asserted());

        pia.write(1, 0x07, 0, 0);
        assert_eq!(pia.read(0, 0x5A, 0), 0x5A);
        assert_eq!(pia.read(1, 0, 0), 0x07);
        assert!(!pia.irq_a_asserted());
    }

    #[test]
    fn pia_default_c1_transition_is_high_to_low() {
        let mut pia = Pia6821::default();

        pia.write(3, 0x01, 0, 0);
        pia.cb1_w(true);
        assert_eq!(pia.read(3, 0, 0), 0x01);
        assert!(!pia.irq_b_asserted());

        pia.cb1_w(false);
        assert_eq!(pia.read(3, 0, 0), PIA_IRQ1 | 0x01);
        assert!(pia.irq_b_asserted());

        pia.write(3, 0x05, 0, 0);
        assert_eq!(pia.read(2, 0, 0xA5), 0xA5);
        assert_eq!(pia.read(3, 0, 0), 0x05);
        assert!(!pia.irq_b_asserted());
    }

    #[test]
    fn pia_c2_input_edges_report_irq2_and_clear_on_port_read() {
        let mut pia = Pia6821::default();

        pia.write(1, 0x1C, 0, 0);
        assert!(c2_input(pia.control_a()));
        pia.ca2_w(false);
        pia.ca2_w(true);

        assert_eq!(pia.read(1, 0, 0), PIA_IRQ2 | 0x1C);
        assert!(pia.irq_a_asserted());

        pia.write(1, 0x04, 0, 0);
        assert_eq!(pia.read(1, 0, 0), PIA_IRQ2 | 0x04);
        assert!(!pia.irq_a_asserted());
        assert_eq!(pia.read(0, 0x12, 0), 0x12);
        assert_eq!(pia.read(1, 0, 0), 0x04);
    }

    #[test]
    fn pia_ca2_output_set_reset_and_read_strobe_modes_follow_mame() {
        let mut pia = Pia6821::default();

        pia.write(1, 0x30, 0, 0);
        assert!(c2_output(pia.control_a()));
        assert!(c2_set_mode(pia.control_a()));
        assert!(!pia.ca2_output());
        assert!(!pia.ca2_output_z());

        pia.write(1, 0x38, 0, 0);
        assert!(pia.ca2_output());
        assert!(pia.ca2_output_z());

        pia.write(1, 0x24, 0, 0);
        assert!(c2_strobe_mode(pia.control_a()));
        assert!(strobe_c1_reset(pia.control_a()));
        assert!(pia.ca2_output());

        assert_eq!(pia.read(0, 0x5A, 0), 0x5A);
        assert!(!pia.ca2_output());
        assert!(!pia.ca2_output_z());

        pia.ca1_w(false);
        assert!(pia.ca2_output());
        assert!(pia.ca2_output_z());

        pia.write(1, 0x2C, 0, 0);
        assert!(strobe_e_reset(pia.control_a()));
        assert!(pia.ca2_output());
        assert_eq!(pia.read(0, 0xA5, 0), 0xA5);
        assert!(pia.ca2_output());
    }

    #[test]
    fn pia_cb2_write_strobe_modes_follow_mame_port_b_rules() {
        let mut pia = Pia6821::default();

        pia.write(3, 0x30, 0, 0);
        assert!(c2_output(pia.control_b()));
        assert!(c2_set_mode(pia.control_b()));
        assert!(!pia.cb2_output());
        assert!(!pia.cb2_output_z());

        pia.write(3, 0x38, 0, 0);
        assert!(pia.cb2_output());

        pia.write(3, 0x24, 0, 0);
        assert!(c2_strobe_mode(pia.control_b()));
        assert!(strobe_c1_reset(pia.control_b()));
        assert!(pia.cb2_output());

        assert_eq!(
            pia.write(2, 0xAA, 0, 0x55),
            Some(PiaOutputEvent::PortB { data: 0, ddr: 0 })
        );
        assert!(!pia.cb2_output());
        pia.cb1_w(true);
        pia.cb1_w(false);
        assert!(!pia.cb2_output());
        assert_eq!(pia.read(2, 0, 0x55), 0x55);
        assert!(pia.cb2_output());

        pia.write(3, 0x2C, 0, 0);
        assert!(strobe_e_reset(pia.control_b()));
        assert!(pia.cb2_output());
        assert_eq!(
            pia.write(2, 0x55, 0, 0xAA),
            Some(PiaOutputEvent::PortB { data: 0, ddr: 0 })
        );
        assert!(pia.cb2_output());
    }
}
