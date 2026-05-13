//! Source-scheduler process routing helpers.

use super::*;

impl RedLabelScheduledProcess {
    pub fn from_source_routine(process_address: u16, routine_address: u16) -> Result<Self, String> {
        Ok(Self {
            process_address,
            routine_address,
            entry_registers: red_label_source_entry_registers_for_routine(
                process_address,
                routine_address,
            )?,
        })
    }

    pub(super) fn validate_source_disp_context(self, current_process: u16) -> Result<(), String> {
        if self.entry_registers.u == Some(current_process)
            && self.process_address == current_process
        {
            return Ok(());
        }

        Err(format!(
            "red-label scheduled process 0x{:04X} does not match source DISP U/CRPROC 0x{current_process:04X}",
            self.process_address
        ))
    }
}

pub(super) fn red_label_source_entry_registers_for_routine(
    process_address: u16,
    routine_address: u16,
) -> Result<RedLabelCpuRegisters, String> {
    let mut registers = RedLabelCpuRegisters::from_source_disp(process_address);
    if routine_address == red_label_routine_address("PLS1")? {
        registers.b = Some(RED_LABEL_PLS1_ENTRY_B_REGISTER);
    }
    Ok(registers)
}

pub(super) fn red_label_live_player_start_process_routines() -> Result<[u16; 5], String> {
    Ok([
        red_label_routine_address("PLSTRT")?,
        red_label_routine_address("PLST1A")?,
        red_label_routine_address("PLSTR3")?,
        red_label_routine_address("PLS01")?,
        red_label_routine_address("PLS1")?,
    ])
}

pub(super) fn red_label_live_start_switch_process_routines() -> Result<[u16; 2], String> {
    Ok([
        red_label_routine_address("ST1")?,
        red_label_routine_address("ST2")?,
    ])
}

pub(super) fn red_label_live_coin_door_process_routines() -> Result<[u16; 6], String> {
    Ok([
        red_label_routine_address("LCOIN")?,
        red_label_routine_address("RCOIN")?,
        red_label_routine_address("CCOIN")?,
        red_label_routine_address("CN1")?,
        red_label_routine_address("ADVSW")?,
        red_label_routine_address("HSRES")?,
    ])
}

pub(super) fn red_label_live_game_over_attract_process_routines() -> Result<[u16; 2], String> {
    Ok([
        red_label_routine_address("PLE3")?,
        red_label_routine_address("HALL13")?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scheduled_source_routine_seeds_pls1_b_register() {
        let routine_address = red_label_routine_address("PLS1").expect("PLS1 routine exists");
        let scheduled = RedLabelScheduledProcess::from_source_routine(0xA05F, routine_address)
            .expect("PLS1 source registers are modeled");

        assert_eq!(scheduled.entry_registers.u, Some(0xA05F));
        assert_eq!(
            scheduled.entry_registers.b,
            Some(RED_LABEL_PLS1_ENTRY_B_REGISTER)
        );
    }

    #[test]
    fn live_process_routine_sets_are_source_named() {
        assert_eq!(
            red_label_live_player_start_process_routines()
                .expect("player-start routines are modeled")
                .len(),
            5
        );
        assert_eq!(
            red_label_live_coin_door_process_routines()
                .expect("coin-door routines are modeled")
                .len(),
            6
        );
    }
}
