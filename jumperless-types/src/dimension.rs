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

    /// Index of this dimension within data structures
    ///
    /// X is 0, Y is 1.
    pub fn index(&self) -> usize {
        match self {
            Dimension::X => 0,
            Dimension::Y => 1,
        }
    }

    /// Turn an index (0 or 1) into a dimension (X or Y)
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Dimension::X,
            1 => Dimension::Y,
            _ => panic!("Invalid dimension index"),
        }
    }

    /// Number of ports that a chip has in this dimension
    ///
    /// 16 for X, 8 for Y.
    pub fn port_count(&self) -> u8 {
        match self {
            Dimension::X => 16,
            Dimension::Y => 8,
        }
    }
}
