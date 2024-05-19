use crate::{
    Net,
    ChipStatus,
    Lane,
    Edge,
    NetId,
    util::{EdgeSet, LaneSet},
    layout::PortMap,
};

use heapless::Vec;

use log::debug;

const MAX_NETS: usize = 60;

/// Turn given list of `nets` into connections. The connections are made by modifying the given `chip_status` (which is expected to be empty to begin with).
///
/// The given `lanes` are specific to the board, and tell the algorithm how to the chips are interconnected.
pub fn nets_to_connections(nets: impl Iterator<Item = Net>, chip_status: &mut ChipStatus, layout_lanes: &[Lane], port_map: &PortMap) {
    // list of edges that need to be connected at the very end (these are for nets which are only on a single chip)
    let mut pending_edge_nets: Vec<(Edge, NetId), MAX_NETS> = Vec::new();
    // list of pairs of edges that need a bounce in between
    let mut pending_bounces: Vec<(Edge, Edge, NetId), MAX_NETS> = Vec::new();

    // set of lanes that are available (initially all of them, we take them away as they are being assigned to nets)
    let mut lanes = LaneSet::new(layout_lanes);

    // For now, just go net-by-net, in the order they are given. Later on this could become more clever and route more complex nets first.
    for net in nets {
        debug!("Ports: {:?}", net.ports);

        // set of edges that need to be connected to satisfy the net
        let mut edges = EdgeSet::empty();

        for port in net.ports {
            // mark each port as belonging to this net
            chip_status.set(port, net.id);

            // to hook up this port, it's orthogonal edge must be connected
            edges.insert(port.edge().orthogonal());
        }

        debug!("Net {:?} has {} edges: {:?}", net.id, edges.len(), edges);

        if edges.len() == 1 { // single-chip net. Will be connected at the very end, using an arbitrary free lane port.
            pending_edge_nets.push((edges.pop().unwrap(), net.id)).ok().unwrap();
        } else {
            let mut connected_edges = EdgeSet::empty();

            connected_edges.insert(edges.pop().unwrap());

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
                    connected_edges.insert(edge);
                    edges.remove(edge);
                } else {
                    // no direct lane found, add the first pair as a bounce candidate, and try again
                    // (this will likely fail, but it's a start)
                    pending_bounces.push((connected_edges.iter().next().unwrap(), edges.pop().unwrap(), net.id)).ok().unwrap();
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
            pending_edge_nets.push((edge_a, net_id)).ok().unwrap();
        } else if let Some(lane) = lanes.take(|lane| lane.connects(edge_a, alt_edge_b)) {
            chip_status.set_lane(lane, net_id);
            pending_edge_nets.push((edge_b, net_id)).ok().unwrap();
        } else {

            let mut success = false;

            'outer: for port in edge_a.ports() {
                if let Some(index0) = port_map.get(port).lane_index() && lanes.has_index(index0) {
                    let lane0 = layout_lanes[index0];
                    // destination edge on the target chip of lane0
                    let dest0_edge = lane0.opposite(port).edge();
                    debug!("Candidate lane0 {} going to {:?}", index0, dest0_edge);

                    // first check if there is an orthogonal lane leading to edge B
                    for port in dest0_edge.orthogonal().ports() {
                        if let Some(index1) = port_map.get(port).lane_index() && lanes.has_index(index1) {
                            let lane1 = layout_lanes[index1];
                            let dest1_edge = lane1.opposite(port).edge();
                            debug!("  Candidate lane1 {} going to {:?} (orthogonal)", index1, dest1_edge);

                            if dest1_edge == edge_b { // success!
                                chip_status.set_lane(lane0, net_id);
                                chip_status.set_lane(lane1, net_id);

                                // mark lanes as used
                                lanes.clear_index(index0);
                                lanes.clear_index(index1);

                                success = true;
                                break 'outer;
                            }
                        }
                    }

                    // next check if there is a lane on the same edge, leading to edge B
                    for port in dest0_edge.ports() {
                        if let Some(index1) = port_map.get(port).lane_index() && lanes.has_index(index1) {
                            let lane1 = layout_lanes[index1];
                            let dest1_edge = lane1.opposite(port).edge();
                            debug!("  Candidate lane1 {} going to {:?} (adjacent)", index1, dest1_edge);

                            if dest1_edge == edge_b {
                                // found an adjacent edge that goes to the right place.
                                // now find an orthogonal edge to this adjacent one, to complete the bounce
                                for port in dest0_edge.orthogonal().ports() {
                                    if let Some(index2) = port_map.get(port).lane_index() && lanes.has_index(index2) {
                                        let lane2 = layout_lanes[index2];
                                        debug!("Found a path from {edge_a:?} to {edge_b:?} via {lane0:?} and {lane1:?}, with support of {lane2:?}");
                                        chip_status.set_lane(lane0, net_id);
                                        chip_status.set_lane(lane1, net_id);
                                        chip_status.set_lane(lane2, net_id);

                                        lanes.clear_index(index0);
                                        lanes.clear_index(index1);
                                        lanes.clear_index(index2);

                                        success = true;
                                        break 'outer;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !success {
                panic!("No viable bounce path to connect {:?} with {:?} (net {:?})", edge_a, edge_b, net_id);
            }

        }
        // - iterate through all ports on edge A
        // - check if the port points to a free lane:
        //   - yes: follow!
        //     - check if target chip has a lane to edge B on the orthogonal edge, if yes, take both lanes & by done
        //     - otherwise check if chip has a lane to edge B on the same side *and* a free orthogonal lane slot to bounce. If yes, take all three lanes & be done
        //   - no: continue with next one

        // else if let Some(bounce_path) = lanes.find_bounce_lanes(edge_a, edge_b) {
            
        //     // for lane in bounce_path {
        //     //     chip_status.set_lane(lane, net_id);
        //     // }
        //     // let mut candidates_a = EdgeSet::empty();
        //     // let mut candidates_b = EdgeSet::empty();
        //     // for Lane(port0, port1) in lanes.iter() {
        //     //     if port0.edge() == edge_a {
        //     //         candidates_a.insert(port1.edge());
        //     //     } else if port1.edge() == edge_a {
        //     //         candidates_a.insert(port0.edge());
        //     //     } else if port0.edge() == edge_b {
        //     //         candidates_b.insert(port1.edge());
        //     //     } else if port1.edge() == edge_b {
        //     //         candidates_b.insert(port0.edge());
        //     //     }
        //     // }
        // } 
        // else {
        //     panic!("No viable bounce path to connect {:?} with {:?} (net {:?})", edge_a, edge_b, net_id);
        // }
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
