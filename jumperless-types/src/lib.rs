#![cfg_attr(not(feature = "std"), no_std)]
#![feature(type_alias_impl_trait)]
#![feature(impl_trait_in_assoc_type)]

mod chip_id;
pub use chip_id::ChipId;

mod net_id;
pub use net_id::NetId;

mod dimension;
pub use dimension::Dimension;

mod edge;
pub use edge::Edge;

mod port;
pub use port::Port;

mod lane;
pub use lane::Lane;

mod node;
pub use node::Node;

pub mod board_spec;
pub mod board;

pub mod set {
    mod edge_set;
    pub use edge_set::EdgeSet;

    mod port_set;
    pub use port_set::PortSet;

    mod lane_set;
    pub use lane_set::LaneSet;

    mod node_set;
    pub use node_set::NodeSet;
}

pub mod map {
    mod port_map;
    pub use port_map::PortMap;
}
