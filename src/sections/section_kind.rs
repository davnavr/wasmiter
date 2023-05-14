use crate::allocator::OwnOrRef;

/// A [section *id*](https://webassembly.github.io/spec/core/binary/modules.html#sections)
/// is a byte value that indicates what kind of contents are contained within a WebAssembly
/// [`Section`](crate::Section).
pub type SectionId = core::num::NonZeroU8;

/// Indicates what kind of contents are contained within a WebAssembly [`Section`](crate::Section).
#[derive(Clone, Eq, Hash, PartialEq)]
pub enum SectionKind<S: AsRef<str>> {
    /// The section is a known value documented in the
    /// [WebAssembly specification](https://webassembly.github.io/spec/core/binary/modules.html#sections)
    Id(SectionId),
    /// The section is a
    /// [custom section](https://webassembly.github.io/spec/core/binary/modules.html#binary-customsec)
    /// with the given name.
    Custom(OwnOrRef<'static, str, S>),
}

impl<S: AsRef<str>> core::fmt::Debug for SectionKind<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Id(id) => f.debug_tuple("Id").field(id).finish(),
            Self::Custom(name) => f.debug_tuple("Custom").field(&name.as_ref()).finish(),
        }
    }
}

macro_rules! known_ids {
    ($(
        $(#[$meta:meta])*
        $name:ident = $value:literal;
    )*) => {
        pub(crate) mod section_id {
            use crate::SectionId;

            $(
                pub(crate) const $name: SectionId = {
                    // Safety: value should not be zero
                    unsafe {
                        core::num::NonZeroU8::new_unchecked($value)
                    }
                };
            )*
        }

        impl<S: AsRef<str>> SectionKind<S> {
            $(
                $(#[$meta])*
                pub const $name: Self = Self::Id(section_id::$name);
            )*
        }
    };
}

// Id should not be 0
known_ids! {
    /// [The *type* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-typesec).
    TYPE = 1;
    /// [The *import* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-importsec).
    IMPORT = 2;
    /// [The *function* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-funcsec).
    FUNC = 3;
    /// [The *table* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-tablesec).
    TABLE = 4;
    /// [The *memory* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-memsec).
    MEMORY = 5;
    /// [The *global* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-globalsec).
    GLOBAL = 6;
    /// [The *export* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-exportsec).
    EXPORT = 7;
    /// [The *start* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-startsec).
    START = 8;
    /// [The *element* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-elemsec).
    ELEMENT = 9;
    /// [The *code* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-codesec).
    CODE = 10;
    /// [The *data* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-datasec).
    DATA = 11;
    /// [The *data count* section](https://webassembly.github.io/spec/core/binary/modules.html#binary-datacountsec).
    DATA_COUNT = 12;

    // Proposed

    /// [The *tag* section](https://webassembly.github.io/exception-handling/core/binary/modules.html#binary-tagsec),
    /// introduced as part of the
    /// [exception handling proposal](https://github.com/WebAssembly/exception-handling).
    TAG = 13;
}

macro_rules! known_custom_ids {
    ($(
        $(#[$meta:meta])*
        $name:ident = $value:literal;
    )*) => {
        pub(in crate::sections) fn cached_custom_name(s: &str) -> Option<&'static str> {
            match s {
                $($value => Some($value),)*
                _ => None,
            }
        }

        impl<S: AsRef<str>> SectionKind<S> {
            $(
                $(#[$meta])*
                pub const $name: Self = Self::Custom(OwnOrRef::Reference($value));
            )*
        }
    };
}

known_custom_ids! {
    /// [The `name` custom section](https://webassembly.github.io/spec/core/appendix/custom.html#name-section),
    /// described in the WebAssembly specification appendix.
    NAME = "name";
    /// [The `build_id` custom section](https://github.com/WebAssembly/tool-conventions/blob/main/BuildId.md),
    /// described in the [WebAssembly tool conventions](https://github.com/WebAssembly/tool-conventions).
    BUILD_ID = "build_id";
    /// [The `producers` custom section](https://github.com/WebAssembly/tool-conventions/blob/main/ProducersSection.md),
    /// described in the [WebAssembly tool conventions](https://github.com/WebAssembly/tool-conventions).
    PRODUCERS = "producers";
    /// [The `core` custom section](https://github.com/WebAssembly/tool-conventions/blob/main/Coredump.md#process-information),
    /// described in the [WebAssembly tool conventions](https://github.com/WebAssembly/tool-conventions) for core dumps.
    CORE = "core";
    /// [The `corestack` custom section](https://github.com/WebAssembly/tool-conventions/blob/main/Coredump.md#threads-and-stack-frames),
    /// described in the [WebAssembly tool conventions](https://github.com/WebAssembly/tool-conventions) for core dumps.
    CORESTACK = "corestack";
    /// [The `dylink.0` custom section](https://github.com/WebAssembly/tool-conventions/blob/main/DynamicLinking.md),
    /// described in the [WebAssembly tool conventions](https://github.com/WebAssembly/tool-conventions) for dynamic linking.
    DYLINK_0 = "dylink.0";
    /// [The `linking` custom section](https://github.com/WebAssembly/tool-conventions/blob/main/Linking.md#linking-metadata-section),
    /// described in the [WebAssembly tool conventions](https://github.com/WebAssembly/tool-conventions) for static linking.
    LINKING = "linking";
}
