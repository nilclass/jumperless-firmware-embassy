
trait CrosspointArray {
    /// Select chip with index `i`
    ///
    /// Subsequent `set_*` calls should target that chip.
    fn select_chip(&mut self, i: usize);

    /// Set state of a single switch.
    ///
    /// Targets the chip selected with [`select_chip`].
    fn set_single(&mut self, x: usize, y: usize, closed: bool);
}
