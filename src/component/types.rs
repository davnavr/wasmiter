use core::fmt::{Display, Formatter};

mod block_type;
mod global_type;
mod table_type;

pub use block_type::BlockType;
pub use global_type::{GlobalMutability, GlobalType};
pub use table_type::TableType;

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

impl ValType {
    pub(crate) fn try_to_ref_type(self) -> Option<RefType> {
        match self {
            Self::FuncRef => Some(RefType::Func),
            Self::ExternRef => Some(RefType::Extern),
            _ => None,
        }
    }
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

impl Display for ValType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::I32 => "i32",
            Self::I64 => "i64",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::FuncRef => "funcref",
            Self::ExternRef => "externref",
            Self::V128 => "v128",
        })
    }
}

impl Display for RefType {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Self::Func => "funcref",
            Self::Extern => "externref",
        })
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
