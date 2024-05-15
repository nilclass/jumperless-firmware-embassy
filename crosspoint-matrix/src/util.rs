use crate::{ChipPort, ChipId, Dimension};

/// A bitmap that can hold a boolean for every chip port.
///
/// Useful for various algorithms.
#[derive(Eq, PartialEq)]
pub struct ChipPortBitMap([u8; 36]);

impl ChipPortBitMap {
    /// Construct a new ChipPortBitMap with all bits 0
    pub fn empty() -> Self {
        Self([0; 36])
    }

    /// Construct a new ChipPortBitMap with all bits 1
    pub fn full() -> Self {
        Self([0xFF; 36])
    }

    /// Check if bit for given port address is set
    pub fn get(&self, port: ChipPort) -> bool {
        let (i, j) = Self::address(port);
        (self.0[i] >> j) & 1 == 1
    }

    /// Set the bit for given port address
    pub fn set(&mut self, port: ChipPort) {
        let (i, j) = Self::address(port);
        self.0[i] |= 1 << j
    }

    /// Clear the bit for given port address
    pub fn clear(&mut self, port: ChipPort) {
        let (i, j) = Self::address(port);
        self.0[i] &= !(1 << j)
    }

    /// Check if all the bits that are set in `other` are also set in `self` (i.e. `self` is a superset of `other`)
    pub fn contains(&self, other: &Self) -> bool {
        for (i, byte) in self.0.iter().enumerate() {
            if other.0[i] & byte != other.0[i] {
                return false
            }
        }
        true
    }

    pub fn print_diff(&self, other: &Self) {
        println!("BEGIN DIFF");
        for i in 0..12 {
            let chip = ChipId::from_index(i);
            for x in 0..16 {
                let port = chip.port_x(x);
                let a = self.get(port);
                let b = other.get(port);
                if a && !b {
                    println!("+{:?}", port);
                } else if !a && b {
                    println!("-{:?}", port);
                }
            }
            for y in 0..8 {
                let port = chip.port_y(y);
                let a = self.get(port);
                let b = other.get(port);
                if a && !b {
                    println!("+{:?}", port);
                } else if !a && b {
                    println!("-{:?}", port);
                }
            }
        }
        println!("END DIFF");
    }

    fn address(port: ChipPort) -> (usize, usize) {
        let ChipPort(chip, dimension, index) = port;
        let bit_address = chip.index() * 24 + if dimension == Dimension::X { 0 } else { 16 } + index as usize;
        (bit_address / 8, bit_address % 8)
    }
}
