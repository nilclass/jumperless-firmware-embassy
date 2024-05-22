use crate::Node;
use core::marker::PhantomData;

#[derive(PartialEq, Eq)]
pub struct NodeSet<Node>([u8; 16], PhantomData<Node>);

impl<N: Node> Default for NodeSet<N> {
    fn default() -> Self {
        Self([0; 16], PhantomData)
    }
}

impl<N: Node> NodeSet<N> {
    /// Number of nodes in this set
    pub fn len(&self) -> usize {
        self.0.iter().fold(0, |a, i| a + i.count_ones() as usize)
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|i| *i == 0)
    }

    /// Does this set contain given node?
    pub fn contains(&self, node: N) -> bool {
        let (i, j) = Self::address(node);
        (self.0[i] >> j) & 1 == 1
    }

    /// Insert given node
    pub fn insert(&mut self, node: N) {
        let (i, j) = Self::address(node);
        self.0[i] |= 1 << j;
    }

    /// Remove given node
    pub fn remove(&mut self, node: N) {
        let (i, j) = Self::address(node);
        self.0[i] &= !(1 << j);
    }

    /// Iterate over nodes in this set
    pub fn iter(&self) -> impl Iterator<Item = N> + '_ {
        (0..16).flat_map(move |i| {
            (0..8).filter_map(move |j| {
                if (self.0[i] >> j) & 1 == 1 {
                    Some(Node::from_id((i * 8 + j) as u8))
                } else {
                    None
                }
            })
        })
    }

    /// Removes all nodes from `self`, and returns a copy of self before the removal
    pub fn take(&mut self) -> Self {
        let copy = NodeSet(self.0, PhantomData);
        self.0.fill(0);
        copy
    }

    fn address(node: N) -> (usize, usize) {
        let value = node.id() as usize;
        (value / 8, value % 8)
    }
}

impl<N: Node> FromIterator<N> for NodeSet<N> {
    fn from_iter<T: IntoIterator<Item = N>>(iter: T) -> Self {
        let mut set = Self::default();
        for node in iter {
            set.insert(node);
        }
        set
    }
}

impl<N: Node> core::fmt::Debug for NodeSet<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.iter()
            .fold(&mut f.debug_list(), |list, node| list.entry(&node))
            .finish()
    }
}
