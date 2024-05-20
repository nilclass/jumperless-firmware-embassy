use jumperless_common::{NetId, layout::{Net, Node}};

use heapless::Vec;

const MAX_NODES_PER_NET: usize = 64;
const MAX_NETS: usize = 64;

pub struct Nets {
    pub supply_switch_pos: SupplySwitchPos,
    pub nets: Vec<Net, MAX_NETS>,
    pub colors: Vec<(u8, u8, u8), MAX_NETS>,
}

impl Nets {
    fn add_net<I: Into<NetId>, N: IntoIterator<Item = Node>>(&mut self, id: I, nodes: N, color: (u8, u8, u8)) {
        _ = self.nets.push(Net::from_iter(id.into(), nodes.into_iter()));
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

        nets.add_net(1, [Node::Gnd], (0x00, 0x1c, 0x04));
        nets.add_net(2, [Node::Supply5V], (0x1c, 0x07, 0x02));
        nets.add_net(3, [Node::Supply3V3], (0x1c, 0x01, 0x07));
        nets.add_net(4, [Node::Dac05V], (0x23, 0x11, 0x11));
        nets.add_net(5, [Node::Dac18V], (0x23, 0x09, 0x13));
        nets.add_net(6, [Node::CurrentSensePlus], (0x23, 0x23, 0x23));
        nets.add_net(7, [Node::CurrentSenseMinus], (0x23, 0x23, 0x23));
        nets.add_net(8, [Node::Top2, Node::Top3], (0x13, 0x00, 0x23));

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
