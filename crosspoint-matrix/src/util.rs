use crate::{ChipPort, ChipId, Dimension, Edge, Lane};

/// A bitmap that can hold a boolean for every [`ChipPort`].
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

    #[cfg(feature = "std")]
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

/// A bitmap holding a boolean for every [`Edge`].
pub struct EdgeBitMap([u8; 3]);

impl EdgeBitMap {
    pub fn empty() -> Self {
        Self([0; 3])
    }

    pub fn get(&self, edge: Edge) -> bool {
        let (i, j) = Self::address(edge);
        (self.0[i] >> j) & 1 == 1
    }

    pub fn set(&mut self, edge: Edge) {
        let (i, j) = Self::address(edge);
        self.0[i] |= 1 << j
    }

    pub fn clear(&mut self, edge: Edge) {
        let (i, j) = Self::address(edge);
        self.0[i] &= !(1 << j)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Edge> + 'a {
        (0..24).filter_map(|address| {
            let edge = Self::edge_from_address(address);
            if self.get(edge) {
                Some(edge)
            } else {
                None
            }
        })
    }

    pub fn len(&self) -> usize {
        self.iter().count()
    }

    pub fn pop(&mut self) -> Option<Edge> {
        let first = self.iter().next();
        first.map(|edge| { self.clear(edge); edge })
    }

    fn address(Edge(chip, dimension): Edge) -> (usize, usize) {
        let bit_address = chip.index() * 2 + if dimension == Dimension::X { 0 } else { 1 };
        (bit_address / 8, bit_address % 8)
    }

    fn edge_from_address(address: usize) -> Edge {
        Edge(ChipId::from_index(address >> 1), if address & 1 == 0 { Dimension::X } else { Dimension::Y })
    }
}

impl core::fmt::Debug for EdgeBitMap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.iter().fold(
            &mut f.debug_list(),
            |list, edge| list.entry(&edge),
        ).finish()
    }
}

pub struct LaneSet<'a>(&'a [Lane], [u8; 16]);

impl<'a> LaneSet<'a> {
    /// Construct a new LaneSet holding the given lanes
    pub fn new(lanes: &'a [Lane]) -> Self {
        assert!(lanes.len() < 128);
        Self(lanes, [0xFF; 16])
    }

    /// Remove the first lane that matches given predicate and return it
    pub fn take<F: Fn(Lane) -> bool>(&mut self, predicate: F) -> Option<Lane> {
        for i in 0..self.0.len() {
            if self.has_index(i) {
                let lane = self.0[i];
                if predicate(lane) {
                    self.clear_index(i);
                    return Some(lane)
                }
            }
        }
        None
    }

    fn has_index(&self, index: usize) -> bool {
        let (i, j) = (index / 8, index % 8);
        (self.1[i] >> j) & 1 == 1
    }

    fn clear_index(&mut self, index: usize) {
        let (i, j) = (index / 8, index % 8);
        self.1[i] &= !(1 << j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_bit_map_empty() {
        let edges = EdgeBitMap::empty();
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_edge_bit_map_single_edge() {
        let mut edges = EdgeBitMap::empty();
        let edge = Edge(ChipId(b'C'), Dimension::X);
        edges.set(edge);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![edge]);

        let mut edges = EdgeBitMap::empty();
        let edge = Edge(ChipId(b'C'), Dimension::Y);
        edges.set(edge);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![edge]);

        let mut edges = EdgeBitMap::empty();
        let edge = Edge(ChipId(b'A'), Dimension::X);
        edges.set(edge);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![edge]);
    }

    #[test]
    fn test_edge_bit_map_multiple_edges() {
        let mut edges = EdgeBitMap::empty();
        let ax = Edge(ChipId(b'A'), Dimension::X);
        let ay = Edge(ChipId(b'A'), Dimension::Y);
        let iy = Edge(ChipId(b'I'), Dimension::Y);
        edges.set(ax);
        edges.set(ay);
        edges.set(iy);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![ax, ay, iy]);
    }

    #[test]
    fn test_edge_bit_map_clear() {
        let mut edges = EdgeBitMap::empty();
        let ax = Edge(ChipId(b'A'), Dimension::X);
        let ay = Edge(ChipId(b'A'), Dimension::Y);
        let iy = Edge(ChipId(b'I'), Dimension::Y);
        edges.set(ax);
        edges.set(ay);
        edges.set(iy);
        edges.clear(ay);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![ax, iy]);
    }

    #[test]
    fn test_edge_bit_map_pop_and_len() {
        let mut edges = EdgeBitMap::empty();
        let ax = Edge(ChipId(b'A'), Dimension::X);
        let ay = Edge(ChipId(b'A'), Dimension::Y);
        let iy = Edge(ChipId(b'I'), Dimension::Y);
        edges.set(ax);
        edges.set(ay);
        edges.set(iy);
        assert_eq!(edges.len(), 3);
        assert_eq!(edges.pop(), Some(ax));
        assert_eq!(edges.len(), 2);
        assert_eq!(edges.pop(), Some(ay));
        assert_eq!(edges.len(), 1);
        assert_eq!(edges.pop(), Some(iy));
        assert_eq!(edges.len(), 0);
        assert_eq!(edges.pop(), None);
    }
}
