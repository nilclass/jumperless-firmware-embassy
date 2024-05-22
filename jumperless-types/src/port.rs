use crate::{ChipId, Dimension, Edge};

/// Represents a single connection point on one of the chip edges
///
/// Examples: Ay0, Bx7
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Port(ChipId, Dimension, u8);

impl Port {
    /// Construct port for given chip, dimension and index
    pub fn new(chip_id: ChipId, dimension: Dimension, index: u8) -> Self {
        Self(chip_id, dimension, index)
    }

    /// The chip on which this port resides
    pub fn chip_id(&self) -> ChipId {
        self.0
    }

    /// Dimension of this port
    pub fn dimension(&self) -> Dimension {
        self.1
    }

    /// Index of the port (0..=15 if `dimension` is X, 0..=7 if it is Y)
    pub fn index(&self) -> u8 {
        self.2
    }

    /// The edge on which this port resides
    ///
    /// # Examples
    ///
    /// ```
    /// # use jumperless_types::{Port, Edge, ChipId, Dimension};
    /// let port = Port::new(ChipId::from_ascii(b'C'), Dimension::Y, 4);
    /// assert_eq!(port.edge(), Edge::new(ChipId::from_ascii(b'C'), Dimension::Y));
    /// ```
    pub fn edge(&self) -> Edge {
        Edge::new(self.0, self.1)
    }

    /// Iterate over all possible ports
    ///
    /// # Examples
    ///
    /// ```
    /// # use jumperless_types::{Port, Edge, ChipId, Dimension};
    /// let mut ports = Port::all();
    /// assert_eq!(ports.next(), Some(Port::new(ChipId::from_ascii(b'A'), Dimension::X, 0)));
    /// assert_eq!(ports.next(), Some(Port::new(ChipId::from_ascii(b'A'), Dimension::X, 1)));
    /// assert_eq!(ports.next(), Some(Port::new(ChipId::from_ascii(b'A'), Dimension::X, 2)));
    /// assert_eq!(ports.next(), Some(Port::new(ChipId::from_ascii(b'A'), Dimension::X, 3)));
    /// assert_eq!(ports.next(), Some(Port::new(ChipId::from_ascii(b'A'), Dimension::X, 4)));
    /// assert_eq!(Port::all().nth(16), Some(Port::new(ChipId::from_ascii(b'A'), Dimension::Y, 0)));
    /// assert_eq!(Port::all().nth(24), Some(Port::new(ChipId::from_ascii(b'B'), Dimension::X, 0)));
    /// assert_eq!(Port::all().count(), 24 * 12);
    /// ```
    pub fn all() -> impl Iterator<Item = Port> {
        (0..12).flat_map(|chip_index| {
            let chip = ChipId::from_index(chip_index);
            let xs = (0..16).map(move |x_index| Port(chip, Dimension::X, x_index));
            let ys = (0..8).map(move |y_index| Port(chip, Dimension::Y, y_index));
            xs.chain(ys)
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidPort;

impl core::str::FromStr for Port {
    type Err = InvalidPort;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = s.as_bytes();
        let chip_id = ChipId::try_from_ascii(bytes[0]).ok_or(InvalidPort)?;
        let dimension = match bytes[1] {
            b'x' | b'X' => Dimension::X,
            b'y' | b'Y' => Dimension::Y,
            _ => return Err(InvalidPort),
        };
        let index: u8 = s[2..].parse().map_err(|_| InvalidPort)?;
        if index > dimension.port_count() - 1 {
            return Err(InvalidPort);
        }
        Ok(Port(chip_id, dimension, index))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_str() {
        let a = ChipId::from_ascii(b'A');
        let l = ChipId::from_ascii(b'L');
        assert_eq!("Ax0".parse(), Ok(Port::new(a, Dimension::X, 0)));
        assert_eq!("AX0".parse(), Ok(Port::new(a, Dimension::X, 0)));
        assert_eq!("Ax1".parse(), Ok(Port::new(a, Dimension::X, 1)));
        assert_eq!("Ax15".parse(), Ok(Port::new(a, Dimension::X, 15)));
        assert_eq!("Ly7".parse(), Ok(Port::new(l, Dimension::Y, 7)));
        assert_eq!("LY7".parse(), Ok(Port::new(l, Dimension::Y, 7)));

        // chip out of range
        assert_eq!("Mx0".parse::<Port>(), Err(InvalidPort));
        // dimension out of range
        assert_eq!("Az0".parse::<Port>(), Err(InvalidPort));
        // X index out of range
        assert_eq!("Ax16".parse::<Port>(), Err(InvalidPort));
        // Y index out of range
        assert_eq!("Ay8".parse::<Port>(), Err(InvalidPort));
    }
}
