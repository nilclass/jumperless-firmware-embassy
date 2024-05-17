use crate::{ChipPort, Lane, ChipId, Dimension, Net, ChipStatus, NetId, util::ChipPortBitMap};

pub struct NodeMapping(Node, ChipPort);

pub struct Layout<const NODE_COUNT: usize, const LANE_COUNT: usize> {
    pub nodes: [NodeMapping; NODE_COUNT],
    pub lanes: [Lane; LANE_COUNT],
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
    pub fn nets_to_connections(&self, nets: &[NodeNet], chip_status: &mut ChipStatus) {
        super::nets_to_connections(nets.iter().map(|net| {
            Net {
                id: net.id,
                ports: net.nodes.iter().map(|node| self.node_to_port(*node).unwrap()).collect(),
            }
        }), chip_status, &self.lanes)
    }

    pub fn node_to_port(&self, node: Node) -> Option<ChipPort> {
        self.nodes.iter().find(|NodeMapping(n, _)| *n == node).map(|NodeMapping(_, p)| p).copied()
    }

    pub fn port_to_node(&self, port: ChipPort) -> Option<Node> {
        self.nodes.iter().find(|NodeMapping(_, p)| *p == port).map(|NodeMapping(n, _)| n).copied()
    }

    /// If the given port is part of a lane, returns the port on the other side of that lane
    pub fn lane_destination(&self, port: ChipPort) -> Option<ChipPort> {
        if let Some(Lane(a, b)) = self.lanes.iter().find(|l| l.0 == port || l.1 == port).copied() {
            if a == port {
                Some(b)
            } else {
                Some(a)
            }
        } else {
            None
        }
    }

    /// Verify that all possible ports are referenced by exactly one node mapping or exactly one lane.
    ///
    /// Prints problems to stdout and panics if a check has failed.
    pub fn sanity_check(&self) {
        let mut problems = vec![];
        let mut used_ports = ChipPortBitMap::empty();
        for NodeMapping(node, port) in &self.nodes {
            if used_ports.get(*port) {
                problems.push(format!("Port {port:?} used more than once (last use in node mapping {node:?})"));
            }
            used_ports.set(*port);
        }
        for Lane(a, b) in &self.lanes {
            if used_ports.get(*a) {
                problems.push(format!("Port {a:?} used more than once (last use in lane with port {b:?})"));
            }
            used_ports.set(*a);
            if used_ports.get(*b) {
                problems.push(format!("Port {b:?} used more than once (last use in lane with port {a:?})"));
            }
            used_ports.set(*b);
        }
        for problem in &problems {
            println!("Found problem: {}", problem);
        }
        let expected = ChipPortBitMap::full();
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

impl Layout<120, 84> {
    pub fn v4() -> Self {
        Self {
            nodes: [
                NodeMapping(Node::Top2, ChipPort(ChipId(b'A'), Dimension::Y, 1)),
                NodeMapping(Node::Top3, ChipPort(ChipId(b'A'), Dimension::Y, 2)),
                NodeMapping(Node::Top4, ChipPort(ChipId(b'A'), Dimension::Y, 3)),
                NodeMapping(Node::Top5, ChipPort(ChipId(b'A'), Dimension::Y, 4)),
                NodeMapping(Node::Top6, ChipPort(ChipId(b'A'), Dimension::Y, 5)),
                NodeMapping(Node::Top7, ChipPort(ChipId(b'A'), Dimension::Y, 6)),
                NodeMapping(Node::Top8, ChipPort(ChipId(b'A'), Dimension::Y, 7)),
                NodeMapping(Node::Top9, ChipPort(ChipId(b'B'), Dimension::Y, 1)),
                NodeMapping(Node::Top10, ChipPort(ChipId(b'B'), Dimension::Y, 2)),
                NodeMapping(Node::Top11, ChipPort(ChipId(b'B'), Dimension::Y, 3)),
                NodeMapping(Node::Top12, ChipPort(ChipId(b'B'), Dimension::Y, 4)),
                NodeMapping(Node::Top13, ChipPort(ChipId(b'B'), Dimension::Y, 5)),
                NodeMapping(Node::Top14, ChipPort(ChipId(b'B'), Dimension::Y, 6)),
                NodeMapping(Node::Top15, ChipPort(ChipId(b'B'), Dimension::Y, 7)),
                NodeMapping(Node::Top16, ChipPort(ChipId(b'C'), Dimension::Y, 1)),
                NodeMapping(Node::Top17, ChipPort(ChipId(b'C'), Dimension::Y, 2)),
                NodeMapping(Node::Top18, ChipPort(ChipId(b'C'), Dimension::Y, 3)),
                NodeMapping(Node::Top19, ChipPort(ChipId(b'C'), Dimension::Y, 4)),
                NodeMapping(Node::Top20, ChipPort(ChipId(b'C'), Dimension::Y, 5)),
                NodeMapping(Node::Top21, ChipPort(ChipId(b'C'), Dimension::Y, 6)),
                NodeMapping(Node::Top22, ChipPort(ChipId(b'C'), Dimension::Y, 7)),
                NodeMapping(Node::Top23, ChipPort(ChipId(b'D'), Dimension::Y, 1)),
                NodeMapping(Node::Top24, ChipPort(ChipId(b'D'), Dimension::Y, 2)),
                NodeMapping(Node::Top25, ChipPort(ChipId(b'D'), Dimension::Y, 3)),
                NodeMapping(Node::Top26, ChipPort(ChipId(b'D'), Dimension::Y, 4)),
                NodeMapping(Node::Top27, ChipPort(ChipId(b'D'), Dimension::Y, 5)),
                NodeMapping(Node::Top28, ChipPort(ChipId(b'D'), Dimension::Y, 6)),
                NodeMapping(Node::Top29, ChipPort(ChipId(b'D'), Dimension::Y, 7)),
                NodeMapping(Node::Bottom2, ChipPort(ChipId(b'E'), Dimension::Y, 1)),
                NodeMapping(Node::Bottom3, ChipPort(ChipId(b'E'), Dimension::Y, 2)),
                NodeMapping(Node::Bottom4, ChipPort(ChipId(b'E'), Dimension::Y, 3)),
                NodeMapping(Node::Bottom5, ChipPort(ChipId(b'E'), Dimension::Y, 4)),
                NodeMapping(Node::Bottom6, ChipPort(ChipId(b'E'), Dimension::Y, 5)),
                NodeMapping(Node::Bottom7, ChipPort(ChipId(b'E'), Dimension::Y, 6)),
                NodeMapping(Node::Bottom8, ChipPort(ChipId(b'E'), Dimension::Y, 7)),
                NodeMapping(Node::Bottom9, ChipPort(ChipId(b'F'), Dimension::Y, 1)),
                NodeMapping(Node::Bottom10, ChipPort(ChipId(b'F'), Dimension::Y, 2)),
                NodeMapping(Node::Bottom11, ChipPort(ChipId(b'F'), Dimension::Y, 3)),
                NodeMapping(Node::Bottom12, ChipPort(ChipId(b'F'), Dimension::Y, 4)),
                NodeMapping(Node::Bottom13, ChipPort(ChipId(b'F'), Dimension::Y, 5)),
                NodeMapping(Node::Bottom14, ChipPort(ChipId(b'F'), Dimension::Y, 6)),
                NodeMapping(Node::Bottom15, ChipPort(ChipId(b'F'), Dimension::Y, 7)),
                NodeMapping(Node::Bottom16, ChipPort(ChipId(b'G'), Dimension::Y, 1)),
                NodeMapping(Node::Bottom17, ChipPort(ChipId(b'G'), Dimension::Y, 2)),
                NodeMapping(Node::Bottom18, ChipPort(ChipId(b'G'), Dimension::Y, 3)),
                NodeMapping(Node::Bottom19, ChipPort(ChipId(b'G'), Dimension::Y, 4)),
                NodeMapping(Node::Bottom20, ChipPort(ChipId(b'G'), Dimension::Y, 5)),
                NodeMapping(Node::Bottom21, ChipPort(ChipId(b'G'), Dimension::Y, 6)),
                NodeMapping(Node::Bottom22, ChipPort(ChipId(b'G'), Dimension::Y, 7)),
                NodeMapping(Node::Bottom23, ChipPort(ChipId(b'H'), Dimension::Y, 1)),
                NodeMapping(Node::Bottom24, ChipPort(ChipId(b'H'), Dimension::Y, 2)),
                NodeMapping(Node::Bottom25, ChipPort(ChipId(b'H'), Dimension::Y, 3)),
                NodeMapping(Node::Bottom26, ChipPort(ChipId(b'H'), Dimension::Y, 4)),
                NodeMapping(Node::Bottom27, ChipPort(ChipId(b'H'), Dimension::Y, 5)),
                NodeMapping(Node::Bottom28, ChipPort(ChipId(b'H'), Dimension::Y, 6)),
                NodeMapping(Node::Bottom29, ChipPort(ChipId(b'H'), Dimension::Y, 7)),
                NodeMapping(Node::NanoA0, ChipPort(ChipId(b'I'), Dimension::X, 0)),
                NodeMapping(Node::NanoD1, ChipPort(ChipId(b'I'), Dimension::X, 1)),
                NodeMapping(Node::NanoA2, ChipPort(ChipId(b'I'), Dimension::X, 2)),
                NodeMapping(Node::NanoD3, ChipPort(ChipId(b'I'), Dimension::X, 3)),
                NodeMapping(Node::NanoA4, ChipPort(ChipId(b'I'), Dimension::X, 4)),
                NodeMapping(Node::NanoD5, ChipPort(ChipId(b'I'), Dimension::X, 5)),
                NodeMapping(Node::NanoA6, ChipPort(ChipId(b'I'), Dimension::X, 6)),
                NodeMapping(Node::NanoD7, ChipPort(ChipId(b'I'), Dimension::X, 7)),
                NodeMapping(Node::NanoD11, ChipPort(ChipId(b'I'), Dimension::X, 8)),
                NodeMapping(Node::NanoD9, ChipPort(ChipId(b'I'), Dimension::X, 9)),
                NodeMapping(Node::NanoD13, ChipPort(ChipId(b'I'), Dimension::X, 10)),
                NodeMapping(Node::NanoReset, ChipPort(ChipId(b'I'), Dimension::X, 11)),
                NodeMapping(Node::Dac05V, ChipPort(ChipId(b'I'), Dimension::X, 12)),
                NodeMapping(Node::Adc05V, ChipPort(ChipId(b'I'), Dimension::X, 13)),
                NodeMapping(Node::Supply3V3, ChipPort(ChipId(b'I'), Dimension::X, 14)),
                NodeMapping(Node::Gnd, ChipPort(ChipId(b'I'), Dimension::X, 15)),
                NodeMapping(Node::NanoD0, ChipPort(ChipId(b'J'), Dimension::X, 0)),
                NodeMapping(Node::NanoA1, ChipPort(ChipId(b'J'), Dimension::X, 1)),
                NodeMapping(Node::NanoD2, ChipPort(ChipId(b'J'), Dimension::X, 2)),
                NodeMapping(Node::NanoA3, ChipPort(ChipId(b'J'), Dimension::X, 3)),
                NodeMapping(Node::NanoD4, ChipPort(ChipId(b'J'), Dimension::X, 4)),
                NodeMapping(Node::NanoA5, ChipPort(ChipId(b'J'), Dimension::X, 5)),
                NodeMapping(Node::NanoD6, ChipPort(ChipId(b'J'), Dimension::X, 6)),
                NodeMapping(Node::NanoA7, ChipPort(ChipId(b'J'), Dimension::X, 7)),
                NodeMapping(Node::NanoD8, ChipPort(ChipId(b'J'), Dimension::X, 8)),
                NodeMapping(Node::NanoD10, ChipPort(ChipId(b'J'), Dimension::X, 9)),
                NodeMapping(Node::NanoD12, ChipPort(ChipId(b'J'), Dimension::X, 10)),
                NodeMapping(Node::NanoAref, ChipPort(ChipId(b'J'), Dimension::X, 11)),
                NodeMapping(Node::Dac18V, ChipPort(ChipId(b'J'), Dimension::X, 12)),
                NodeMapping(Node::Adc15V, ChipPort(ChipId(b'J'), Dimension::X, 13)),
                NodeMapping(Node::Supply5V, ChipPort(ChipId(b'J'), Dimension::X, 14)),
                NodeMapping(Node::Gnd, ChipPort(ChipId(b'J'), Dimension::X, 15)),
                NodeMapping(Node::NanoA0, ChipPort(ChipId(b'K'), Dimension::X, 0)),
                NodeMapping(Node::NanoA1, ChipPort(ChipId(b'K'), Dimension::X, 1)),
                NodeMapping(Node::NanoA2, ChipPort(ChipId(b'K'), Dimension::X, 2)),
                NodeMapping(Node::NanoA3, ChipPort(ChipId(b'K'), Dimension::X, 3)),
                NodeMapping(Node::NanoD2, ChipPort(ChipId(b'K'), Dimension::X, 4)),
                NodeMapping(Node::NanoD3, ChipPort(ChipId(b'K'), Dimension::X, 5)),
                NodeMapping(Node::NanoD4, ChipPort(ChipId(b'K'), Dimension::X, 6)),
                NodeMapping(Node::NanoD5, ChipPort(ChipId(b'K'), Dimension::X, 7)),
                NodeMapping(Node::NanoD6, ChipPort(ChipId(b'K'), Dimension::X, 8)),
                NodeMapping(Node::NanoD7, ChipPort(ChipId(b'K'), Dimension::X, 9)),
                NodeMapping(Node::NanoD8, ChipPort(ChipId(b'K'), Dimension::X, 10)),
                NodeMapping(Node::NanoD9, ChipPort(ChipId(b'K'), Dimension::X, 11)),
                NodeMapping(Node::NanoD10, ChipPort(ChipId(b'K'), Dimension::X, 12)),
                NodeMapping(Node::NanoD11, ChipPort(ChipId(b'K'), Dimension::X, 13)),
                NodeMapping(Node::NanoD12, ChipPort(ChipId(b'K'), Dimension::X, 14)),
                NodeMapping(Node::Adc25V, ChipPort(ChipId(b'K'), Dimension::X, 15)),
                NodeMapping(Node::CurrentSenseMinus, ChipPort(ChipId(b'L'), Dimension::X, 0)),
                NodeMapping(Node::CurrentSensePlus, ChipPort(ChipId(b'L'), Dimension::X, 1)),
                NodeMapping(Node::Adc05V, ChipPort(ChipId(b'L'), Dimension::X, 2)),
                NodeMapping(Node::Adc15V, ChipPort(ChipId(b'L'), Dimension::X, 3)),
                NodeMapping(Node::Adc25V, ChipPort(ChipId(b'L'), Dimension::X, 4)),
                NodeMapping(Node::Adc38V, ChipPort(ChipId(b'L'), Dimension::X, 5)),
                NodeMapping(Node::Dac18V, ChipPort(ChipId(b'L'), Dimension::X, 6)),
                NodeMapping(Node::Dac05V, ChipPort(ChipId(b'L'), Dimension::X, 7)),
                NodeMapping(Node::Top1, ChipPort(ChipId(b'L'), Dimension::X, 8)),
                NodeMapping(Node::Top30, ChipPort(ChipId(b'L'), Dimension::X, 9)),
                NodeMapping(Node::Bottom1, ChipPort(ChipId(b'L'), Dimension::X, 10)),
                NodeMapping(Node::Bottom30, ChipPort(ChipId(b'L'), Dimension::X, 11)),
                NodeMapping(Node::RpUartTx, ChipPort(ChipId(b'L'), Dimension::X, 12)),
                NodeMapping(Node::RpUartRx, ChipPort(ChipId(b'L'), Dimension::X, 13)),
                NodeMapping(Node::Supply5V, ChipPort(ChipId(b'L'), Dimension::X, 14)),
                NodeMapping(Node::RpGpio0, ChipPort(ChipId(b'L'), Dimension::X, 15)),
            ],
            lanes: [
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 0), ChipPort(ChipId(b'I'), Dimension::Y, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 1), ChipPort(ChipId(b'J'), Dimension::Y, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 2), ChipPort(ChipId(b'B'), Dimension::X, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 3), ChipPort(ChipId(b'B'), Dimension::X, 1)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 4), ChipPort(ChipId(b'C'), Dimension::X, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 5), ChipPort(ChipId(b'C'), Dimension::X, 1)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 6), ChipPort(ChipId(b'D'), Dimension::X, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 7), ChipPort(ChipId(b'D'), Dimension::X, 1)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 8), ChipPort(ChipId(b'E'), Dimension::X, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 9), ChipPort(ChipId(b'K'), Dimension::Y, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 10), ChipPort(ChipId(b'F'), Dimension::X, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 11), ChipPort(ChipId(b'F'), Dimension::X, 1)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 12), ChipPort(ChipId(b'G'), Dimension::X, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 13), ChipPort(ChipId(b'G'), Dimension::X, 1)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 14), ChipPort(ChipId(b'H'), Dimension::X, 0)),
                Lane(ChipPort(ChipId(b'A'), Dimension::X, 15), ChipPort(ChipId(b'H'), Dimension::X, 1)),
                Lane(ChipPort(ChipId(b'A'), Dimension::Y, 0), ChipPort(ChipId(b'L'), Dimension::Y, 0)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 2), ChipPort(ChipId(b'I'), Dimension::Y, 1)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 3), ChipPort(ChipId(b'J'), Dimension::Y, 1)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 4), ChipPort(ChipId(b'C'), Dimension::X, 2)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 5), ChipPort(ChipId(b'C'), Dimension::X, 3)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 6), ChipPort(ChipId(b'D'), Dimension::X, 2)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 7), ChipPort(ChipId(b'D'), Dimension::X, 3)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 8), ChipPort(ChipId(b'E'), Dimension::X, 2)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 9), ChipPort(ChipId(b'E'), Dimension::X, 3)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 10), ChipPort(ChipId(b'F'), Dimension::X, 2)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 11), ChipPort(ChipId(b'K'), Dimension::Y, 1)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 12), ChipPort(ChipId(b'G'), Dimension::X, 2)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 13), ChipPort(ChipId(b'G'), Dimension::X, 3)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 14), ChipPort(ChipId(b'H'), Dimension::X, 2)),
                Lane(ChipPort(ChipId(b'B'), Dimension::X, 15), ChipPort(ChipId(b'H'), Dimension::X, 3)),
                Lane(ChipPort(ChipId(b'B'), Dimension::Y, 0), ChipPort(ChipId(b'L'), Dimension::Y, 1)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 4), ChipPort(ChipId(b'I'), Dimension::Y, 2)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 5), ChipPort(ChipId(b'J'), Dimension::Y, 2)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 6), ChipPort(ChipId(b'D'), Dimension::X, 4)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 7), ChipPort(ChipId(b'D'), Dimension::X, 5)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 8), ChipPort(ChipId(b'E'), Dimension::X, 4)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 9), ChipPort(ChipId(b'E'), Dimension::X, 5)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 10), ChipPort(ChipId(b'F'), Dimension::X, 4)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 11), ChipPort(ChipId(b'F'), Dimension::X, 5)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 12), ChipPort(ChipId(b'G'), Dimension::X, 4)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 13), ChipPort(ChipId(b'K'), Dimension::Y, 2)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 14), ChipPort(ChipId(b'H'), Dimension::X, 4)),
                Lane(ChipPort(ChipId(b'C'), Dimension::X, 15), ChipPort(ChipId(b'H'), Dimension::X, 5)),
                Lane(ChipPort(ChipId(b'C'), Dimension::Y, 0), ChipPort(ChipId(b'L'), Dimension::Y, 2)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 6), ChipPort(ChipId(b'I'), Dimension::Y, 3)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 7), ChipPort(ChipId(b'J'), Dimension::Y, 3)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 8), ChipPort(ChipId(b'E'), Dimension::X, 6)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 9), ChipPort(ChipId(b'E'), Dimension::X, 7)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 10), ChipPort(ChipId(b'F'), Dimension::X, 6)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 11), ChipPort(ChipId(b'F'), Dimension::X, 7)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 12), ChipPort(ChipId(b'G'), Dimension::X, 6)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 13), ChipPort(ChipId(b'G'), Dimension::X, 7)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 14), ChipPort(ChipId(b'H'), Dimension::X, 6)),
                Lane(ChipPort(ChipId(b'D'), Dimension::X, 15), ChipPort(ChipId(b'K'), Dimension::Y, 3)),
                Lane(ChipPort(ChipId(b'D'), Dimension::Y, 0), ChipPort(ChipId(b'L'), Dimension::Y, 3)),
                Lane(ChipPort(ChipId(b'E'), Dimension::X, 1), ChipPort(ChipId(b'K'), Dimension::Y, 4)),
                Lane(ChipPort(ChipId(b'E'), Dimension::X, 8), ChipPort(ChipId(b'I'), Dimension::Y, 4)),
                Lane(ChipPort(ChipId(b'E'), Dimension::X, 9), ChipPort(ChipId(b'J'), Dimension::Y, 4)),
                Lane(ChipPort(ChipId(b'E'), Dimension::X, 10), ChipPort(ChipId(b'F'), Dimension::X, 8)),
                Lane(ChipPort(ChipId(b'E'), Dimension::X, 11), ChipPort(ChipId(b'F'), Dimension::X, 9)),
                Lane(ChipPort(ChipId(b'E'), Dimension::X, 12), ChipPort(ChipId(b'G'), Dimension::X, 8)),
                Lane(ChipPort(ChipId(b'E'), Dimension::X, 13), ChipPort(ChipId(b'G'), Dimension::X, 9)),
                Lane(ChipPort(ChipId(b'E'), Dimension::X, 14), ChipPort(ChipId(b'H'), Dimension::X, 8)),
                Lane(ChipPort(ChipId(b'E'), Dimension::X, 15), ChipPort(ChipId(b'H'), Dimension::X, 9)),
                Lane(ChipPort(ChipId(b'E'), Dimension::Y, 0), ChipPort(ChipId(b'L'), Dimension::Y, 4)),
                Lane(ChipPort(ChipId(b'F'), Dimension::X, 3), ChipPort(ChipId(b'K'), Dimension::Y, 5)),
                Lane(ChipPort(ChipId(b'F'), Dimension::X, 10), ChipPort(ChipId(b'I'), Dimension::Y, 5)),
                Lane(ChipPort(ChipId(b'F'), Dimension::X, 11), ChipPort(ChipId(b'J'), Dimension::Y, 5)),
                Lane(ChipPort(ChipId(b'F'), Dimension::X, 12), ChipPort(ChipId(b'G'), Dimension::X, 10)),
                Lane(ChipPort(ChipId(b'F'), Dimension::X, 13), ChipPort(ChipId(b'G'), Dimension::X, 11)),
                Lane(ChipPort(ChipId(b'F'), Dimension::X, 14), ChipPort(ChipId(b'H'), Dimension::X, 10)),
                Lane(ChipPort(ChipId(b'F'), Dimension::X, 15), ChipPort(ChipId(b'H'), Dimension::X, 11)),
                Lane(ChipPort(ChipId(b'F'), Dimension::Y, 0), ChipPort(ChipId(b'L'), Dimension::Y, 5)),
                Lane(ChipPort(ChipId(b'G'), Dimension::X, 5), ChipPort(ChipId(b'K'), Dimension::Y, 6)),
                Lane(ChipPort(ChipId(b'G'), Dimension::X, 12), ChipPort(ChipId(b'I'), Dimension::Y, 6)),
                Lane(ChipPort(ChipId(b'G'), Dimension::X, 13), ChipPort(ChipId(b'J'), Dimension::Y, 6)),
                Lane(ChipPort(ChipId(b'G'), Dimension::X, 14), ChipPort(ChipId(b'H'), Dimension::X, 12)),
                Lane(ChipPort(ChipId(b'G'), Dimension::X, 15), ChipPort(ChipId(b'H'), Dimension::X, 13)),
                Lane(ChipPort(ChipId(b'G'), Dimension::Y, 0), ChipPort(ChipId(b'L'), Dimension::Y, 6)),
                Lane(ChipPort(ChipId(b'H'), Dimension::X, 7), ChipPort(ChipId(b'K'), Dimension::Y, 7)),
                Lane(ChipPort(ChipId(b'H'), Dimension::X, 14), ChipPort(ChipId(b'I'), Dimension::Y, 7)),
                Lane(ChipPort(ChipId(b'H'), Dimension::X, 15), ChipPort(ChipId(b'J'), Dimension::Y, 7)),
                Lane(ChipPort(ChipId(b'H'), Dimension::Y, 0), ChipPort(ChipId(b'L'), Dimension::Y, 7)),
            ],
        }
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

#[derive(Debug)]
pub struct InvalidNode;

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
