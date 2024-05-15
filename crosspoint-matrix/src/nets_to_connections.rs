use crate::{
    Net,
    ChipStatus,
    Lane,
    ChipPort,
    Dimension,
    ChipId,
    Edge,
};
use std::collections::HashMap;

/// Turn given list of `nets` into connections. The connections are made by modifying the given `chip_status` (which is expected to be empty to begin with).
///
/// The given `lanes` represent
pub fn nets_to_connections(nets: impl Iterator<Item = Net>, chip_status: &mut ChipStatus, lanes: &[Lane]) {
    // list of edges that need to be connected at the very end (these are for nets which are only on a single chip)
    let mut pending_edge_nets = vec![];
    // list of pairs of edges that need a bounce in between
    let mut pending_bounces = vec![];

    // Copy list of lanes to a heap-allocated list. We remove lanes that are used as we go, so this always contains
    // lanes that are still available.
    let mut lanes = lanes.to_vec();

    for net in nets {
        let mut by_chip: HashMap<ChipId, Vec<ChipPort>> = HashMap::new();
        for port in &net.ports {
            by_chip.entry(port.0).or_default().push(*port);
        }

        println!("Ports: {:?}", net.ports);

        let mut edges = vec![];

        for (chip, ports) in by_chip {
            let (mut x_used, mut y_used) = (false, false);
            for port in ports {
                chip_status.set(port.0, port.1, port.2, net.id);

                if port.1 == Dimension::X {
                    x_used = true;
                } else {
                    y_used = true;
                }
            }

            if x_used && y_used {
                todo!("Handle multiple edges on the same chip");
            }

            if x_used {
                edges.push(Edge(chip, Dimension::Y));
            }

            if y_used {
                edges.push(Edge(chip, Dimension::X));
            }
        }

        println!("Net {:?} has {} edges: {:?}", net.id, edges.len(), edges);

        if edges.len() == 1 { // single-chip net. Will be connected at the very end.
            pending_edge_nets.push((edges[0], net.id));
        } else if edges.len() == 2 {
            if let Some(lane) = take_lane(&mut lanes, |lane| lane.touches(edges[0]) && lane.touches(edges[1])) {
                // connect directly, with a lane
                chip_status.set_lane(lane, net.id);
            } else {
                // connect later, bounced through another chip
                pending_bounces.push((edges[0], edges[1], net.id));
            }
        } else {
            todo!("More than 2 edges");
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
