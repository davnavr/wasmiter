use std::io::Read;

/// An extension of the [`Read`] trait that provides support for creating new readers over the same
/// input at a particular location.
pub trait Input: Read {
    /// A reader used to read a portion of the [`Input`] at a particular location.
    type Portion: Read;

    /// Creates a new reader that reads byte starting at the current location.
    fn portion(&mut self) -> Self::Portion;
}

impl<'a> Input for &'a [u8] {
    type Portion = &'a [u8];

    fn portion(&mut self) -> Self::Portion {
        self
    }
}
