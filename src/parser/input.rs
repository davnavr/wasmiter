use std::fs::File;
use std::io::{Cursor, Read, Result, Seek, SeekFrom};

/// Conversion to an [`Input`].
pub trait ToInput<'a> {
    /// The [`Input`] implementation.
    type In: Input;

    /// Attempts to create to an [`Input`].
    fn to_input(&'a self) -> Result<Self::In>;
}

impl<'a, I: ToInput<'a>> ToInput<'a> for &I {
    type In = I::In;

    #[inline]
    fn to_input(&'a self) -> Result<Self::In> {
        Ok(I::to_input(&self))
    }
}

impl<'a, I: ToInput<'a>> ToInput<'a> for &mut I {
    type In = I::In;

    #[inline]
    fn to_input(&'a self) -> Result<Self::In> {
        Ok(I::to_input(&self))
    }
}

impl<'a> ToInput<'a> for &'a [u8] {
    type In = &'a [u8];

    #[inline]
    fn to_input(&'a self) -> Result<Self::In> {
        Ok(self)
    }
}

impl<'a> ToInput<'a> for Cursor<&'a [u8]> {
    type In = Self;

    #[inline]
    fn to_input(&self) -> Result<Self::In> {
        Ok(self.clone())
    }
}

/// Allows reading bytes from some source.
///
/// [`Input`] implementations are expected to keep track of their position within the
/// original source.
pub trait Input: Sized {
    /// The [`Read`] implementation used to read bytes.
    ///
    /// Calls to read bytes from the input **must** advance the [`Input`]'s position.
    type Reader<'a>: Read + ToInput<'a>
    where
        Self: 'a;

    /// Returns a duplicate of the [`Input`] at the current position.
    ///
    /// Using the [`Reader`](Input::Reader) of the duplicate **does not** change the position of
    /// the original.
    fn fork(&self) -> Result<Self>;

    /// Returns a reader used to read bytes.
    ///
    /// Consuming bytes from this reader advances the [`position`](Input::position).
    fn reader(&mut self) -> Result<Self::Reader<'_>>;

    /// Gets current position, which is a byte offset from the start of the input.
    fn position(&self) -> Result<u64>;
}

impl<'a> Input for Cursor<&'a [u8]> {
    type Reader<'b> = &'b mut Self where 'a: 'b;

    #[inline]
    fn fork(&self) -> Result<Self> {
        Ok(self.clone())
    }

    #[inline]
    fn reader(&mut self) -> Result<Self::Reader<'_>> {
        Ok(self)
    }

    #[inline]
    fn position(&self) -> Result<u64> {
        Ok(self.position())
    }
}

#[inline]
fn clone_result<T: Clone>(result: &Result<T>) -> Result<T> {
    #[inline(never)]
    #[cold]
    fn clone_error(error: &std::io::Error) -> std::io::Error {
        std::io::Error::new(
            error.kind(),
            format!("unable to access underlying reader: {error}"),
        )
    }

    match result {
        Ok(value) => Ok(value.clone()),
        Err(e) => Err(clone_error(e)),
    }
}

/// Reads bytes from a [`File`] starting at a given location.
#[derive(Debug)]
pub struct FileReader<'a> {
    offset: &'a mut Result<u64>,
    file: &'a File,
}

impl Read for FileReader<'_> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.file.read(buf)
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
        self.file.read_exact(buf)
    }
}

impl<'a> ToInput<'a> for FileReader<'a> {
    type In = FileInput;

    fn to_input(&self) -> Result<Self::In> {
        Ok(FileInput {
            offset: clone_result(self.offset),
            file: self.file,
        })
    }
}

impl Drop for FileReader<'_> {
    fn drop(&mut self) {
        *self.offset = self.file.stream_position();
    }
}

/// Allows reading bytes from different parts of a [`File`].
#[derive(Debug)]
pub struct FileInput {
    offset: Result<u64>,
    file: File,
}

impl FileInput {
    /// Opens the file at the given [`Path`](std::path::Path) as [`Input`].
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Ok(Self {
            offset: Ok(0),
            file: File::open(path)?,
        })
    }
}

impl Input for FileInput {
    type Reader<'b> = FileReader<'b> where Self: 'b;

    fn position(&self) -> Result<u64> {
        clone_result(&self.offset)
    }

    fn fork(&self) -> Result<Self> {
        Ok(FileInput {
            offset: Ok(self.position()?),
            file: self.file.try_clone()?,
        })
    }

    fn reader(&mut self) -> Result<FileReader<'_>> {
        self.file.seek(SeekFrom::Start(self.position()?))?;
        Ok(FileReader {
            offset: &mut self.offset,
            file: &self.file,
        })
    }
}
