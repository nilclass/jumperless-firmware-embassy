use crate::Port;

/// A set of [`crate::Port`]s. Implemented as a bitmap.
///
/// Useful for various algorithms.
#[derive(Eq, PartialEq)]
pub struct PortSet([u8; 36]);

impl PortSet {
    /// Construct a new PortSet containing no ports
    pub fn empty() -> Self {
        Self([0; 36])
    }

    /// Construct a new PortSet containing all ports
    pub fn full() -> Self {
        Self([0xFF; 36])
    }

    /// Check if given port is included
    pub fn contains(&self, port: Port) -> bool {
        let (i, j) = Self::address(port);
        (self.0[i] >> j) & 1 == 1
    }

    /// Add given port
    pub fn insert(&mut self, port: Port) {
        let (i, j) = Self::address(port);
        self.0[i] |= 1 << j
    }

    /// Remove given port
    pub fn remove(&mut self, port: Port) {
        let (i, j) = Self::address(port);
        self.0[i] &= !(1 << j)
    }

    /// Check if all of the ports of `other` are also part of `self`
    pub fn is_superset(&self, other: &Self) -> bool {
        for (i, byte) in self.0.iter().enumerate() {
            if other.0[i] & byte != other.0[i] {
                return false;
            }
        }
        true
    }

    #[cfg(feature = "std")]
    pub fn print_diff(&self, other: &Self) {
        use crate::ChipId;

        println!("BEGIN DIFF");
        for i in 0..12 {
            let chip = ChipId::from_index(i);
            for x in 0..16 {
                let port = chip.port_x(x);
                let a = self.contains(port);
                let b = other.contains(port);
                if a && !b {
                    println!("+{:?}", port);
                } else if !a && b {
                    println!("-{:?}", port);
                }
            }
            for y in 0..8 {
                let port = chip.port_y(y);
                let a = self.contains(port);
                let b = other.contains(port);
                if a && !b {
                    println!("+{:?}", port);
                } else if !a && b {
                    println!("-{:?}", port);
                }
            }
        }
        println!("END DIFF");
    }

    fn address(port: Port) -> (usize, usize) {
        let bit_address =
            port.chip_id().index() * 24 + port.dimension().index() * 16 + port.index() as usize;
        (bit_address / 8, bit_address % 8)
    }
}
