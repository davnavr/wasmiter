//! Traits and type definitions for reading bytes from a source.

mod error;

pub use error::{Error, ErrorKind};

/// Result type used when an operation with an [`Input`] fails.
///
/// This type is meant to be a mirror of
/// [`std::io::Result`](https://doc.rust-lang.org/std/io/type.Result.html).
pub type Result<T> = core::result::Result<T, Error>;

/// Trait for reading bytes at specific locations from a source.
pub trait Input {
    /// Moves the reader to a location specified by a byte `offset` from the start of the source.
    fn seek(&mut self, offset: u64) -> Result<()>;

    /// Reads bytes starting at the current [`position`](Input::position) without advancing the
    /// reader. Returns the portion of the buffer filled with the bytes read from the source, and
    /// the remaining portion of the buffer.
    fn peek(&mut self, buffer: &mut [u8]) -> Result<(&[u8], &[u8])>;

    /// Advances the reader by the given byte `amount`. This is equivalent to calling
    /// [`seek`](Input::seek) with the current [`position`](Input::position) plus `amount`.
    fn read(&mut self, amount: u64) -> Result<()>;

    /// Returns the current position of the reader, as a byte offset from the start of the source.
    fn position(&self) -> Result<u64>; // u64?

    // TODO: functions to help with caches/buffers, no-op default impl

    //fn read_bytes(&mut self, buffer: &mut [u8])
    // fn read_bytes_exact(&mut self, buffer: &mut [u8]) -> Result<()> {
    //     match self.peek(buffer)? {
    //         (_, []) => Ok(()),
    //         (_, remaining) => todo!("error for remaining")
    //     }
    // }
}

//impl Input for core::io::Cursor // Oops, no core::io::cursor
