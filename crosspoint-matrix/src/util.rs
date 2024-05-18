use crate::{Port, ChipId, Dimension, Edge, Lane};

/// A set of ports. Implementeed as a bitmap.
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

    fn address(Port(chip, dimension, index): Port) -> (usize, usize) {
        let bit_address = chip.index() * 24 + dimension.index() * 16 + index as usize;
        (bit_address / 8, bit_address % 8)
    }
}

/// A set of edges. Implemented as a bitmap.
pub struct EdgeSet([u8; 3]);

impl EdgeSet {
    pub fn empty() -> Self {
        Self([0; 3])
    }

    pub fn contains(&self, edge: Edge) -> bool {
        let (i, j) = Self::address(edge);
        (self.0[i] >> j) & 1 == 1
    }

    pub fn insert(&mut self, edge: Edge) {
        let (i, j) = Self::address(edge);
        self.0[i] |= 1 << j
    }

    pub fn remove(&mut self, edge: Edge) {
        let (i, j) = Self::address(edge);
        self.0[i] &= !(1 << j)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = Edge> + 'a {
        (0..24).filter_map(|address| {
            let edge = Self::edge_from_address(address);
            if self.contains(edge) {
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
        first.map(|edge| { self.remove(edge); edge })
    }

    fn address(Edge(chip, dimension): Edge) -> (usize, usize) {
        let bit_address = chip.index() * 2 + dimension.index();
        (bit_address / 8, bit_address % 8)
    }

    fn edge_from_address(address: usize) -> Edge {
        Edge(ChipId::from_index(address >> 1), Dimension::from_index(address & 1))
    }
}

impl core::fmt::Debug for EdgeSet {
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
        let edges = EdgeSet::empty();
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![]);
    }

    #[test]
    fn test_edge_bit_map_single_edge() {
        let mut edges = EdgeSet::empty();
        let edge = Edge(ChipId(b'C'), Dimension::X);
        edges.insert(edge);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![edge]);

        let mut edges = EdgeSet::empty();
        let edge = Edge(ChipId(b'C'), Dimension::Y);
        edges.insert(edge);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![edge]);

        let mut edges = EdgeSet::empty();
        let edge = Edge(ChipId(b'A'), Dimension::X);
        edges.insert(edge);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![edge]);
    }

    #[test]
    fn test_edge_bit_map_multiple_edges() {
        let mut edges = EdgeSet::empty();
        let ax = Edge(ChipId(b'A'), Dimension::X);
        let ay = Edge(ChipId(b'A'), Dimension::Y);
        let iy = Edge(ChipId(b'I'), Dimension::Y);
        edges.insert(ax);
        edges.insert(ay);
        edges.insert(iy);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![ax, ay, iy]);
    }

    #[test]
    fn test_edge_bit_map_clear() {
        let mut edges = EdgeSet::empty();
        let ax = Edge(ChipId(b'A'), Dimension::X);
        let ay = Edge(ChipId(b'A'), Dimension::Y);
        let iy = Edge(ChipId(b'I'), Dimension::Y);
        edges.insert(ax);
        edges.insert(ay);
        edges.insert(iy);
        edges.remove(ay);
        assert_eq!(edges.iter().collect::<Vec<_>>(), vec![ax, iy]);
    }

    #[test]
    fn test_edge_bit_map_pop_and_len() {
        let mut edges = EdgeSet::empty();
        let ax = Edge(ChipId(b'A'), Dimension::X);
        let ay = Edge(ChipId(b'A'), Dimension::Y);
        let iy = Edge(ChipId(b'I'), Dimension::Y);
        edges.insert(ax);
        edges.insert(ay);
        edges.insert(iy);
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
