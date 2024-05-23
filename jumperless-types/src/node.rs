/// Common trait for Nodes.
///
/// Concrete node types are board-specific, and implement this trait.
pub trait Node: Copy + core::fmt::Debug + PartialEq + Eq {
    /// Identifier of a Node
    ///
    /// Must be `<= 127`.
    fn id(&self) -> u8;

    /// Construct node from it's `id`.
    ///
    /// Panics if `id` is out of range.
    fn from_id(id: u8) -> Self;
}
