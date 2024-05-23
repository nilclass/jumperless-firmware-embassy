#![cfg_attr(not(feature = "std"), no_std)]
#![feature(let_chains)]

use jumperless_types::{ChipId, NetId};

pub use jumperless_types as types;

pub mod board;

mod chip_status;
pub use chip_status::ChipStatus;

mod nets_to_connections;
pub use nets_to_connections::nets_to_connections;

/// A single crosspoint coordinate, with associated NetId.
///
/// Represents a unique switch (by Chip, X and Y coordinate) on the board.
///
/// The [`ChipStatus`] structure provides an iterator over these.
#[derive(Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Crosspoint {
    pub chip: ChipId,
    pub net_id: NetId,
    pub x: u8,
    pub y: u8,
}

#[cfg(feature = "std")]
/// Pretty-print crosspoint matrix configuration
pub fn print_crosspoints(crosspoints: impl Iterator<Item = Crosspoint>) {
    let mut matrices = [
        (ChipId::from_ascii(b'A'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'B'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'C'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'D'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'E'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'F'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'G'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'H'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'I'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'J'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'K'), [[None; 16]; 8]),
        (ChipId::from_ascii(b'L'), [[None; 16]; 8]),
    ];

    for crosspoint in crosspoints {
        let matrix = &mut matrices
            .iter_mut()
            .find(|(chip, _)| *chip == crosspoint.chip)
            .unwrap()
            .1;
        matrix[crosspoint.y as usize][crosspoint.x as usize] = Some(crosspoint.net_id);
    }

    for (chip, matrix) in matrices {
        println!("Chip: {}", chip);
        for line in matrix {
            for cell in line {
                if let Some(net_id) = cell {
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
    use jumperless_types::{Port, Net};
    use crate::board::{Board, Node};

    fn setup() {
        _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    /// Nets within this test are all either on the same chip, or span two chips with
    /// a direct lane between them.
    fn test_direct_routes() {
        setup();

        let a = ChipId::from_ascii(b'A');
        let i = ChipId::from_ascii(b'I');

        test_netlist(
            &mut [
                // two nodes, on different chips
                Net {
                    id: 1.into(),
                    nodes: vec![
                        // Ix15
                        Node::GND,
                        // Ay1
                        Node::_2,
                    ]
                    .into_iter()
                    .collect(),
                },
                // single chip
                Net {
                    id: 2.into(),
                    nodes: vec![
                        // Ay6
                        Node::_7,
                        // Ay7
                        Node::_8,
                    ]
                    .into_iter()
                    .collect(),
                },
            ],
            &[
                Crosspoint {
                    chip: a,
                    x: 0,
                    y: 1,
                    net_id: 1.into(),
                }, // hook up Ay1 to lane leading to Iy0
                Crosspoint {
                    chip: a,
                    x: 1,
                    y: 6,
                    net_id: 2.into(),
                }, // hook up first available lane (Ax1) to node at Ay6
                Crosspoint {
                    chip: a,
                    x: 1,
                    y: 7,
                    net_id: 2.into(),
                }, // hook the same line up to node at Ay7
                Crosspoint {
                    chip: i,
                    x: 15,
                    y: 0,
                    net_id: 1.into(),
                }, // finally connect GND node (Ix15) to lane going to chip A
            ],
        );
    }

    #[test]
    fn test_bounce_orthogonal() {
        setup();

        let a = ChipId::from_ascii(b'A');
        let l = ChipId::from_ascii(b'L');
        let net_id = 1.into();

        test_netlist(
            &mut [
                // Just two chips are involved here, but there is no direct lane to connect them.
                // There is a connection between chip A and chip L though, using Ay0.
                Net {
                    id: 1.into(),
                    nodes: vec![
                        // Lx8
                        Node::_1,
                        // Ay1
                        Node::_2,
                    ]
                    .into_iter()
                    .collect(),
                },
            ],
            &[
                Crosspoint {
                    chip: a,
                    x: 0,
                    y: 0,
                    net_id,
                }, // connect first free lane on Ax with lane to Ly
                Crosspoint {
                    chip: a,
                    x: 0,
                    y: 1,
                    net_id,
                }, // connect the same lane on Ax with node at Ay1 (Top2)
                Crosspoint {
                    chip: l,
                    x: 8,
                    y: 0,
                    net_id,
                }, // connect the L side of the lane with node at Lx8 (Top1)
            ],
        );
    }

    #[test]
    fn test_bounce_other_chip() {
        setup();

        let a = ChipId::from_ascii(b'A');
        let b = ChipId::from_ascii(b'B');
        let c = ChipId::from_ascii(b'C');

        test_netlist(
            &mut [
                // exhaust direct lanes between A and B
                Net {
                    id: 1.into(),
                    nodes: vec![Node::_2, Node::_9].into_iter().collect(),
                },
                Net {
                    id: 2.into(),
                    nodes: vec![Node::_3, Node::_10].into_iter().collect(),
                },
                // this one will need to be bounced via another chip
                Net {
                    id: 3.into(),
                    nodes: vec![Node::_4, Node::_11].into_iter().collect(),
                },
            ],
            &[
                Crosspoint {
                    chip: a,
                    x: 2,
                    y: 1,
                    net_id: 1.into(),
                }, // Top2 to lane AB0
                Crosspoint {
                    chip: a,
                    x: 3,
                    y: 2,
                    net_id: 2.into(),
                }, // Top3 to lane AB1
                Crosspoint {
                    chip: a,
                    x: 4,
                    y: 3,
                    net_id: 3.into(),
                }, // Top4 to lane AC0
                Crosspoint {
                    chip: b,
                    x: 0,
                    y: 1,
                    net_id: 1.into(),
                }, // Lane AB0 to Top9
                Crosspoint {
                    chip: b,
                    x: 1,
                    y: 2,
                    net_id: 2.into(),
                }, // Lane AB1 to Top10
                Crosspoint {
                    chip: b,
                    x: 4,
                    y: 3,
                    net_id: 3.into(),
                }, // Lane BC0 to Top11
                Crosspoint {
                    chip: c,
                    x: 0,
                    y: 0,
                    net_id: 3.into(),
                }, // Lane AC0 to lane CL
                Crosspoint {
                    chip: c,
                    x: 2,
                    y: 0,
                    net_id: 3.into(),
                }, // Lane BC0 to lane CL
            ],
        );
    }

    #[test]
    /// Connect all nodes on a chip to the same net
    fn test_all_nodes_on_chip_single_net() {
        setup();

        let chip = ChipId::from_ascii(b'D');
        let net_id = 1.into();
        let x = 0; // first available lane on edge `Dx`

        test_netlist(
            &mut [Net {
                id: 1.into(),
                nodes: vec![
                    // Dy1
                    Node::_23,
                    // Dy2
                    Node::_24,
                    // Dy3
                    Node::_25,
                    // Dy4
                    Node::_26,
                    // Dy5
                    Node::_27,
                    // Dy6
                    Node::_28,
                    // Dy7
                    Node::_29,
                ]
                .into_iter()
                .collect(),
            }],
            &[
                Crosspoint {
                    chip,
                    net_id,
                    x,
                    y: 1,
                },
                Crosspoint {
                    chip,
                    net_id,
                    x,
                    y: 2,
                },
                Crosspoint {
                    chip,
                    net_id,
                    x,
                    y: 3,
                },
                Crosspoint {
                    chip,
                    net_id,
                    x,
                    y: 4,
                },
                Crosspoint {
                    chip,
                    net_id,
                    x,
                    y: 5,
                },
                Crosspoint {
                    chip,
                    net_id,
                    x,
                    y: 6,
                },
                Crosspoint {
                    chip,
                    net_id,
                    x,
                    y: 7,
                },
            ],
        );
    }

    #[test]
    fn test_multiple_chips() {
        setup();

        let a = ChipId::from_ascii(b'A');
        let e = ChipId::from_ascii(b'E');
        let j = ChipId::from_ascii(b'J');
        let net_id = 1.into();

        test_netlist(
            &mut [
                // two nodes on the same chip, two other nodes on other chips each
                Net {
                    id: 1.into(),
                    nodes: vec![
                        // Jx14
                        Node::SUPPLY_5V,
                        // Ay2
                        Node::_3,
                        // Ay3
                        Node::_4,
                        // Ey4
                        Node::_35,
                    ]
                    .into_iter()
                    .collect(),
                },
            ],
            &[
                // lane to J, with node top3
                Crosspoint {
                    chip: a,
                    net_id,
                    x: 1,
                    y: 2,
                },
                // lane to J, with node top4
                Crosspoint {
                    chip: a,
                    net_id,
                    x: 1,
                    y: 3,
                },
                // lane to E, with node top3
                Crosspoint {
                    chip: a,
                    net_id,
                    x: 8,
                    y: 2,
                },
                // lane to E, with node top4
                Crosspoint {
                    chip: a,
                    net_id,
                    x: 8,
                    y: 3,
                },
                // lane to A, with node bottom5
                Crosspoint {
                    chip: e,
                    net_id,
                    x: 0,
                    y: 4,
                },
                // lane to A, with node supply5v
                Crosspoint {
                    chip: j,
                    net_id,
                    x: 14,
                    y: 0,
                },
            ],
        );
    }

    fn test_netlist(nets: &mut [Net<Node>], expected_crosspoints: &[Crosspoint]) {
        // normalize nets, to make it comparisons easier
        normalize_nets(nets);

        let board = crate::board::init_board();

        // create chip status from netlist
        let mut chip_status = ChipStatus::default();
        nets_to_connections(nets.iter(), &mut chip_status, &board).unwrap();

        print_crosspoints(chip_status.crosspoints());

        // reconstruct netlist from ChipStatus, and compare it with the input
        let extracted_nets = node_nets_from_chip_status(&chip_status, &board);
        assert_eq!(nets, &extracted_nets[..]);

        // verify that the ChipStatus is sound given the the list of nets.
        // this ensures that each net is fully connected (no disjoint islands)
        check_connectivity(&chip_status, nets, &board);

        // finally verify that the netlist lead to the expected crosspoint connections
        let crosspoints: Vec<_> = chip_status.crosspoints().collect();
        assert_eq!(&crosspoints[..], expected_crosspoints);
    }

    fn node_nets_from_chip_status(
        chip_status: &ChipStatus,
        board: &Board,
    ) -> Vec<Net<Node>> {
        let mut nets = std::collections::HashMap::new();
        for port in Port::all() {
            if let Some(net_id) = chip_status.get(port) {
                if let Some(node) = board.port_to_node(port) {
                    nets.entry(net_id)
                        .or_insert(Net::new(net_id))
                        .nodes
                        .insert(node);
                }
            }
        }

        let mut nets: Vec<Net<Node>> = nets.into_values().collect();
        normalize_nets(&mut nets);
        nets
    }

    fn check_connectivity(
        chip_status: &ChipStatus,
        nets: &[Net<Node>],
        board: &Board,
    ) {
        for net in nets {
            chip_status.check_connectivity(net.id, board);
        }
    }

    /// Normalizes a list of Nets to ease comparison
    ///
    /// A normalized netlist has all nets ordered by ID,
    /// and all nodes within each net ordered by node number.
    fn normalize_nets(nets: &mut [Net<Node>]) {
        nets.sort_by_key(|net| net.id.index());
    }
}
