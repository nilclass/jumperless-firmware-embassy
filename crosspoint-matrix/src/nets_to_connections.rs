use crate::{
    Net,
    ChipStatus,
    Lane,
    util::{EdgeBitMap, LaneSet},
};

use log::debug;

/// Turn given list of `nets` into connections. The connections are made by modifying the given `chip_status` (which is expected to be empty to begin with).
///
/// The given `lanes` are specific to the board, and tell the algorithm how to the chips are interconnected.
pub fn nets_to_connections(nets: impl Iterator<Item = Net>, chip_status: &mut ChipStatus, lanes: &[Lane]) {
    // list of edges that need to be connected at the very end (these are for nets which are only on a single chip)
    let mut pending_edge_nets = vec![];
    // list of pairs of edges that need a bounce in between
    let mut pending_bounces = vec![];

    // set of lanes that are available
    let mut lanes = LaneSet::new(lanes);

    // For now, just go net-by-net, in the order they are given. Later on this could become more clever and route more complex nets first.
    for net in nets {
        debug!("Ports: {:?}", net.ports);

        // set of edges that need to be connected to satisfy the net
        let mut edges = EdgeBitMap::empty();

        for port in net.ports {
            // mark each port as belonging to this net
            chip_status.set(port, net.id);

            // to hook up this port, it's orthogonal edge must be connected
            edges.set(port.edge().orthogonal());
        }

        debug!("Net {:?} has {} edges: {:?}", net.id, edges.len(), edges);

        if edges.len() == 1 { // single-chip net. Will be connected at the very end, using an arbitrary free lane port.
            pending_edge_nets.push((edges.pop().unwrap(), net.id));
        } else {
            let mut connected_edges = EdgeBitMap::empty();

            connected_edges.set(edges.pop().unwrap());

            while edges.len() > 0 {
                // attempt to find a direct lane for one of the edge pairs
                let mut direct = None;
                'outer: for unconnected in edges.iter() {
                    for connected in connected_edges.iter() {
                        if let Some(lane) = lanes.take(|lane| lane.connects(connected, unconnected)) {
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
        if let Some(lane) = lanes.take(|lane| lane.connects(alt_edge_a, edge_b)) {
            chip_status.set_lane(lane, net_id);
            pending_edge_nets.push((edge_a, net_id));
        } else if let Some(lane) = lanes.take(|lane| lane.connects(edge_a, alt_edge_b)) {
            chip_status.set_lane(lane, net_id);
            pending_edge_nets.push((edge_b, net_id));
        } else { // bounce via orthagonal edge not possible. Try to find a path via another chip.
            todo!("Bounce from {:?} to {:?}", edge_a, edge_b);
        }
    }

    // Connect the remaining edges
    for (edge, net_id) in pending_edge_nets {
        if let Some(lane) = lanes.take(|lane| lane.touches(edge)) {
            chip_status.set_lane(lane, net_id);
        } else {
            todo!("No available lane ports on edge {:?}", edge);
        }
    }
}
