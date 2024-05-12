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

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ChipId(u8);

impl ChipId {
    pub fn index(&self) -> usize {
        (self.0 - b'A') as usize
    }

    pub fn from_index(index: usize) -> Self {
        Self(b'A' + index as u8)
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

// #[derive(Copy, Clone)]
// struct Port(ChipId, Dimension, usize);

trait CrosspointSwitch {
    fn set(&mut self, x: usize, y: usize, closed: bool);
}

// #[derive(Copy, Clone)]
// struct Lane<ChipId: Copy>(Port<ChipId>, Port<ChipId>);

// trait CrosspointMatrix {
//     type ChipId: Copy;
//     type CrosspointSwitch: CrosspointSwitch;

//     /// Reset all chips
//     fn reset(&mut self);

//     /// Iterate over all lanes that connect the given chip edges
//     fn lanes(&self, from: (Self::ChipId, Dimension), to: (Self::ChipId, Dimension)) -> impl Iterator<Item = Lane<Self::ChipId>>;

//     fn select(&mut self, chip: Self::ChipId) -> &mut Self::CrosspointSwitch;
// }

// trait NetsToCrosspoints {
//     type NodeId: Copy;
//     type SwitchId;

//     fn route_net(&mut self, net: &[Self::NodeId]) -> impl Iterator<Item = Self::SwitchId>;
// }

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

#[derive(Default)]
struct ChipStatusEntry {
    x: [Option<NetId>; 16],
    y: [Option<NetId>; 8],
}

// impl Display for ChipStatusEntry {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//     }
// }

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
}

impl From<ChipStatus> for Vec<Net> {
    fn from(value: ChipStatus) -> Self {
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

pub struct Crosspoint {
    pub chip: ChipId,
    pub net_id: NetId,
    pub x: u8,
    pub y: u8,
}

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct Edge(ChipId, Dimension);

#[derive(Copy, Clone)]
struct Lane(ChipPort, ChipPort);

impl Lane {
    fn touches(&self, edge: Edge) -> bool {
        self.0.edge() == edge || self.1.edge() == edge
    }
}

fn nets_to_connections(nets: impl Iterator<Item = Net>, chip_status: &mut ChipStatus, lanes: &[Lane]) {
    // list of edges that need to be connected at the very end (these are for nets which are only on a single chip)
    let mut pending_edge_nets = vec![];
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
            let mut lane_index = None;
            for (i, lane) in lanes.iter().enumerate() {
                if lane.touches(edges[0]) && lane.touches(edges[1]) {
                    lane_index = Some(i);
                    break;
                }
            }
            if let Some(i) = lane_index {
                chip_status.set_lane(lanes.remove(i), net.id);
            } else {
                panic!("No available lanes!");
            }
        } else {
            panic!("More than 2 edges not yet implemented");
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
            panic!("No available lanes!");
        }
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;
    use layout::{Layout, NodeNet, Node};

    #[test]
    fn it_works() {
        let layout = Layout::v4();
        let mut chip_status = ChipStatus::default();
        let mut nets = [
            NodeNet {
                id: 1.into(),
                nodes: vec![
                    Node::Gnd,
                    Node::Top2,
                ],
            },
            NodeNet {
                id: 2.into(),
                nodes: vec![
                    Node::Supply5V,
                    Node::Top3,
                    Node::Top4,
                    Node::Bottom5,
                ],
            },
        ];
        normalize_nets(&mut nets);
        layout.nets_to_connections(&nets, &mut chip_status);

        print_crosspoints(chip_status.crosspoints());

        let extracted: Vec<Net> = chip_status.into();
        let mut converted: Vec<NodeNet> = extracted.into_iter().map(|net| NodeNet::from_net(&net, &layout)).collect();
        normalize_nets(&mut converted);
        assert_eq!(&nets[..], &converted[..]);
    }

    fn normalize_nets(nets: &mut [NodeNet]) {
        nets.sort_by_key(|net| net.id.0);
        for net in nets {
            net.sort();
        }
    }
}
