use crate::parser::{Input, Parser, Result, ResultExt};

/// A [section *id*](https://webassembly.github.io/spec/core/binary/modules.html#sections)
/// is a byte value that indicates what kind of contents are contained within a WebAssembly
/// [`Section`].
pub type SectionId = std::num::NonZeroU8;

/// Indicates what kind of contents are contained within a WebAssembly [`Section`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum SectionKind {
    /// The section is a known value documented in the
    /// [WebAssembly specification](https://webassembly.github.io/spec/core/binary/modules.html#sections)
    Id(SectionId),
    /// The section is a
    /// [custom section](https://webassembly.github.io/spec/core/binary/modules.html#binary-customsec)
    /// with the given name.
    Custom(std::borrow::Cow<'static, str>),
}

macro_rules! known_ids {
    ($(
        $(#[$meta:meta])*
        $name:ident = $value:literal;
    )*) => {
        impl SectionKind {
            $(
                $(#[$meta])*
                pub const $name: Self = Self::Id({
                    // Safety: value should not be zero
                    unsafe {
                        std::num::NonZeroU8::new_unchecked($value)
                    }
                });
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
        impl SectionKind {
            $(
                $(#[$meta])*
                pub const $name: Self = Self::Custom(std::borrow::Cow::Borrowed($value));
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

/// Represents a
/// [WebAssembly section](https://webassembly.github.io/spec/core/binary/modules.html#sections).
#[derive(Debug)]
pub struct Section<I: Input> {
    kind: SectionKind,
    contents: I,
}

impl<I: Input> Section<I> {
    /// Gets the
    /// [*id* or custom section name](https://webassembly.github.io/spec/core/binary/modules.html#sections)
    /// for this section.
    pub fn kind(&self) -> &SectionKind {
        &self.kind
    }
}

//pub trait SectionParser
//impl Section { fn select_parser(&self) -> dyn SectionParser }

/// Represents the
/// [sequence of sections](https://webassembly.github.io/spec/core/binary/modules.html#binary-module)
/// in a WebAssembly module.
#[derive(Debug)]
pub struct SectionSequence<I: Input> {
    parser: Parser<I>,
}

impl<I: Input> SectionSequence<I> {
    /// Creates a sequence of sections read from the given [`Input`].
    pub fn new(input: I) -> Self {
        Self {
            parser: Parser::new(input),
        }
    }

    fn parse(&mut self) -> Result<Option<Section<I>>> {
        let mut id_byte = 0u8;
        self.parser
            .bytes_exact(std::slice::from_mut(&mut id_byte))
            .context("section id byte")?;

        let kind = SectionId::new(id_byte);

        //let size = self.parser.leb128_u64;

        todo!()
    }
}

impl<I: Input> Iterator for SectionSequence<I> {
    type Item = Result<Section<I>>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.parse().transpose()
    }
}
