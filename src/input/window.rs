use crate::input::Input;

/// Adapts an [`Input`] implementation to limit the amount of bytes that can be read to a specific
/// range.
#[derive(Clone, Copy)]
pub struct Window<I: Input> {
    base: u64,
    length: u64,
    inner: I,
}

impl<I: Input> Window<I> {
    /// Creates a new [`Window`] into the specified [`Input`] that ensures reads can only occur at
    /// the given `offset` for `length` bytes.
    pub fn with_offset_and_length(inner: I, offset: u64, length: u64) -> Self {
        Self {
            base: offset,
            length,
            inner,
        }
    }

    /// Creates a new [`Window`] into the specified [`Input`] that ensures reads can only occur
    /// starting at the given `offset`.
    ///
    /// The [`length`](Window::length) is set to [`u64::MAX`].
    #[inline]
    pub fn with_offset(inner: I, offset: u64) -> Self {
        Self::with_offset_and_length(inner, offset, u64::MAX)
    }

    /// Gets the offset at which the [`Window`]'s content begins.
    #[inline]
    pub fn base(&self) -> u64 {
        self.base
    }

    /// Gets length, in bytes, of the [`Window`].
    #[inline]
    pub fn length(&self) -> u64 {
        self.length
    }

    /// Gets a reference to the inner [`Input`], bypassing the [`Window`]'s restrictions.
    #[inline]
    pub fn as_inner(&self) -> &I {
        &self.inner
    }

    /// Returns the inner [`Input`], bypassing the [`Window`]'s restrictions.
    #[inline]
    pub fn into_inner(self) -> I {
        self.inner
    }
}

impl<I: Input> From<I> for Window<I> {
    /// Creates a new [`Window`] into the specified [`Input`] that allows reading the first
    /// [`u64::MAX`] bytes.
    fn from(inner: I) -> Self {
        Self::with_offset(inner, 0)
    }
}
