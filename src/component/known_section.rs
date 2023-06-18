use crate::component;
use crate::input::{Input, Window};
use crate::parser::{self, ResultExt as _};
use crate::sections::{id as section_id, Section};

macro_rules! known_section {
    ($input:ident: $(
        $(#[$meta:meta])*
        $name:ident($component:ty $(=> $from:ident)?) = $id:ident,
    )*) => {
        /// Represents a well-known WebAssembly [`Section`].
        #[non_exhaustive]
        pub enum KnownSection<$input: Input> {$(
            $(#[$meta])*
            $name($component),
        )*}

        impl<$input: Input> KnownSection<$input> {
            /// Gets the [*id*](https://webassembly.github.io/spec/core/binary/modules.html#sections) for
            /// the section.
            pub fn id(&self) -> u8 {
                match self {
                    $(Self::$name(_) => section_id::$id,)*
                }
            }
        }

        $($(
            impl<$input: Input> core::convert::$from<$component> for KnownSection<$input> {
                #[inline]
                fn from(component: $component) -> Self {
                    Self::$name(component)
                }
            }
        )?)*

        impl<$input: Input> core::fmt::Debug for KnownSection<$input> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                match self {
                    $(Self::$name(component) => f.debug_tuple(stringify!($name)).field(component).finish(),)*
                }
            }
        }

        impl<$input: Input + Clone> Clone for KnownSection<$input> {
            fn clone(&self) -> Self {
                match self {
                    $(Self::$name(component) => Self::$name(Clone::clone(component)),)*
                }
            }
        }
    };
}

known_section! {
    I:
    /// The
    /// [*type section*](https://webassembly.github.io/spec/core/binary/modules.html#type-section).
    Type(component::TypesComponent<I> => From) = TYPE,
    /// The
    /// [*import section*](https://webassembly.github.io/spec/core/binary/modules.html#import-section).
    Import(component::ImportsComponent<I> => From) = IMPORT,
    /// The
    /// [*function section*](https://webassembly.github.io/spec/core/binary/modules.html#function-section).
    Function(component::FunctionSection<I> => From) = FUNC,
    /// The
    /// [*table section*](https://webassembly.github.io/spec/core/binary/modules.html#table-section).
    Table(component::TablesComponent<I> => From) = TABLE,
    /// The
    /// [*memory section*](https://webassembly.github.io/spec/core/binary/modules.html#memory-section).
    Memory(component::MemsComponent<I> => From) = MEMORY,
    /// The
    /// [*global section*](https://webassembly.github.io/spec/core/binary/modules.html#global-section).
    Global(component::GlobalsComponent<I> => From) = GLOBAL,
    /// The
    /// [*export section*](https://webassembly.github.io/spec/core/binary/modules.html#export-section).
    Export(component::ExportsComponent<I> => From) = EXPORT,
    /// Represents the
    /// [**start** component](https://webassembly.github.io/spec/core/syntax/modules.html#start-function)
    /// of a WebAssembly module, encoded in the
    /// [*start section*](https://webassembly.github.io/spec/core/binary/modules.html#start-section).
    Start(crate::index::FuncIdx) = START,
    /// The
    /// [*element section*](https://webassembly.github.io/spec/core/binary/modules.html#element-section).
    Element(component::ElemsComponent<I> => From) = ELEMENT,
    /// The
    /// [*code section*](https://webassembly.github.io/spec/core/binary/modules.html#code-section).
    Code(component::CodeSection<I> => From) = CODE,
    /// The
    /// [*data section*](https://webassembly.github.io/spec/core/binary/modules.html#data-section).
    Data(component::DatasComponent<I> => From) = DATA,
    /// The
    /// [*data count section*](https://webassembly.github.io/spec/core/binary/modules.html#data-count-section)
    /// specifies the number of of entries in the [*data section*](KnownSection::Data).
    DataCount(u32) = DATA_COUNT,
    /// The
    /// [*tag section*](https://webassembly.github.io/exception-handling/core/binary/modules.html#tag-section).
    Tag(component::TagsComponent<I> => From) = TAG,
}

impl<I: Input> KnownSection<I> {
    /// Returns `true` if the section was introduced in WebAssembly 1.0 (the 2017 MVP).
    pub fn is_mvp_section(&self) -> bool {
        matches!(
            self,
            Self::Type(_)
                | Self::Import(_)
                | Self::Function(_)
                | Self::Table(_)
                | Self::Memory(_)
                | Self::Global(_)
                | Self::Export(_)
                | Self::Start(_)
                | Self::Element(_)
                | Self::Code(_)
                | Self::Data(_)
                | Self::DataCount(_)
        )
    }
}

impl<I: Input> KnownSection<Window<I>> {
    /// Attempts to interpret the contents of the given WebAssembly [`Section`].
    ///
    /// Returns `Err(_)` if the section is a custom section, or if the section's
    /// [*id*](https://webassembly.github.io/spec/core/binary/modules.html#sections) is not
    /// recognized.
    ///
    /// Returns `Ok(Err(_))` if the section **was** recognized, but an attempt to parse a length field
    /// failed.
    pub fn interpret(section: Section<I>) -> Result<parser::Parsed<Self>, Section<I>> {
        Ok(match section.id() {
            section_id::TYPE => {
                let contents = section.into_contents();
                component::TypesComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::IMPORT => {
                let contents = section.into_contents();
                component::ImportsComponent::new(contents.base(), contents)
            }
            .map(Self::from),
            section_id::FUNC => {
                let contents = section.into_contents();
                component::FunctionSection::new(contents.base(), contents).map(Self::from)
            }
            section_id::TABLE => {
                let contents = section.into_contents();
                component::TablesComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::MEMORY => {
                let contents = section.into_contents();
                component::MemsComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::GLOBAL => {
                let contents = section.into_contents();
                component::GlobalsComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::EXPORT => {
                let contents = section.into_contents();
                component::ExportsComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::START => {
                let contents = section.into_contents();
                component::index(&mut contents.base(), contents)
                    .context("start section")
                    .map(Self::Start)
            }
            section_id::ELEMENT => {
                let contents = section.into_contents();
                component::ElemsComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::CODE => {
                let contents = section.into_contents();
                component::CodeSection::new(contents.base(), contents).map(Self::from)
            }
            section_id::DATA => {
                let contents = section.into_contents();
                component::DatasComponent::new(contents.base(), contents).map(Self::from)
            }
            section_id::DATA_COUNT => {
                let contents = section.into_contents();
                parser::leb128::u32(&mut contents.base(), contents)
                    .context("data count section")
                    .map(Self::DataCount)
            }
            section_id::TAG => {
                let contents = section.into_contents();
                component::TagsComponent::new(contents.base(), contents).map(Self::from)
            }
            _ => return Err(section),
        })
    }
}

impl<I: Input + Copy> Copy for KnownSection<I> {}
