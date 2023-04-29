use std::fs::File;
use std::io::{Cursor, Read, Result, Seek, SeekFrom};

/// Conversion into an [`Input`].
pub trait IntoInput {
    /// The [`Input`] implementation.
    type In: Input;

    /// Gets an [`Input`] from a value.
    fn into_input(self) -> Self::In;
}

impl<I: Input> IntoInput for I {
    type In = Self;

    #[inline]
    fn into_input(self) -> Self::In {
        self
    }
}

impl<'a> IntoInput for &'a [u8] {
    type In = Cursor<&'a [u8]>;

    #[inline]
    fn into_input(self) -> Self::In {
        Cursor::new(self)
    }
}

/// An extension of the [`Read`] trait that allows creating new readers to read bytes starting at
/// the current position.
pub trait Input: Read + Seek + Sized {
    /// Returns a new reader that reads bytes from the original source, starting at the current location.
    fn fork(&self) -> Result<Self>;

    //fn fork_with_length(&self, length: usize) -> Result<std::io::Take<Self>>;
}

impl<'a> Input for Cursor<&'a [u8]> {
    #[inline]
    fn fork(&self) -> Result<Self> {
        Ok(self.clone())
    }
}

/// An extension to the [`Read`] trait that allows creating a new reader that shares the same
/// underlying source.
///
/// Certain [`Read`] implementations, such as [`std::fs::File`], provide `try_clone` methods that
/// create a new reader that is affected by changes to the original. This makes implementing
/// [`Input::fork`] difficult, which is what the [`SeekingInput<I>`] struct is for.
pub trait SharedInput: Read + Seek + Sized {
    /// Attempts to create a new reader to read bytes just like the original.
    ///
    /// Changes to the original **will** result in changes to the clone.
    fn duplicate(&self) -> Result<Self>;
}

impl SharedInput for File {
    fn duplicate(&self) -> Result<Self> {
        self.try_clone()
    }
}

impl<'a, I> SharedInput for &'a I
where
    I: SharedInput,
    &'a I: Read + Seek,
{
    fn duplicate(&self) -> Result<Self> {
        Ok(self)
    }
}

/// An [`Input`] implementation that implements [`Read`] methods by first seeking to a cached
/// location.
///
/// Note that relying on certain [`Read`] implementations such as [`std::fs::File`], which can be
/// shared and mutated across threads, may cause **race conditions** leading to incorrect bytes
/// being returned by a [`SeekingInput<I>`].
#[derive(Debug)]
pub struct SeekingInput<I: SharedInput> {
    position: u64,
    reader: I,
}

impl<I: SharedInput> SeekingInput<I> {
    /// Creates a new [`SeekingInput<I>`] that reads bytes from the start of the stream.
    #[inline]
    pub fn new(reader: I) -> Self {
        Self {
            position: 0,
            reader,
        }
    }

    /// Gets a byte offset from the start of the stream representing the current position.
    #[inline]
    pub fn position(&self) -> u64 {
        self.position
    }

    /// Unwraps this [`SeekingInput<I>`], returning the underlying [`SharedInput`].
    #[inline]
    pub fn into_inner(self) -> I {
        self.reader
    }
}

impl<I: SharedInput> From<I> for SeekingInput<I> {
    #[inline]
    fn from(reader: I) -> Self {
        Self::new(reader)
    }
}

fn increment_position(position: &mut u64, amount: usize) -> Result<()> {
    #[inline]
    #[cold]
    fn position_overflowed() -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, "position overflowed")
    }

    *position = position
        .checked_add(u64::try_from(amount).map_err(|_| position_overflowed())?)
        .ok_or_else(position_overflowed)?;
    Ok(())
}

impl<I: SharedInput> Read for SeekingInput<I> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.seek(SeekFrom::Start(self.position))?;
        let amount = self.reader.read(buf)?;
        increment_position(&mut self.position, amount)?;
        Ok(amount)
    }
}

impl<I: SharedInput> Seek for SeekingInput<I> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let offset = self.reader.seek(pos)?;
        self.position = offset;
        Ok(offset)
    }

    #[inline]
    fn stream_position(&mut self) -> Result<u64> {
        Ok(self.position)
    }
}

impl<I: SharedInput> Input for SeekingInput<I> {
    fn fork(&self) -> Result<Self> {
        Ok(Self {
            position: self.position,
            reader: self.reader.duplicate()?,
        })
    }
}

/// An [`Input`] implementation that reads bytes from a [`File`].
pub type FileInput = SeekingInput<File>;
