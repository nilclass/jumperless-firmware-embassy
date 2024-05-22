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

pub mod set {
    mod edge_set;
    pub use edge_set::EdgeSet;

    mod port_set;
    pub use port_set::PortSet;

    mod lane_set;
    pub use lane_set::LaneSet;
}
