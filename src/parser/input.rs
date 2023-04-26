use std::fs::File;
use std::io::{Read, Result, Seek, SeekFrom};

#[allow(unreachable_pub)]
pub trait Input: Sized {
    type Reader<'a>: Read
    where
        Self: 'a;

    fn fork(&self) -> Result<Self>;

    fn reader(&mut self) -> Result<Self::Reader<'_>>;

    fn position(&self) -> Result<u64>;
}

impl<'a> Input for std::io::Cursor<&'a [u8]> {
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
    pub(crate) fn from_path<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        Ok(Self {
            offset: Ok(0),
            file: File::open(path)?,
        })
    }
}

impl Input for FileInput {
    type Reader<'b> = FileReader<'b> where Self: 'b;

    fn position(&self) -> Result<u64> {
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
