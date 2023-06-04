/// Helper trait for keeping track of offsets during parsing.
pub trait Offset {
    /// Gets the offset.
    fn offset(&self) -> u64;

    /// Gets a mutable reference to the offset.
    fn offset_mut(&mut self) -> &mut u64;
}

impl<O: Offset> Offset for &mut O {
    #[inline]
    fn offset(&self) -> u64 {
        O::offset(self)
    }

    #[inline]
    fn offset_mut(&mut self) -> &mut u64 {
        O::offset_mut(self)
    }
}

impl Offset for u64 {
    #[inline]
    fn offset(&self) -> u64 {
        *self
    }

    #[inline]
    fn offset_mut(&mut self) -> &mut u64 {
        self
    }
}
