use crate::component::{Limits, RefType};

/// Represents a
/// [WebAssembly table type](https://webassembly.github.io/spec/core/binary/types.html#table-types).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TableType {
    element_type: RefType,
    limits: Limits,
}

impl TableType {
    /// Creates a new table type.
    pub const fn new(element_type: RefType, limits: Limits) -> Self {
        Self {
            element_type,
            limits,
        }
    }
    /// Gets the type of elements stored in the table.
    pub const fn element_type(&self) -> RefType {
        self.element_type
    }

    /// Gets the minimum and maximum number of elements for the table.
    pub const fn limits(&self) -> &Limits {
        &self.limits
    }
}
