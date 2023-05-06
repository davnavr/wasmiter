use crate::allocator::{Allocator, Vector};
use crate::component::ValType;

enum FuncTypeInner<S: Vector<ValType>> {
    Inline {
        /// The number of parameters (stored in bits 0 to 3) and results (stored in bits 4 to 7).
        counts: u8,
        types: [ValType; 15],
    },
    Allocated {
        parameter_count: usize,
        types: S,
    },
}

/// Represents a
/// [WebAssembly function type](https://webassembly.github.io/spec/core/syntax/types.html#function-types),
/// which specifies the parameter and result types of a function.
pub struct FuncType<S: Vector<ValType>> {
    inner: FuncTypeInner<S>,
}

impl<S: Vector<ValType>> FuncType<S> {
    /// Creates a new `functype` from the given parameter and result types, using the `Allocator` if necessary.
    ///
    /// The `types` must contain the parameter types followed by the result types.
    pub fn from_slice_in<A: Allocator<Vec<ValType> = S>>(
        parameter_count: usize,
        types: &[ValType],
        allocator: &A,
    ) -> Self {
        debug_assert!(parameter_count <= types.len());
        Self {
            inner: if types.len() <= 15 {
                let mut inline = [ValType::I32; 15];
                inline[0..types.len()].copy_from_slice(types);
                FuncTypeInner::Inline {
                    counts: (((types.len() - parameter_count) as u8) << 4)
                        | (parameter_count as u8),
                    types: inline,
                }
            } else {
                FuncTypeInner::Allocated {
                    parameter_count,
                    types: allocator.allocate_vector_from_slice(types),
                }
            },
        }
    }

    /// Gets the parameter types.
    pub fn parameter_types(&self) -> &[ValType] {
        match &self.inner {
            FuncTypeInner::Inline { counts, types } => &types[..usize::from(*counts & 0xF)],
            FuncTypeInner::Allocated {
                parameter_count,
                types,
            } => &types.as_ref()[..*parameter_count],
        }
    }

    /// Gets the result types.
    pub fn result_types(&self) -> &[ValType] {
        match &self.inner {
            FuncTypeInner::Inline { counts, types } => {
                let parameter_count = usize::from(*counts & 0xF);
                &types[parameter_count..parameter_count + usize::from(*counts >> 4)]
            }
            FuncTypeInner::Allocated {
                parameter_count,
                types,
            } => &types.as_ref()[*parameter_count..],
        }
    }
}

impl<S: Vector<ValType>> core::fmt::Debug for FuncType<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FuncType")
            .field("parameters", &self.parameter_types())
            .field("results", &self.result_types())
            .finish()
    }
}

impl<S: Vector<ValType>> Default for FuncType<S> {
    fn default() -> Self {
        Self {
            inner: FuncTypeInner::Inline {
                counts: 0,
                types: [ValType::I32; 15],
            },
        }
    }
}

impl<S: Vector<ValType>> core::cmp::PartialEq for FuncType<S> {
    fn eq(&self, other: &Self) -> bool {
        self.parameter_types() == other.parameter_types()
            && self.result_types() == other.result_types()
    }
}

impl<S: Vector<ValType>> core::cmp::Eq for FuncType<S> {}

impl<S: Vector<ValType>> core::hash::Hash for FuncType<S> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        core::hash::Hash::hash(self.parameter_types(), state);
        core::hash::Hash::hash(self.result_types(), state);
    }
}
