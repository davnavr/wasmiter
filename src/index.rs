//! Contains types representing
//! [indices in WebAssembly](https://webassembly.github.io/spec/core/syntax/modules.html#syntax-index).

/// Error type used when an attempt to convert an integer into an [`Index`] fails.
#[derive(Debug)]
pub struct IndexConversionError(&'static &'static str);

impl core::fmt::Display for IndexConversionError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "cannot convert into a {}", self.0)
    }
}

#[cfg(feature = "std")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "std")))]
impl std::error::Error for IndexConversionError {}

impl From<IndexConversionError> for crate::parser::Error {
    #[inline]
    fn from(error: IndexConversionError) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")] {
                Self::new(crate::parser::ErrorKind::BadIndexConversion(error))
            } else {
                let _ = error;
                Self::new(crate::parser::ErrorKind::InvalidFormat)
            }
        }
    }
}

/// A [WebAssembly index](https://webassembly.github.io/spec/core/binary/modules.html#indices).
pub trait Index:
    From<u8>
    + From<u16>
    + Into<u32>
    + Into<usize>
    + TryFrom<u32, Error = IndexConversionError>
    + core::fmt::Debug
    + Eq
    + core::hash::Hash
    + Copy
    + Ord
    + PartialEq<u32>
    + PartialOrd<u32>
    + PartialEq<usize>
    + PartialOrd<usize>
    + Send
    + Sync
    + 'static
{
    /// A human readable string that indicates what this [`Index`] refers to.
    const NAME: &'static str;
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
            /// Returns the index as a `u32`.
            #[inline]
            pub const fn to_u32(self) -> u32 {
                self.0
            }

            /// Returns the index as a `usize`.
            #[inline]
            pub const fn to_usize(self) -> usize {
                self.0 as usize
            }
        }

        impl Index for $name {
            const NAME: &'static str = $descriptor;
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                if f.alternate() {
                    f.debug_tuple(stringify!($name)).field(&self.0).finish()
                } else {
                    core::fmt::Debug::fmt(&self.0, f)
                }
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
                index.to_usize()
            }
        }

        impl From<$name> for u32 {
            #[inline]
            fn from(index: $name) -> u32 {
                index.to_u32()
            }
        }

        impl TryFrom<u32> for $name {
            type Error = IndexConversionError;

            fn try_from(index: u32) -> Result<Self, Self::Error> {
                if usize::try_from(index).is_ok() {
                    Ok(Self(index))
                } else {
                    Err(IndexConversionError(&Self::NAME))
                }
            }
        }

        impl TryFrom<u64> for $name {
            type Error = IndexConversionError;

            fn try_from(index: u64) -> Result<Self, Self::Error> {
                match u32::try_from(index) {
                    Ok(actual_index) if usize::try_from(index).is_ok() => {
                        Ok(Self(actual_index))
                    }
                    _ => Err(IndexConversionError(&Self::NAME)),
                }
            }
        }

        impl core::cmp::PartialEq<u32> for $name {
            #[inline]
            fn eq(&self, other: &u32) -> bool {
                self.0 == *other
            }
        }

        impl core::cmp::PartialOrd<u32> for $name {
            #[inline]
            fn partial_cmp(&self, other: &u32) -> Option<core::cmp::Ordering> {
                core::cmp::PartialOrd::partial_cmp(&self.0, other)
            }
        }

        impl core::cmp::PartialEq<usize> for $name {
            #[inline]
            fn eq(&self, other: &usize) -> bool {
                usize::from(*self) == *other
            }
        }

        impl core::cmp::PartialOrd<usize> for $name {
            #[inline]
            fn partial_cmp(&self, other: &usize) -> Option<core::cmp::Ordering> {
                core::cmp::PartialOrd::partial_cmp(&usize::from(*self), other)
            }
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

    /// A [`tagidx`](https://webassembly.github.io/exception-handling/core/syntax/modules.html#syntax-tagidx)
    /// refers to a
    /// [*tag*s](https://webassembly.github.io/exception-handling/core/syntax/modules.html#tags)
    /// in the
    /// [*tag section*](https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section),
    /// introduced as part of the
    /// [exception handling proposal](https://github.com/WebAssembly/exception-handling).
    struct TagIdx = "tag index";
}
