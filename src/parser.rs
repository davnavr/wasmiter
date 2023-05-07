//! Low-level types and functions for parsing.

mod decoder;
mod error;
mod result_ext;
mod vector;

pub mod input;

pub use decoder::Decoder;
pub use error::{Context, Error, ErrorKind};
pub use result_ext::ResultExt;
pub use vector::Vector;

/// Result type used when parsing input.
pub type Result<T> = core::result::Result<T, Error>;
