#![doc = include_str!("../README.md")]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(clippy::undocumented_unsafe_blocks)]

pub mod input;

/// Contains commonly used traits and types.
pub mod prelude {
    #[doc(no_inline)]
    pub use crate::input::Source;
}
