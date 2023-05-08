//! Low-level types and functions for parsing.

mod decoder;
mod error;
mod result_ext;
mod simple_parse;
mod vector;

pub mod input;

pub use decoder::Decoder;
pub use error::{Context, Error, ErrorKind};
pub use result_ext::ResultExt;
pub use simple_parse::SimpleParse;
pub use vector::{Sequence, Vector};

/// Result type used when parsing input.
pub type Result<T> = core::result::Result<T, Error>;

/// Trait for parsers.
pub trait Parse: Clone {
    /// The result of the parser.
    type Output;

    /// Parses the given input.
    fn parse<I: input::Input>(&mut self, input: &mut Decoder<I>) -> Result<Self::Output>;
}
