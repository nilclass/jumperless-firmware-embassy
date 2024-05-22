use jumperless_types::{set::PortSet, ChipId, Dimension, Lane, NetId, Port};

use crate::{layout, Crosspoint};

/// Assigns a net to every port on every chip.
///
/// This is an intermediate structure, used to build chip connections from netlists.
///
/// By default all ports are unassigned. To assign a net id to one or two ports, [`ChipStatus::set`] and [`ChipStatus::set_lane`] are called.
///
/// A given port can only be set once (reassigning causes a panic). This ensures that distinct nets are not accidentally connected
/// by accident, due to bugs in the routing code.
///
/// Once a ChipStatus is complete, the [`ChipStatus::crosspoints`] method provides a way to iterate over the resulting switch positions.
#[derive(Default)]
pub struct ChipStatus([ChipStatusEntry; 12]);

#[derive(Default)]
struct ChipStatusEntry {
    x: [Option<NetId>; 16],
    y: [Option<NetId>; 8],
}

impl ChipStatus {
    /// Remove all net ID assignments
    pub fn clear(&mut self) {
        for entry in &mut self.0 {
            entry.x.fill(None);
            entry.y.fill(None);
        }
    }

    /// Retrieve net id assigned to given port
    pub fn get(&self, port: Port) -> Option<NetId> {
        let entry = &self.0[port.chip_id().index()];
        match port.dimension() {
            Dimension::X => entry.x[port.index() as usize],
            Dimension::Y => entry.y[port.index() as usize],
        }
    }

    /// Assign net id to given port
    ///
    /// Panics if the port is already assigned to a different net.
    pub fn set(&mut self, port: Port, net: NetId) {
        if self.get(port).is_some() {
            panic!("Port already set");
        }

        let entry = &mut self.0[port.chip_id().index()];
        match port.dimension() {
            Dimension::X => entry.x[port.index() as usize] = Some(net),
            Dimension::Y => entry.y[port.index() as usize] = Some(net),
        }
        // println!("SET {:?} to {:?}", port, net)
    }

    /// Assign given net id to both ends of given lane
    ///
    /// Panics if one of the ports is already assigned to a different net.
    pub fn set_lane(&mut self, lane: Lane, net: NetId) {
        self.set(lane.0, net);
        self.set(lane.1, net);
    }

    /// Is the given port available? (i.e. no net assigned to it?)
    pub fn available(&self, port: Port) -> bool {
        self.get(port).is_none()
    }

    /// Iterate over all the crosspoint which must be set (switch closed)
    pub fn crosspoints(&self) -> CrosspointIterator {
        CrosspointIterator {
            cs: self,
            i: 0,
            x: 0,
            y: 0,
        }
    }

    /// Validate that all nodes that are marked to belong to the given net are interconnected.
    ///
    /// This is used as a sanity check within tests.
    ///
    /// First build a set of ports that *must* be part of the net (because they are node ports, not lane ports, and they are marked to belong to the net).
    ///
    /// Then it picks the first node, and does a depth-first walk following any crosspoint connections and lanes that it encounters, keeping track of all the
    /// chip ports that it already visited (so it doesn't get stuck in loops).
    ///
    /// Once the walk finds no more paths to follow, we compare the set of ports that were visited by the walk with the set of required ones collected earlier.
    /// If all of the required ports have been visited, then all of them must be connected by at least one path.
    #[cfg(feature = "std")]
    pub(crate) fn check_connectivity<const NODE_COUNT: usize, const LANE_COUNT: usize>(
        &self,
        net_id: NetId,
        layout: &layout::Layout<NODE_COUNT, LANE_COUNT>,
    ) {
        // println!("Check connectivity of net {:?}", net_id);
        // contains every port that this net must contain (those that resolve to Nodes)
        let mut required = PortSet::empty();

        let mut first = None;

        for (i, chip_status) in self.0.iter().enumerate() {
            let chip = ChipId::from_index(i);
            for (x, value) in chip_status.x.iter().enumerate() {
                if *value == Some(net_id) {
                    let port = chip.port_x(x as u8);
                    if layout.port_to_node(port).is_some() {
                        // println!("REQUIRED: {:?}", port);
                        required.insert(port);
                        if first.is_none() {
                            first = Some(port);
                        }
                    }
                }
            }
            for (y, value) in chip_status.y.iter().enumerate() {
                if *value == Some(net_id) {
                    let port = chip.port_y(y as u8);
                    if layout.port_to_node(port).is_some() {
                        // println!("REQUIRED: {:?}", port);
                        required.insert(port);
                        if first.is_none() {
                            first = Some(port);
                        }
                    }
                }
            }
        }

        let first = first.expect("net must be connected to at least one port");

        // println!("Starting with port: {:?}", first);

        // println!("Required bitmap: {:?}", required.0);

        // keep track of nodes that were visited
        let mut visited = PortSet::empty();

        self.visit_port(first, &mut visited, &mut |port, value| {
            if value == Some(net_id) {
                if let Some(lane) = layout.port_to_lane(port) {
                    return Visit::MarkAndFollow(lane.opposite(port));
                }
                Visit::Mark
            } else {
                Visit::Skip
            }
        });

        // println!("Visited bitmap: {:?}", visited.0);

        if !visited.is_superset(&required) {
            visited.print_diff(&required);

            panic!("Net {:?} is not fully connected", net_id);
        }
    }

    /// Performs a depth-first search of ports that are connected to the same net, guided by the given `visit` closure.
    fn visit_port<F: FnMut(Port, Option<NetId>) -> Visit>(
        &self,
        start: Port,
        visited: &mut PortSet,
        visit: &mut F,
    ) {
        // println!("Visit port {:?}", start);
        visited.insert(start);

        // keep track if we "marked" any of the ports on the orthogonal edge
        // (if so, we also visit the adjacent edge)
        let mut marked_orthogonal = false;

        // first visit all ports on the "other" edge
        for port in start.edge().orthogonal().ports() {
            // println!("CHECK {:?}", port);
            if visited.contains(port) {
                // println!("VISITED");
                // skip this one, we've already been here!
                continue;
            }

            let net_id = self.get(port);
            match visit(port, net_id) {
                Visit::Skip => {}
                Visit::Mark => {
                    // println!("MARK {:?}", port);
                    marked_orthogonal = true;
                    visited.insert(port);
                }
                Visit::MarkAndFollow(follow) => {
                    // println!("MARK {:?}, FOLLOW {:?}", port, follow);
                    marked_orthogonal = true;
                    visited.insert(port);
                    self.visit_port(follow, visited, visit);
                }
            }
        }

        if marked_orthogonal {
            for port in start.edge().ports() {
                if visited.contains(port) {
                    // skip this one, we've already been here!
                    continue;
                }

                let net_id = self.get(port);
                match visit(port, net_id) {
                    Visit::Skip => {}
                    Visit::Mark => {
                        // println!("MARK {:?}", port);
                        visited.insert(port);
                    }
                    Visit::MarkAndFollow(follow) => {
                        // println!("MARK {:?}, FOLLOW {:?}", port, follow);
                        visited.insert(port);
                        self.visit_port(follow, visited, visit);
                    }
                }
            }
        }
    }
}

enum Visit {
    Skip,
    Mark,
    MarkAndFollow(Port),
}

// #[cfg(feature = "std")]
// impl From<&ChipStatus> for Vec<Net> {
//     fn from(value: &ChipStatus) -> Self {
//         use std::collections::HashMap;

//         let mut nets: HashMap<NetId, Net> = HashMap::new();
//         for (chip_index, chip) in value.0.iter().enumerate() {
//             for (i, x) in chip.x.iter().enumerate() {
//                 if let Some(net_id) = x {
//                     nets.entry(*net_id).or_insert(Net { id: *net_id, ports: vec![] }).ports.push(Port(ChipId::from_index(chip_index), Dimension::X, i as u8));
//                 }
//             }

//             for (i, y) in chip.y.iter().enumerate() {
//                 if let Some(net_id) = y {
//                     nets.entry(*net_id).or_insert(Net { id: *net_id, ports: vec![] }).ports.push(Port(ChipId::from_index(chip_index), Dimension::Y, i as u8));
//                 }
//             }
//         }
//         nets.into_values().collect()
//     }
// }

/// Iterator over all connected crosspoints
///
/// Yields all crosspoints that need to be connected.
pub struct CrosspointIterator<'a> {
    cs: &'a ChipStatus,
    i: usize,
    x: usize,
    y: usize,
}

impl<'a> Iterator for CrosspointIterator<'a> {
    type Item = Crosspoint;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.cs.0.len() {
            return None;
        }
        let chip_status = &self.cs.0[self.i];
        let chip = ChipId::from_index(self.i);
        // println!("CP {},{},{}", self.i, self.x, self.y);
        if let Some(x_net) = chip_status.x[self.x] {
            if let Some(y_net) = chip_status.y[self.y]
                && x_net == y_net
            {
                let (x, y) = (self.x as u8, self.y as u8);
                self.advance_y();
                Some(Crosspoint {
                    chip,
                    x,
                    y,
                    net_id: x_net,
                })
            } else {
                self.advance_y();
                self.next()
            }
        } else {
            self.advance_x();
            self.next()
        }
    }
}

impl<'a> CrosspointIterator<'a> {
    fn advance_x(&mut self) {
        if self.x == 15 {
            self.i += 1;
            self.x = 0;
        } else {
            self.x += 1;
            self.y = 0;
        }
    }

    fn advance_y(&mut self) {
        if self.y == 7 {
            self.advance_x();
        } else {
            self.y += 1;
        }
    }
}
