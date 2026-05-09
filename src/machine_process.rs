//! Source-scheduler process data contracts.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RedLabelCpuRegisters {
    pub a: Option<u8>,
    pub b: Option<u8>,
    pub x: Option<u16>,
    pub y: Option<u16>,
    pub u: Option<u16>,
    pub s: Option<u16>,
    pub cc: Option<u8>,
}

impl RedLabelCpuRegisters {
    pub const fn from_source_disp(process_address: u16) -> Self {
        Self {
            a: None,
            b: None,
            x: None,
            y: None,
            u: Some(process_address),
            s: None,
            cc: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RedLabelScheduledProcess {
    pub process_address: u16,
    pub routine_address: u16,
    pub entry_registers: RedLabelCpuRegisters,
}

impl RedLabelScheduledProcess {
    pub const fn from_source_disp(process_address: u16, routine_address: u16) -> Self {
        Self {
            process_address,
            routine_address,
            entry_registers: RedLabelCpuRegisters::from_source_disp(process_address),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RedLabelCpuRegisters, RedLabelScheduledProcess};

    #[test]
    fn source_disp_registers_seed_only_process_pointer() {
        let registers = RedLabelCpuRegisters::from_source_disp(0xA05F);

        assert_eq!(registers.u, Some(0xA05F));
        assert_eq!(registers.a, None);
        assert_eq!(registers.b, None);
        assert_eq!(registers.x, None);
        assert_eq!(registers.y, None);
        assert_eq!(registers.s, None);
        assert_eq!(registers.cc, None);
    }

    #[test]
    fn scheduled_process_from_source_disp_carries_entry_register_context() {
        let scheduled = RedLabelScheduledProcess::from_source_disp(0xA05F, 0xC123);

        assert_eq!(scheduled.process_address, 0xA05F);
        assert_eq!(scheduled.routine_address, 0xC123);
        assert_eq!(
            scheduled.entry_registers,
            RedLabelCpuRegisters::from_source_disp(0xA05F)
        );
    }
}
