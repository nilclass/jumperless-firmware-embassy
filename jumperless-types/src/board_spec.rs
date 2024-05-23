use crate::{Node, Port, Lane, map::PortMap};

#[cfg(feature = "board-spec-generator")]
pub mod generator;

pub struct BoardSpec<
    N: Node,
    const NODE_PORT_COUNT: usize,
    const LANE_COUNT: usize,
    const BOUNCE_PORT_COUNT: usize,
> {
    pub node_ports: [NodePort<N>; NODE_PORT_COUNT],
    pub lanes: [Lane; LANE_COUNT],
    pub bounce_ports: [Port; BOUNCE_PORT_COUNT],
}

#[derive(Debug)]
pub struct NodePort<N: Node>(pub N, pub Port);

impl<
    N: Node,
    const NODE_PORT_COUNT: usize,
    const LANE_COUNT: usize,
    const BOUNCE_PORT_COUNT: usize,
> BoardSpec<N, NODE_PORT_COUNT, LANE_COUNT, BOUNCE_PORT_COUNT> {
    pub fn create_port_map(&self) -> PortMap<N> {
        let mut port_map = PortMap::default();

        for NodePort(node, port) in &self.node_ports {
            port_map.set_node(*port, *node);
        }
        for (index, Lane(a, b)) in self.lanes.iter().enumerate() {
            port_map.set_lane_index(*a, index);
            port_map.set_lane_index(*b, index);
        }

        port_map
    }
}
