use crate::{Port, Lane, ChipId, Dimension, Net, ChipStatus, NetId, util::PortSet};

pub struct NodeMapping(Node, Port);

pub struct Layout<const NODE_COUNT: usize, const LANE_COUNT: usize> {
    pub nodes: [NodeMapping; NODE_COUNT],
    pub lanes: [Lane; LANE_COUNT],
    port_map: PortMap,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NodeNet {
    pub id: NetId,
    pub nodes: Vec<Node>,
}

impl NodeNet {
    pub fn from_net<const NODE_COUNT: usize, const LANE_COUNT: usize>(net: &Net, layout: &Layout<NODE_COUNT, LANE_COUNT>) -> Self {
        Self {
            id: net.id,
            nodes: net.ports.iter().filter_map(|port| layout.port_to_node(*port)).collect(),
        }
    }

    pub fn sort(&mut self) {
        self.nodes.sort();
    }
}

impl<const NODE_COUNT: usize, const LANE_COUNT: usize> Layout<NODE_COUNT, LANE_COUNT> {
    pub fn new(nodes: [NodeMapping; NODE_COUNT], lanes: [Lane; LANE_COUNT]) -> Self {
        let port_map = PortMap::new(&nodes, &lanes);
        Self { nodes, lanes, port_map }
    }

    pub fn nets_to_connections(&self, nets: &[NodeNet], chip_status: &mut ChipStatus) {
        super::nets_to_connections(nets.iter().map(|net| {
            Net {
                id: net.id,
                ports: net.nodes.iter().map(|node| self.node_to_port(*node).unwrap()).collect(),
            }
        }), chip_status, &self.lanes, &self.port_map)
    }

    pub fn node_to_port(&self, node: Node) -> Option<Port> {
        self.nodes.iter().find(|NodeMapping(n, _)| *n == node).map(|NodeMapping(_, p)| p).copied()
    }

    pub fn port_to_node(&self, port: Port) -> Option<Node> {
        self.port_map.get_node(port)
    }

    /// If the given port is part of a lane, returns the port on the other side of that lane
    pub fn lane_destination(&self, port: Port) -> Option<Port> {
        if let Some(index) = self.port_map.get_lane_index(port) {
            let Lane(a, b) = self.lanes[index];
            if a == port {
                Some(b)
            } else {
                Some(a)
            }
        } else {
            None
        }
    }

}

#[cfg(feature = "std")]
impl<const NODE_COUNT: usize, const LANE_COUNT: usize> Layout<NODE_COUNT, LANE_COUNT> {
    /// Verify that all possible ports are referenced by exactly one node mapping or exactly one lane.
    ///
    /// Prints problems to stdout and panics if a check has failed.
    pub fn sanity_check(&self) {
        let mut problems = vec![];
        let mut used_ports = PortSet::empty();
        for NodeMapping(node, port) in &self.nodes {
            if used_ports.contains(*port) {
                problems.push(format!("Port {port:?} used more than once (last use in node mapping {node:?})"));
            }
            used_ports.insert(*port);
        }
        for Lane(a, b) in &self.lanes {
            if used_ports.contains(*a) {
                problems.push(format!("Port {a:?} used more than once (last use in lane with port {b:?})"));
            }
            used_ports.insert(*a);
            if used_ports.contains(*b) {
                problems.push(format!("Port {b:?} used more than once (last use in lane with port {a:?})"));
            }
            used_ports.insert(*b);
        }
        for problem in &problems {
            println!("Found problem: {}", problem);
        }
        let expected = PortSet::full();
        if used_ports != expected {
            println!("Not all ports have been used. Diff:");
            expected.print_diff(&used_ports);

            panic!("Sanity check failed");
        }
        if problems.len() > 0 {
            panic!("All ports were used, but problems have been detected");
        }
    }
}

#[derive(Copy, Clone)]
struct PortMapEntry(u8);

// highest possible lane index (127) is used to indicate
// the port is not mapped anywhere.
const PORT_USE_NONE: u8 = 0x7F << 1;

impl PortMapEntry {
    /// Construct PortUse pointing to nothing
    fn new_none() -> Self {
        Self(PORT_USE_NONE)
    }

    /// Construct PortUse pointing to a node
    fn new_node(node: Node) -> Self {
        Self(((node as u8) << 1) | 1)
    }

    /// Construct PortUse pointing to a lane
    ///
    /// (Lanes don't fit into 7 bits, so instead we keep an index into the list of lanes of the layout)
    fn new_lane_index(index: usize) -> Self {
        assert!(index < 0x7F);
        Self((index as u8) << 1)
    }

    fn node(&self) -> Option<Node> {
        if self.0 & 1 == 1 {
            // Safety: values are constructed through `node as u8` in `new_node`, so only valid values exist
            Some(unsafe { Node::from_u8(self.0 >> 1) })
        } else {
            None
        }
    }

    fn lane_index(&self) -> Option<usize> {
        if self.0 & 1 == 1 || self.0 == PORT_USE_NONE {
            None
        } else {
            Some((self.0 >> 1) as usize)
        }
    }
}

/// Maps every port to either a node or a lane
pub struct PortMap([PortMapEntry; 24 * 12]);

impl PortMap {
    pub fn new(nodes: &[NodeMapping], lanes: &[Lane]) -> Self {
        let mut m = PortMap([PortMapEntry::new_none(); 24 * 12]);
        for NodeMapping(node, port) in nodes {
            m.set_node(*port, *node);
        }
        for (index, Lane(a, b)) in lanes.into_iter().enumerate() {
            m.set_lane_index(*a, index);
            m.set_lane_index(*b, index);
        }
        m
    }

    pub fn get_node(&self, port: Port) -> Option<Node> {
        self.0[Self::address(port)].node()
    }

    pub fn get_lane_index(&self, port: Port) -> Option<usize> {
        self.0[Self::address(port)].lane_index()
    }

    pub fn set_node(&mut self, port: Port, node: Node) {
        self.0[Self::address(port)] = PortMapEntry::new_node(node);
    }

    pub fn set_lane_index(&mut self, port: Port, index: usize) {
        self.0[Self::address(port)] = PortMapEntry::new_lane_index(index);
    }

    fn address(Port(chip, dimension, index): Port) -> usize {
        chip.index() * 24 + dimension.index() * 16 + index as usize
    }
}

impl Layout<120, 84> {
    pub fn v4() -> Self {
        Self::new([
                NodeMapping(Node::Top2, Port(ChipId(b'A'), Dimension::Y, 1)),
                NodeMapping(Node::Top3, Port(ChipId(b'A'), Dimension::Y, 2)),
                NodeMapping(Node::Top4, Port(ChipId(b'A'), Dimension::Y, 3)),
                NodeMapping(Node::Top5, Port(ChipId(b'A'), Dimension::Y, 4)),
                NodeMapping(Node::Top6, Port(ChipId(b'A'), Dimension::Y, 5)),
                NodeMapping(Node::Top7, Port(ChipId(b'A'), Dimension::Y, 6)),
                NodeMapping(Node::Top8, Port(ChipId(b'A'), Dimension::Y, 7)),
                NodeMapping(Node::Top9, Port(ChipId(b'B'), Dimension::Y, 1)),
                NodeMapping(Node::Top10, Port(ChipId(b'B'), Dimension::Y, 2)),
                NodeMapping(Node::Top11, Port(ChipId(b'B'), Dimension::Y, 3)),
                NodeMapping(Node::Top12, Port(ChipId(b'B'), Dimension::Y, 4)),
                NodeMapping(Node::Top13, Port(ChipId(b'B'), Dimension::Y, 5)),
                NodeMapping(Node::Top14, Port(ChipId(b'B'), Dimension::Y, 6)),
                NodeMapping(Node::Top15, Port(ChipId(b'B'), Dimension::Y, 7)),
                NodeMapping(Node::Top16, Port(ChipId(b'C'), Dimension::Y, 1)),
                NodeMapping(Node::Top17, Port(ChipId(b'C'), Dimension::Y, 2)),
                NodeMapping(Node::Top18, Port(ChipId(b'C'), Dimension::Y, 3)),
                NodeMapping(Node::Top19, Port(ChipId(b'C'), Dimension::Y, 4)),
                NodeMapping(Node::Top20, Port(ChipId(b'C'), Dimension::Y, 5)),
                NodeMapping(Node::Top21, Port(ChipId(b'C'), Dimension::Y, 6)),
                NodeMapping(Node::Top22, Port(ChipId(b'C'), Dimension::Y, 7)),
                NodeMapping(Node::Top23, Port(ChipId(b'D'), Dimension::Y, 1)),
                NodeMapping(Node::Top24, Port(ChipId(b'D'), Dimension::Y, 2)),
                NodeMapping(Node::Top25, Port(ChipId(b'D'), Dimension::Y, 3)),
                NodeMapping(Node::Top26, Port(ChipId(b'D'), Dimension::Y, 4)),
                NodeMapping(Node::Top27, Port(ChipId(b'D'), Dimension::Y, 5)),
                NodeMapping(Node::Top28, Port(ChipId(b'D'), Dimension::Y, 6)),
                NodeMapping(Node::Top29, Port(ChipId(b'D'), Dimension::Y, 7)),
                NodeMapping(Node::Bottom2, Port(ChipId(b'E'), Dimension::Y, 1)),
                NodeMapping(Node::Bottom3, Port(ChipId(b'E'), Dimension::Y, 2)),
                NodeMapping(Node::Bottom4, Port(ChipId(b'E'), Dimension::Y, 3)),
                NodeMapping(Node::Bottom5, Port(ChipId(b'E'), Dimension::Y, 4)),
                NodeMapping(Node::Bottom6, Port(ChipId(b'E'), Dimension::Y, 5)),
                NodeMapping(Node::Bottom7, Port(ChipId(b'E'), Dimension::Y, 6)),
                NodeMapping(Node::Bottom8, Port(ChipId(b'E'), Dimension::Y, 7)),
                NodeMapping(Node::Bottom9, Port(ChipId(b'F'), Dimension::Y, 1)),
                NodeMapping(Node::Bottom10, Port(ChipId(b'F'), Dimension::Y, 2)),
                NodeMapping(Node::Bottom11, Port(ChipId(b'F'), Dimension::Y, 3)),
                NodeMapping(Node::Bottom12, Port(ChipId(b'F'), Dimension::Y, 4)),
                NodeMapping(Node::Bottom13, Port(ChipId(b'F'), Dimension::Y, 5)),
                NodeMapping(Node::Bottom14, Port(ChipId(b'F'), Dimension::Y, 6)),
                NodeMapping(Node::Bottom15, Port(ChipId(b'F'), Dimension::Y, 7)),
                NodeMapping(Node::Bottom16, Port(ChipId(b'G'), Dimension::Y, 1)),
                NodeMapping(Node::Bottom17, Port(ChipId(b'G'), Dimension::Y, 2)),
                NodeMapping(Node::Bottom18, Port(ChipId(b'G'), Dimension::Y, 3)),
                NodeMapping(Node::Bottom19, Port(ChipId(b'G'), Dimension::Y, 4)),
                NodeMapping(Node::Bottom20, Port(ChipId(b'G'), Dimension::Y, 5)),
                NodeMapping(Node::Bottom21, Port(ChipId(b'G'), Dimension::Y, 6)),
                NodeMapping(Node::Bottom22, Port(ChipId(b'G'), Dimension::Y, 7)),
                NodeMapping(Node::Bottom23, Port(ChipId(b'H'), Dimension::Y, 1)),
                NodeMapping(Node::Bottom24, Port(ChipId(b'H'), Dimension::Y, 2)),
                NodeMapping(Node::Bottom25, Port(ChipId(b'H'), Dimension::Y, 3)),
                NodeMapping(Node::Bottom26, Port(ChipId(b'H'), Dimension::Y, 4)),
                NodeMapping(Node::Bottom27, Port(ChipId(b'H'), Dimension::Y, 5)),
                NodeMapping(Node::Bottom28, Port(ChipId(b'H'), Dimension::Y, 6)),
                NodeMapping(Node::Bottom29, Port(ChipId(b'H'), Dimension::Y, 7)),
                NodeMapping(Node::NanoA0, Port(ChipId(b'I'), Dimension::X, 0)),
                NodeMapping(Node::NanoD1, Port(ChipId(b'I'), Dimension::X, 1)),
                NodeMapping(Node::NanoA2, Port(ChipId(b'I'), Dimension::X, 2)),
                NodeMapping(Node::NanoD3, Port(ChipId(b'I'), Dimension::X, 3)),
                NodeMapping(Node::NanoA4, Port(ChipId(b'I'), Dimension::X, 4)),
                NodeMapping(Node::NanoD5, Port(ChipId(b'I'), Dimension::X, 5)),
                NodeMapping(Node::NanoA6, Port(ChipId(b'I'), Dimension::X, 6)),
                NodeMapping(Node::NanoD7, Port(ChipId(b'I'), Dimension::X, 7)),
                NodeMapping(Node::NanoD11, Port(ChipId(b'I'), Dimension::X, 8)),
                NodeMapping(Node::NanoD9, Port(ChipId(b'I'), Dimension::X, 9)),
                NodeMapping(Node::NanoD13, Port(ChipId(b'I'), Dimension::X, 10)),
                NodeMapping(Node::NanoReset, Port(ChipId(b'I'), Dimension::X, 11)),
                NodeMapping(Node::Dac05V, Port(ChipId(b'I'), Dimension::X, 12)),
                NodeMapping(Node::Adc05V, Port(ChipId(b'I'), Dimension::X, 13)),
                NodeMapping(Node::Supply3V3, Port(ChipId(b'I'), Dimension::X, 14)),
                NodeMapping(Node::Gnd, Port(ChipId(b'I'), Dimension::X, 15)),
                NodeMapping(Node::NanoD0, Port(ChipId(b'J'), Dimension::X, 0)),
                NodeMapping(Node::NanoA1, Port(ChipId(b'J'), Dimension::X, 1)),
                NodeMapping(Node::NanoD2, Port(ChipId(b'J'), Dimension::X, 2)),
                NodeMapping(Node::NanoA3, Port(ChipId(b'J'), Dimension::X, 3)),
                NodeMapping(Node::NanoD4, Port(ChipId(b'J'), Dimension::X, 4)),
                NodeMapping(Node::NanoA5, Port(ChipId(b'J'), Dimension::X, 5)),
                NodeMapping(Node::NanoD6, Port(ChipId(b'J'), Dimension::X, 6)),
                NodeMapping(Node::NanoA7, Port(ChipId(b'J'), Dimension::X, 7)),
                NodeMapping(Node::NanoD8, Port(ChipId(b'J'), Dimension::X, 8)),
                NodeMapping(Node::NanoD10, Port(ChipId(b'J'), Dimension::X, 9)),
                NodeMapping(Node::NanoD12, Port(ChipId(b'J'), Dimension::X, 10)),
                NodeMapping(Node::NanoAref, Port(ChipId(b'J'), Dimension::X, 11)),
                NodeMapping(Node::Dac18V, Port(ChipId(b'J'), Dimension::X, 12)),
                NodeMapping(Node::Adc15V, Port(ChipId(b'J'), Dimension::X, 13)),
                NodeMapping(Node::Supply5V, Port(ChipId(b'J'), Dimension::X, 14)),
                NodeMapping(Node::Gnd, Port(ChipId(b'J'), Dimension::X, 15)),
                NodeMapping(Node::NanoA0, Port(ChipId(b'K'), Dimension::X, 0)),
                NodeMapping(Node::NanoA1, Port(ChipId(b'K'), Dimension::X, 1)),
                NodeMapping(Node::NanoA2, Port(ChipId(b'K'), Dimension::X, 2)),
                NodeMapping(Node::NanoA3, Port(ChipId(b'K'), Dimension::X, 3)),
                NodeMapping(Node::NanoD2, Port(ChipId(b'K'), Dimension::X, 4)),
                NodeMapping(Node::NanoD3, Port(ChipId(b'K'), Dimension::X, 5)),
                NodeMapping(Node::NanoD4, Port(ChipId(b'K'), Dimension::X, 6)),
                NodeMapping(Node::NanoD5, Port(ChipId(b'K'), Dimension::X, 7)),
                NodeMapping(Node::NanoD6, Port(ChipId(b'K'), Dimension::X, 8)),
                NodeMapping(Node::NanoD7, Port(ChipId(b'K'), Dimension::X, 9)),
                NodeMapping(Node::NanoD8, Port(ChipId(b'K'), Dimension::X, 10)),
                NodeMapping(Node::NanoD9, Port(ChipId(b'K'), Dimension::X, 11)),
                NodeMapping(Node::NanoD10, Port(ChipId(b'K'), Dimension::X, 12)),
                NodeMapping(Node::NanoD11, Port(ChipId(b'K'), Dimension::X, 13)),
                NodeMapping(Node::NanoD12, Port(ChipId(b'K'), Dimension::X, 14)),
                NodeMapping(Node::Adc25V, Port(ChipId(b'K'), Dimension::X, 15)),
                NodeMapping(Node::CurrentSenseMinus, Port(ChipId(b'L'), Dimension::X, 0)),
                NodeMapping(Node::CurrentSensePlus, Port(ChipId(b'L'), Dimension::X, 1)),
                NodeMapping(Node::Adc05V, Port(ChipId(b'L'), Dimension::X, 2)),
                NodeMapping(Node::Adc15V, Port(ChipId(b'L'), Dimension::X, 3)),
                NodeMapping(Node::Adc25V, Port(ChipId(b'L'), Dimension::X, 4)),
                NodeMapping(Node::Adc38V, Port(ChipId(b'L'), Dimension::X, 5)),
                NodeMapping(Node::Dac18V, Port(ChipId(b'L'), Dimension::X, 6)),
                NodeMapping(Node::Dac05V, Port(ChipId(b'L'), Dimension::X, 7)),
                NodeMapping(Node::Top1, Port(ChipId(b'L'), Dimension::X, 8)),
                NodeMapping(Node::Top30, Port(ChipId(b'L'), Dimension::X, 9)),
                NodeMapping(Node::Bottom1, Port(ChipId(b'L'), Dimension::X, 10)),
                NodeMapping(Node::Bottom30, Port(ChipId(b'L'), Dimension::X, 11)),
                NodeMapping(Node::RpUartTx, Port(ChipId(b'L'), Dimension::X, 12)),
                NodeMapping(Node::RpUartRx, Port(ChipId(b'L'), Dimension::X, 13)),
                NodeMapping(Node::Supply5V, Port(ChipId(b'L'), Dimension::X, 14)),
                NodeMapping(Node::RpGpio0, Port(ChipId(b'L'), Dimension::X, 15)),
            ], [
                Lane(Port(ChipId(b'A'), Dimension::X, 0), Port(ChipId(b'I'), Dimension::Y, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 1), Port(ChipId(b'J'), Dimension::Y, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 2), Port(ChipId(b'B'), Dimension::X, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 3), Port(ChipId(b'B'), Dimension::X, 1)),
                Lane(Port(ChipId(b'A'), Dimension::X, 4), Port(ChipId(b'C'), Dimension::X, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 5), Port(ChipId(b'C'), Dimension::X, 1)),
                Lane(Port(ChipId(b'A'), Dimension::X, 6), Port(ChipId(b'D'), Dimension::X, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 7), Port(ChipId(b'D'), Dimension::X, 1)),
                Lane(Port(ChipId(b'A'), Dimension::X, 8), Port(ChipId(b'E'), Dimension::X, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 9), Port(ChipId(b'K'), Dimension::Y, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 10), Port(ChipId(b'F'), Dimension::X, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 11), Port(ChipId(b'F'), Dimension::X, 1)),
                Lane(Port(ChipId(b'A'), Dimension::X, 12), Port(ChipId(b'G'), Dimension::X, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 13), Port(ChipId(b'G'), Dimension::X, 1)),
                Lane(Port(ChipId(b'A'), Dimension::X, 14), Port(ChipId(b'H'), Dimension::X, 0)),
                Lane(Port(ChipId(b'A'), Dimension::X, 15), Port(ChipId(b'H'), Dimension::X, 1)),
                Lane(Port(ChipId(b'A'), Dimension::Y, 0), Port(ChipId(b'L'), Dimension::Y, 0)),
                Lane(Port(ChipId(b'B'), Dimension::X, 2), Port(ChipId(b'I'), Dimension::Y, 1)),
                Lane(Port(ChipId(b'B'), Dimension::X, 3), Port(ChipId(b'J'), Dimension::Y, 1)),
                Lane(Port(ChipId(b'B'), Dimension::X, 4), Port(ChipId(b'C'), Dimension::X, 2)),
                Lane(Port(ChipId(b'B'), Dimension::X, 5), Port(ChipId(b'C'), Dimension::X, 3)),
                Lane(Port(ChipId(b'B'), Dimension::X, 6), Port(ChipId(b'D'), Dimension::X, 2)),
                Lane(Port(ChipId(b'B'), Dimension::X, 7), Port(ChipId(b'D'), Dimension::X, 3)),
                Lane(Port(ChipId(b'B'), Dimension::X, 8), Port(ChipId(b'E'), Dimension::X, 2)),
                Lane(Port(ChipId(b'B'), Dimension::X, 9), Port(ChipId(b'E'), Dimension::X, 3)),
                Lane(Port(ChipId(b'B'), Dimension::X, 10), Port(ChipId(b'F'), Dimension::X, 2)),
                Lane(Port(ChipId(b'B'), Dimension::X, 11), Port(ChipId(b'K'), Dimension::Y, 1)),
                Lane(Port(ChipId(b'B'), Dimension::X, 12), Port(ChipId(b'G'), Dimension::X, 2)),
                Lane(Port(ChipId(b'B'), Dimension::X, 13), Port(ChipId(b'G'), Dimension::X, 3)),
                Lane(Port(ChipId(b'B'), Dimension::X, 14), Port(ChipId(b'H'), Dimension::X, 2)),
                Lane(Port(ChipId(b'B'), Dimension::X, 15), Port(ChipId(b'H'), Dimension::X, 3)),
                Lane(Port(ChipId(b'B'), Dimension::Y, 0), Port(ChipId(b'L'), Dimension::Y, 1)),
                Lane(Port(ChipId(b'C'), Dimension::X, 4), Port(ChipId(b'I'), Dimension::Y, 2)),
                Lane(Port(ChipId(b'C'), Dimension::X, 5), Port(ChipId(b'J'), Dimension::Y, 2)),
                Lane(Port(ChipId(b'C'), Dimension::X, 6), Port(ChipId(b'D'), Dimension::X, 4)),
                Lane(Port(ChipId(b'C'), Dimension::X, 7), Port(ChipId(b'D'), Dimension::X, 5)),
                Lane(Port(ChipId(b'C'), Dimension::X, 8), Port(ChipId(b'E'), Dimension::X, 4)),
                Lane(Port(ChipId(b'C'), Dimension::X, 9), Port(ChipId(b'E'), Dimension::X, 5)),
                Lane(Port(ChipId(b'C'), Dimension::X, 10), Port(ChipId(b'F'), Dimension::X, 4)),
                Lane(Port(ChipId(b'C'), Dimension::X, 11), Port(ChipId(b'F'), Dimension::X, 5)),
                Lane(Port(ChipId(b'C'), Dimension::X, 12), Port(ChipId(b'G'), Dimension::X, 4)),
                Lane(Port(ChipId(b'C'), Dimension::X, 13), Port(ChipId(b'K'), Dimension::Y, 2)),
                Lane(Port(ChipId(b'C'), Dimension::X, 14), Port(ChipId(b'H'), Dimension::X, 4)),
                Lane(Port(ChipId(b'C'), Dimension::X, 15), Port(ChipId(b'H'), Dimension::X, 5)),
                Lane(Port(ChipId(b'C'), Dimension::Y, 0), Port(ChipId(b'L'), Dimension::Y, 2)),
                Lane(Port(ChipId(b'D'), Dimension::X, 6), Port(ChipId(b'I'), Dimension::Y, 3)),
                Lane(Port(ChipId(b'D'), Dimension::X, 7), Port(ChipId(b'J'), Dimension::Y, 3)),
                Lane(Port(ChipId(b'D'), Dimension::X, 8), Port(ChipId(b'E'), Dimension::X, 6)),
                Lane(Port(ChipId(b'D'), Dimension::X, 9), Port(ChipId(b'E'), Dimension::X, 7)),
                Lane(Port(ChipId(b'D'), Dimension::X, 10), Port(ChipId(b'F'), Dimension::X, 6)),
                Lane(Port(ChipId(b'D'), Dimension::X, 11), Port(ChipId(b'F'), Dimension::X, 7)),
                Lane(Port(ChipId(b'D'), Dimension::X, 12), Port(ChipId(b'G'), Dimension::X, 6)),
                Lane(Port(ChipId(b'D'), Dimension::X, 13), Port(ChipId(b'G'), Dimension::X, 7)),
                Lane(Port(ChipId(b'D'), Dimension::X, 14), Port(ChipId(b'H'), Dimension::X, 6)),
                Lane(Port(ChipId(b'D'), Dimension::X, 15), Port(ChipId(b'K'), Dimension::Y, 3)),
                Lane(Port(ChipId(b'D'), Dimension::Y, 0), Port(ChipId(b'L'), Dimension::Y, 3)),
                Lane(Port(ChipId(b'E'), Dimension::X, 1), Port(ChipId(b'K'), Dimension::Y, 4)),
                Lane(Port(ChipId(b'E'), Dimension::X, 8), Port(ChipId(b'I'), Dimension::Y, 4)),
                Lane(Port(ChipId(b'E'), Dimension::X, 9), Port(ChipId(b'J'), Dimension::Y, 4)),
                Lane(Port(ChipId(b'E'), Dimension::X, 10), Port(ChipId(b'F'), Dimension::X, 8)),
                Lane(Port(ChipId(b'E'), Dimension::X, 11), Port(ChipId(b'F'), Dimension::X, 9)),
                Lane(Port(ChipId(b'E'), Dimension::X, 12), Port(ChipId(b'G'), Dimension::X, 8)),
                Lane(Port(ChipId(b'E'), Dimension::X, 13), Port(ChipId(b'G'), Dimension::X, 9)),
                Lane(Port(ChipId(b'E'), Dimension::X, 14), Port(ChipId(b'H'), Dimension::X, 8)),
                Lane(Port(ChipId(b'E'), Dimension::X, 15), Port(ChipId(b'H'), Dimension::X, 9)),
                Lane(Port(ChipId(b'E'), Dimension::Y, 0), Port(ChipId(b'L'), Dimension::Y, 4)),
                Lane(Port(ChipId(b'F'), Dimension::X, 3), Port(ChipId(b'K'), Dimension::Y, 5)),
                Lane(Port(ChipId(b'F'), Dimension::X, 10), Port(ChipId(b'I'), Dimension::Y, 5)),
                Lane(Port(ChipId(b'F'), Dimension::X, 11), Port(ChipId(b'J'), Dimension::Y, 5)),
                Lane(Port(ChipId(b'F'), Dimension::X, 12), Port(ChipId(b'G'), Dimension::X, 10)),
                Lane(Port(ChipId(b'F'), Dimension::X, 13), Port(ChipId(b'G'), Dimension::X, 11)),
                Lane(Port(ChipId(b'F'), Dimension::X, 14), Port(ChipId(b'H'), Dimension::X, 10)),
                Lane(Port(ChipId(b'F'), Dimension::X, 15), Port(ChipId(b'H'), Dimension::X, 11)),
                Lane(Port(ChipId(b'F'), Dimension::Y, 0), Port(ChipId(b'L'), Dimension::Y, 5)),
                Lane(Port(ChipId(b'G'), Dimension::X, 5), Port(ChipId(b'K'), Dimension::Y, 6)),
                Lane(Port(ChipId(b'G'), Dimension::X, 12), Port(ChipId(b'I'), Dimension::Y, 6)),
                Lane(Port(ChipId(b'G'), Dimension::X, 13), Port(ChipId(b'J'), Dimension::Y, 6)),
                Lane(Port(ChipId(b'G'), Dimension::X, 14), Port(ChipId(b'H'), Dimension::X, 12)),
                Lane(Port(ChipId(b'G'), Dimension::X, 15), Port(ChipId(b'H'), Dimension::X, 13)),
                Lane(Port(ChipId(b'G'), Dimension::Y, 0), Port(ChipId(b'L'), Dimension::Y, 6)),
                Lane(Port(ChipId(b'H'), Dimension::X, 7), Port(ChipId(b'K'), Dimension::Y, 7)),
                Lane(Port(ChipId(b'H'), Dimension::X, 14), Port(ChipId(b'I'), Dimension::Y, 7)),
                Lane(Port(ChipId(b'H'), Dimension::X, 15), Port(ChipId(b'J'), Dimension::Y, 7)),
                Lane(Port(ChipId(b'H'), Dimension::Y, 0), Port(ChipId(b'L'), Dimension::Y, 7)),
            ],
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Node {
    Top1 = 1,
    Top2 = 2,
    Top3 = 3,
    Top4 = 4,
    Top5 = 5,
    Top6 = 6,
    Top7 = 7,
    Top8 = 8,
    Top9 = 9,
    Top10 = 10,
    Top11 = 11,
    Top12 = 12,
    Top13 = 13,
    Top14 = 14,
    Top15 = 15,
    Top16 = 16,
    Top17 = 17,
    Top18 = 18,
    Top19 = 19,
    Top20 = 20,
    Top21 = 21,
    Top22 = 22,
    Top23 = 23,
    Top24 = 24,
    Top25 = 25,
    Top26 = 26,
    Top27 = 27,
    Top28 = 28,
    Top29 = 29,
    Top30 = 30,
    Bottom1 = 31,
    Bottom2 = 32,
    Bottom3 = 33,
    Bottom4 = 34,
    Bottom5 = 35,
    Bottom6 = 36,
    Bottom7 = 37,
    Bottom8 = 38,
    Bottom9 = 39,
    Bottom10 = 40,
    Bottom11 = 41,
    Bottom12 = 42,
    Bottom13 = 43,
    Bottom14 = 44,
    Bottom15 = 45,
    Bottom16 = 46,
    Bottom17 = 47,
    Bottom18 = 48,
    Bottom19 = 49,
    Bottom20 = 50,
    Bottom21 = 51,
    Bottom22 = 52,
    Bottom23 = 53,
    Bottom24 = 54,
    Bottom25 = 55,
    Bottom26 = 56,
    Bottom27 = 57,
    Bottom28 = 58,
    Bottom29 = 59,
    Bottom30 = 60,

    NanoD0 = 70,
    NanoD1 = 71,
    NanoD2 = 72,
    NanoD3 = 73,
    NanoD4 = 74,
    NanoD5 = 75,
    NanoD6 = 76,
    NanoD7 = 77,
    NanoD8 = 78,
    NanoD9 = 79,
    NanoD10 = 80,
    NanoD11 = 81,
    NanoD12 = 82,
    NanoD13 = 83,
    NanoReset = 84,
    NanoAref = 85,
    NanoA0 = 86,
    NanoA1 = 87,
    NanoA2 = 88,
    NanoA3 = 89,
    NanoA4 = 90,
    NanoA5 = 91,
    NanoA6 = 92,
    NanoA7 = 93,

    Gnd = 100,
    Supply3V3 = 103,
    Supply5V = 105,
    Dac05V = 106,
    Dac18V = 107,
    CurrentSensePlus = 108,
    CurrentSenseMinus = 109,
    Adc05V = 110,
    Adc15V = 111,
    Adc25V = 112,
    Adc38V = 113,
    RpGpio0 = 114,
    RpUartTx = 116,
    RpUartRx = 117,
}

impl Node {
    unsafe fn from_u8(value: u8) -> Self {
        core::mem::transmute(value)
    }
}

#[derive(Debug)]
pub struct InvalidNode;

#[cfg(feature = "std")]
impl std::str::FromStr for Node {
    type Err = InvalidNode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Node::*;
        match s {
            "GND" => Ok(Gnd),
            "3V3" => Ok(Supply3V3),
            "5V" => Ok(Supply5V),
            "DAC0" => Ok(Dac05V),
            "DAC1" => Ok(Dac18V),
            "1" => Ok(Top1),
            "2" => Ok(Top2),
            "3" => Ok(Top3),
            "4" => Ok(Top4),
            "5" => Ok(Top5),
            "6" => Ok(Top6),
            "7" => Ok(Top7),
            "8" => Ok(Top8),
            "9" => Ok(Top9),
            "10" => Ok(Top10),
            "11" => Ok(Top11),
            "12" => Ok(Top12),
            "13" => Ok(Top13),
            "14" => Ok(Top14),
            "15" => Ok(Top15),
            "16" => Ok(Top16),
            "17" => Ok(Top17),
            "18" => Ok(Top18),
            "19" => Ok(Top19),
            "20" => Ok(Top20),
            "21" => Ok(Top21),
            "22" => Ok(Top22),
            "23" => Ok(Top23),
            "24" => Ok(Top24),
            "25" => Ok(Top25),
            "26" => Ok(Top26),
            "27" => Ok(Top27),
            "28" => Ok(Top28),
            "29" => Ok(Top29),
            "30" => Ok(Top30),
            "31" => Ok(Bottom1),
            "32" => Ok(Bottom2),
            "33" => Ok(Bottom3),
            "34" => Ok(Bottom4),
            "35" => Ok(Bottom5),
            "36" => Ok(Bottom6),
            "37" => Ok(Bottom7),
            "38" => Ok(Bottom8),
            "39" => Ok(Bottom9),
            "40" => Ok(Bottom10),
            "41" => Ok(Bottom11),
            "42" => Ok(Bottom12),
            "43" => Ok(Bottom13),
            "44" => Ok(Bottom14),
            "45" => Ok(Bottom15),
            "46" => Ok(Bottom16),
            "47" => Ok(Bottom17),
            "48" => Ok(Bottom18),
            "49" => Ok(Bottom19),
            "50" => Ok(Bottom20),
            "51" => Ok(Bottom21),
            "52" => Ok(Bottom22),
            "53" => Ok(Bottom23),
            "54" => Ok(Bottom24),
            "55" => Ok(Bottom25),
            "56" => Ok(Bottom26),
            "57" => Ok(Bottom27),
            "58" => Ok(Bottom28),
            "59" => Ok(Bottom29),
            "60" => Ok(Bottom30),

            "D0" => Ok(NanoD0),
            "D1" => Ok(NanoD1),
            "D2" => Ok(NanoD2),
            "D3" => Ok(NanoD3),
            "D4" => Ok(NanoD4),
            "D5" => Ok(NanoD5),
            "D6" => Ok(NanoD6),
            "D7" => Ok(NanoD7),
            "D8" => Ok(NanoD8),
            "D9" => Ok(NanoD9),
            "D10" => Ok(NanoD10),
            "D11" => Ok(NanoD11),
            "D12" => Ok(NanoD12),
            "D13" => Ok(NanoD13),

            "A0" => Ok(NanoA0),
            "A1" => Ok(NanoA1),
            "A2" => Ok(NanoA2),
            "A3" => Ok(NanoA3),
            "A4" => Ok(NanoA4),
            "A5" => Ok(NanoA5),
            "A6" => Ok(NanoA6),
            "A7" => Ok(NanoA7),

            _ => Err(InvalidNode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_use_none() {
        let none = PortMapEntry::new_none();
        assert_eq!(none.node(), None);
        assert_eq!(none.lane_index(), None);
    }

    #[test]
    fn test_port_use_node() {
        let node = PortMapEntry::new_node(Node::Supply5V);
        assert_eq!(node.node(), Some(Node::Supply5V));
        assert_eq!(node.lane_index(), None);
    }

    #[test]
    fn test_port_use_index() {
        let lane_index = PortMapEntry::new_lane_index(27);
        assert_eq!(lane_index.node(), None);
        assert_eq!(lane_index.lane_index(), Some(27));
    }
}

