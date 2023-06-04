use crate::bytes::{Bytes, Result};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::sync::Mutex;

/// Trait for [`Read`] and [`Seek`] implementations that allow limited cloning.
pub trait SharedInput: Read + Seek {
    /// Returns a clone of `Self`.
    ///
    /// [`Read`] and [`Seek`] methods that modify the clone will also modify `self`.
    fn try_clone(&self) -> std::io::Result<Self>
    where
        Self: Sized;
}

impl SharedInput for File {
    fn try_clone(&self) -> std::io::Result<Self> {
        <std::fs::File>::try_clone(self)
    }
}

struct SharedBytesInner<I> {
    cached_position: u64,
    source: I,
}

impl<I: SharedInput> SharedBytesInner<I> {
    fn check_cached_position(&mut self, offset: u64) -> std::io::Result<()> {
        if offset == self.cached_position {
            return Ok(());
        }

        self.cached_position = self.source.seek(SeekFrom::Start(offset))?;
        Ok(())
    }
}

/// A [`Bytes`] instance that reads bytes from a [`SharedInput`].
///
/// Note that calls to any [`Bytes`] methods are potentially expensive and may result in blocking.
/// See [`Mutex::lock`] for more information.
pub struct SharedBytes<I: SharedInput> {
    inner: Mutex<SharedBytesInner<I>>,
}

impl<I: SharedInput> SharedBytes<I> {
    fn with_input(input: I) -> Self {
        Self {
            inner: Mutex::new(SharedBytesInner {
                cached_position: 0,
                source: input,
            }),
        }
    }

    /// Returns the underlying [`SharedInput`].
    #[inline]
    pub fn into_inner(self) -> I {
        self.inner.into_inner().unwrap().source
    }
}

impl SharedBytes<File> {
    /// Opens the given [`File`] as a [`SharedInput`].
    pub fn open_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        File::open(path).map(Self::with_input)
    }

    /// Opens the [`File`] with the given [`OpenOptions`](std::fs::OpenOptions) as a [`SharedInput`].
    pub fn open_file_with<P: AsRef<std::path::Path>>(
        options: &std::fs::OpenOptions,
        path: P,
    ) -> std::io::Result<Self> {
        options.open(path).map(Self::with_input)
    }
}

impl<I: SharedInput> Bytes for SharedBytes<I> {
    fn read_at<'b>(&self, offset: u64, buffer: &'b mut [u8]) -> Result<&'b mut [u8]> {
        let mut guard = self.inner.lock().unwrap();
        guard.check_cached_position(offset)?;
        let amount = Read::read(&mut guard.source, buffer)?;
        guard.cached_position += amount as u64;
        Ok(&mut buffer[..amount])
    }

    fn length_at(&self, offset: u64) -> Result<u64> {
        let mut guard = self.inner.lock().unwrap();
        guard.check_cached_position(offset)?;

        // Stream is assumed to be positioned at offset

        let end = guard.source.seek(SeekFrom::End(0))?;
        let length = if end < offset {
            // Nonsense, no data left (length is zero) past the end of the stream
            0
        } else {
            end - offset
        };

        // Need to move stream back to cached position
        if offset != end {
            guard.source.seek(SeekFrom::Start(offset))?;
        }

        Ok(length)
    }
}

impl<I: SharedInput + std::fmt::Debug> std::fmt::Debug for SharedBytes<I> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let guard;
        let failure;
        f.debug_tuple("SharedBytes")
            .field(match self.inner.try_lock() {
                Ok(inner) => {
                    guard = inner;
                    &guard.source
                }
                Err(err) => {
                    failure = err;
                    &failure
                }
            })
            .finish()
    }
}
