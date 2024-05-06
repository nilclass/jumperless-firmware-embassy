use core::num::NonZeroU8;

use heapless::Vec;

use crate::ch446q::Chip;

const MAX_NODES_PER_NET: usize = 64;
const MAX_NETS: usize = 64;

/// Represents a set of connected nodes
#[derive(Clone)]
pub struct Net {
    pub id: NetId,
    pub nodes: Vec<Node, MAX_NODES_PER_NET>,
    pub color: (u8, u8, u8),
}

#[derive(Copy, Clone)]
pub struct NetId(NonZeroU8);

impl From<u8> for NetId {
    fn from(value: u8) -> Self {
        Self(NonZeroU8::new(value).unwrap())
    }
}

impl Net {
    fn new<I: Into<NetId>>(
        id: I,
        nodes: Vec<Node, MAX_NODES_PER_NET>,
        color: (u8, u8, u8),
    ) -> Self {
        Net {
            id: id.into(),
            nodes,
            color,
        }
    }
}

pub struct Nets {
    pub supply_switch_pos: SupplySwitchPos,
    pub nets: Vec<Net, MAX_NETS>,
}

impl Default for Nets {
    fn default() -> Self {
        Self {
            supply_switch_pos: SupplySwitchPos::_5V,
            nets: Vec::from_slice(&[
                Net::new(
                    1,
                    Vec::from_slice(&[Node::GND]).unwrap(),
                    (0x00, 0x1c, 0x04),
                ),
                Net::new(
                    2,
                    Vec::from_slice(&[Node::SUPPLY_5V]).unwrap(),
                    (0x1c, 0x07, 0x02),
                ),
                Net::new(
                    3,
                    Vec::from_slice(&[Node::SUPPLY_3V3]).unwrap(),
                    (0x1c, 0x01, 0x07),
                ),
                Net::new(
                    4,
                    Vec::from_slice(&[Node::DAC0]).unwrap(),
                    (0x23, 0x11, 0x11),
                ),
                Net::new(
                    5,
                    Vec::from_slice(&[Node::DAC1]).unwrap(),
                    (0x23, 0x09, 0x13),
                ),
                Net::new(
                    6,
                    Vec::from_slice(&[Node::ISENSE_PLUS]).unwrap(),
                    (0x23, 0x23, 0x23),
                ),
                Net::new(
                    7,
                    Vec::from_slice(&[Node::ISENSE_MINUS]).unwrap(),
                    (0x23, 0x23, 0x23),
                ),
                Net::new(
                    8,
                    Vec::from_slice(&[Node::bb(2).unwrap(), Node::bb(3).unwrap()]).unwrap(),
                    (0x13, 0x00, 0x23),
                ),
            ])
            .unwrap(),
        }
    }
}

/// Represents a node on the jumperless.
///
/// A node is everything that can be connected to any other nodes
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[allow(unused)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Node {
    GND,
    SUPPLY_5V,
    SUPPLY_3V3,
    DAC0,
    DAC1,
    ISENSE_MINUS,
    ISENSE_PLUS,
    ADC0,
    ADC1,
    ADC2,
    ADC3,
    NANO_D0,
    NANO_D1,
    NANO_D2,
    NANO_D3,
    NANO_D4,
    NANO_D5,
    NANO_D6,
    NANO_D7,
    NANO_D8,
    NANO_D9,
    NANO_D10,
    NANO_D11,
    NANO_D12,
    NANO_D13,
    NANO_A0,
    NANO_A1,
    NANO_A2,
    NANO_A3,
    NANO_A4,
    NANO_A5,
    NANO_A6,
    NANO_A7,
    NANO_RESET,
    NANO_AREF,
    RP_GPIO_0,
    RP_UART_Rx,
    RP_UART_Tx,
    Breadboard(u8),
}

impl Node {
    pub fn bb(n: u8) -> Option<Self> {
        if (1..=60).contains(&n) {
            Some(Node::Breadboard(n))
        } else {
            None
        }
    }

    pub fn id(&self) -> u8 {
        match self {
            Node::Breadboard(n) => *n,
            Node::GND => 100,
            Node::SUPPLY_5V => 105,
            Node::SUPPLY_3V3 => 103,
            Node::DAC0 => 106,
            Node::DAC1 => 107,
            Node::ISENSE_MINUS => 108,
            Node::ISENSE_PLUS => 109,
            Node::ADC0 => 110,
            Node::ADC1 => 111,
            Node::ADC2 => 112,
            Node::ADC3 => 113,
            Node::NANO_D0 => 70,
            Node::NANO_D1 => 71,
            Node::NANO_D2 => 72,
            Node::NANO_D3 => 73,
            Node::NANO_D4 => 74,
            Node::NANO_D5 => 75,
            Node::NANO_D6 => 76,
            Node::NANO_D7 => 77,
            Node::NANO_D8 => 78,
            Node::NANO_D9 => 79,
            Node::NANO_D10 => 80,
            Node::NANO_D11 => 81,
            Node::NANO_D12 => 82,
            Node::NANO_D13 => 83,
            Node::NANO_A0 => 86,
            Node::NANO_A1 => 87,
            Node::NANO_A2 => 88,
            Node::NANO_A3 => 89,
            Node::NANO_A4 => 90,
            Node::NANO_A5 => 91,
            Node::NANO_A6 => 92,
            Node::NANO_A7 => 93,
            Node::NANO_RESET => 84,
            Node::NANO_AREF => 85,
            Node::RP_GPIO_0 => 114,
            Node::RP_UART_Rx => 117,
            Node::RP_UART_Tx => 116,
        }
    }

    /// Returns the pixel (i.e. LED number) for this node
    ///
    /// Only works for breadboard and NANO nodes, which have a dedicated LED.
    ///
    /// Power rails etc. are handled elsewhere.
    pub fn pixel(&self) -> Option<u8> {
        match self {
            Node::Breadboard(n) => Some(*n - 1),
            Node::NANO_D0 => Some(81),
            Node::NANO_D1 => Some(80),
            Node::NANO_D2 => Some(84),
            Node::NANO_D3 => Some(85),
            Node::NANO_D4 => Some(86),
            Node::NANO_D5 => Some(87),
            Node::NANO_D8 => Some(88),
            Node::NANO_D9 => Some(89),
            Node::NANO_D10 => Some(90),
            Node::NANO_D11 => Some(91),
            Node::NANO_D12 => Some(92),
            Node::NANO_RESET => Some(93),
            Node::NANO_AREF => Some(94),
            Node::NANO_A0 => Some(95),
            Node::NANO_A1 => Some(82),
            Node::NANO_A2 => Some(97),
            Node::NANO_A3 => Some(98),
            Node::NANO_A4 => Some(99),
            Node::NANO_A5 => Some(100),
            Node::NANO_A6 => Some(101),
            Node::NANO_A7 => Some(102),
            _ => None,
        }
    }
}

/// Represents position of the supply switch
///
/// This value cannot be detected, it must be set manually by the user.
/// Only affects the color of the rail LEDs.
pub enum SupplySwitchPos {
    _3V3,
    _5V,
    _8V,
}

impl SupplySwitchPos {
    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "3V3" => Some(Self::_3V3),
            "5V" => Some(Self::_5V),
            "8V" => Some(Self::_8V),
            _ => None,
        }
    }
}

/*
{ 0, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,
                                    30,31,32,33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48,49,50,51,52,53,54,55,56,57,58,59,60,61,
                                    0,0,0,0,0,0,0,

                                    81,80,84,85,86,87,88,89,90,91,92,93,94,
                                    95,82,97,98,99,100,101,102,103,104,105,106,107,108,109,110,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,-1,

                                    }
*/

struct ChipStatus([ChipStatusEntry; 12]);

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

    fn set(&mut self, chip: Chip, dimension: Dimension, index: u8, net: NetId) {
        let entry = &mut self.0[chip as u8 as usize];
        match dimension {
            Dimension::X => entry.x[index as usize] = Some(net),
            Dimension::Y => entry.y[index as usize] = Some(net),
        }
    }
}

enum Dimension {
    X,
    Y,
}

trait ChipLayout {
    fn node_to_chip_port(&self, node: Node) -> (Chip, Dimension, u8);
}

fn nets_to_chip_connections<L: ChipLayout>(layout: L, nets: &Nets, chip_status: &mut ChipStatus) {
    chip_status.clear();

    // populate net id for all node ports
    for net in &nets.nets {
        for node in &net.nodes {
            let (chip, dimension, index) = layout.node_to_chip_port(*node);
            chip_status.set(chip, dimension, index, net.id);
        }
    }
}
