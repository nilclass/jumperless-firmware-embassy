use crate::{
    ChipId,
    NetId,
    Net,
    Dimension,
    ChipPort,
    Crosspoint,
    Lane,
    util::ChipPortBitMap,
    layout,
};
use std::collections::HashMap;

/// Assigns a net to every chip port
///
/// This is an intermediate structure, used to build 
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
    pub fn get(&self, port: ChipPort) -> Option<NetId> {
        let entry = &self.0[port.0.index()];
        match port.1 {
            Dimension::X => entry.x[port.2 as usize],
            Dimension::Y => entry.y[port.2 as usize],
        }
    }

    /// Assign net id to given port
    ///
    /// Panics if the port is already assigned to a different net.
    pub fn set(&mut self, port: ChipPort, net: NetId) {
        if let Some(existing) = self.get(port) {
            panic!("Already set: {:?}, have {:?}, want {:?}", port, existing, net);
        }

        let entry = &mut self.0[port.0.index()];
        match port.1 {
            Dimension::X => entry.x[port.2 as usize] = Some(net),
            Dimension::Y => entry.y[port.2 as usize] = Some(net),
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
    pub fn available(&self, port: ChipPort) -> bool {
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
    #[allow(unused)]
    pub(crate) fn check_connectivity<const NODE_COUNT: usize, const LANE_COUNT: usize>(&self, net_id: NetId, layout: &layout::Layout<NODE_COUNT, LANE_COUNT>) {
        println!("Check connectivity of net {:?}", net_id);
        // contains every port that this net must contain (those that resolve to Nodes)
        let mut required = ChipPortBitMap::empty();

        let mut first = None;

        for (i, chip_status) in self.0.iter().enumerate() {
            let chip = ChipId::from_index(i);
            for (x, value) in chip_status.x.iter().enumerate() {
                if *value == Some(net_id) {
                    let port = chip.port_x(x as u8);
                    if layout.port_to_node(port).is_some() {
                        // println!("REQUIRED: {:?}", port);
                        required.set(port);
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
                        required.set(port);
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
        let mut visited = ChipPortBitMap::empty();

        self.visit_port(first, &mut visited, &mut |port, value| {
            if value == Some(net_id) {
                if let Some(dest) = layout.lane_destination(port) {
                    return Visit::MarkAndFollow(dest)
                }
                Visit::Mark
            } else {
                Visit::Skip
            }
        });

        // println!("Visited bitmap: {:?}", visited.0);

        if !visited.contains(&required) {

            visited.print_diff(&required);

            panic!("Not connected");
        }
    }

    fn visit_port<F: FnMut(ChipPort, Option<NetId>) -> Visit>(&self, start: ChipPort, visited: &mut ChipPortBitMap, visit: &mut F) {
        // println!("Visit port {:?}", start);
        visited.set(start);

        // keep track if we "marked" any of the ports on the orthogonal edge
        // (if so, we also visit the adjacent edge)
        let mut marked_orthogonal = false;

        // first visit all ports on the "other" edge
        for port in start.edge().orthogonal().ports() {
            // println!("CHECK {:?}", port);
            if visited.get(port) {
                // println!("VISITED");
                // skip this one, we've already been here!
                continue;
            }

            let net_id = self.get(port);
            match visit(port, net_id) {
                Visit::Skip => {},
                Visit::Mark => {
                    // println!("MARK {:?}", port);
                    marked_orthogonal = true;
                    visited.set(port);
                },
                Visit::MarkAndFollow(follow) => {
                    // println!("MARK {:?}, FOLLOW {:?}", port, follow);
                    marked_orthogonal = true;
                    visited.set(port);
                    self.visit_port(follow, visited, visit);
                },
            }
        }

        if marked_orthogonal {
            for port in start.edge().ports() {
                if visited.get(port) {
                    // skip this one, we've already been here!
                    continue;
                }

                let net_id = self.get(port);
                match visit(port, net_id) {
                    Visit::Skip => {},
                    Visit::Mark => {
                        // println!("MARK {:?}", port);
                        visited.set(port);
                    },
                    Visit::MarkAndFollow(follow) => {
                        // println!("MARK {:?}, FOLLOW {:?}", port, follow);
                        visited.set(port);
                        self.visit_port(follow, visited, visit);
                    },
                }
            }
        }
    }
}

enum Visit {
    Skip,
    Mark,
    MarkAndFollow(ChipPort),
}

impl From<&ChipStatus> for Vec<Net> {
    fn from(value: &ChipStatus) -> Self {
        let mut nets: HashMap<NetId, Net> = HashMap::new();
        for (chip_index, chip) in value.0.iter().enumerate() {
            for (i, x) in chip.x.iter().enumerate() {
                if let Some(net_id) = x {
                    nets.entry(*net_id).or_insert(Net { id: *net_id, ports: vec![] }).ports.push(ChipPort(ChipId::from_index(chip_index), Dimension::X, i as u8));
                }
            }

            for (i, y) in chip.y.iter().enumerate() {
                if let Some(net_id) = y {
                    nets.entry(*net_id).or_insert(Net { id: *net_id, ports: vec![] }).ports.push(ChipPort(ChipId::from_index(chip_index), Dimension::Y, i as u8));
                }
            }
        }
        nets.into_values().collect()
    }
}

/// Iterator over all connected crosspoints
///
/// Yields all crosspoints that need to be connected.
pub struct CrosspointIterator<'a>{
    cs: &'a ChipStatus,
    i: usize,
    x: usize,
    y: usize,
}

impl<'a> Iterator for CrosspointIterator<'a> {
    type Item = Crosspoint;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i == self.cs.0.len() {
            return None
        }
        let chip_status = &self.cs.0[self.i];
        let chip = ChipId::from_index(self.i);
        // println!("CP {},{},{}", self.i, self.x, self.y);
        if let Some(x_net) = chip_status.x[self.x] {
            if let Some(y_net) = chip_status.y[self.y] && x_net == y_net {
                let (x, y) = (self.x as u8, self.y as u8);
                self.advance_y();
                Some(Crosspoint { chip, x, y, net_id: x_net })
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
