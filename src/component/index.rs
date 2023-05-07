use crate::parser::{self, ResultExt as _};
use core::fmt::Debug;

/// A [WebAssembly index](https://webassembly.github.io/spec/core/binary/modules.html#indices).
pub trait Index:
    From<u8>
    + From<u16>
    + Into<u32>
    + Into<usize>
    + TryFrom<u32, Error = parser::Error>
    + Debug
    + Eq
    + core::hash::Hash
    + Copy
    + Ord
{
    /// A human readable string that indicates what this [`Index`] refers to.
    const NAME: &'static str;
}

impl<I: parser::input::Input> parser::Decoder<I> {
    /// Parses a
    /// [WebAssembly index](https://webassembly.github.io/spec/core/binary/modules.html#indices).
    pub fn index<N: Index>(&mut self) -> parser::Result<N> {
        self.leb128_u32().context(N::NAME).and_then(N::try_from)
    }
}

macro_rules! indices {
    ($(
        $(#[$meta:meta])*
        struct $name:ident = $descriptor:literal;
    )*) => {$(
        $(#[$meta])*
        #[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
        #[repr(transparent)]
        pub struct $name(u32);

        impl $name {
            #[inline]
            fn error_too_large(index: impl core::fmt::Display) -> parser::Error {
                crate::parser_bad_format!("{} {index} is too large", $descriptor)
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                Debug::fmt(&self.0, f)
            }
        }

        impl From<u8> for $name {
            #[inline]
            fn from(index: u8) -> Self {
                Self(u32::from(index))
            }
        }

        impl From<u16> for $name {
            #[inline]
            fn from(index: u16) -> Self {
                Self(u32::from(index))
            }
        }

        impl From<$name> for usize {
            #[inline]
            fn from(index: $name) -> usize {
                index.0 as usize
            }
        }

        impl From<$name> for u32 {
            #[inline]
            fn from(index: $name) -> u32 {
                index.0
            }
        }

        impl TryFrom<u32> for $name {
            type Error = parser::Error;

            fn try_from(index: u32) -> parser::Result<Self> {
                if usize::try_from(index).is_ok() {
                    Ok(Self(index))
                } else {
                    Err(Self::error_too_large(index))
                }
            }
        }

        impl TryFrom<u64> for $name {
            type Error = parser::Error;

            fn try_from(index: u64) -> parser::Result<Self> {
                match u32::try_from(index) {
                    Ok(actual_index) if usize::try_from(index).is_ok() => {
                        Ok(Self(actual_index))
                    }
                    _ => Err(Self::error_too_large(index)),
                }
            }
        }

        impl Index for $name {
            const NAME: &'static str = $descriptor;
        }
    )*};
}

indices! {
    /// A [`typeidx`](https://webassembly.github.io/spec/core/binary/modules.html#binary-typeidx),
    /// which is an index into the
    /// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
    struct TypeIdx = "type index";
    /// A [`funcidx`](https://webassembly.github.io/spec/core/binary/modules.html#binary-funcidx)
    /// refers to an
    /// [imported function](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-importdesc)
    /// or a function defined in the
    /// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section).
    struct FuncIdx = "function index";
    /// A [`tableidx`](https://webassembly.github.io/spec/core/binary/modules.html#binary-tableidx)
    /// refers to an
    /// [imported table](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-importdesc)
    /// or a table defined in the
    /// [*table section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
    struct TableIdx = "table index";
    /// A [`memidx`](https://webassembly.github.io/spec/core/binary/modules.html#binary-memidx)
    /// refers to an
    /// [imported memory](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-importdesc)
    /// or a memory defined in the
    /// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
    struct MemIdx = "memory index";
    /// A [`globalidx`](https://webassembly.github.io/spec/core/binary/modules.html#binary-globalidx)
    /// refers to an
    /// [imported global](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-importdesc)
    /// or a global defined in the
    /// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
    struct GlobalIdx = "global index";
    /// An [`elemidx`](https://webassembly.github.io/spec/core/binary/modules.html#binary-elemidx)
    /// refers to
    /// [element segments](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-elem)
    /// in the
    /// [*element section*](https://webassembly.github.io/spec/core/binary/modules.html#element-section).
    struct ElemIdx = "element index";
    /// A [`dataidx`](https://webassembly.github.io/spec/core/binary/modules.html#binary-dataidx)
    /// refers to
    /// [data segments](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-data)
    /// in the
    /// [*data section*](https://webassembly.github.io/spec/core/binary/modules.html#data-section).
    struct DataIdx = "data index";
    /// A [`localidx`](https://webassembly.github.io/spec/core/binary/modules.html#binary-localidx)
    /// refers to the parameters and local variables of a function. The types of each local
    /// variable are defined in the
    /// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section)
    struct LocalIdx = "local variable index";
    /// A [`labelidx`](https://webassembly.github.io/spec/core/binary/modules.html#binary-labelidx)
    /// refers to
    /// [structured control instructions](https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-instr-control)
    /// within the code of a function.
    struct LabelIdx = "label index";
}
