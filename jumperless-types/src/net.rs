use crate::{NetId, Node, set::NodeSet};

#[derive(PartialEq, Eq, Debug)]
pub struct Net<N: Node> {
    pub id: NetId,
    pub nodes: NodeSet<N>,
}

impl<N: Node> Net<N> {
    pub fn new(id: NetId) -> Self {
        Self {
            id,
            nodes: NodeSet::default(),
        }
    }

    pub fn from_iter(id: NetId, nodes: impl Iterator<Item = N>) -> Self {
        Self {
            id,
            nodes: nodes.collect(),
        }
    }
}
