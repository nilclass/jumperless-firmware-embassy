use crate::{ChipId, Dimension, Port};

/// Represents one of the sides (X/Y) of a specific chip.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Edge(ChipId, Dimension);

impl Edge {
    pub fn new(chip_id: ChipId, dimension: Dimension) -> Self {
        Self(chip_id, dimension)
    }

    pub fn chip_id(&self) -> ChipId {
        self.0
    }

    pub fn dimension(&self) -> Dimension {
        self.1
    }

    /// Returns the orthogonal edge on the same chip
    ///
    /// # Examples
    ///
    /// ```
    /// # use jumperless_types::{Edge, ChipId, Dimension};
    /// let chip = ChipId::from_ascii(b'A');
    /// let ax = Edge::new(chip, Dimension::X);
    /// let ay = Edge::new(chip, Dimension::Y);
    /// assert_eq!(ax.orthogonal(), ay);
    /// assert_eq!(ax, ay.orthogonal());
    /// ```
    pub fn orthogonal(&self) -> Self {
        Self(self.0, self.1.orthogonal())
    }

    /// Iterate over all the ports on this edge
    pub fn ports(&self) -> impl Iterator<Item = Port> {
        let Edge(chip, dimension) = *self;
        match dimension {
            Dimension::X => 0..16,
            Dimension::Y => 0..8,
        }
        .map(move |index| Port::new(chip, dimension, index))
    }
}
