// #![no_std]
#![feature(let_chains)]

use std::collections::HashMap;
use std::fmt::Display;
use std::num::NonZeroU8;

pub mod layout;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Dimension {
    X,
    Y,
}

impl Dimension {
    fn orthogonal(&self) -> Self {
        match self {
            Dimension::X => Dimension::Y,
            Dimension::Y => Dimension::X,
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ChipId(u8);

impl ChipId {
    pub fn index(&self) -> usize {
        (self.0 - b'A') as usize
    }

    pub fn from_index(index: usize) -> Self {
        Self(b'A' + index as u8)
    }

    pub fn port_x(&self, x: u8) -> ChipPort {
        ChipPort(*self, Dimension::X, x)
    }

    pub fn port_y(&self, y: u8) -> ChipPort {
        ChipPort(*self, Dimension::Y, y)
    }
}

impl Display for ChipId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", core::str::from_utf8(&[self.0]).unwrap())
    }
}

impl std::fmt::Debug for ChipId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChipId({})", core::str::from_utf8(&[self.0]).unwrap())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NetId(NonZeroU8);

impl From<u8> for NetId {
    fn from(value: u8) -> Self {
        NetId(NonZeroU8::new(value).unwrap())
    }
}

#[derive(Default)]
pub struct ChipStatus([ChipStatusEntry; 12]);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChipPort(ChipId, Dimension, u8);

impl ChipPort {
    fn edge(&self) -> Edge {
        Edge(self.0, self.1)
    }
}

/// A bitmap that can hold a boolean for every chip port.
///
/// Useful for various algorithms.
#[derive(Eq, PartialEq)]
pub struct ChipPortBitMap([u8; 36]);

impl ChipPortBitMap {
    pub fn empty() -> Self {
        Self([0; 36])
    }

    pub fn full() -> Self {
        Self([0xFF; 36])
    }

    /// Check if bit for given port address is set
    pub fn get(&self, port: ChipPort) -> bool {
        let (i, j) = Self::address(port);
        (self.0[i] >> j) & 1 == 1
    }

    /// Set the bit for given port address
    pub fn set(&mut self, port: ChipPort) {
        let (i, j) = Self::address(port);
        self.0[i] |= 1 << j
    }

    /// Clear the bit for given port address
    pub fn clear(&mut self, port: ChipPort) {
        let (i, j) = Self::address(port);
        self.0[i] &= !(1 << j)
    }

    /// Check if all the bits that are set in `other` are also set in `self` (i.e. `self` is a superset of `other`)
    pub fn contains(&self, other: &Self) -> bool {
        for (i, byte) in self.0.iter().enumerate() {
            if other.0[i] & byte != other.0[i] {
                return false
            }
        }
        true
    }

    pub fn print_diff(&self, other: &Self) {
        println!("BEGIN DIFF");
        for i in 0..12 {
            let chip = ChipId::from_index(i);
            for x in 0..16 {
                let port = chip.port_x(x);
                let a = self.get(port);
                let b = other.get(port);
                if a && !b {
                    println!("+{:?}", port);
                } else if !a && b {
                    println!("-{:?}", port);
                }
            }
            for y in 0..8 {
                let port = chip.port_y(y);
                let a = self.get(port);
                let b = other.get(port);
                if a && !b {
                    println!("+{:?}", port);
                } else if !a && b {
                    println!("-{:?}", port);
                }
            }
        }
        println!("END DIFF");
    }

    fn address(port: ChipPort) -> (usize, usize) {
        let ChipPort(chip, dimension, index) = port;
        let bit_address = chip.index() * 24 + if dimension == Dimension::X { 0 } else { 16 } + index as usize;
        (bit_address / 8, bit_address % 8)
    }
}

#[derive(Default)]
struct ChipStatusEntry {
    x: [Option<NetId>; 16],
    y: [Option<NetId>; 8],
}

impl ChipStatus {
    fn clear(&mut self) {
        for entry in &mut self.0 {
            entry.x.fill(None);
            entry.y.fill(None);
        }
    }

    fn get(&self, chip: ChipId, dimension: Dimension, index: u8) -> Option<NetId> {
        let entry = &self.0[chip.index()];
        match dimension {
            Dimension::X => entry.x[index as usize],
            Dimension::Y => entry.y[index as usize],
        }
    }

    fn set(&mut self, chip: ChipId, dimension: Dimension, index: u8, net: NetId) {
        if let Some(existing) = self.get(chip, dimension, index) {
            panic!("Already set: {:?} {:?} {}, have {:?}, want {:?}", chip, dimension, index, existing, net);
        }

        let entry = &mut self.0[chip.index()];
        match dimension {
            Dimension::X => entry.x[index as usize] = Some(net),
            Dimension::Y => entry.y[index as usize] = Some(net),
        }
        println!("SET {} {:?}{} to {:?}", chip, dimension, index, net)
    }

    fn set_lane(&mut self, lane: Lane, net: NetId) {
        self.set(lane.0.0, lane.0.1, lane.0.2, net);
        self.set(lane.1.0, lane.1.1, lane.1.2, net);
    }

    fn available(&self, port: ChipPort) -> bool {
        self.get(port.0, port.1, port.2).is_none()
    }

    fn crosspoints(&self) -> CrosspointIterator {
        CrosspointIterator {
            cs: self,
            i: 0,
            x: 0,
            y: 0,
        }
    }

    fn check_connectivity<const NODE_COUNT: usize, const LANE_COUNT: usize>(&self, net_id: NetId, layout: &layout::Layout<NODE_COUNT, LANE_COUNT>) {
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
                        println!("REQUIRED: {:?}", port);
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
                        println!("REQUIRED: {:?}", port);
                        required.set(port);
                        if first.is_none() {
                            first = Some(port);
                        }
                    }
                }
            }
        }

        let first = first.expect("net must be connected to at least one port");

        println!("Starting with port: {:?}", first);

        println!("Required bitmap: {:?}", required.0);

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

        println!("Visited bitmap: {:?}", visited.0);

        if !visited.contains(&required) {

            visited.print_diff(&required);

            panic!("Not connected");
        }
    }

    fn visit_port<F: FnMut(ChipPort, Option<NetId>) -> Visit>(&self, start: ChipPort, visited: &mut ChipPortBitMap, visit: &mut F) {
        println!("Visit port {:?}", start);
        visited.set(start);

        // keep track if we "marked" any of the ports on the orthogonal edge
        // (if so, we also visit the adjacent edge)
        let mut marked_orthogonal = false;

        // first visit all ports on the "other" edge
        for port in start.edge().orthogonal().ports() {
            println!("CHECK {:?}", port);
            if visited.get(port) {
                println!("VISITED");
                // skip this one, we've already been here!
                continue;
            }

            let net_id = self.get(port.0, port.1, port.2);
            match visit(port, net_id) {
                Visit::Skip => {},
                Visit::Mark => {
                    println!("MARK {:?}", port);
                    marked_orthogonal = true;
                    visited.set(port);
                },
                Visit::MarkAndFollow(follow) => {
                    println!("MARK {:?}, FOLLOW {:?}", port, follow);
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

                let net_id = self.get(port.0, port.1, port.2);
                match visit(port, net_id) {
                    Visit::Skip => {},
                    Visit::Mark => {
                        println!("MARK {:?}", port);
                        visited.set(port);
                    },
                    Visit::MarkAndFollow(follow) => {
                        println!("MARK {:?}, FOLLOW {:?}", port, follow);
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
        println!("CP {},{},{}", self.i, self.x, self.y);
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Crosspoint {
    pub chip: ChipId,
    pub net_id: NetId,
    pub x: u8,
    pub y: u8,
}

/// A net is a collection of ChipPorts which are supposed to be
/// interconnected.
pub struct Net {
    id: NetId,
    ports: Vec<ChipPort>,
}

fn print_crosspoints(crosspoints: impl Iterator<Item = Crosspoint>) {
    let mut matrices = [
        (ChipId(b'A'), [[None; 16]; 8]),
        (ChipId(b'B'), [[None; 16]; 8]),
        (ChipId(b'C'), [[None; 16]; 8]),
        (ChipId(b'D'), [[None; 16]; 8]),
        (ChipId(b'E'), [[None; 16]; 8]),
        (ChipId(b'F'), [[None; 16]; 8]),
        (ChipId(b'G'), [[None; 16]; 8]),
        (ChipId(b'H'), [[None; 16]; 8]),
        (ChipId(b'I'), [[None; 16]; 8]),
        (ChipId(b'J'), [[None; 16]; 8]),
        (ChipId(b'K'), [[None; 16]; 8]),
        (ChipId(b'L'), [[None; 16]; 8]),
    ];

    for crosspoint in crosspoints {
        println!("Lookup chip {:?}", crosspoint.chip);
        let matrix = &mut matrices.iter_mut().find(|(chip, _)| *chip == crosspoint.chip).unwrap().1;
        matrix[crosspoint.y as usize][crosspoint.x as usize] = Some(crosspoint.net_id);
    }

    for (chip, matrix) in matrices {
        println!("Chip: {}", chip);
        for line in matrix {
            for cell in line {
                if let Some(NetId(net_id)) = cell {
                    print!("{}\t", net_id);
                } else {
                    print!("-\t");
                }
            }
            println!("");
        }
        println!("");
    }
}

/// Represents one of the sides (X/Y) of a specific chip.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Edge(ChipId, Dimension);

impl Edge {
    fn orthogonal(&self) -> Self {
        Self(self.0, self.1.orthogonal())
    }

    fn ports(&self) -> impl Iterator<Item = ChipPort> {
        let Edge(chip, dimension) = *self;
        let range = match dimension {
            Dimension::X => 0..16,
            Dimension::Y => 0..8,
        };
        range.map(move |index| ChipPort(chip, dimension, index))
    }
}

/// Represents a connection between two ChipPorts on different chips
#[derive(Copy, Clone)]
struct Lane(ChipPort, ChipPort);

impl Lane {
    fn touches(&self, edge: Edge) -> bool {
        self.0.edge() == edge || self.1.edge() == edge
    }

    fn connects(&self, from: Edge, to: Edge) -> bool {
        let (a, b) = (self.0.edge(), self.1.edge());
        (a, b) == (from, to) || (a, b) == (to, from)
    }
}

fn nets_to_connections(nets: impl Iterator<Item = Net>, chip_status: &mut ChipStatus, lanes: &[Lane]) {
    // list of edges that need to be connected at the very end (these are for nets which are only on a single chip)
    let mut pending_edge_nets = vec![];
    // list of pairs of edges that need a bounce in between
    let mut pending_bounces = vec![];
    let mut lanes = lanes.to_vec(); // we remove lanes that are allocated

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

#[cfg(test)]
mod tests {
    use super::*;
    use layout::{Layout, NodeNet, Node};

    #[test]
    /// Nets within this test are all either on the same chip, or span two chips with
    /// a direct lane between them.
    fn test_direct_routes() {
        let a = ChipId(b'A');
        let i = ChipId(b'I');

        test_netlist(&mut [
            // two nodes, on different chips
            NodeNet {
                id: 1.into(),
                nodes: vec![
                    // Ix15
                    Node::Gnd,
                    // Ay1
                    Node::Top2,
                ],
            },
            // single chip
            NodeNet {
                id: 2.into(),
                nodes: vec![
                    // Ay6
                    Node::Top7,
                    // Ay7
                    Node::Top8,
                ],
            },
        ], &[
            Crosspoint { chip: a, x: 0, y: 1, net_id: 1.into() }, // hook up Ay1 to lane leading to Iy0
            Crosspoint { chip: a, x: 1, y: 6, net_id: 2.into() }, // hook up first available lane (Ax1) to node at Ay6
            Crosspoint { chip: a, x: 1, y: 7, net_id: 2.into() }, // hook the same line up to node at Ay7
            Crosspoint { chip: i, x: 15, y: 0, net_id: 1.into() }, // finally connect GND node (Ix15) to lane going to chip A
        ]);
    }

    #[test]
    fn test_bounce_orthogonal() {
        let a = ChipId(b'A');
        let l = ChipId(b'L');
        let net_id = 1.into();

        test_netlist(&mut [
            // Just two chips are involved here, but there is no direct lane to connect them.
            // There is a connection between chip A and chip L though, using Ay0.
            NodeNet {
                id: 1.into(),
                nodes: vec![
                    // Lx8
                    Node::Top1,
                    // Ay1
                    Node::Top2,
                ],
            },
        ], &[
            Crosspoint { chip: a, x: 0, y: 0, net_id }, // connect first free lane on Ax with lane to Ly
            Crosspoint { chip: a, x: 0, y: 1, net_id }, // connect the same lane on Ax with node at Ay1 (Top2)
            Crosspoint { chip: l, x: 8, y: 0, net_id }, // connect the L side of the lane with node at Lx8 (Top1)
        ]);
    }

    #[test]
    /// Connect all nodes on a chip to the same net
    fn test_all_nodes_on_chip_single_net() {
        let chip = ChipId(b'D');
        let net_id = 1.into();
        let x = 0; // first available lane on edge `Dx`

        test_netlist(&mut [
            NodeNet {
                id: 1.into(),
                nodes: vec![
                    // Dy1
                    Node::Top23,
                    // Dy2
                    Node::Top24,
                    // Dy3
                    Node::Top25,
                    // Dy4
                    Node::Top26,
                    // Dy5
                    Node::Top27,
                    // Dy6
                    Node::Top28,
                    // Dy7
                    Node::Top29,
                ],
            },
        ], &[
            Crosspoint { chip, net_id, x, y: 1 },
            Crosspoint { chip, net_id, x, y: 2 },
            Crosspoint { chip, net_id, x, y: 3 },
            Crosspoint { chip, net_id, x, y: 4 },
            Crosspoint { chip, net_id, x, y: 5 },
            Crosspoint { chip, net_id, x, y: 6 },
            Crosspoint { chip, net_id, x, y: 7 },
        ]);
    }

    /// THIS ONE DOESN'T WORK YET:

    // #[test]
    // fn test_multiple_chips() {
    //     let layout = Layout::v4();
    //     let mut chip_status = ChipStatus::default();
    //     let mut nets = [
    //         // two nodes on the same chip, two other nodes on other chips each
    //         NodeNet {
    //             id: 1.into(),
    //             nodes: vec![
    //                 // Jx14
    //                 Node::Supply5V,
    //                 // Ay2
    //                 Node::Top3,
    //                 // Ay3
    //                 Node::Top4,
    //                 // Ey4
    //                 Node::Bottom5,
    //             ],
    //         },
    //     ];
    //     normalize_nets(&mut nets);
    //     layout.nets_to_connections(&nets, &mut chip_status);
    //     let extracted = node_nets_from_chip_status(&chip_status, &layout);
    //     assert_eq!(&nets[..], &extracted[..]);
    //     check_connectivity(&chip_status, &nets, &layout);
    // }

    fn test_netlist(nets: &mut [NodeNet], expected_crosspoints: &[Crosspoint]) {
        // normalize nets, to make it comparisons easier
        normalize_nets(nets);

        let layout = Layout::v4();
        layout.sanity_check();

        // create chip status from netlist
        let mut chip_status = ChipStatus::default();
        layout.nets_to_connections(nets, &mut chip_status);

        // reconstruct netlist from ChipStatus, and compare it with the input
        let extracted_nets = node_nets_from_chip_status(&chip_status, &layout);
        assert_eq!(nets, &extracted_nets[..]);

        // verify that the ChipStatus is sound given the the list of nets.
        // this ensures that each net is fully connected (no disjoint islands)
        check_connectivity(&chip_status, nets, &layout);

        // finally verify that the netlist lead to the expected crosspoint connections
        let crosspoints: Vec<_> = chip_status.crosspoints().collect();
        assert_eq!(&crosspoints[..], expected_crosspoints);
    }

    fn node_nets_from_chip_status<const NODE_COUNT: usize, const LANE_COUNT: usize>(chip_status: &ChipStatus, layout: &Layout<NODE_COUNT, LANE_COUNT>) -> Vec<NodeNet> {
        let nets: Vec<Net> = chip_status.into();
        let mut converted: Vec<NodeNet> = nets.into_iter().map(|net| NodeNet::from_net(&net, layout)).collect();
        normalize_nets(&mut converted);
        converted
    }

    fn check_connectivity<const NODE_COUNT: usize, const LANE_COUNT: usize>(chip_status: &ChipStatus, nets: &[NodeNet], layout: &Layout<NODE_COUNT, LANE_COUNT>) {
        for net in nets {
            chip_status.check_connectivity(net.id, layout);
        }
    }

    /// Normalizes a list of NodeNets to ease comparison
    ///
    /// A normalized netlist has all nets ordered by ID,
    /// and all nodes within each net ordered by node number.
    fn normalize_nets(nets: &mut [NodeNet]) {
        nets.sort_by_key(|net| net.id.0);
        for net in nets {
            net.sort();
        }
    }
}
