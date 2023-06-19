use core::fmt::{Debug, Display, Formatter};

#[cfg(feature = "alloc")]
use alloc::{boxed::Box, vec::Vec};

#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;

#[derive(Debug)]
pub(crate) enum ErrorKind {
    #[cfg(feature = "std")]
    IO(std::io::Error),
    #[cfg(feature = "alloc")]
    BadIndexConversion(crate::index::IndexConversionError),
    #[cfg(feature = "alloc")]
    BadStringEncoding(alloc::string::FromUtf8Error),
    BadInput(crate::input::Error),
    BadWasmMagic,
    UnsupportedWasmVersion(u32),
    InvalidOpcode(crate::instruction_set::InvalidOpcode),
    EmptyBlockTypeInValType,
    TypeIndexInValType(crate::index::TypeIdx),
    ExpectedRefType(crate::types::ValType),
    BadElementKind(u8),
    BadTagAttribute(u8),
    BadDataSegmentMode(u32),
    BadElementSegmentMode(u32),
    BadExportKind(u8),
    BadImportKind(u8),
    BadGlobalMutability(u8),
    BadLimitFlags(u8),
    BadFuncTypeTag(u8),
    BadMemArgAlignPower(u32),
    BranchTableCountOverflow,
    BlockNestingCounterOverflow,
    ExpectedEndInstructionButGotDelegate,
    MissingEndInstructions(u32),
    VarLenIntTooLarge {
        bits: u8,
        signed: bool,
    },
    InvalidFormat,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "std")]
            Self::IO(err) => write!(f, "an I/O error occured: {err}"),
            #[cfg(feature = "alloc")]
            Self::BadIndexConversion(err) => Display::fmt(err, f),
            #[cfg(feature = "alloc")]
            Self::BadStringEncoding(err) => Display::fmt(err, f),
            Self::BadInput(err) => Display::fmt(err, f),
            Self::BadWasmMagic => f.write_str("not a valid WebAssembly module"),
            Self::UnsupportedWasmVersion(bad) => {
                write!(f, "unsupported WebAssembly version {bad} ({bad:#010X})")
            }
            Self::InvalidOpcode(err) => Display::fmt(err, f),
            Self::EmptyBlockTypeInValType => {
                f.write_str("expected value type but got empty block type")
            }
            Self::TypeIndexInValType(idx) => {
                write!(f, "expected value type but got type index {idx:?}")
            }
            Self::ExpectedRefType(actual) => write!(f, "expected reference type but got {actual}"),
            Self::BadElementKind(bad) => write!(f, "{bad:#04X} is not a valid elemkind"),
            Self::BadTagAttribute(bad) => write!(f, "{bad:#04X} is not a valid tag attribute"),
            Self::BadDataSegmentMode(bad) => {
                write!(f, "{bad} is not a supported data segment mode")
            }
            Self::BadElementSegmentMode(bad) => {
                write!(f, "{bad} is not a supported element segment mode")
            }
            Self::BadExportKind(bad) => write!(f, "{bad:#04X} is not a known export kind"),
            Self::BadImportKind(bad) => write!(f, "{bad:#04X} is not a known import kind"),
            Self::BadGlobalMutability(bad) => {
                write!(f, "{bad:#04X} is not a valid global mutability flag")
            }
            Self::BadLimitFlags(bad) => write!(f, "{bad:#04X} is not a known limit flag"),
            Self::BadFuncTypeTag(bad) => {
                write!(f, "expected function type (0x60) but got {bad:#04X}")
            }
            Self::BadMemArgAlignPower(a) => {
                write!(f, "{a} is too large to be a valid alignment power")
            }
            Self::BranchTableCountOverflow => write!(
                f,
                "branch table has a label count of {}, which is too large",
                u32::MAX as u64 + 1
            ),
            Self::BlockNestingCounterOverflow => f.write_str("block nesting counter overflowed"),
            Self::ExpectedEndInstructionButGotDelegate => {
                f.write_str("expected end instruction to mark end of expression, but got delegate")
            }
            Self::MissingEndInstructions(1) => f.write_str(
                "missing end instruction for expression, or blocks were not structured correctly",
            ),
            Self::MissingEndInstructions(missing) => {
                write!(
                    f,
                    "blocks are not structured correctly, {missing} end instructions were missing"
                )
            }
            Self::VarLenIntTooLarge { bits, signed } => {
                let signedness = if *signed { "signed" } else { "unsigned" };
                write!(
                    f,
                    "decoded value cannot fit into a {bits}-bit {signedness} integer"
                )
            }
            Self::InvalidFormat => f.write_str("input was malformed"),
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        pub(crate) enum Context {
            Literal(&'static str),
            Boxed(Box<dyn Display + Send + Sync>),
        }

        #[repr(transparent)]
        struct ContextFn<F>(F);

        impl<F: Fn(&mut Formatter) -> core::fmt::Result> Display for ContextFn<F> {
            fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
                self.0(f)
            }
        }

        impl Context {
            #[inline]
            pub(crate) fn from_display<D: core::fmt::Display + Send + Sync + 'static>(d: D) -> Self {
                Self::Boxed(Box::new(d))
            }

            pub(crate) fn from_closure<F>(f: F) -> Self
            where
                F: Fn(&mut Formatter) -> core::fmt::Result + Send + Sync + 'static
            {
                Self::from_display(ContextFn(f))
            }
        }

        impl From<&'static str> for Context {
            #[inline]
            fn from(message: &'static str) -> Self {
                Self::Literal(message)
            }
        }

        impl Debug for Context {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                match self {
                    Self::Literal(s) => Debug::fmt(s, f),
                    Self::Boxed(b) => write!(f, "r#\"{b}\"#"),
                }
            }
        }

        impl Display for Context {
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                match self {
                    Self::Literal(s) => Display::fmt(s, f),
                    Self::Boxed(b) => b.fmt(f),
                }
            }
        }

        struct BoxedError {
            kind: ErrorKind,
            context: Vec<Context>,
            #[cfg(feature = "backtrace")]
            backtrace: Backtrace,
        }

        type ErrorInner = alloc::boxed::Box<BoxedError>;
    } else {
        pub(crate) struct Context;

        impl Context {
            #[inline]
            pub(crate) fn from_display<D: core::fmt::Display + Send + Sync + 'static>(_: D) -> Self {
                Self
            }

            #[inline]
            pub(crate) fn from_closure<F>(_: F) -> Self
            where
                F: Fn(&mut Formatter) -> core::fmt::Result + Send + Sync
            {
                Self
            }
        }

        impl From<&str> for Context {
            #[inline]
            fn from(_: &str) -> Self {
                Self
            }
        }

        #[repr(transparent)]
        struct ErrorInner {
            kind: ErrorKind,
        }
    }
}

/// Describes an error that occured during parsing.
#[repr(transparent)]
pub struct Error {
    inner: ErrorInner,
}

impl Error {
    const _SIZE_CHECK: [(); 1] =
        [(); (core::mem::size_of::<Option<Self>>() == core::mem::size_of::<usize>()) as usize];

    pub(crate) fn new(kind: ErrorKind) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(feature = "alloc")] {
                Self {
                    inner: Box::new(BoxedError {
                        kind,
                        context: Vec::new(),
                        #[cfg(feature = "backtrace")]
                        backtrace: Backtrace::capture(),
                    }),
                }
            } else {
                Self { inner: ErrorInner { kind } }
            }
        }
    }

    /// Gets a [`Backtrace`] describing where in the code the error occured.
    #[cfg(feature = "backtrace")]
    #[cfg_attr(doc_cfg, doc(cfg(feature = "backtrace")))]
    #[inline]
    pub fn backtrace(&self) -> &Backtrace {
        &self.inner.backtrace
    }

    #[inline]
    pub(crate) fn with_location_context(self, description: &'static str, offset: u64) -> Self {
        self.with_context(Context::from_closure(move |f| {
            write!(f, "within the {description}, at offset {offset:#X}")
        }))
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut s = f.debug_struct("Error");

        s.field("kind", &self.inner.kind);

        #[cfg(feature = "alloc")]
        s.field("context", &self.inner.context);

        #[cfg(feature = "backtrace")]
        s.field("backtrace", self.backtrace());

        s.finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "{}", &self.inner.kind)?;

        #[cfg(feature = "alloc")]
        {
            for context in self.inner.context.iter() {
                writeln!(f, "- {context}")?;
            }

            #[cfg(feature = "backtrace")]
            if !f.alternate()
                && self.backtrace().status() == std::backtrace::BacktraceStatus::Captured
            {
                writeln!(f, "with backtrace:")?;
                writeln!(f, "{}", self.backtrace())?;
            }
        }

        Ok(())
    }
}

impl From<crate::input::Error> for Error {
    #[inline]
    fn from(error: crate::input::Error) -> Self {
        Self::new(ErrorKind::BadInput(error))
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        impl Error {
            #[inline]
            pub(crate) fn with_context(mut self, context: Context) -> Self {
                self.inner.context.push(context);
                self
            }
        }

        impl From<alloc::string::FromUtf8Error> for Error {
            #[inline]
            fn from(error: alloc::string::FromUtf8Error) -> Self {
                Self::new(ErrorKind::BadStringEncoding(error))
            }
        }
    } else {
        impl Error {
            #[inline]
            pub(crate) fn with_context(self, _: Context) -> Self {
                self
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        impl std::error::Error for Error {
            fn cause(&self) -> Option<&dyn std::error::Error> {
                match &self.inner.kind {
                    ErrorKind::IO(err) => Some(err),
                    ErrorKind::BadIndexConversion(err) => Some(err),
                    ErrorKind::BadStringEncoding(err) => Some(err),
                    ErrorKind::BadInput(err) => Some(err),
                    _ => None,
                }
            }
        }

        impl From<std::io::Error> for Error {
            #[inline]
            fn from(error: std::io::Error) -> Self {
                Self::new(ErrorKind::IO(error))
            }
        }
    }
}
