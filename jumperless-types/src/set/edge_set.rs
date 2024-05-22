use crate::{ChipId, Dimension, Edge};

/// A set of edges. Implemented as a bitmap.
pub struct EdgeSet(u32);

impl EdgeSet {
    pub fn empty() -> Self {
        Self(0)
    }

    pub fn contains(&self, edge: Edge) -> bool {
        (self.0 >> Self::address(edge)) & 1 == 1
    }

    pub fn insert(&mut self, edge: Edge) {
        self.0 |= 1 << Self::address(edge)
    }

    pub fn remove(&mut self, edge: Edge) {
        self.0 &= !(1 << Self::address(edge))
    }

    pub fn iter(&self) -> impl Iterator<Item = Edge> + '_ {
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
        self.0.count_ones() as usize
    }

    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub fn pop(&mut self) -> Option<Edge> {
        let first = self.iter().next();
        first.map(|edge| {
            self.remove(edge);
            edge
        })
    }

    fn address(edge: Edge) -> usize {
        edge.chip_id().index() * 2 + edge.dimension().index()
    }

    fn edge_from_address(address: usize) -> Edge {
        Edge::new(
            ChipId::from_index(address >> 1),
            Dimension::from_index(address & 1),
        )
    }
}

impl core::fmt::Debug for EdgeSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.iter()
            .fold(&mut f.debug_list(), |list, edge| list.entry(&edge))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        let edges = EdgeSet::empty();
        assert_eq!(edges.iter().next(), None);
    }

    #[test]
    fn test_single_edge() {
        let mut edges = EdgeSet::empty();
        let edge = Edge::new(ChipId::from_ascii(b'C'), Dimension::X);
        edges.insert(edge);
        let mut iter = edges.iter();
        assert_eq!(iter.next(), Some(edge));
        assert_eq!(iter.next(), None);

        let mut edges = EdgeSet::empty();
        let edge = Edge::new(ChipId::from_ascii(b'C'), Dimension::Y);
        edges.insert(edge);
        let mut iter = edges.iter();
        assert_eq!(iter.next(), Some(edge));
        assert_eq!(iter.next(), None);

        let mut edges = EdgeSet::empty();
        let edge = Edge::new(ChipId::from_ascii(b'A'), Dimension::X);
        edges.insert(edge);
        let mut iter = edges.iter();
        assert_eq!(iter.next(), Some(edge));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_multiple_edges() {
        let mut edges = EdgeSet::empty();
        let ax = Edge::new(ChipId::from_ascii(b'A'), Dimension::X);
        let ay = Edge::new(ChipId::from_ascii(b'A'), Dimension::Y);
        let iy = Edge::new(ChipId::from_ascii(b'I'), Dimension::Y);
        edges.insert(ax);
        edges.insert(ay);
        edges.insert(iy);
        let mut iter = edges.iter();
        assert_eq!(iter.next(), Some(ax));
        assert_eq!(iter.next(), Some(ay));
        assert_eq!(iter.next(), Some(iy));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_remove() {
        let mut edges = EdgeSet::empty();
        let ax = Edge::new(ChipId::from_ascii(b'A'), Dimension::X);
        let ay = Edge::new(ChipId::from_ascii(b'A'), Dimension::Y);
        let iy = Edge::new(ChipId::from_ascii(b'I'), Dimension::Y);
        edges.insert(ax);
        edges.insert(ay);
        edges.insert(iy);
        edges.remove(ay);
        let mut iter = edges.iter();
        assert_eq!(iter.next(), Some(ax));
        assert_eq!(iter.next(), Some(iy));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_pop_and_len() {
        let mut edges = EdgeSet::empty();
        let ax = Edge::new(ChipId::from_ascii(b'A'), Dimension::X);
        let ay = Edge::new(ChipId::from_ascii(b'A'), Dimension::Y);
        let iy = Edge::new(ChipId::from_ascii(b'I'), Dimension::Y);
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
