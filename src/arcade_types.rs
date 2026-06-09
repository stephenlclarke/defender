//! Typed arcade value wrappers shared across actor and renderer boundaries.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScreenAddress(u16);

impl ScreenAddress {
    pub const fn new(word: u16) -> Self {
        Self(word)
    }

    pub const fn from_bytes(column: u8, row: u8) -> Self {
        Self(u16::from_be_bytes([column, row]))
    }

    pub const fn word(self) -> u16 {
        self.0
    }

    pub const fn wrapping_add(self, rhs: u16) -> Self {
        Self(self.0.wrapping_add(rhs))
    }

    pub const fn wrapping_sub(self, rhs: u16) -> Self {
        Self(self.0.wrapping_sub(rhs))
    }
}
