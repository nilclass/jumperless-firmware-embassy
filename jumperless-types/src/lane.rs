use crate::{Edge, Port};

/// A Lane is a physical connection between two ports (on distinct chips)
#[derive(Copy, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Lane(pub Port,  pub Port);

impl Lane {
    /// Is one of the ports of this lane on the given edge?
    pub fn touches(&self, edge: Edge) -> bool {
        self.0.edge() == edge || self.1.edge() == edge
    }

    /// Does this lane connect these two edges?
    pub fn connects(&self, from: Edge, to: Edge) -> bool {
        let (a, b) = (self.0.edge(), self.1.edge());
        (a, b) == (from, to) || (a, b) == (to, from)
    }

    /// Given one of the ports of the lane, return the opposite one.
    ///
    /// Panics if the given port is not part of the lane.
    pub fn opposite(&self, port: Port) -> Port {
        if self.0 == port {
            self.1
        } else if self.1 == port {
            self.0
        } else {
            panic!("Given port must be one of the endpoints of the lane");
        }
    }
}
