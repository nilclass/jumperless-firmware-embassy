use crate::board_spec::{BoardSpec, NodePort};
use crate::map::PortMap;
use crate::{Node, Port, Lane};

pub struct Board<N: Node, const NODE_PORT_COUNT: usize, const LANE_COUNT: usize, const BOUNCE_PORT_COUNT: usize> {
    spec: BoardSpec<N, NODE_PORT_COUNT, LANE_COUNT, BOUNCE_PORT_COUNT>,
    port_map: PortMap<N>,
}

impl<N: Node, const NODE_PORT_COUNT: usize, const LANE_COUNT: usize, const BOUNCE_PORT_COUNT: usize> Board<N, NODE_PORT_COUNT, LANE_COUNT, BOUNCE_PORT_COUNT> {
    pub fn new(spec: BoardSpec<N, NODE_PORT_COUNT, LANE_COUNT, BOUNCE_PORT_COUNT>) -> Self {
        let port_map = spec.create_port_map();
        Self {
            spec,
            port_map,
        }
    }

    pub fn node_to_port(&self, node: N) -> Option<Port> {
        self.spec.node_ports.iter().find(|NodePort(n, _)| *n == node).map(|NodePort(_, p)| p).copied()
    }

    pub fn port_to_lane(&self, port: Port) -> Option<Lane> {
        self.port_map.get_lane_index(port).map(move |index| self.spec.lanes[index])
    }
}
