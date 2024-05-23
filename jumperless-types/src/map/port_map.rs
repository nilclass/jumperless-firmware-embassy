use core::marker::PhantomData;
use crate::{Node, Port};

/// Maps every port to either a node or a lane (index)
///
/// Nodes are held by the map directly, while for lanes only the index is stored.
/// Thus a PortMap is only valid in conjunction with the lanes of the board that it was constructed for.
pub struct PortMap<N>([PortMapEntry<N>; 24 * 12]);

impl<N: Node> Default for PortMap<N> {
    fn default() -> Self {
        Self([PortMapEntry::new_none(); 24 * 12])
    }
}

impl<N: Node> PortMap<N> {
    pub fn get_node(&self, port: Port) -> Option<N> {
        self.0[Self::address(port)].node()
    }

    pub fn get_lane_index(&self, port: Port) -> Option<usize> {
        self.0[Self::address(port)].lane_index()
    }

    pub fn set_node(&mut self, port: Port, node: N) {
        self.0[Self::address(port)] = PortMapEntry::new_node(node);
    }

    pub fn set_lane_index(&mut self, port: Port, index: usize) {
        self.0[Self::address(port)] = PortMapEntry::new_lane_index(index);
    }

    fn address(port: Port) -> usize {
        port.chip_id().index() * 24 + port.dimension().index() * 16 + port.index() as usize
    }
}

#[derive(Copy, Clone)]
struct PortMapEntry<N>(u8, PhantomData<N>);

// highest possible lane index (127) is used to indicate the port is not mapped anywhere:
const ENTRY_VALUE_NONE: u8 = 0x7F << 1;

impl<N: Node> PortMapEntry<N> {
    /// Construct entry pointing to nothing
    fn new_none() -> Self {
        Self(ENTRY_VALUE_NONE, PhantomData)
    }

    /// Construct entry pointing to a node
    fn new_node(node: N) -> Self {
        Self((node.id() << 1) | 1, PhantomData)
    }

    /// Construct entry pointing to a lane
    ///
    /// (Lanes don't fit into 7 bits, so instead we keep an index into the list of lanes of the layout)
    fn new_lane_index(index: usize) -> Self {
        assert!(index < 0x7F);
        Self((index as u8) << 1, PhantomData)
    }

    /// Retrieve node that this entry points to
    fn node(&self) -> Option<N> {
        if self.0 & 1 == 1 {
            Some(Node::from_id(self.0 >> 1))
        } else {
            None
        }
    }

    fn lane_index(&self) -> Option<usize> {
        if self.0 & 1 == 1 || self.0 == ENTRY_VALUE_NONE {
            None
        } else {
            Some((self.0 >> 1) as usize)
        }
    }
}
