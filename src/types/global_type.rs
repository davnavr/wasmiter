use crate::types::ValType;

/// Indicates whether a
/// [WebAssembly global](https://webassembly.github.io/spec/core/syntax/modules.html#globals) is
/// mutable.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum GlobalMutability {
    /// A [`const`](https://webassembly.github.io/spec/core/syntax/types.html#syntax-mut) global is
    /// one whose value is only assigned once, when the module is instantiated.
    Constant,
    /// A [`var`](https://webassembly.github.io/spec/core/syntax/types.html#syntax-mut) global is
    /// mutable, and can have a value assigned any time.
    ///
    /// This requires support for the
    /// [mutable globals proposal](https://github.com/WebAssembly/mutable-global).
    Variable,
}

/// Represents a
/// [`globaltype`](https://webassembly.github.io/spec/core/syntax/types.html#global-types), which
/// indicates the type of value stored in a
/// [WebAssembly global](https://webassembly.github.io/spec/core/syntax/modules.html#globals) and
/// whether it is mutable.
#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub struct GlobalType {
    mutability: GlobalMutability,
    r#type: ValType,
}

impl GlobalType {
    /// Creates a new [`globaltype`](GlobalType).
    pub const fn new(mutability: GlobalMutability, value_type: ValType) -> Self {
        Self {
            mutability,
            r#type: value_type,
        }
    }

    /// Gets whether or not the global is mutable.
    pub const fn mutability(&self) -> GlobalMutability {
        self.mutability
    }

    /// Gets the type of the value stored in the global.
    pub const fn value_type(&self) -> ValType {
        self.r#type
    }
}
