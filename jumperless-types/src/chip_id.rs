use crate::{Dimension, Port};

/// Represents one of the crosspoint chip on the jumperless
///
/// Chips can be identified either by index (`0..=11`) or by letter (`'A'..='L'`).
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ChipId(u8);

impl core::fmt::Display for ChipId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", core::str::from_utf8(&[self.0]).unwrap())
    }
}

impl core::fmt::Debug for ChipId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ChipId({})", core::str::from_utf8(&[self.0]).unwrap())
    }
}

impl ChipId {
    /// Returns the letter of this chip as an ASCII character ('A' through 'L')
    pub fn ascii(&self) -> u8 {
        self.0
    }

    /// Returns the index of this chip
    ///
    /// Indices are 0 (for chip A) through 11 (for chip L).
    pub fn index(&self) -> usize {
        (self.0 - b'A') as usize
    }

    /// Construct ChipId from given index
    ///
    /// The index must be in the 0..=11 range. 0 is chip A, 11 is chip L.
    ///
    /// Panics if index is out of range.
    pub fn from_index(index: usize) -> Self {
        assert!(index < 12);
        Self(b'A' + index as u8)
    }

    /// Construct ChipId from given ASCII letter
    ///
    /// The letter must be in the 'A'..='L' range.
    ///
    /// Panics if letter is out of range.
    pub fn from_ascii(ascii: u8) -> Self {
        assert!((b'A'..=b'L').contains(&ascii));
        Self(ascii)
    }

    pub fn try_from_ascii(ascii: u8) -> Option<Self> {
        if (b'A'..=b'L').contains(&ascii) {
            Some(Self(ascii))
        } else {
            None
        }
    }

    /// Get port on the X edge, at given index
    pub fn port_x(&self, x: u8) -> Port {
        Port::new(*self, Dimension::X, x)
    }

    /// Get port on the Y edge, at given index
    pub fn port_y(&self, y: u8) -> Port {
        Port::new(*self, Dimension::Y, y)
    }
}
