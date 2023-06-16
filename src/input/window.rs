use crate::input::{self, BorrowInput, Input, Result};

/// Adapts an [`Input`] implementation to limit the amount of bytes that can be read to a specific
/// range.
///
/// # Example
///
/// ```
/// use wasmiter::input::{Input, Window};
///
/// let bytes: &[u8] = b"This is a test of the Window struct";
/// let window = Window::with_offset_and_length(bytes, 10, 18); // "test of the Window"
///
/// assert_eq!(window.length_at(22)?, 6);
///
/// let mut buffer = [0u8; 6];
/// let copied = window.read_at(15, &mut buffer)?;
/// assert_eq!(copied, b"of the");
/// # wasmiter::input::Result::Ok(())
/// ```
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

    #[inline]
    pub(super) fn advance(&mut self, amount: u64) -> Result<()> {
        if self.length > 0 {
            input::increment_offset(&mut self.base, amount)?;
            self.shrink(amount);
            Ok(())
        } else {
            Err(input::offset_overflowed(self.base))
        }
    }

    /// Reduces the [`length`](Window::length) of the [`Window`] by the given `amount`, returning
    /// the new length.
    #[inline]
    pub(super) fn shrink(&mut self, amount: u64) {
        self.length = self.length.saturating_sub(amount);
    }

    #[inline]
    fn bounds_check(&self, offset: u64) -> Result<u64> {
        let (end, overflow) = self.base.overflowing_add(self.length);
        if offset >= self.base && offset < end && !overflow {
            Ok(self.length - (offset - self.base))
        } else {
            Err(input::Error::new(
                input::error::ErrorKind::OutOfBounds,
                offset,
                self.inner.length_at(offset).ok(),
            ))
        }
    }
}

impl<I: Input> From<I> for Window<I> {
    /// Creates a new [`Window`] into the specified [`Input`] that allows reading the first
    /// [`u64::MAX`] bytes.
    #[inline]
    fn from(inner: I) -> Self {
        Self::with_offset(inner, 0)
    }
}

impl<I: Input> Input for Window<I> {
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        let remaining = usize::try_from(self.bounds_check(offset)?).unwrap_or(usize::MAX);
        let actual_len = core::cmp::min(remaining, buffer.len());
        self.inner.read_at(offset, &mut buffer[0..actual_len])
    }

    fn length_at(&self, offset: u64) -> Result<u64> {
        Ok(core::cmp::min(
            self.bounds_check(offset)?,
            self.inner.length_at(offset)?,
        ))
    }
}

impl<I: Input> BorrowInput for Window<I> {
    type Borrowed<'a> = Window<&'a I> where I: 'a;

    fn borrow_input(&self) -> Self::Borrowed<'_> {
        Window {
            base: self.base,
            length: self.length,
            inner: &self.inner,
        }
    }
}

impl<I: Input> core::fmt::Debug for Window<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&input::HexDump::from(self.borrow_input()), f)
    }
}
