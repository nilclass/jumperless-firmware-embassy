use crate::{
    Net,
    ChipStatus,
    Lane,
    ChipPort,
    Dimension,
    ChipId,
    Edge, util::EdgeBitMap,
};
use std::collections::HashMap;

/// Turn given list of `nets` into connections. The connections are made by modifying the given `chip_status` (which is expected to be empty to begin with).
///
/// The given `lanes` are specific to the board, and tell the algorithm how to the chips are interconnected.
pub fn nets_to_connections(nets: impl Iterator<Item = Net>, chip_status: &mut ChipStatus, lanes: &[Lane]) {
    // list of edges that need to be connected at the very end (these are for nets which are only on a single chip)
    let mut pending_edge_nets = vec![];
    // list of pairs of edges that need a bounce in between
    let mut pending_bounces = vec![];

    // Copy list of lanes to a heap-allocated list. We remove lanes that are used as we go, so this always contains
    // lanes that are still available.
    let mut lanes = lanes.to_vec();

    // For now, just go net-by-net, in the order they are given. Later on this could become more clever and route more complex nets first.
    for net in nets {
        // group the ports by chip
        let mut by_chip: HashMap<ChipId, Vec<ChipPort>> = HashMap::new();
        for port in &net.ports {
            by_chip.entry(port.0).or_default().push(*port);
        }

        println!("Ports: {:?}", net.ports);

        // set of edges that need to be connected to satisfy the net
        let mut edges = EdgeBitMap::empty();

        // go through each port by chip, and detect which edges need to be connected
        for (chip, ports) in by_chip {
            for port in ports {
                // mark each port as belonging to this net
                chip_status.set(port, net.id);

                // to hook up this port, it's orthogonal edge must be connected
                edges.set(port.edge().orthogonal());
            }
        }

        println!("Net {:?} has {} edges: {:?}", net.id, edges.len(), edges);

        if edges.len() == 1 { // single-chip net. Will be connected at the very end.
            pending_edge_nets.push((edges.pop().unwrap(), net.id));
        } else {
            let mut connected_edges = EdgeBitMap::empty();

            connected_edges.set(edges.pop().unwrap());

            while edges.len() > 0 {
                // attempt to find a direct lane for one of the edge pairs
                let mut direct = None;
                'outer: for unconnected in edges.iter() {
                    for connected in connected_edges.iter() {
                        if let Some(lane) = take_lane(&mut lanes, |lane| lane.connects(connected, unconnected)) {
                            direct = Some((unconnected, lane));
                            break 'outer;
                        }
                    }
                }
                if let Some((edge, lane)) = direct {
                    chip_status.set_lane(lane, net.id);
                    connected_edges.set(edge);
                    edges.clear(edge);
                } else {
                    // no direct lane found, add the first pair as a bounce candidate, and try again
                    // (this will likely fail, but it's a start)
                    pending_bounces.push((connected_edges.iter().next().unwrap(), edges.pop().unwrap(), net.id));
                }
            }
        }
    }

    // Produce missing lanes via bounces
    for (edge_a, edge_b, net_id) in pending_bounces {
        // first try to find an orthogonal edge on one of the chips that can connect us.
        // if one is found, it can be hooked up to the target nodes via any other free lane at the very end.
        let alt_edge_a = edge_a.orthogonal();
        let alt_edge_b = edge_b.orthogonal();
        if let Some(lane) = take_lane(&mut lanes, |lane| lane.connects(alt_edge_a, edge_b)) {
            chip_status.set_lane(lane, net_id);
            pending_edge_nets.push((edge_a, net_id));
        } else if let Some(lane) = take_lane(&mut lanes, |lane| lane.connects(edge_a, alt_edge_b)) {
            chip_status.set_lane(lane, net_id);
            pending_edge_nets.push((edge_b, net_id));
        } else { // bounce via orthagonal edge not possible. Try to find a path via another chip.
            todo!("Bounce from {:?} to {:?}", edge_a, edge_b);
        }
    }

    // Connect the remaining edges
    for (edge, net_id) in pending_edge_nets {
        let mut lane_index = None;
        // find a free lane that touches the edge
        for (i, lane) in lanes.iter().enumerate() {
            if lane.0.edge() == edge || lane.1.edge() == edge {
                lane_index = Some(i);
                break;
            }
        }
        if let Some(i) = lane_index {
            chip_status.set_lane(lanes.remove(i), net_id);
        } else {
            todo!("No available lane ports on edge {:?}", edge);
        }
    }
}

/// Take (remove) the first lane from the given list of lanes that matches the predicate.
fn take_lane<F: Fn(&Lane) -> bool>(lanes: &mut Vec<Lane>, predicate: F) -> Option<Lane> {
    let mut index = None;
    for (i, lane) in lanes.iter().enumerate() {
        if predicate(lane) {
            index = Some(i);
            break;
        }
    }
    if let Some(index) = index {
        Some(lanes.remove(index))
    } else {
        None
    }
}
