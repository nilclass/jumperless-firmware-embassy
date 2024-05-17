// #![no_std]
#![feature(let_chains)]

use std::fmt::Display;
use std::num::NonZeroU8;

pub mod layout;

mod chip_status;
pub use chip_status::ChipStatus;

pub mod util;

mod nets_to_connections;
pub use nets_to_connections::nets_to_connections;

/// Represents a chip
///
/// The value inside is the ASCII character of the chip's letter (A-L).
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ChipId(u8);

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

impl ChipId {
    /// Returns the index of this chip
    ///
    /// Indices are 0 (for chip A) through 11 (for chip L).
    pub fn index(&self) -> usize {
        (self.0 - b'A') as usize
    }

    /// Construct ChipId from given index
    ///
    /// The index must be in the 0..=11 range.
    pub fn from_index(index: usize) -> Self {
        assert!(index < 12);
        Self(b'A' + index as u8)
    }

    /// Get chip port on the X edge, at given index
    pub fn port_x(&self, x: u8) -> ChipPort {
        ChipPort(*self, Dimension::X, x)
    }

    /// Get chip port on the Y edge, at given index
    pub fn port_y(&self, y: u8) -> ChipPort {
        ChipPort(*self, Dimension::Y, y)
    }
}

/// Identifier for a net. Should be unique within a netlist.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct NetId(NonZeroU8);

impl From<u8> for NetId {
    fn from(value: u8) -> Self {
        NetId(NonZeroU8::new(value).unwrap())
    }
}

/// Either X or Y. Used to specify ports and edges.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Dimension {
    X,
    Y,
}

impl Dimension {
    /// Orthogonal dimension
    ///
    /// X returns Y; Y returns X.
    pub fn orthogonal(&self) -> Self {
        match self {
            Dimension::X => Dimension::Y,
            Dimension::Y => Dimension::X,
        }
    }
}

/// Represents one of the sides (X/Y) of a specific chip.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Edge(ChipId, Dimension);

impl Edge {
    /// Returns the orthogonal edge on the same chip
    pub fn orthogonal(&self) -> Self {
        Self(self.0, self.1.orthogonal())
    }

    /// Iterate over all the ports on this edge
    pub fn ports(&self) -> impl Iterator<Item = ChipPort> {
        let Edge(chip, dimension) = *self;
        let range = match dimension {
            Dimension::X => 0..16,
            Dimension::Y => 0..8,
        };
        range.map(move |index| ChipPort(chip, dimension, index))
    }
}

/// Represents a single connection point on one of the chip edges
///
/// Examples: Ay0, Bx7
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ChipPort(ChipId, Dimension, u8);

impl ChipPort {
    /// The edge on which this port resides
    ///
    /// Example:
    ///   ChipPort(ChipId(b'C'), Dimension::Y, 4).edge() //=> Edge(ChipId(b'C'), Dimension::Y)
    pub fn edge(&self) -> Edge {
        Edge(self.0, self.1)
    }
}


/// A Lane connects to ChipPorts (on different chips)
#[derive(Copy, Clone)]
pub struct Lane(ChipPort, ChipPort);

impl Lane {
    /// Is one of the endpoints of this lane on the given edge?
    pub fn touches(&self, edge: Edge) -> bool {
        self.0.edge() == edge || self.1.edge() == edge
    }

    /// Does this lane connect these two edges?
    pub fn connects(&self, from: Edge, to: Edge) -> bool {
        let (a, b) = (self.0.edge(), self.1.edge());
        (a, b) == (from, to) || (a, b) == (to, from)
    }
}

/// Represents a crossing 
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Crosspoint {
    pub chip: ChipId,
    pub net_id: NetId,
    pub x: u8,
    pub y: u8,
}

/// A net is a collection of ChipPorts which are supposed to be interconnected.
///
/// The routing code does not know (or care) about nodes, just about chip ports.
/// The `layout` module provides a `NodeNet` type, which represents a net in terms of nodes.
/// A board-specific layout converts between the two.
pub struct Net {
    id: NetId,
    ports: Vec<ChipPort>,
}

/// Pretty-print crosspoint matrix configuration
pub fn print_crosspoints(crosspoints: impl Iterator<Item = Crosspoint>) {
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

/// A crosspoint config holds one bit for each of the 1536 switches on the jumperless.
pub struct CrosspointConfig([u8; 192]);

impl FromIterator<Crosspoint> for CrosspointConfig {
    fn from_iter<T: IntoIterator<Item = Crosspoint>>(iter: T) -> Self {
        let mut config = CrosspointConfig([0; 192]);

        for crosspoint in iter {
            config.set(crosspoint);
        }

        config
    }
}

const HEX_CHARS: &[u8; 16] = b"0123456789ABCDEF";

impl CrosspointConfig {
    pub fn get(&self, crosspoint: Crosspoint) -> bool {
        (self.0[crosspoint.chip.index() * 16 + crosspoint.x as usize] >> crosspoint.y) & 1 == 1
    }

    pub fn set(&mut self, crosspoint: Crosspoint) {
        self.0[crosspoint.chip.index() * 16 + crosspoint.x as usize] |= 1 << crosspoint.y;
    }

    pub fn clear(&mut self, crosspoint: Crosspoint) {
        self.0[crosspoint.chip.index() * 16 + crosspoint.x as usize] &= !(1 << crosspoint.y);
    }

    pub fn to_hex_bytes(&self) -> [u8; 384] {
        let mut buf = [0; 384];
        for (i, byte) in self.0.iter().enumerate() {
            buf[i * 2] = HEX_CHARS[((byte >> 4) & 0xF) as usize];
            buf[i * 2 + 1] = HEX_CHARS[(byte & 0xF) as usize];
        }
        buf
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

    #[test]
    fn test_multiple_chips() {
        let layout = Layout::v4();
        let mut chip_status = ChipStatus::default();
        let mut nets = [
            // two nodes on the same chip, two other nodes on other chips each
            NodeNet {
                id: 1.into(),
                nodes: vec![
                    // Jx14
                    Node::Supply5V,
                    // Ay2
                    Node::Top3,
                    // Ay3
                    Node::Top4,
                    // Ey4
                    Node::Bottom5,
                ],
            },
        ];
        normalize_nets(&mut nets);
        layout.nets_to_connections(&nets, &mut chip_status);
        let extracted = node_nets_from_chip_status(&chip_status, &layout);
        assert_eq!(&nets[..], &extracted[..]);
        check_connectivity(&chip_status, &nets, &layout);
        print_crosspoints(chip_status.crosspoints());
    }

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
