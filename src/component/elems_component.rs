use crate::bytes::Bytes;
use crate::component;
use crate::parser::{self, Vector, Result, ResultExt};
use crate::instruction_set::InstructionSequence;

pub struct ElementExpressions {

}

pub enum ElementInit<'a, 'b, B: Bytes> {
    Functions(&'b mut Vector<&'a mut u64, B, parser::SimpleParse<component::FuncIdx>>),
    Expressions(ElementExpressions),
}

pub enum ElementKind<'a, B: Bytes> {
    Passive,
    Active(component::TableIdx, &'a mut InstructionSequence<B>),
    Declarative,
}

/// Represents the
/// [**elems** component](https://webassembly.github.io/spec/core/syntax/modules.html#element-segments) of a
/// WebAssembly module, stored in and parsed from the
/// [*element section*](https://webassembly.github.io/spec/core/binary/modules.html#element-section).
#[derive(Clone, Copy)]
pub struct ElemsComponent<B: Bytes> {
    count: usize,
    offset: u64,
    bytes: B,
}

impl<B: Bytes> ElemsComponent<B> {
    /// Uses the given [`Bytes`] to read the contents of the *element section* of a module, starting
    /// at the specified `offset`.
    pub fn new(mut offset: u64, bytes: B) -> Result<Self> {
        Ok(Self {
            count: parser::leb128::usize(&mut offset, &bytes).context("type section count")?,
            bytes,
            offset,
        })
    }

    pub fn next<K, I>(&mut self, kind: K, init: I) -> Result<()>
    where
        K: FnOnce(ElementKind<'_, &B>) -> Result<()>,
        I: FnOnce(ElementInit<'_, '_, &B>) -> Result<()>,
    {
        todo!()
    }
}

