/// Helper trait for keeping track of offsets during parsing.
pub trait Offset {
    /// Gets the offset.
    fn offset(&mut self) -> &mut u64;
}

impl<O: Offset> Offset for &mut O {
    #[inline]
    fn offset(&mut self) -> &mut u64 {
        O::offset(self)
    }
}

impl Offset for u64 {
    #[inline]
    fn offset(&mut self) -> &mut u64 {
        self
    }
}
