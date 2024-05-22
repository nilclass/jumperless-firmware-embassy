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

    pub fn index(&self) -> usize {
        match self {
            Dimension::X => 0,
            Dimension::Y => 1,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Dimension::X,
            1 => Dimension::Y,
            _ => panic!("Invalid dimension index"),
        }
    }

    pub fn port_count(&self) -> u8 {
        match self {
            Dimension::X => 16,
            Dimension::Y => 8,
        }
    }
}
