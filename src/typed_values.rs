//! Typed value wrappers shared across actor and renderer boundaries.

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SoundCommand(u8);

impl SoundCommand {
    pub const fn new(byte: u8) -> Self {
        Self(byte)
    }

    pub const fn byte(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpriteFrameIndex(u8);

impl SpriteFrameIndex {
    pub const fn new(index: u8) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TimelineStep(u16);

impl TimelineStep {
    pub const fn new(step: u16) -> Self {
        Self(step)
    }

    pub const fn step(self) -> u16 {
        self.0
    }
}
