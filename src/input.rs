//! Traits and types for reading bytes from a source.

use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};

/// Allows reading bytes from a source, and creating new readers to read bytes at specific
/// locations within the source.
pub trait Input: Sized {
    /// The [`Read`] implementation that provides the bytes.
    type Reader<'a>: Read
    where
        Self: 'a;

    /// Gets a reader used to read bytes at the current location.
    fn reader(&mut self) -> Result<Self::Reader<'_>>;

    /// Returns a reader used to read bytes starting at the current location.
    fn fork(&self) -> Result<Self>;
}

impl<'a> Input for &'a [u8] {
    type Reader<'b> = &'b [u8] where Self: 'b;

    #[inline]
    fn reader(&mut self) -> Result<Self::Reader<'_>> {
        Ok(self)
    }

    #[inline]
    fn fork(&self) -> Result<Self> {
        Ok(self)
    }
}

/// Ensures exclusive access to a reader when reading bytes from a [`Source<I>`].
pub trait SourceLock: Sized {
    /// The reader used to read bytes from the [`Source<I>`].
    type Reader<'a>: Read + Seek
    where
        Self: 'a;

    /// Gets the reader.
    fn lock(&mut self) -> Result<Self::Reader<'_>>;

    /// Returns a new reader, which may point to any location within the [`Source<I>`].
    fn duplicate(&self) -> Result<Self>;
}

impl SourceLock for File {
    type Reader<'a> = &'a mut File;

    #[inline]
    fn lock(&mut self) -> Result<&mut Self> {
        Ok(self)
    }

    #[inline]
    fn duplicate(&self) -> Result<Self> {
        self.try_clone()
    }
}

impl<S: SourceLock + Read + Seek> SourceLock for std::io::BufReader<S> {
    type Reader<'a> = &'a mut Self where Self: 'a;

    #[inline]
    fn lock(&mut self) -> Result<Self::Reader<'_>> {
        Ok(self)
    }

    #[inline]
    fn duplicate(&self) -> Result<Self> {
        Ok(std::io::BufReader::new(self.get_ref().duplicate()?))
    }
}

/// Reads bytes from a specific location within a [`Source<I>`].
#[derive(Debug)]
pub struct SourceReader<'a, R: Seek> {
    offset: &'a mut Result<u64>,
    reader: R,
}

impl<'a, R: Read + Seek> Read for SourceReader<'a, R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf)
    }
}

impl<'a, R: Seek> Drop for SourceReader<'a, R> {
    fn drop(&mut self) {
        *self.offset = self.reader.stream_position();
    }
}

/// Reads bytes from a particular location within a stream.
#[derive(Debug)]
pub struct Source<I> {
    offset: Result<u64>,
    input: I,
}

impl<I: SourceLock> Source<I> {
    /// Creates a new [`Source<I>`] that reads bytes from the beginning of the stream.
    pub fn from_start(input: I) -> Self {
        Self {
            offset: Ok(0),
            input,
        }
    }

    fn offset(&self) -> Result<u64> {
        #[inline(never)]
        #[cold]
        fn clone_error(error: &std::io::Error) -> std::io::Error {
            std::io::Error::new(
                error.kind(),
                format!("unable to access underlying reader: {error}"),
            )
        }

        match self.offset.as_ref() {
            Ok(offset) => Ok(*offset),
            Err(e) => Err(clone_error(e)),
        }
    }
}

impl<I: SourceLock> Input for Source<I> {
    type Reader<'a> = SourceReader<'a, I::Reader<'a>> where I: 'a;

    fn reader(&mut self) -> Result<Self::Reader<'_>> {
        let offset = self.offset()?;
        let mut reader = self.input.lock()?;
        reader.seek(SeekFrom::Start(offset))?;
        Ok(SourceReader {
            offset: &mut self.offset,
            reader,
        })
    }

    fn fork(&self) -> Result<Self> {
        Ok(Self {
            offset: self.offset(),
            input: self.input.duplicate()?,
        })
    }
}

/// Opens the [`File`] at the given [`Path`](std::path::Path) as an [`Input`].
pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Source<std::io::BufReader<File>>> {
    Ok(Source::from_start(std::io::BufReader::new(File::open(
        path,
    )?)))
}
