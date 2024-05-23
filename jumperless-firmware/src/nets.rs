use jumperless_common::{types::NetId, board::Node, types::Net};

use heapless::Vec;

const MAX_NETS: usize = 64;

pub struct Nets {
    pub supply_switch_pos: SupplySwitchPos,
    pub nets: Vec<Net<Node>, MAX_NETS>,
    pub colors: Vec<(u8, u8, u8), MAX_NETS>,
}

pub enum AddNodeError {
    NodeAlreadyAssigned(NetId),
}

impl Nets {
    pub fn add_node(&mut self, net_id: NetId, node: Node) -> Result<(), AddNodeError> {
        for net in &self.nets {
            if net.nodes.contains(node) {
                return Err(AddNodeError::NodeAlreadyAssigned(net.id));
            }
        }
        self.nets[net_id.index()].nodes.insert(node);
        Ok(())
    }

    pub fn remove_node(&mut self, net_id: NetId, node: Node) {
        self.nets[net_id.index()].nodes.remove(node);
    }

    pub fn new_net(&mut self, color: (u8, u8, u8)) -> NetId {
        let net_id = ((self.nets.len() as u8) + 1).into();
        _ = self.nets.push(Net::new(net_id));
        _ = self.colors.push(color);
        net_id
    }

    pub fn with_node(&self, node: Node) -> Option<NetId> {
        for net in &self.nets {
            if net.nodes.contains(node) {
                return Some(net.id);
            }
        }
        None
    }

    pub fn merge(&mut self, dest: NetId, src: NetId) {
        let src_nodes = self.nets[src.index()].nodes.take();
        let dest_net = &mut self.nets[dest.index()];
        for node in src_nodes.iter() {
            dest_net.nodes.insert(node);
        }
    }

    fn add_net<N: IntoIterator<Item = Node>>(&mut self, nodes: N, color: (u8, u8, u8)) {
        let net_id = ((self.nets.len() as u8) + 1).into();
        _ = self.nets.push(Net::from_iter(net_id, nodes.into_iter()));
        _ = self.colors.push(color);
    }

    pub fn color(&self, net_id: NetId) -> (u8, u8, u8) {
        self.colors[net_id.index()]
    }
}

impl Default for Nets {
    fn default() -> Self {
        let mut nets = Self {
            supply_switch_pos: SupplySwitchPos::_5V,
            nets: Vec::new(),
            colors: Vec::new(),
        };

        nets.add_net([Node::GND], (0x00, 0x1c, 0x04));
        #[cfg(feature = "board-v4")]
        {
            nets.add_net([Node::SUPPLY_5V], (0x1c, 0x07, 0x02));
            nets.add_net([Node::SUPPLY_3V3], (0x1c, 0x01, 0x07));
        }
        #[cfg(feature = "board-v5")]
        {
            nets.add_net([Node::TOP_RAIL], (0x30, 0x1A, 0x02));
            nets.add_net([Node::BOTTOM_RAIL], (0x12, 0x09, 0x32));
        }
        nets.add_net([Node::DAC0], (0x23, 0x11, 0x11));
        nets.add_net([Node::DAC1], (0x23, 0x09, 0x13));
        nets.add_net([Node::ISENSE_PLUS], (0x23, 0x23, 0x23));
        nets.add_net([Node::ISENSE_MINUS], (0x23, 0x23, 0x23));
        // nets.add_net([Node::Top2, Node::Top3], (0x13, 0x00, 0x23));

        nets
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

const SSP_3V3: &str = "3V3";
const SSP_5V: &str = "5V";
const SSP_8V: &str = "8V";

impl SupplySwitchPos {
    pub fn parse(input: &str) -> Option<Self> {
        match input {
            SSP_3V3 => Some(Self::_3V3),
            SSP_5V => Some(Self::_5V),
            SSP_8V => Some(Self::_8V),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SupplySwitchPos::_3V3 => SSP_3V3,
            SupplySwitchPos::_5V => SSP_5V,
            SupplySwitchPos::_8V => SSP_8V,
        }
    }
}
