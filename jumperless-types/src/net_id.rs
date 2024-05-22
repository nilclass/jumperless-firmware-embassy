use core::num::NonZeroU8;

/// Identifier for a net. Must be unique within a netlist.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NetId(NonZeroU8);

impl From<u8> for NetId {
    fn from(value: u8) -> Self {
        NetId(NonZeroU8::new(value).unwrap())
    }
}

impl NetId {
    /// Nets with numbers 1..=7 are "special"
    pub fn is_special(&self) -> bool {
        self.0.get() <= 7
    }

    /// Return index of this net
    ///
    /// The index is always the net number minus one.
    pub fn index(&self) -> usize {
        self.0.get() as usize - 1
    }

    /// Construct NetId from index.
    pub fn from_index(index: usize) -> Self {
        Self(NonZeroU8::new(index as u8 + 1).unwrap())
    }
}

impl core::fmt::Display for NetId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0.get())
    }
}
