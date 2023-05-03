use crate::allocator::{Allocator, Vector};

/// Represents a
/// [WebAssembly number type](https://webassembly.github.io/spec/core/syntax/types.html#number-types).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum NumType {
    /// 32-bit integer.
    I32,
    /// 64-bit integer.
    I64,
    /// 32-bit IEEE-754 floating point, sometimes referred to as `float`.
    F32,
    /// 64-bit IEEE-754 floating point, sometimes referred to as `double`.
    F64,
}

/// Represents a
/// [WebAssembly vector type](https://webassembly.github.io/spec/core/syntax/types.html#vector-types).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VecType {
    /// A 128-bit vector.
    V128,
}

/// Represents a
/// [WebAssembly reference type](https://webassembly.github.io/spec/core/syntax/types.html#reference-types).
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RefType {
    /// A `funcref`, a reference to a function.
    Func,
    /// An `externref`, an opaque reference to some object provided by the WebAssembly embedder.
    Extern,
}

/// Represents a
/// [WebAssembly value type](https://webassembly.github.io/spec/core/syntax/types.html#value-types),
/// which indicate the types of values.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValType {
    /// [`i32`](NumType::I32)
    I32,
    /// [`i64`](NumType::I64)
    I64,
    /// [`f32`](NumType::F32)
    F32,
    /// [`f64`](NumType::F64)
    F64,
    /// [`funcref`](RefType::Func)
    FuncRef,
    /// [`externref`](RefType::Extern)
    ExternRef,
    /// [`v128`](VecType::V128)
    V128,
}

impl From<NumType> for ValType {
    fn from(ty: NumType) -> Self {
        match ty {
            NumType::I32 => Self::I32,
            NumType::I64 => Self::I64,
            NumType::F32 => Self::F32,
            NumType::F64 => Self::F64,
        }
    }
}

impl From<RefType> for ValType {
    fn from(ty: RefType) -> Self {
        match ty {
            RefType::Extern => Self::ExternRef,
            RefType::Func => Self::FuncRef,
        }
    }
}

impl From<VecType> for ValType {
    fn from(ty: VecType) -> Self {
        match ty {
            VecType::V128 => Self::V128,
        }
    }
}

// TODO: maybe make crate::allocator::SmallVec?

/*
enum ResultTypeInner<S: Vector<ValType>> {
    Inline { length: u8, types: [ValType; 7] },
    Allocated(S),
}

/// Represents a
/// [WebAssembly result type](https://webassembly.github.io/spec/core/syntax/types.html#reference-types).
pub struct ResultType<S: Vector<ValType>> {
    inner: ResultTypeInner<S>,
}

impl<S: Vector<ValType>> ResultType<S> {
    /// Creates a new `resulttype` from the given `types`, using the specified [`Allocator`] if necessary.
    pub fn from_slice_in<A: Allocator<Vec<ValType> = S>>(types: &[ValType], allocator: &A) -> Self {
        Self {
            inner: if types.len() <= 7 {
                let mut inline = [ValType::I32; 7];
                inline[0..types.len()].copy_from_slice(types);
                ResultTypeInner::Inline {
                    length: types.len() as u8,
                    types: inline,
                }
            } else {
                ResultTypeInner::Allocated(allocator.allocate_vector_from_slice(types))
            },
        }
    }

    /// Gets the types.
    pub fn types(&self) -> &[ValType] {
        match &self.inner {
            ResultTypeInner::Inline { length, types } => &types[..usize::from(*length)],
            ResultTypeInner::Allocated(allocated) => allocated.as_ref(),
        }
    }
}

impl<S: Vector<ValType>> Default for ResultType<S> {
    fn default() -> Self {
        Self {
            inner: ResultTypeInner::Inline {
                length: 0,
                types: [ValType::I32; 7],
            },
        }
    }
}

impl<S: Vector<ValType>> Debug for ResultType<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.types()).finish()
    }
}
*/

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
